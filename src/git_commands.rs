use std::{
    path::{Path, PathBuf},
    process::Output,
};

use anyhow::Context;
use reqwest::Client;

use crate::{
    types::{BranchAndRemote, GitHubResponse},
    utils::{make_request, normalize_pr_title, with_uuid},
    APP_NAME,
};

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

pub fn spawn_git(args: &[&str], git_dir: &Path) -> Result<Output, std::io::Error> {
    std::process::Command::new("git")
        .args(args)
        .current_dir(git_dir)
        .output()
}

pub fn get_git_root() -> anyhow::Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    let args = ["rev-parse", "--show-toplevel"];

    let root = spawn_git(&args, &current_dir)?;

    get_git_output(root, &args).map(|output| output.into())
}

pub fn git(args: &[&str]) -> anyhow::Result<String> {
    let root = get_git_root()?;
    get_git_output(spawn_git(args, &root)?, args)
}

pub fn add_remote_branch(
    local_remote: &str,
    local_branch: &str,
    remote_remote: &str,
    remote_branch: &str,
) -> anyhow::Result<()> {
    match git(&["remote", "add", local_remote, remote_remote]) {
        Ok(_) => match git(&[
            "fetch",
            remote_remote,
            &format!("{remote_branch}:{local_branch}"),
        ]) {
            Ok(_) => Ok(()),
            Err(err) => {
                git(&["branch", "-D", local_branch])?;
                Err(anyhow::anyhow!("Could not fetch branch from remote: {err}"))
            }
        },
        Err(err) => {
            git(&["remote", "remove", local_remote])?;
            Err(anyhow::anyhow!("Could not add remote: {err}"))
        }
    }
}

pub fn checkout_from_remote(branch: &str, remote: &str) -> anyhow::Result<String> {
    let current_branch = git(&["rev-parse", "--abbrev-ref", "HEAD"])?;

    match git(&["checkout", branch]) {
        Ok(_) => Ok(current_branch),
        Err(err) => {
            git(&["branch", "-D", branch])?;
            git(&["remote", "remove", remote])?;
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
    match git(&["merge", local_branch, "--no-commit", "--no-ff"]) {
        Ok(_) => Ok(format!("Merged {remote_branch} successfully")),
        Err(_) => {
            let files_with_conflicts = git(&["diff", "--name-only", "--diff-filter=U"])?;
            for file_with_conflict in files_with_conflicts.lines() {
                if file_with_conflict.ends_with(".md") {
                    git(&["checkout", "--ours", file_with_conflict])?;
                    git(&["add", file_with_conflict])?;
                } else {
                    git(&["merge", "--abort"])?;
                    return Err(anyhow::anyhow!(
                        "Unresolved conflict in {file_with_conflict}"
                    ));
                }
            }
            Ok("Merged {remote_branch} successfully and disregarded conflicts".into())
        }
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
    custom_branch_name: Option<&str>,
) -> anyhow::Result<(GitHubResponse, BranchAndRemote)> {
    let url = format!("https://api.github.com/repos/{}/pulls/{pull_request}", repo);

    let response = make_request(client, &url).await.context(format!(
        "Couldn't fetch required data from remote for pull request #{pull_request}, skipping.
Url fetched: {url}"
    ))?;

    let remote_remote = &response.head.repo.clone_url;

    let local_remote = with_uuid(&format!(
        "{title}-{}",
        pull_request,
        title = normalize_pr_title(&response.html_url)
    ));

    let remote_branch = &response.head.r#ref;

    let local_branch = custom_branch_name
        .map(|s| s.into())
        .unwrap_or(with_uuid(&format!(
            "{title}-{}",
            pull_request,
            title = normalize_pr_title(&response.title)
        )));

    add_remote_branch(&local_remote, &local_branch, remote_remote, remote_branch).context(
        format!("Could not add remove branch for pull request #{pull_request}, skipping."),
    )?;

    let info = BranchAndRemote::new(&local_branch, remote_branch, &local_remote, remote_remote);

    Ok((response, info))
}
