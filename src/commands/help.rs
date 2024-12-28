use colored::Colorize;

use crate::{types::CommandArgs, APP_NAME};

fn subcommand(command: &str, description: &str) -> String {
    let command = command.yellow();
    format!("{command}\n    {}", make_description(description))
}

fn make_description(description: &str) -> String {
    format!("{} {description}", "»".black())
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
    format!("{flags}\n    {}", make_description(description))
}

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
    let help_flag = flags(&["-h", "--help"], "Print this message");
    let version_flag = flags(&["-v", "--version"], "Get package version");

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
                make_description("Generate a several .patch files from one commit hashes")
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

  Flags:

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
            let branch_name_flag = flags(
                &["-b=<name>", "--branch-name=<name>"],
                "Choose local name for the branch belong to the preceding pull request",
            );
            let checkout_flag = flags(
                &["-c", "--checkout"],
                "Automatically check out the branch belonging to the first pull request",
            );
            let remote_name_flag = flags(
                &["-r=<name>", "--remote-name=<name>"],
                "Choose a github repository, using the `origin` remote of the current repository by default",
            );
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

  Flags:

    {branch_name_flag}

    {checkout_flag}

    {remote_name_flag}

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
