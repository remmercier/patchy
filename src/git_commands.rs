use crate::INDENT;
use colored::Colorize;
use std::{
    path::{Path, PathBuf},
    process::Output,
};

use anyhow::{anyhow, Context, Result};
use once_cell::sync::Lazy;
use reqwest::Client;

use crate::{
    flags::IS_VERBOSE,
    trace,
    types::{Branch, BranchAndRemote, GitHubResponse, Remote},
    utils::{make_request, normalize_commit_msg, with_uuid},
    APP_NAME,
};

pub fn is_valid_branch_name(branch_name: &str) -> bool {
    branch_name
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '.' || ch == '-' || ch == '/' || ch == '_')
}

pub static GITHUB_REMOTE_PREFIX: &str = "git@github.com:";
pub static GITHUB_REMOTE_SUFFIX: &str = ".git";

pub fn spawn_git(args: &[&str], git_dir: &Path) -> Result<Output, std::io::Error> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(git_dir)
        .output()
}

pub fn get_git_output(output: Output, args: &[&str]) -> anyhow::Result<String> {
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned())
    } else {
        Err(anyhow::anyhow!(
            "Git command failed.\nCommand: git {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

pub fn get_git_root() -> anyhow::Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    let args = ["rev-parse", "--show-toplevel"];

    let root = spawn_git(&args, &current_dir)?;

    get_git_output(root, &args).map(|output| output.into())
}

pub static GIT_ROOT: Lazy<PathBuf> =
    Lazy::new(|| get_git_root().expect("Failed to determine Git root directory"));

type Git = Lazy<Box<dyn Fn(&[&str]) -> Result<String> + Send + Sync>>;

pub static GIT: Git = Lazy::new(|| {
    Box::new(move |args: &[&str]| -> Result<String> {
        get_git_output(spawn_git(args, &GIT_ROOT)?, args)
    })
});

/// Fetches a branch of a remote into local. Optionally accepts a commit hash for versioning.
pub fn add_remote_branch(
    info: &BranchAndRemote,
    commit_hash: &Option<String>,
) -> anyhow::Result<()> {
    match GIT(&[
        "remote",
        "add",
        &info.remote.local_remote_alias,
        &info.remote.repository_url,
    ]) {
        Ok(_) => {
            trace!(
                "Added remote {} for repository {}",
                &info.remote.repository_url,
                &info.remote.local_remote_alias
            );

            match GIT(&[
            "fetch",
            &info.remote.repository_url,
            &format!("{}:{}", info.branch.upstream_branch_name, info.branch.local_branch_name),
        ]) {
            Ok(_) => {
                trace!(
                    "Fetched branch {} as {} from repository {}",
                      info.branch.upstream_branch_name, info.branch.local_branch_name,&info.remote.repository_url
                );

                if let Some(commit_hash) = commit_hash {
                    GIT(&["branch", "--force", &info.branch.local_branch_name, commit_hash]).map_err(|err| {
                        anyhow!("We couldn't find commit {} of branch {}. Are you sure it exists?\n{err}", commit_hash, info.branch.local_branch_name)
                    })?;

                    trace!(
                        "...and did a hard reset to commit {commit_hash}",
                    );
                    
                };
                Ok(())
            },
            Err(err) => Err(anyhow!("We couldn't find branch {} of GitHub repository {}. Are you sure it exists?\n{err}", info.branch.upstream_branch_name, info.remote.repository_url)),
        }
        }
        Err(err) => {
            GIT(&["remote", "remove", &info.remote.local_remote_alias])?;
            Err(anyhow!("Could not fetch remote: {err}"))
        }
    }
}

pub fn checkout_from_remote(branch: &str, remote: &str) -> anyhow::Result<String> {
    let current_branch = match GIT(&["rev-parse", "--abbrev-ref", "HEAD"]) {
        Ok(current_branch) => current_branch,
        Err(err) => {
            return Err(anyhow!("Couldn't get the current branch. This usually happens when you have no commits.\n{err}"))
        }
    };

    match GIT(&["checkout", branch]) {
        Ok(_) => Ok(current_branch),
        Err(err) => {
            GIT(&["branch", "--delete", "--force", branch])?;
            GIT(&["remote", "remove", remote])?;
            Err(anyhow::anyhow!(
                "Could not checkout branch: {branch}, which belongs to remote {remote}\n{err}"
            ))
        }
    }
}

pub fn merge_into_main(
    local_branch: &str,
    remote_branch: &str,
) -> anyhow::Result<String, anyhow::Error> {
    match GIT(&["merge", local_branch, "--no-commit", "--no-ff"]) {
        Ok(_) => Ok(format!("Merged {remote_branch} successfully")),
        Err(_) => {
            let files_with_conflicts = GIT(&["diff", "--name-only", "--diff-filter=U"])?;
            for file_with_conflict in files_with_conflicts.lines() {
                if file_with_conflict.ends_with(".md") {
                    GIT(&["checkout", "--ours", file_with_conflict])?;
                    GIT(&["add", file_with_conflict])?;
                } else {
                    GIT(&["merge", "--abort"])?;
                    return Err(anyhow::anyhow!(
                        "Unresolved conflict in {file_with_conflict}"
                    ));
                }
            }
            Ok("Merged {remote_branch} successfully and disregarded conflicts".into())
        }
    }
}

pub async fn merge_pull_request(info: BranchAndRemote, pull_request: &str) -> anyhow::Result<()> {
    if let Err(err) = merge_into_main(
        &info.branch.local_branch_name,
        &info.branch.upstream_branch_name,
    ) {
        return Err(anyhow!("Could not merge branch {} into the current branch for pull request #{pull_request}, skipping\n{err}", &info.branch.local_branch_name));
    }

    let has_unstaged_changes = GIT(&["diff", "--cached", "--quiet"]).is_err();

    if has_unstaged_changes {
        GIT(&[
            "commit",
            "--message",
            &format!(
                "{APP_NAME}: Merge branch {} of {}",
                &info.branch.upstream_branch_name, &info.remote.repository_url
            ),
        ])?;
    }

    GIT(&["remote", "remove", &info.remote.local_remote_alias])?;
    GIT(&[
        "branch",
        "--delete",
        "--force",
        &info.branch.local_branch_name,
    ])?;

    Ok(())
}

pub async fn fetch_pull_request(
    repo: &str,
    pull_request: &str,
    client: &Client,
    custom_branch_name: Option<&str>,
    commit_hash: &Option<String>,
) -> anyhow::Result<(GitHubResponse, BranchAndRemote)> {
    let url = format!("https://api.github.com/repos/{}/pulls/{pull_request}", repo);

    let response = match make_request(client, &url).await {
        Ok(res) => res,
        Err(res) => {
            return Err(anyhow!(
                "Could not fetch pull request #{pull_request}\n{res}\n"
            ))
        }
    };

    let info = BranchAndRemote {
        branch: Branch {
            upstream_branch_name: response.head.r#ref.clone(),
            local_branch_name: custom_branch_name
                .map(|s| s.into())
                .unwrap_or(with_uuid(&format!(
                    "{title}-{}",
                    pull_request,
                    title = normalize_commit_msg(&response.title)
                ))),
        },
        remote: Remote {
            repository_url: response.head.repo.clone_url.clone(),
            local_remote_alias: with_uuid(&format!(
                "{title}-{}",
                pull_request,
                title = normalize_commit_msg(&response.html_url)
            )),
        },
    };

    add_remote_branch(&info, commit_hash).context(format!(
        "Could not add remote branch for pull request #{pull_request}, skipping."
    ))?;

    Ok((response, info))
}
