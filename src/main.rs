use colored::Colorize;
use patchy::commands::{gen_patch, help, init, pr_fetch, run};
use patchy::fail;
use std::{env, path::Path};

use anyhow::Result;
use patchy::git_commands::{get_git_output, get_git_root, spawn_git};
use patchy::types::CommandArgs;
use patchy::INDENT;

async fn process_subcommand(
    subcommand: &str,
    args: CommandArgs,
    root: &Path,
    git: &impl Fn(&[&str]) -> anyhow::Result<String>,
) -> Result<()> {
    match subcommand {
        // main commands
        "init" => init(&args, root)?,
        "run" => run(&args, root, &git).await?,
        "gen-patch" => gen_patch(&args, root, git)?,
        // lower level commands
        "pr-fetch" => pr_fetch(&args, &git).await?,
        unrecognized => {
            if !unrecognized.is_empty() {
                anyhow::bail!(
                    "{}",
                    format!(
                        "  Unknown {unknown}: {}",
                        unrecognized,
                        unknown = if unrecognized.starts_with("-") {
                            "flag".red()
                        } else {
                            "command".red()
                        }
                    )
                    .red()
                )
            }

            help(&args, None)?;
        }
    }

    Ok(())
}

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
        // We're not using any command, only flags
        args.insert(subcommand.clone());
    }

    if args.contains("-h") || args.contains("--help") {
        help(&args, Some(&subcommand))
    } else if args.contains("-v") || args.contains("--version") {
        print!("{}", env!("CARGO_PKG_VERSION"));

        Ok(())
    } else {
        match process_subcommand(subcommand.as_str(), args, &root, &git).await {
            Ok(()) => Ok(()),
            Err(msg) => {
                fail!("{msg}");
                std::process::exit(1);
            }
        }
    }
}
