mod backup;
mod commands;
mod git_commands;
mod types;
mod utils;

use colored::Colorize;
use commands::{gen_patch, help, init, pr_fetch, run};
use std::env;

use anyhow::Result;
use git_commands::{get_git_output, get_git_root, spawn_git};
use types::CommandArgs;

static CONFIG_ROOT: &str = ".patchy";
static CONFIG_FILE: &str = "config.toml";
static APP_NAME: &str = env!("CARGO_PKG_NAME");
static INDENT: &str = "  ";

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let _command_name = args.next();
    let subcommand = args.next().unwrap_or_default();

    let root = get_git_root()?;

    let git =
        |args: &[&str]| -> anyhow::Result<String> { get_git_output(spawn_git(args, &root)?, args) };

    let mut args: CommandArgs = args.collect();

    if subcommand.starts_with("-") {
        args.insert(subcommand.clone());
    }

    if args.contains("-h") || args.contains("--help") {
        help(&args)
    } else if args.contains("-v") || args.contains("--version") {
        print!("{}", env!("CARGO_PKG_VERSION"));

        Ok(())
    } else {
        match subcommand.as_str() {
            // main commands
            "init" => init(&args)?,
            "run" => run(&args, &root, &git).await?,
            "gen-patch" => gen_patch(&args)?,
            // lower level commands
            "pr-fetch" => pr_fetch(&args)?,
            unrecognized => {
                if !unrecognized.is_empty() {
                    let message = format!(
                        "  Unknown {unknown}: {}",
                        unrecognized,
                        unknown = if unrecognized.starts_with("-") {
                            "flag".red()
                        } else {
                            "command".red()
                        }
                    )
                    .red();

                    eprintln!("{message}");
                }

                help(&args)?;
                std::process::exit(1)
            }
        }

        Ok(())
    }
}
