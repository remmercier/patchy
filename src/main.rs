use colored::Colorize;
use patchy::commands::help::{HELP_FLAG, VERSION_FLAG};
use patchy::commands::{gen_patch, help, init, pr_fetch, run};
use patchy::fail;
use patchy::flags::contains_flag;
use std::env;

use patchy::types::CommandArgs;
use patchy::INDENT;

use anyhow::Result;

async fn process_subcommand(subcommand: &str, args: CommandArgs) -> Result<()> {
    match subcommand {
        // main commands
        "init" => init(&args)?,
        "run" => run(&args).await?,
        "gen-patch" => gen_patch(&args)?,
        // lower level commands
        "pr-fetch" => pr_fetch(&args).await?,
        unrecognized => {
            if !unrecognized.is_empty() {
                fail!(
                    "{}",
                    format!(
                        "  Unknown {unknown}: {}",
                        unrecognized,
                        unknown = if unrecognized.starts_with("-") {
                            "flag".bright_red()
                        } else {
                            "command".bright_red()
                        }
                    )
                    .bright_red()
                )
            }

            help(None)?;
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let _command_name = args.next();
    let subcommand = args.next().unwrap_or_default();

    let mut args: CommandArgs = args.collect();

    if subcommand.starts_with("-") {
        // We're not using any command, only flags
        args.insert(subcommand.clone());
    }

    if contains_flag(&args, &HELP_FLAG) {
        help(Some(&subcommand))
    } else if contains_flag(&args, &VERSION_FLAG) {
        print!("{}", env!("CARGO_PKG_VERSION"));

        Ok(())
    } else {
        match process_subcommand(subcommand.as_str(), args).await {
            Ok(()) => Ok(()),
            Err(msg) => {
                fail!("{msg}");
                std::process::exit(1);
            }
        }
    }
}
