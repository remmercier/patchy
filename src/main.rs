use std::sync::Arc;

use anyhow::{anyhow, bail, Context, Result};
use reqwest::{get, header::USER_AGENT};
use serde::{Deserialize, Serialize};
use tokio::{sync::Semaphore, task::JoinSet};

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

    while let Some(res) = set.join_next().await {
        let out = res??.text().await?;
        let response: GitHubResponse = serde_json::from_str(&out).unwrap();
        dbg!(response);
    }

    // const TASKS_LIMIT: usize = 3;

    // let semaphore = Arc::new(Semaphore::new(TASKS_LIMIT));

    // for _ in 0..5 {
    //     let permit = semaphore.clone().acquire_owned().await.unwrap();
    //     tokio::spawn(async move {
    //         let resp = client
    //             .clone()
    //             .get(format!(
    //                 "https://api.github.com/repos/helix-editor/helix/pulls/12309"
    //             ))
    //             .header(USER_AGENT, "gitpatcher")
    //             .send()
    //             .await?
    //             .text()
    //             .await?;

    //         drop(permit);
    //     });
    // }

    // semaphore.acquire_many(TASKS_LIMIT as u32).await.unwrap();

    Ok(())
}
