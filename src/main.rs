use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use reqwest::header::USER_AGENT;
use serde::{Deserialize, Serialize};
use tokio::task::JoinSet;

fn git<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
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
            "Git command failed.\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

static CONFIG_FILE: &str = ".gitpatcher.toml";

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

#[tokio::main]
async fn main() -> Result<()> {
    if git(["rev-parse", "--is-inside-work-tree"]).is_err() {
        bail!("Not in a git repository");
    }

    let config_raw = std::env::current_dir()
        .map(|z| z.join(CONFIG_FILE))
        .and_then(std::fs::read_to_string)
        .context(format!("Could not find `{CONFIG_FILE}` configuration file"))?;

    let config = toml::from_str::<Configuration>(&config_raw).context(format!(
        "Could not parse `{CONFIG_FILE}` configuration file"
    ))?;

    let client = Arc::new(reqwest::Client::new());

    let mut set = JoinSet::new();

    let requests = config.pull_requests.iter().map(|pr| {
        client
            .clone()
            .get(format!(
                "https://api.github.com/repos/{}/pulls/{pr}",
                config.repo
            ))
            .header(USER_AGENT, "gitpatcher")
            .send()
    });

    for fut in requests {
        set.spawn(fut);
    }

    // first backup the config file

    let backup_branch = "gitpatcher-config-file-backup";

    git(["switch", "--create", backup_branch])?;
    git(["add", CONFIG_FILE])?;
    git(["commit", "--message", &format!("Backup {CONFIG_FILE}")])?;

    // fetch and checkout the main repository

    let local_main_temp_branch = "gitpatcher-main-4412503";

    git([
        "remote",
        "add",
        local_main_temp_branch,
        &format!("https://github.com/{}.git", config.repo),
    ])?;

    git([
        "fetch",
        local_main_temp_branch,
        &format!("{0}:{0}", config.remote_branch),
    ])?;

    git([
        "checkout",
        &format!("{local_main_temp_branch}/{}", config.remote_branch),
    ])?;

    // fetch each pull request and merge it into the temporary branch
    while let Some(res) = set.join_next().await {
        let out = res??.text().await?;
        let response: GitHubResponse = serde_json::from_str(&out).unwrap();

        let local_branch = format!(
            "gitpatcher-{}-{}",
            response.head.repo.clone_url, response.head.r#ref
        );

        let remote_name = &response.head.repo.clone_url;
        let branch = &response.head.r#ref;

        // Fetch all of the remotes for each of the pull requests
        git(["remote", "add", &local_branch, remote_name])?;

        // Fetch the pull request branches for each of the remotes
        git(["fetch", remote_name, &format!("{0}:{0}", branch)])?;

        // Merge all remotes into main repository
        git([
            "merge",
            &format!("{remote_name}/{branch}"),
            "--message",
            &format!("gitpatcher: Merge {branch} of {remote_name}"),
        ])?;
    }

    let another_temp = "another-temporary-branch";

    git(["switch", "--create", another_temp])?;

    // replace the original branch with our new branch
    git([
        "branch",
        "--move",
        "--force",
        another_temp,
        &config.local_branch,
    ])?;

    // Restore our configuration file
    git(["cherry-pick", "--no-commit", backup_branch])?;
    git(["commit", "--message", &format!("Restore {CONFIG_FILE}")])?;

    Ok(())
}
