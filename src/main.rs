use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;

fn git<I, S>(args: I) -> Result<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let current_dir = std::env::current_dir().unwrap();
    let output = std::process::Command::new("git")
        .args(args)
        .current_dir(current_dir)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_owned())
    } else {
        Err(anyhow!(
            "Git command failed.\nStdout: {}\nStderr: {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

#[derive(Deserialize)]
struct Configuration {
    repo: String,
    local_branch: String,
    pull_requests: Vec<String>,
}

fn main() -> Result<()> {
    if git(["rev-parse", "--is-inside-work-tree"]).is_err() {
        bail!("Not in a git repository");
    }

    let config_raw = std::env::current_dir()
        .map(|z| z.join(".gitpatcher.toml"))
        .and_then(std::fs::read_to_string)
        .context("Could not find `.gitpatcher.toml` configuration file")?;

    let config = toml::from_str::<Configuration>(&config_raw)?;

    Ok(())
}
