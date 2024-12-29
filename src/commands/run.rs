use std::fs;

use anyhow::Context;
use colored::Colorize;
use dialoguer::Confirm;

use crate::{
    backup::{backup_files, restore_backup},
    fail,
    git_commands::{
        add_remote_branch, checkout_from_remote, fetch_pull_request, merge_pull_request, GIT,
        GIT_ROOT,
    },
    info, success,
    types::{CommandArgs, Configuration},
    utils::{display_link, with_uuid},
    APP_NAME, CONFIG_FILE, CONFIG_ROOT, INDENT,
};

pub async fn run(_args: &CommandArgs) -> anyhow::Result<()> {
    println!();

    let config_path = GIT_ROOT.join(CONFIG_ROOT);

    let config_file_path = config_path.join(CONFIG_FILE);

    let config_raw = fs::read_to_string(config_file_path.clone()).context(format!(
        "Could not find `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    let config = toml::from_str::<Configuration>(&config_raw).context(format!(
        "Could not parse `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    if config.repo.is_empty() {
        return Err(anyhow::anyhow!(
            r#"You haven't specified a `repo` in your config, which can be for example:
  - "helix-editor/helix"
  - "microsoft/vscode"

  For more information see this guide: https://github.com/NikitaRevenco/patchy/blob/main/README.md""#
        ));
    }

    dbg!(&config);

    let config_files = fs::read_dir(&config_path).context(format!(
        "Could not read files in directory {:?}",
        &config_path
    ))?;

    let backed_up_files = backup_files(config_files)
        .context(format!("Could not {} configuration files", crate::APP_NAME))?;

    let local_remote = with_uuid(&config.repo);

    let remote_remote = format!("https://github.com/{}.git", config.repo);

    let local_branch = with_uuid(&config.remote_branch);

    dbg!("1");

    // TODO: consider case where user has not specified any pull requests in their config
    add_remote_branch(
        &local_remote,
        &local_branch,
        &remote_remote,
        &config.remote_branch,
    )?;
    dbg!("2");

    let previous_branch = checkout_from_remote(&local_branch, &local_remote)?;

    let client = reqwest::Client::new();

    if config.pull_requests.is_empty() {
        info!("You haven't specified any pull requests to fetch in your config.")
    } else {
        // TODO: make this concurrent, see https://users.rust-lang.org/t/processing-subprocesses-concurrently/79638/3
        // Git cannot handle multiple threads executing commands in the same repository, so we can't use threads, but we can run processes in the background
        for pull_request in config.pull_requests.iter() {
            // TODO: refactor this to not use such deep nesting
            match fetch_pull_request(&config.repo, pull_request, &client, None).await {
                Ok((response, info)) => {
                    match merge_pull_request(info).await {
                        Ok(()) => {
                            success!(
                                "Merged pull request {}",
                                display_link(
                                    &format!(
                                        "{}{} {}",
                                        "#".bright_blue(),
                                        pull_request.bright_blue(),
                                        &response.title.blue().italic()
                                    ),
                                    &response.html_url
                                ),
                            )
                        }
                        Err(err) => {
                            fail!(
                                "Could not merge pull request {pr}\n\n{err:#?}",
                                pr = pull_request.bright_blue()
                            );
                            continue;
                        }
                    };
                }
                Err(err) => {
                    fail!("Could not fetch branch from remote\n\n{err:#?}");
                    continue;
                }
            }
        }
    }

    if let Err(err) = fs::create_dir(GIT_ROOT.join(CONFIG_ROOT)) {
        GIT(&["checkout", &previous_branch])?;
        GIT(&["remote", "remove", &local_remote])?;
        GIT(&["branch", "--delete", "--force", &local_branch])?;
        return Err(anyhow::anyhow!(err).context("Could not create directory {CONFIG_ROOT}"));
    };

    for (file_name, _file, contents) in backed_up_files.iter() {
        restore_backup(file_name, contents).context("Could not restore backups")?;

        // apply patches if they exist
        if let Some(ref patches) = config.patches {
            let file_name = file_name
                .to_str()
                .and_then(|file_name| file_name.get(0..file_name.len() - 6))
                .unwrap_or_default();

            if patches.contains(file_name) {
                GIT(&[
                    "am",
                    "--keep-cr",
                    "--signoff",
                    &format!(
                        "{}/{file_name}.patch",
                        GIT_ROOT.join(CONFIG_ROOT).to_str().unwrap_or_default()
                    ),
                ])
                .context(format!("Could not apply patch {file_name}, skipping"))?;

                let last_commit_message = GIT(&["log", "-1", "--format=%B"])?;
                success!(
                    "Applied patch {file_name} {}",
                    last_commit_message
                        .lines()
                        .next()
                        .unwrap_or_default()
                        .blue()
                        .italic()
                );
            }
        }
    }

    GIT(&["add", CONFIG_ROOT])?;
    GIT(&[
        "commit",
        "--message",
        &format!("{APP_NAME}: Restore configuration files"),
    ])?;

    let temporary_branch = with_uuid("temp-branch");

    GIT(&["switch", "--create", &temporary_branch])?;

    GIT(&["remote", "remove", &local_remote])?;
    GIT(&["branch", "--delete", "--force", &local_branch])?;

    let confirmation = Confirm::new()
        .with_prompt(format!(
            "\n{INDENT}{} Overwrite branch {}? This is irreversible.",
            "»".black(),
            config.local_branch.cyan()
        ))
        .interact()
        .unwrap();

    if confirmation {
        // forcefully renames the branch we are currently on into the branch specified by the user.
        // WARNING: this is a destructive action which erases the original branch
        GIT(&[
            "branch",
            "--move",
            "--force",
            &temporary_branch,
            &config.local_branch,
        ])?;
        println!("\n{INDENT}{}", "  Success!\n".green().bold());
    } else {
        let command = format!(
            "  git branch --move --force {temporary_branch} {}",
            config.local_branch
        );
        let command = format!("\n{INDENT}{}\n", command.magenta(),);
        println!(
            "\n{INDENT}  You can still manually overwrite {} with the following command:\n  {command}",
            config.local_branch.cyan(),
        );
        std::process::exit(1)
    }

    Ok(())
}
