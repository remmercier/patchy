use colored::Colorize;

use crate::{types::CommandArgs, APP_NAME};

pub fn help(_args: &CommandArgs, command: Option<&str>) -> anyhow::Result<()> {
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
    let help_flag = flags(&["-h", "--help"], "print this message");
    let version_flag = flags(&["-v", "--version"], "get package version");

    match command {
        Some("init") => (),
        Some("run") => (),
        Some("gen-patch") => (),
        Some("pr-fetch") => {
            let branch_name_flag = flags(
                &["-b", "--branch-name"],
                "choose name for the branch of the first fetched pull request",
            );
            let checkout_flag = flags(
                &["-c", "--checkout"],
                "check out the first fetched pull request",
            );
            let remote_name_flag = flags(
                &["-r", "--remote-name"],
                "choose a remote, by default it uses remote of the current repository",
            );
            let this_command_name = format!("{app_name} {}", "pr-fetch".yellow());

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

            println!(
                "
{header}
        
  Usage:

    {this_command_name} {args} {flags_label}

  Examples:

    {this_command_name} {example_1}

    {this_command_name} {example_2}

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
