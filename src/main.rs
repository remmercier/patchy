mod backup;
mod commands;
mod types;
mod utils;

use colored::Colorize;
use std::fs::{create_dir, read_dir};
use tokio::task;

use anyhow::{Context, Result};
use backup::{backup_files, restore_backup};
use commands::{add_remote_branch, checkout, git, merge_into_main};
use types::Configuration;
use utils::{make_request, with_uuid};

static CONFIG_ROOT: &str = ".gitpatcher";
static CONFIG_FILE: &str = "config.toml";
static APP_NAME: &str = "gitpatcher";

macro_rules! success {
    ($($arg:tt)*) => {{
        format!("    {}{}", "✓ ".bright_green().bold(), format!($($arg)*))
    }};
}

fn display_link(text: &str, url: &str) -> String {
    format!("\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\", url, text)
}

#[tokio::main]
async fn main() -> Result<()> {
    let config_path = std::env::current_dir().map(|cd| cd.join(CONFIG_ROOT))?;

    let config_file_path = config_path.join(CONFIG_FILE);

    let config_raw = std::fs::read_to_string(config_file_path.clone()).context(format!(
        "Could not find `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    let config = toml::from_str::<Configuration>(&config_raw).context(format!(
        "Could not parse `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    let config_files = read_dir(&config_path).context(format!(
        "Could not read files in directory {:?}",
        &config_path
    ))?;

    let backed_up_files =
        backup_files(config_files).context(format!("Could not {APP_NAME} configuration files"))?;

    let local_remote = with_uuid(&config.repo);

    let remote_remote = format!("https://github.com/{}.git", config.repo);

    let local_branch = with_uuid(&config.remote_branch);

    add_remote_branch(
        &local_remote,
        &local_branch,
        &remote_remote,
        &config.remote_branch,
    )?;

    checkout(&local_branch, &local_remote)?;

    let client = reqwest::Client::new();

    let futures = config.pull_requests.iter().map(|pull_request| {
        let client = client.clone();
        let config_repo = config.repo.clone();
        let pull_request = pull_request.clone();

        task::spawn(async move {
            let response = match make_request(
                &client,
                &format!(
                    "https://api.github.com/repos/{}/pulls/{}",
                    config_repo, pull_request
                ),
            )
            .await
            {
                Ok(response) => response,
                Err(err) => {
                    eprintln!(
                        "Couldn't fetch required data from remote, skipping. #{}. Skipping.\n{err}",
                        pull_request
                    );
                    return;
                }
            };

            let remote_remote = &response.head.repo.clone_url;
            let local_remote = with_uuid(&response.head.r#ref);
            let remote_branch = &response.head.r#ref;
            let local_branch = with_uuid(remote_branch);

            if let Err(err) = async {
                add_remote_branch(&local_remote, &local_branch, remote_remote, remote_branch)?;
                merge_into_main(&local_branch, remote_branch)?;
                Ok::<(), anyhow::Error>(())
            }
            .await
            {
                eprintln!(
                    "Couldn't merge remote branch from pull request #{}. Skipping.\n{err}",
                    pull_request
                );
                return;
            } else {
                let success_message = success!(
                    "Merged pull request {}",
                    display_link(
                        &format!(
                            "{}{} {}",
                            "#".bright_blue(),
                            pull_request.bright_blue(),
                            response.title.blue().italic()
                        ),
                        &response.html_url
                    ),
                );
                println!("{success_message}")
            }

            // Commit changes if necessary
            let has_unstaged_changes = git(&["diff", "--cached", "--quiet"]).is_err();

            if has_unstaged_changes {
                if let Err(err) = git(&[
                    "commit",
                    "--message",
                    &format!("{APP_NAME}: Merge branch {remote_branch} of {remote_remote}"),
                ]) {
                    eprintln!("Failed to commit changes: {err}");
                }
            }

            // Clean up
            if let Err(err) = git(&["remote", "remove", &local_remote]) {
                eprintln!("Failed to remove remote {local_remote}: {err}");
            }

            if let Err(err) = git(&["branch", "-D", &local_branch]) {
                eprintln!("Failed to delete branch {local_branch}: {err}");
            }
        })
    });

    futures::future::join_all(futures).await;

    let temporary_branch = with_uuid("temp-branch");

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

    create_dir(CONFIG_ROOT)?;

    for (file_name, _, contents) in backed_up_files.iter() {
        restore_backup(file_name, contents).context("Could not restore backups")?;

        // apply patches if they exist
        if let Some(ref patches) = config.patches {
            let file_name = file_name
                .to_str()
                .and_then(|s| s.get(0..s.len() - 6))
                .unwrap_or_default();

            if patches.contains(file_name) {
                git(&[
                    "am",
                    "--keep-cr",
                    "--signoff",
                    &format!("{CONFIG_ROOT}/{file_name}.patch"),
                ])
                .context(format!("Could not apply patch {file_name}, skipping"))?;

                let last_commit_message = git(&["log", "-1", "--format=%B"])?;
                let success_message = success!(
                    "Applied patch {file_name} {}",
                    last_commit_message
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .blue()
                        .italic()
                );

                println!("{success_message}")
            }
        }
    }

    // clean up
    git(&["remote", "remove", &local_remote])?;
    git(&["branch", "-D", &local_branch])?;

    git(&["add", CONFIG_ROOT])?;

    git(&[
        "commit",
        "--message",
        &format!("{APP_NAME}: Restore configuration files"),
    ])?;

    Ok(())
}
