pub fn git(args: &[&str]) -> anyhow::Result<String> {
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
        Err(anyhow::anyhow!(
            "Git command failed.\nCommand: git {}\nStdout: {}\nStderr: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        ))
    }
}

pub fn add_remote_branch(
    local_remote: &str,
    local_branch: &str,
    remote: &str,
    branch: &str,
) -> anyhow::Result<()> {
    match git(&["remote", "add", local_remote, remote]) {
        Ok(_) => match git(&["fetch", remote, &format!("{branch}:{local_branch}")]) {
            Ok(_) => Ok(()),
            Err(err) => {
                git(&["branch", "-D", local_branch])?;
                Err(anyhow::anyhow!(
                    "Could not fetch branch for pull request: {err}"
                ))
            }
        },
        Err(err) => {
            git(&["remote", "remove", local_remote])?;
            Err(anyhow::anyhow!("Could not add remote: {err}"))
        }
    }
}
