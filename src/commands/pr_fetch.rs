use crate::fail;
use crate::git_commands::fetch_pull_request;
use crate::success;
use crate::utils::display_link;
use crate::CommandArgs;
use crate::INDENT;
use colored::Colorize;

fn is_valid_branch_name(branch_name: &str) -> bool {
    branch_name
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '.' || ch == '-' || ch == '/' || ch == '_')
}

pub async fn pr_fetch(
    args: &CommandArgs,
    _git: impl Fn(&[&str]) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let mut args = args.iter().peekable();

    let mut pull_requests_with_maybe_custom_branch_names = vec![];

    while let Some(arg) = args.next() {
        let next = args.peek();
        let maybe_custom_branch_name: Option<&str> = next.and_then(|next_arg| {
            let range = if next_arg.starts_with("-b=") {
                3..
            } else if next_arg.starts_with("--branch-name=") {
                14..
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

    let client = reqwest::Client::new();

    for (pull_request, maybe_custom_branch_name) in pull_requests_with_maybe_custom_branch_names {
        match fetch_pull_request(repo, pull_request, &client, maybe_custom_branch_name).await {
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
