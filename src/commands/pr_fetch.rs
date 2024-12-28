use crate::fail;
use crate::flags::{extract_value_from_flag, Flag};
use crate::git_commands::{
    fetch_pull_request, is_valid_branch_name, GITHUB_REMOTE_PREFIX, GITHUB_REMOTE_SUFFIX,
};
use crate::success;
use crate::types::CommandArgs;
use crate::utils::display_link;
use crate::INDENT;
use anyhow::anyhow;
use colored::Colorize;

pub static PR_FETCH_BRANCH_NAME_FLAG: Flag<'static> = Flag {
    short: "-b=",
    long: "--branch-name=",
    description: "Choose local name for the branch belonging to the preceding pull request",
};

pub static PR_FETCH_CHECKOUT_FLAG: Flag<'static> = Flag {
    short: "-c",
    long: "--checkout",
    description: "Check out the branch belonging to the first pull request",
};

pub static PR_FETCH_REPO_NAME_FLAG: Flag<'static> = Flag {
    short: "-r=",
    long: "--repo-name=",
    description:
        "Choose a github repository, using the `origin` remote of the current repository by default",
};

pub async fn pr_fetch(
    args: &CommandArgs,
    git: impl Fn(&[&str]) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let checkout_flag =
        args.contains(PR_FETCH_CHECKOUT_FLAG.short) || args.contains(PR_FETCH_CHECKOUT_FLAG.long);

    let mut args = args.iter().peekable();

    let mut pull_requests_with_maybe_custom_branch_names = vec![];

    let mut remote_name: Option<String> = None;

    while let Some(arg) = args.next() {
        if let Some(flag) = extract_value_from_flag(arg, &PR_FETCH_REPO_NAME_FLAG) {
            remote_name = Some(flag);
        }

        let next_arg = args.peek();
        let maybe_custom_branch_name: Option<String> = next_arg.and_then(|next_arg| {
            extract_value_from_flag(next_arg, &PR_FETCH_BRANCH_NAME_FLAG)
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
            remote_name = remote.get(start..end).map(|remote| remote.into());
        };
    }

    let Some(remote_name) = remote_name else {
        return Err(anyhow!(
            "Could not get the remote, it should be in the form e.g. helix-editor/helix.",
        ));
    };

    let client = reqwest::Client::new();

    for (i, (pull_request, maybe_custom_branch_name)) in
        pull_requests_with_maybe_custom_branch_names
            .iter()
            .enumerate()
    {
        match fetch_pull_request(
            &remote_name,
            pull_request,
            &client,
            maybe_custom_branch_name.as_deref(),
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
                );

                if i == 0 && checkout_flag {
                    if git(&["checkout", &info.branch.local_name]).is_ok() {
                        success!(
                            "Automatically checked out the first branch: {}",
                            info.branch.local_name
                        )
                    } else {
                        fail!("Could not check out branch {}", info.branch.local_name)
                    }
                }
            }
            Err(err) => {
                fail!("{err:#?}");
                continue;
            }
        };
    }

    Ok(())
}
