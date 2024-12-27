use crate::git_commands::fetch_pull_request;
use crate::success;
use crate::utils::display_link;
use crate::CommandArgs;
use crate::INDENT;
use colored::Colorize;

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
