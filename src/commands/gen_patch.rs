use std::{
    fs::{self, File},
    io::Write,
    path,
};

use crate::{
    fail,
    flags::{extract_value_from_flag, Flag},
    git_commands::is_valid_branch_name,
    success,
    types::CommandArgs,
    utils::normalize_commit_msg,
};
use crate::{CONFIG_ROOT, INDENT};
use colored::Colorize;

static GEN_PATCH_NAME: Flag<'static> = Flag {
    short: "-n=",
    long: "--patch-name=",
};

pub fn gen_patch(
    args: &CommandArgs,
    root: &path::Path,
    git: impl Fn(&[&str]) -> anyhow::Result<String>,
) -> anyhow::Result<()> {
    let mut args = args.iter().peekable();
    let mut commit_hashes_with_maybe_custom_patch_filenames = vec![];

    let config_path = root.join(CONFIG_ROOT);

    while let Some(arg) = args.next() {
        let next_arg = args.peek();
        let maybe_custom_patch_filename: Option<String> = next_arg.and_then(|next_arg| {
            extract_value_from_flag(next_arg, &GEN_PATCH_NAME)
                .filter(|branch_name| is_valid_branch_name(branch_name))
        });

        if maybe_custom_patch_filename.is_some() {
            args.next();
        };

        commit_hashes_with_maybe_custom_patch_filenames.push((arg, maybe_custom_patch_filename));
    }

    if !config_path.exists() {
        success!(
            "Config directory {} does not exist, creating it...",
            config_path.to_string_lossy()
        );
        fs::create_dir(&config_path)?;
    }

    for (patch_commit_hash, maybe_custom_patch_name) in
        commit_hashes_with_maybe_custom_patch_filenames
    {
        let Ok(patch_contents) = git(&[
            "diff",
            &format!("{}^", patch_commit_hash),
            patch_commit_hash,
        ]) else {
            fail!("Could not get patch output for patch {}", patch_commit_hash);
            continue;
        };

        // 1. if the user provides a custom filename for the patch file, use that
        // 2. otherwise use the commit message
        // 3. if all fails use the commit hash
        let patch_filename = maybe_custom_patch_name.unwrap_or({
            git(&["log", "--format=%B", "-n", "1", patch_commit_hash])
                .map(|commit_msg| normalize_commit_msg(&commit_msg))
                .unwrap_or(patch_commit_hash.to_string())
        });

        let patch_file_path = config_path.join(&patch_filename);

        let mut file = File::create(&patch_file_path)?;

        file.write_all(patch_contents.as_bytes())?;

        success!("Created patch file at {}", patch_filename)
    }

    Ok(())
}
