use std::{
    fs::{self, File},
    io::Write,
    path,
};

use crate::{
    backup::{backup_files, restore_backup},
    git_commands::{add_remote_branch, checkout_from_remote, merge_into_main},
    types::{CommandArgs, Configuration, GitHubResponse},
    utils::{display_link, make_request, normalize_pr_title, with_uuid},
    APP_NAME, CONFIG_FILE, CONFIG_ROOT, INDENT,
};
use anyhow::Context;
use colored::Colorize;
use dialoguer::Confirm;
use reqwest::Client;

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        println!("{INDENT}{}{}", "✓ ".bright_green().bold(), format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {{
        eprintln!("{INDENT}{}{}", "✗ ".bright_red().bold(), format!($($arg)*))
    }};
}

pub async fn run(
    _args: &CommandArgs,
    root: &path::Path,
    git: impl Fn(&[&str]) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    println!();

    let config_path = root.join(CONFIG_ROOT);

    let config_file_path = config_path.join(CONFIG_FILE);

    let config_raw = fs::read_to_string(config_file_path.clone()).context(format!(
        "Could not find `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    let config = toml::from_str::<Configuration>(&config_raw).context(format!(
        "Could not parse `{CONFIG_ROOT}/{CONFIG_FILE}` configuration file"
    ))?;

    let config_files = fs::read_dir(&config_path).context(format!(
        "Could not read files in directory {:?}",
        &config_path
    ))?;

    let backed_up_files = backup_files(config_files)
        .context(format!("Could not {} configuration files", crate::APP_NAME))?;

    let local_remote = with_uuid(&config.repo);

    let remote_remote = format!("https://github.com/{}.git", config.repo);

    let local_branch = with_uuid(&config.remote_branch);

    add_remote_branch(
        &local_remote,
        &local_branch,
        &remote_remote,
        &config.remote_branch,
    )?;

    let previous_branch = checkout_from_remote(&local_branch, &local_remote)?;

    let client = reqwest::Client::new();

    // Git cannot handle multiple threads executing commands in the same repository, so we can't use threads
    for pull_request in config.pull_requests.iter() {
        // TODO: refactor this to not use such horrible nesting
        match fetch_pull_request(&config.repo, pull_request, &client).await {
            Ok((response, info)) => {
                match merge_pull_request(info, &git).await {
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

    if let Err(err) = fs::create_dir(root.join(CONFIG_ROOT)) {
        git(&["checkout", &previous_branch])?;
        git(&["remote", "remove", &local_remote])?;
        git(&["branch", "--delete", "--force", &local_branch])?;
        return Err(anyhow::anyhow!(err).context("Could not create directory {CONFIG_ROOT}"));
    };

    for (file_name, _file, contents) in backed_up_files.iter() {
        restore_backup(file_name, contents, root).context("Could not restore backups")?;

        // apply patches if they exist
        if let Some(ref patches) = config.patches {
            let file_name = file_name
                .to_str()
                .and_then(|file_name| file_name.get(0..file_name.len() - 6))
                .unwrap_or_default();

            if patches.contains(file_name) {
                git(&[
                    "am",
                    "--keep-cr",
                    "--signoff",
                    &format!(
                        "{}/{file_name}.patch",
                        root.join(CONFIG_ROOT).to_str().unwrap_or_default()
                    ),
                ])
                .context(format!("Could not apply patch {file_name}, skipping"))?;

                let last_commit_message = git(&["log", "-1", "--format=%B"])?;
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

    git(&["add", CONFIG_ROOT])?;
    git(&[
        "commit",
        "--message",
        &format!("{APP_NAME}: Restore configuration files"),
    ])?;

    let temporary_branch = with_uuid("temp-branch");

    git(&["switch", "--create", &temporary_branch])?;

    git(&["remote", "remove", &local_remote])?;
    git(&["branch", "--delete", "--force", &local_branch])?;

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
        git(&[
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

pub fn help(_args: &CommandArgs) -> anyhow::Result<()> {
    fn subcommand(command: &str, args: &str, description: &str) -> String {
        let command = command.yellow();
        let args = args.green();
        format!("{command}{args}\n    {} {description}", "»".black())
    }

    fn flags(flags: &[&str; 2], description: &str) -> String {
        let flags: Vec<_> = flags.iter().map(|flag| flag.magenta()).collect();
        let flag1 = &flags[0];
        let flag2 = &flags[1];
        let flags = format!(
            "{flag1}{}{flag2}",
            if *flag2 == "".into() {
                "".into()
            } else {
                ", ".black()
            }
        );
        format!("{flags}\n    {} {description}", "»".black())
    }

    println!(
        "
  {app_name} {version}
  {author}{less_than}{email}{greater_than}

  Usage:

    {app_name} {flags} {command} {args}

  Commands:

    {init} 

    {run}

    {gen_patch} 

    {pr_fetch} 

  Flags:

    {help_flag}

    {version_flag}
",
        author = "Nikita Revenco ".italic(),
        less_than = "<".black().italic(),
        email = "pm@nikitarevenco.com".italic(),
        greater_than = ">".black().italic(),
        app_name = APP_NAME.blue(),
        flags = "[<flags>]".magenta(),
        command = "<command>".yellow(),
        args = "[<args>]".green(),
        version = env!("CARGO_PKG_VERSION"),
        init = subcommand("init", "", "Create example config file"),
        pr_fetch = subcommand(
            "pr-fetch",
            " <repo-link> <pr-numbers>...",
            "Fetch pull request for a GitHub repository as a local branch",
        ),
        gen_patch = subcommand(
            "gen-patch",
            " <commit-hashes>...",
            "Generate a .patch file from commit hashes",
        ),
        run = subcommand("run", "", &format!("Start {APP_NAME}")),
        help_flag = flags(&["-h", "--help"], "print this message"),
        version_flag = flags(&["-v", "--version"], "get package version"),
    );

    Ok(())
}

pub fn init(_args: &CommandArgs, root: &path::Path) -> anyhow::Result<()> {
    let example_config = include_bytes!("../example-config.toml");

    let config_path = root.join(CONFIG_ROOT);

    let config_file_path = config_path.join(CONFIG_FILE);

    if config_file_path.exists() {
        let confirmation = Confirm::new()
            .with_prompt(format!(
                "\n{INDENT}{} File {config_file_path} already exists. Overwrite it?",
                "»".black(),
                config_file_path = config_file_path.to_string_lossy().blue()
            ))
            .interact()
            .unwrap();
        if !confirmation {
            anyhow::bail!("Did not overwrite {config_file_path:?}");
        }
    }

    let _ = fs::create_dir(config_path);

    let mut file = File::create(&config_file_path)?;

    file.write_all(example_config)?;

    success!("Created config file {config_file_path:?}");

    Ok(())
}

pub fn gen_patch(_args: &CommandArgs) -> anyhow::Result<()> {
    Ok(())
}

pub async fn pr_fetch(
    args: &CommandArgs,
    _git: impl Fn(&[&str]) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let mut args = args.iter();
    let repo = match args.next() {
        Some(repo) => repo,
        None => {
            return Err(anyhow::anyhow!(
                "Please provide a repo-owner/repo for example: helix-editor/helix"
            ))
        }
    };

    let client = reqwest::Client::new();

    for pull_request in args {
        match fetch_pull_request(repo, pull_request, &client).await {
            Ok((response, info)) => {
                success!(
                    "Fetched pull request {} available at branch {}",
                    display_link(
                        &format!(
                            "{}{} {}",
                            "#".bright_blue(),
                            pull_request.bright_blue(),
                            response.title.blue().italic()
                        ),
                        &response.html_url
                    ),
                    info.branch.local_name.cyan()
                )
            }
            Err(err) => {
                eprintln!("{err:#?}");
                continue;
            }
        };
    }

    Ok(())
}

pub struct Branch {
    local_name: String,
    remote_name: String,
}

pub struct Remote {
    local_name: String,
    remote_name: String,
}

pub struct BranchAndRemote {
    branch: Branch,
    remote: Remote,
}

impl BranchAndRemote {
    pub fn new(
        local_branch: &str,
        remote_branch: &str,
        local_remote: &str,
        remote_remote: &str,
    ) -> Self {
        let branch = Branch {
            local_name: local_branch.into(),
            remote_name: remote_branch.into(),
        };
        let remote = Remote {
            local_name: local_remote.into(),
            remote_name: remote_remote.into(),
        };
        Self { branch, remote }
    }
}

pub async fn merge_pull_request(
    info: BranchAndRemote,
    git: &impl Fn(&[&str]) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    merge_into_main(&info.branch.local_name, &info.branch.remote_name).context(
        "Could not merge branch into the current branch for pull request #{pull_request}, skipping",
    )?;

    let has_unstaged_changes = git(&["diff", "--cached", "--quiet"]).is_err();

    if has_unstaged_changes {
        git(&[
            "commit",
            "--message",
            &format!(
                "{APP_NAME}: Merge branch {} of {}",
                &info.branch.remote_name, &info.remote.remote_name
            ),
        ])?;
    }

    git(&["remote", "remove", &info.remote.local_name])?;
    git(&["branch", "--delete", "--force", &info.branch.local_name])?;

    Ok(())
}

pub async fn fetch_pull_request(
    repo: &str,
    pull_request: &str,
    client: &Client,
) -> anyhow::Result<(GitHubResponse, BranchAndRemote)> {
    let url = format!("https://api.github.com/repos/{}/pulls/{pull_request}", repo);

    let response = make_request(client, &url).await.context(format!(
        "Couldn't fetch required data from remote, skipping. #{pull_request}. Url fetched: {url}"
    ))?;

    let remote_remote = &response.head.repo.clone_url;

    let local_remote = with_uuid(&format!(
        "{title}-{}",
        pull_request,
        title = normalize_pr_title(&response.html_url)
    ));

    let remote_branch = &response.head.r#ref;

    let local_branch = with_uuid(&format!(
        "{title}-{}",
        pull_request,
        title = normalize_pr_title(&response.title)
    ));

    add_remote_branch(&local_remote, &local_branch, remote_remote, remote_branch).context(
        format!("Could not add remove branch for pull request #{pull_request}, skipping"),
    )?;

    let info = BranchAndRemote::new(&local_branch, remote_branch, &local_remote, remote_remote);

    Ok((response, info))
}
