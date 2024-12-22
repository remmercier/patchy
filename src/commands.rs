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
    remote_remote: &str,
    remote_branch: &str,
) -> anyhow::Result<()> {
    match git(&["remote", "add", local_remote, remote_remote]) {
        Ok(_) => match git(&[
            "fetch",
            remote_remote,
            &format!("{remote_branch}:{local_branch}"),
        ]) {
            Ok(_) => Ok(()),
            Err(err) => {
                git(&["branch", "-D", local_branch])?;
                Err(anyhow::anyhow!("Could not fetch branch from remote: {err}"))
            }
        },
        Err(err) => {
            git(&["remote", "remove", local_remote])?;
            Err(anyhow::anyhow!("Could not add remote: {err}"))
        }
    }
}

pub fn checkout(branch: &str, remote: &str) -> anyhow::Result<()> {
    match git(&["checkout", branch]) {
        Ok(_) => Ok(()),
        Err(err) => {
            git(&["branch", "-D", branch])?;
            git(&["remote", "remove", remote])?;
            Err(anyhow::anyhow!(
                "Could not checkout branch: {branch}, which belongs to remote {remote}\n{err}"
            ))
        }
    }
}

pub fn merge_into_main(
    local_branch: &str,
    remote_branch: &str,
) -> anyhow::Result<String, anyhow::Error> {
    match git(&["merge", local_branch, "--no-commit", "--no-ff"]) {
        Ok(_) => Ok(format!("Merged {remote_branch} successfully")),
        Err(_) => {
            let files_with_conflicts = git(&["diff", "--name-only", "--diff-filter=U"])?;
            for file_with_conflict in files_with_conflicts.lines() {
                if file_with_conflict.ends_with(".md") {
                    git(&["checkout", "--ours", file_with_conflict])?;
                    git(&["add", file_with_conflict])?;
                } else {
                    git(&["merge", "--abort"])?;
                    return Err(anyhow::anyhow!(
                        "Unresolved conflict in {file_with_conflict}"
                    ));
                }
            }
            Ok("Merged {remote_branch} successfully and disregarded conflicts".into())
        }
    }
}
