use std::{
    fs::{self, File},
    io::Write,
    path,
};

use colored::Colorize;

use crate::{success, types::CommandArgs, CONFIG_FILE, CONFIG_ROOT, INDENT};

pub fn init(_args: &CommandArgs, root: &path::Path) -> anyhow::Result<()> {
    let example_config = include_bytes!("../../example-config.toml");

    let config_path = root.join(CONFIG_ROOT);

    let config_file_path = config_path.join(CONFIG_FILE);

    if config_file_path.exists() {
        let confirmation = dialoguer::Confirm::new()
            .with_prompt(format!(
                "\n{INDENT}{} File {config_file_path} already exists. Overwrite it?",
                "»".black(),
                config_file_path = config_file_path.to_string_lossy().blue()
            ))
            .interact()
            .unwrap();
        if !confirmation {
            anyhow::bail!("Did not overwrite {config_file_path:?}");
        }
    }

    let _ = fs::create_dir(config_path);

    let mut file = File::create(&config_file_path)?;

    file.write_all(example_config)?;

    success!("Created config file {config_file_path:?}");

    Ok(())
}
