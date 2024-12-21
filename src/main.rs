use std::{fs::File, sync::Arc};

use anyhow::{anyhow, bail, Context, Result};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use tokio::task::JoinSet;

fn git(args: &[&str]) -> Result<String> {
    let current_dir = std::env::current_dir().unwrap();

    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(current_dir)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned())
    } else {
        Err(anyhow!(
            "Git command failed.\nCommand: git {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

static CONFIG_FILE: &str = ".gitpatcher.toml";
static APP_NAME: &str = "gitpatcher";

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Configuration {
    repo: String,
    remote_branch: String,
    local_branch: String,
    pull_requests: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GitHubResponse {
    head: Head,
}

#[derive(Serialize, Deserialize, Debug)]
struct Head {
    repo: Repo,
    r#ref: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Repo {
    clone_url: String,
}

fn gen_name(s: &str) -> String {
    let hash: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    format!("gitpatcher-{s}-{hash}")
}

#[tokio::main]
async fn main() -> Result<()> {
    if git(&["rev-parse", "--is-inside-work-tree"]).is_err() {
        bail!("Not in a git repository");
    }

    let config_file_path = std::env::current_dir().map(|cd| cd.join(CONFIG_FILE))?;

    let config_raw = std::fs::read_to_string(config_file_path.clone())
        .context(format!("Could not find `{CONFIG_FILE}` configuration file"))?;

    let config = toml::from_str::<Configuration>(&config_raw).context(format!(
        "Could not parse `{CONFIG_FILE}` configuration file"
    ))?;

    // backup the config file
    let mut backup_file: File = tempfile::tempfile().context("Unable to backup config file")?;
    write!(backup_file, "{config_raw}")?;

    // fetch and checkout the main repository in detached HEAD state from the remote

    let local_main_temp_remote = gen_name(&config.repo);

    git(&[
        "remote",
        "add",
        &local_main_temp_remote,
        &format!("https://github.com/{}.git", config.repo),
    ])?;

    let local_main_temp_branch = gen_name(&config.remote_branch);

    git(&[
        "fetch",
        &local_main_temp_remote,
        &format!("{}:{local_main_temp_branch}", config.remote_branch),
    ])?;

    git(&["checkout", &local_main_temp_branch])?;

    let client = Arc::new(reqwest::Client::new());

    let mut set = JoinSet::new();

    let requests = config.pull_requests.iter().map(|pull_request| {
        client
            .clone()
            .get(format!(
                "https://api.github.com/repos/{}/pulls/{pull_request}",
                config.repo
            ))
            .header(USER_AGENT, "{APP_NAME}")
            .send()
    });

    for fut in requests {
        set.spawn(fut);
    }

    // fetch each pull request and merge it into the detached head remote
    while let Some(res) = set.join_next().await {
        let out = res??.text().await?;
        let response: GitHubResponse = serde_json::from_str(&out).unwrap();

        let local_remote_name = format!("{APP_NAME}-{}", response.head.r#ref);

        let remote = &response.head.repo.clone_url;
        let remote_branch = &response.head.r#ref;

        // Fetch all of the remotes for each of the pull requests
        git(&["remote", "add", &local_remote_name, remote])?;

        let local_branch = gen_name(remote_branch);

        // Fetch the pull request branches for each of the remotes
        git(&["fetch", remote, &format!("{remote_branch}:{local_branch}")])?;

        // Merge all remotes into main repository
        match git(&["merge", &local_branch, "--no-commit", "--no-ff"]) {
            Ok(_) => println!("Merged {local_branch} successfully"),
            Err(_) => {
                let files_with_conflicts = git(&["diff", "--name-only", "--diff-filter=U"])?;
                for file_with_conflict in files_with_conflicts.lines() {
                    if file_with_conflict.ends_with(".md") {
                        git(&["checkout", "--ours", file_with_conflict])?;
                        git(&["add", file_with_conflict])?;
                    } else {
                        eprintln!("Unresolved conflict in {file_with_conflict}")
                    }
                }
            }
        };

        if git(&["diff", "--cached", "--quiet"]).is_ok() {
            println!("No changes to commit after merging");
        } else {
            git(&[
                "commit",
                "--message",
                &format!(
                    "{APP_NAME}: Merge branch {remote_branch} of {remote} [resolved conflicts]"
                ),
            ])?;
        }

        // clean up by removing the temporary remote
        git(&["remote", "remove", &local_remote_name])?;
        git(&["branch", "-D", &local_branch])?;
    }

    let temporary_branch = gen_name("temp");

    git(&["switch", "--create", &temporary_branch])?;

    // forcefully renames the branch we are currently on into the branch specified by the user.
    // WARNING: this is a destructive action which erases the original branch
    git(&[
        "branch",
        "--move",
        "--force",
        &temporary_branch,
        &config.local_branch,
    ])?;

    // Restore our configuration file
    let mut config = String::new();
    backup_file
        .read_to_string(&mut config)
        .context(format!("Unable to restore {CONFIG_FILE} config file"))?;

    File::create(config_file_path).and_then(|mut path| path.write(config.as_bytes()))?;

    git(&["add", CONFIG_FILE])?;

    git(&[
        "commit",
        "--message",
        &format!("{APP_NAME}: Restore {CONFIG_FILE}"),
    ])?;

    // clean up
    git(&["remote", "remove", &local_main_temp_remote])?;
    git(&["branch", "-D", &local_main_temp_branch])?;

    Ok(())
}
