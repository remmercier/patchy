use colored::Colorize;

use crate::{
    commands::{
        gen_patch::GEN_PATCH_NAME_FLAG,
        pr_fetch::{PR_FETCH_BRANCH_NAME_FLAG, PR_FETCH_CHECKOUT_FLAG, PR_FETCH_REPO_NAME_FLAG},
    },
    flags::{format_flag, Flag},
    APP_NAME,
};

fn format_subcommand(command: &str, description: &str) -> String {
    let command = command.yellow();
    format!("{command}\n    {}", format_description(description))
}

pub fn format_description(description: &str) -> String {
    format!("{} {description}", "»".black())
}

pub static HELP_FLAG: Flag<'static> = Flag {
    short: "-h",
    long: "--help",
    description: "Print this message",
};

pub static VERBOSE_FLAG: Flag<'static> = Flag {
    short: "-V",
    long: "--verbose",
    description: "Increased logging information",
};

pub static VERSION_FLAG: Flag<'static> = Flag {
    short: "-v",
    long: "--version",
    description: "Get patchy version",
};

pub fn help(command: Option<&str>) -> anyhow::Result<()> {
    let author = "Nikita Revenco ".italic();
    let less_than = "<".black().italic();
    let email = "pm@nikitarevenco.com".italic();
    let greater_than = ">".black().italic();
    let app_name = APP_NAME.blue();
    let flags_label = "[<flags>]".magenta();
    let command_str = "<command>".yellow();
    let args = "[<args>]".green();
    let version = env!("CARGO_PKG_VERSION");
    let init = format_subcommand("init", "Create example config file");
    let pr_fetch = format_subcommand(
        "pr-fetch",
        "Fetch pull request for a GitHub repository as a local branch",
    );
    let gen_patch = format_subcommand("gen-patch", "Generate a .patch file from commit hashes");
    let run = format_subcommand("run", &format!("Start {APP_NAME}"));
    let header = format!(
        "  {app_name} {version}
  {author}{less_than}{email}{greater_than}"
    );
    let help_flag = format_flag(&HELP_FLAG);
    let version_flag = format_flag(&VERSION_FLAG);

    match command {
        Some(cmd_name @ "init") => {
            let this_command_name = format!("{app_name} {}", cmd_name.yellow());

            let description = format_description("Create example config file");

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
        Some(cmd_name @ "run") => {
            let this_command_name = format!("{app_name} {}", cmd_name.yellow());

            let description = format_description("Create example config file");

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
        Some(cmd_name @ "gen-patch") => {
            let this_command_name = format!("{app_name} {}", cmd_name.yellow());

            let patch_filename_flag = format_flag(&GEN_PATCH_NAME_FLAG);

            let description = format_description("Generate a .patch file from commit hashes");

            let example_1 = format!(
                "{}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4".green(),
                format_description("Generate a single .patch file from one commit hash")
            );

            let example_2 = format!(
                "{}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4 9ad5aa637ccf363b5d6713f66d0c2830736c35a9 cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4".green(),
                format_description("Generate several .patch files from several commit hashes")
            );

            let example_3 = format!(
                "{} {} {} {} {}
    {}",
                "133cbaae83f710b793c98018cea697a04479bbe4".green(),
                "--patch-filename=some-patch".magenta(),
                "9ad5aa637ccf363b5d6713f66d0c2830736c35a9".green(),
                "--patch-filename=another-patch".magenta(),
                "cc75a895f344cf2fe83eaf6d78dfb7aeac8b33a4".green(),
                format_description(
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
        Some(cmd_name @ "pr-fetch") => {
            let description = format_description("Fetch pull requests into a local branch");

            let example_1 = format!(
                "{}
    {}",
                "11745".green(),
                format_description("Fetch a single pull request")
            );

            let example_2 = format!(
                "{}
    {}",
                "11745 10000 9191 600".green(),
                format_description("Fetch several pull requests")
            );

            let example_3 = format!(
                "{} {} {} {} {}
    {}",
                "11745 10000".green(),
                "--branch-name=some-pr".magenta(),
                "9191".green(),
                "--branch-name=another-pr".magenta(),
                "600".green(),
                format_description(
                    "Fetch several pull requests and choose custom branch names for the pull requests #10000 and #9191"
                )
            );

            let example_4 = format!(
                "{} {} {}
    {}",
                "--repo-name=helix-editor/helix".magenta(),
                "11745 10000 9191 600".green(),
                "--checkout".magenta(),
                format_description("Fetch several pull requests, checkout the first one and use a custom github repo: https://github.com/helix-editor/helix")
            );

            let example_5 = format!(
                "{}
    {}",
                "11745 10000@be8f264327f6ae729a0b372ef01f6fde49a78310 9191 600@5d10fa5beb917a0dbe0ef8441d14b3d0dd15227b".green(),
                format_description("Fetch several pull requests at a certain commit")
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

    {this_command_name} {example_5}

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
