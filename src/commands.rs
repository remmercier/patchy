pub fn git(args: &[&str]) -> Result<String> {
    let current_dir = std::env::current_dir()?;

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
            "Git command failed.\nCommand: git {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}
