use std::ffi::OsString;
use std::fs::ReadDir;
use std::io::Write;
mod commands;
mod types;

use std::{
    fs::{create_dir, read_dir, read_to_string, File},
    path::PathBuf,
};

use anyhow::{anyhow, bail, Context, Result};
use commands::{add_remote_branch, git, merge_into_main};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::header::USER_AGENT;
use reqwest::{Error, Response};
use tempfile::tempfile;
use types::{Configuration, GitHubResponse};

static CONFIG_ROOT: &str = ".gitpatcher";
static CONFIG_FILE: &str = "config.toml";
static APP_NAME: &str = "gitpatcher";

fn with_uuid(s: &str) -> String {
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

fn restore_backup(file_name: &OsString, contents: &str) -> Result<()> {
    let path = PathBuf::from(CONFIG_ROOT).join(file_name);
    let mut file = File::create(&path)?;

    write!(file, "{contents}")?;

    Ok(())
}

async fn handle_request(request: Result<Response, Error>) -> Result<GitHubResponse> {
    match request {
        Ok(res) if res.status().is_success() => {
            let out = res.text().await?;

            let response: GitHubResponse =
                serde_json::from_str(&out).context("Could not parse response.\n{out}")?;

            Ok(response)
        }
        Ok(res) => Err(anyhow!(
            "Request failed with status: {}\nResponse: {}",
            res.status(),
            res.text().await?
        )),
        Err(err) => Err(anyhow!("Error sending request: {}", err)),
    }
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

    let local_main_temp_remote = with_uuid(&config.repo);

    git(&[
        "remote",
        "add",
        &local_main_temp_remote,
        &format!("https://github.com/{}.git", config.repo),
    ])?;

    let local_main_temp_branch = with_uuid(&config.remote_branch);

    git(&[
        "fetch",
        &local_main_temp_remote,
        &format!("{}:{local_main_temp_branch}", config.remote_branch),
    ])?;

    git(&["checkout", &local_main_temp_branch])?;

    let client = reqwest::Client::new();

    // fetch each pull request and merge it into the detached head remote
    while let Some(pull_request) = config.pull_requests.iter().next() {
        let request = client
            .get(format!(
                "https://api.github.com/repos/{}/pulls/{pull_request}",
                config.repo
            ))
            .header(USER_AGENT, "{APP_NAME}")
            .send()
            .await;

        let response = match handle_request(request).await {
            Ok(response) => response,
            Err(err) => {
                eprintln!("An error has occured: {err}");
                continue;
            }
        };

        let local_remote_name = with_uuid(&response.head.r#ref);
        let remote = &response.head.repo.clone_url;
        let remote_branch = &response.head.r#ref;
        let local_branch = with_uuid(remote_branch);

        match add_remote_branch(&local_remote_name, &local_branch, remote, remote_branch) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("An error has occured: {err}");
                continue;
            }
        };

        match merge_into_main(&local_branch, remote_branch) {
            Ok(_) => (),
            Err(err) => {
                eprintln!("An error has occured: {err}");
                continue;
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

    let temporary_branch = with_uuid("temp");

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

    for (file_name, _, contents) in backed_up_files.iter() {
        restore_backup(file_name, contents).context("Could not restore backups")?;

        // apply patches if they exist
        if let Some(ref patches) = config.patches {
            if patches.contains(file_name.to_str().unwrap()) {
                git(&["am", "--keep-cr", "--signoff", contents])?;
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
