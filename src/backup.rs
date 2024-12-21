use std::fs::read_to_string;
use std::io::Write;
use std::{
    ffi::OsString,
    fs::{File, ReadDir},
    path::PathBuf,
};
use tempfile::tempfile;

use crate::CONFIG_ROOT;

pub fn backup_files(config_files: ReadDir) -> Vec<(OsString, File, String)> {
    config_files
        .filter_map(|config_file| {
            if let Ok(config_file) = config_file {
                let mut destination_backed_up = tempfile().unwrap();
                let contents = read_to_string(config_file.path()).unwrap();
                let filename = config_file.file_name();
                if write!(destination_backed_up, "{contents}").is_ok() {
                    Some((filename, destination_backed_up, contents))
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

pub fn restore_backup(file_name: &OsString, contents: &str) -> anyhow::Result<()> {
    let path = PathBuf::from(CONFIG_ROOT).join(file_name);
    let mut file = File::create(&path)?;

    write!(file, "{contents}")?;

    Ok(())
}
