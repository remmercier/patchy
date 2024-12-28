use colored::Colorize;

use crate::{
    commands::{
        gen_patch::GEN_PATCH_NAME_FLAG,
        pr_fetch::{PR_FETCH_BRANCH_NAME_FLAG, PR_FETCH_CHECKOUT_FLAG, PR_FETCH_REPO_NAME_FLAG},
    },
    flags::{format_flag, Flag},
    types::CommandArgs,
    APP_NAME,
};

fn subcommand(command: &str, description: &str) -> String {
    let command = command.yellow();
    format!("{command}\n    {}", make_description(description))
}

pub fn make_description(description: &str) -> String {
    format!("{} {description}", "»".black())
}

static HELP_FLAG: Flag<'static> = Flag {
    short: "-h",
    long: "--help",
    description: "Print this message",
};

static VERSION_FLAG: Flag<'static> = Flag {
    short: "-v",
    long: "--version",
    description: "Get patchy version",
};

pub fn help(_args: &CommandArgs, command: Option<&str>) -> anyhow::Result<()> {
    let author = "Nikita Revenco ".italic();
    let less_than = "<".black().italic();
    let email = "pm@nikitarevenco.com".italic();
    let greater_than = ">".black().italic();
    let app_name = APP_NAME.blue();
    let flags_label = "[<flags>]".magenta();
    let command_str = "<command>".yellow();
    let args = "[<args>]".green();
    let version = env!("CARGO_PKG_VERSION");
    let init = subcommand("init", "Create example config file");
    let pr_fetch = subcommand(
        "pr-fetch",
        "Fetch pull request for a GitHub repository as a local branch",
    );
    let gen_patch = subcommand("gen-patch", "Generate a .patch file from commit hashes");
    let run = subcommand("run", &format!("Start {APP_NAME}"));
    let header = format!(
        "  {app_name} {version}
  {author}{less_than}{email}{greater_than}"
    );
    let help_flag = format_flag(&HELP_FLAG);
    let version_flag = format_flag(&VERSION_FLAG);

    match command {
        Some(cmd_name) if cmd_name == "init" => {
            let this_command_name = format!("{app_name} {}", cmd_name.yellow());

            let description = make_description("Create example config file");

            println!(
                "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Flags:

    {help_flag}
",
            );
        }
        Some(cmd_name) if cmd_name == "run" => {
            let this_command_name = format!("{app_name} {}", cmd_name.yellow());

            let description = make_description("Create example config file");

            println!(
                "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Flags:

    {help_flag}
",
            );
        }
        Some(cmd_name) if cmd_name == "gen-patch" => {
            let this_command_name = format!("{app_name} {}", cmd_name.yellow());

            let patch_filename_flag = format_flag(&GEN_PATCH_NAME_FLAG);

            let description = make_description("Generate a .patch file from commit hashes");

            let example_1 = format!(
                "{}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4".green(),
                make_description("Generate a single .patch file from one commit hash")
            );

            let example_2 = format!(
                "{}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4 9ad5aa637ccf363b5d6713f66d0c2830736c35a9 cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4".green(),
                make_description("Generate several .patch files from several commit hashes")
            );

            let example_3 = format!(
                "{} {} {} {} {}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4".green(),
                "--patch-filename=some-patch".magenta(),
                "9ad5aa637ccf363b5d6713f66d0c2830736c35a9".green(),
                "--patch-filename=another-patch".magenta(),
                "cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4".green(),
                make_description(
                    "Generate several .patch files from several commit hashes and give 2 of them custom names"
                )
            );

            println!(
                "
{header}
        
  Usage:

    {this_command_name}
    {description}

  Examples:

    {this_command_name} {example_1}

    {this_command_name} {example_2}

    {this_command_name} {example_3}

  Flags:

    {patch_filename_flag}

    {help_flag}
",
            );
        }
        Some(cmd_name) if cmd_name == "pr-fetch" => {
            let description = make_description("Fetch pull requests into a local branch");

            let example_1 = format!(
                "{}
    {}",
                "11745".green(),
                make_description("Fetch a single pull request")
            );

            let example_2 = format!(
                "{}
    {}",
                "11745 10000 9191 600".green(),
                make_description("Fetch several pull requests")
            );

            let example_3 = format!(
                "{} {} {} {} {}
    {}",
                "11745 10000".green(),
                "--branch-name=some-pr".magenta(),
                "9191".green(),
                "--branch-name=another-pr".magenta(),
                "600".green(),
                make_description(
                    "Fetch several pull requests and choose custom branch names for the pull requests #10000 and #9191"
                )
            );

            let example_4 = format!(
                "{} {} {}
    {}",
                "--repo-name=helix-editor/helix".magenta(),
                "11745 10000 9191 600".green(),
                "--checkout".magenta(),
                make_description("Fetch several pull requests, checkout the first one and use a custom github repo: https://github.com/helix-editor/helix")
            );

            let branch_name_flag = format_flag(&PR_FETCH_BRANCH_NAME_FLAG);
            let checkout_flag = format_flag(&PR_FETCH_CHECKOUT_FLAG);
            let repo_name_flag = format_flag(&PR_FETCH_REPO_NAME_FLAG);
            let this_command_name = format!("{app_name} {}", cmd_name.yellow());

            println!(
                "
{header}
        
  Usage:

    {this_command_name} {args} {flags_label}
    {description}

  Examples:

    {this_command_name} {example_1}

    {this_command_name} {example_2}

    {this_command_name} {example_3}

    {this_command_name} {example_4}

  Flags:

    {branch_name_flag}

    {checkout_flag}

    {repo_name_flag}

    {help_flag}
",
            );
        }
        _ => {
            println!(
                "
{header}
        
  Usage:

    {app_name} {command_str} {args} {flags_label}

  Commands:

    {init} 

    {run}

    {gen_patch} 

    {pr_fetch} 

  Flags:

    {help_flag}

    {version_flag}
"
            );
        }
    }

    Ok(())
}
