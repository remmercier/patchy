use std::ffi::OsString;
use std::fs::ReadDir;
use std::io::Write;
mod commands;
mod types;

use std::{
    fs::{create_dir, read_dir, read_to_string, File},
    path::PathBuf,
    sync::Arc,
};

use anyhow::{bail, Context, Error, Result};
use commands::git;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::header::USER_AGENT;
use reqwest::Response;
use tempfile::tempfile;
use tokio::task::{JoinError, JoinSet};
use types::{Configuration, GitHubResponse};

static CONFIG_ROOT: &str = ".gitpatcher";
static CONFIG_FILE: &str = "config.toml";
static APP_NAME: &str = "gitpatcher";

fn gen_name(s: &str) -> String {
    let hash: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    format!("gitpatcher-{s}-{hash}")
}

fn backup_files(config_files: ReadDir) -> Vec<(OsString, File, String)> {
    config_files
        .filter_map(|config_file| {
            if let Ok(config_file) = config_file {
                let mut destination_backed_up = tempfile().unwrap();
                let contents = read_to_string(config_file.path()).unwrap();
                let filename = config_file.file_name();
                if write!(destination_backed_up, "{contents}").is_ok() {
                    Some((filename, destination_backed_up, contents))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

fn restore_backup(file_name: OsString, contents: String) -> Result<()> {
    let path = PathBuf::from(CONFIG_ROOT).join(&file_name);
    let mut file = File::create(&path)?;

    write!(file, "{contents}")?;

    Ok(())
}

async fn parse_response(
    res: Result<Result<Response, reqwest::Error>, JoinError>,
) -> Result<GitHubResponse> {
    let out = res??.text().await?;

    let response: GitHubResponse =
        serde_json::from_str(&out).context("Could not parse response.\n{out}")?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<()> {
    if git(&["rev-parse", "--is-inside-work-tree"]).is_err() {
        bail!("Not in a git repository");
    }

    let config_path = std::env::current_dir().map(|cd| cd.join(CONFIG_ROOT))?;

    let config_file_path = config_path.join(CONFIG_FILE);

    let config_raw = std::fs::read_to_string(config_file_path.clone()).context(format!(
        "Could not find `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    let config = toml::from_str::<Configuration>(&config_raw).context(format!(
        "Could not parse `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    let config_files = read_dir(config_path)?;

    let backed_up_files = backup_files(config_files);

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
        let response = match parse_response(res).await {
            Ok(res) => res,
            Err(error) => {
                eprintln!("{error}");
                continue;
            }
        };

        let local_remote_name = format!("{APP_NAME}-{}", response.head.r#ref);

        let remote = &response.head.repo.clone_url;
        let remote_branch = &response.head.r#ref;

        // Fetch all of the remotes for each of the pull requests
        match git(&["remote", "add", &local_remote_name, remote]) {
            Ok(_) => (),
            Err(_) => {
                git(&["remote", "remove", &local_remote_name])?;
            }
        };

        let local_branch = gen_name(remote_branch);

        // Fetch the pull request branches for each of the remotes
        git(&["fetch", remote, &format!("{remote_branch}:{local_branch}")])?;

        // Merge all remotes into main repository
        match git(&["merge", &local_branch, "--no-commit", "--no-ff"]) {
            Ok(_) => println!("Merged {remote_branch} successfully"),
            Err(_) => {
                let files_with_conflicts = git(&["diff", "--name-only", "--diff-filter=U"])?;
                for file_with_conflict in files_with_conflicts.lines() {
                    if file_with_conflict.ends_with(".md") {
                        git(&["checkout", "--ours", file_with_conflict])?;
                        git(&["add", file_with_conflict])?;
                        println!("Merged {remote_branch} successfully and disregarded conflicts")
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

    // Restore our configuration files
    create_dir(CONFIG_ROOT)?;

    for (file_name, _, contents) in backed_up_files {
        restore_backup(file_name, contents);

        // apply patches if they exist
        if let Some(ref patches) = config.patches {
            if patches.contains(file_name.to_str().unwrap()) {
                git(&["am", "--keep-cr", "--signoff", &contents])?;
            }
        }
    }

    // clean up
    git(&["remote", "remove", &local_main_temp_remote])?;
    git(&["branch", "-D", &local_main_temp_branch])?;

    git(&["add", CONFIG_ROOT])?;

    git(&[
        "commit",
        "--message",
        &format!("{APP_NAME}: Restore configuration files"),
    ])?;

    Ok(())
}
