use std::fs::read_to_string;
use std::io::Write;
use std::path::Path;
use std::{
    ffi::OsString,
    fs::{File, ReadDir},
    path::PathBuf,
};
use tempfile::tempfile;

use crate::CONFIG_ROOT;

pub fn backup_files(config_files: ReadDir) -> anyhow::Result<Vec<(OsString, File, String)>> {
    let mut backups = Vec::new();

    for entry in config_files {
        let config_file = entry?;

        let path = config_file.path();
        let contents = read_to_string(&path)?;

        let filename = config_file.file_name();
        let mut destination_backed_up = tempfile()?;

        write!(destination_backed_up, "{contents}")?;

        backups.push((filename, destination_backed_up, contents));
    }

    Ok(backups)
}
pub fn restore_backup(file_name: &OsString, contents: &str, root: &Path) -> anyhow::Result<()> {
    let path = root.join(PathBuf::from(CONFIG_ROOT).join(file_name));
    let mut file = File::create(&path)?;

    write!(file, "{contents}")?;

    Ok(())
}
