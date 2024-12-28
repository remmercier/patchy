use crate::git_commands::fetch_pull_request;
use crate::success;
use crate::types::CommandArgs;
use crate::utils::display_link;
use crate::INDENT;
use crate::{extract_value_from_flag, fail};
use anyhow::anyhow;
use colored::Colorize;

fn is_valid_branch_name(branch_name: &str) -> bool {
    branch_name
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '.' || ch == '-' || ch == '/' || ch == '_')
}

pub struct Flag<'a> {
    pub short: &'a str,
    pub long: &'a str,
}

static PR_FETCH_BRANCH_NAME_FLAG: Flag<'static> = Flag {
    short: "-b=",
    long: "--branch-name=",
};

static PR_FETCH_REMOTE_NAME_FLAG: Flag<'static> = Flag {
    short: "-r=",
    long: "--remote-name=",
};

static GITHUB_REMOTE_PREFIX: &str = "git@github.com:";
static GITHUB_REMOTE_SUFFIX: &str = ".git";

pub async fn pr_fetch(
    args: &CommandArgs,
    git: impl Fn(&[&str]) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let mut args = args.iter().peekable();

    let mut pull_requests_with_maybe_custom_branch_names = vec![];

    let mut remote_name: Option<String> = None;

    while let Some(arg) = args.next() {
        if let Some(flag) = extract_value_from_flag(arg, &PR_FETCH_REMOTE_NAME_FLAG) {
            remote_name = Some(flag);
        }
        // if arg.starts_with(PR_FETCH_REMOTE_NAME_FLAG.short) {
        //     remote_name = arg
        //         .get(PR_FETCH_REMOTE_NAME_FLAG.short.len()..)
        //         .map(|m| m.into())
        // } else if arg.starts_with(PR_FETCH_REMOTE_NAME_FLAG.long) {
        //     remote_name = arg
        //         .get(PR_FETCH_REMOTE_NAME_FLAG.long.len()..)
        //         .map(|m| m.into())
        // }

        let next_arg = args.peek();
        let maybe_custom_branch_name: Option<&str> = next_arg.and_then(|next_arg| {
            let range = if next_arg.starts_with(PR_FETCH_BRANCH_NAME_FLAG.short) {
                PR_FETCH_BRANCH_NAME_FLAG.short.len()..
            } else if next_arg.starts_with(PR_FETCH_BRANCH_NAME_FLAG.long) {
                PR_FETCH_BRANCH_NAME_FLAG.long.len()..
            } else {
                return None;
            };

            next_arg
                .get(range)
                .filter(|branch_name| is_valid_branch_name(branch_name))
        });

        if maybe_custom_branch_name.is_some() {
            args.next();
        };

        pull_requests_with_maybe_custom_branch_names.push((arg, maybe_custom_branch_name));
    }

    // The user hasn't provided a custom remote, so we're going to try `origin`
    if remote_name.is_none() {
        let remote = git(&["remote", "get-url", "origin"])?;
        if remote.starts_with(GITHUB_REMOTE_PREFIX) && remote.ends_with(GITHUB_REMOTE_SUFFIX) {
            let start = GITHUB_REMOTE_PREFIX.len();
            let end = remote.len() - GITHUB_REMOTE_SUFFIX.len();
            remote_name = remote.get(start..end).map(|m| m.into());
        };
    }

    let Some(remote_name) = remote_name else {
        return Err(anyhow!(
            "Could not get the remote, it should be in the form helix-editor/helix.",
        ));
    };

    let client = reqwest::Client::new();

    for (pull_request, maybe_custom_branch_name) in pull_requests_with_maybe_custom_branch_names {
        match fetch_pull_request(
            &remote_name,
            pull_request,
            &client,
            maybe_custom_branch_name,
        )
        .await
        {
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
                fail!("{err:#?}");
                continue;
            }
        };
    }

    Ok(())
}
