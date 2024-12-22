use std::{
    path::{Path, PathBuf},
    process::{Child, Command},
};

pub type Args<'a> = &'a [&'a str];

// pub fn batch_git_processes(output: Vec<(Child, Args)>) -> Vec<anyhow::Result<String>> {
//     output
//         .into_iter()
//         .map(|(child, git_args)| run(child, git_args))
//         .collect()
// }

pub fn run(a: Child, args: &[&str]) -> anyhow::Result<String> {
    a.wait_with_output()
        .map_err(|err| anyhow::anyhow!(err))
        .and_then(|output| {
            if !output.status.success() {
                Err(anyhow::anyhow!(
                    "Git command failed.\nCommand: git {}\nStdout: {}\nStderr: {}",
                    args.join(" "),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                ))
            } else {
                Ok(String::from_utf8_lossy(&output.stdout)
                    .trim_end()
                    .to_owned())
            }
        })
}

pub fn spawn<'a>(
    args: &'a [&'a str],
    git_dir: &'a Path,
) -> (Result<Child, std::io::Error>, Args<'a>) {
    (
        Command::new("git").args(args).current_dir(git_dir).spawn(),
        args,
    )
}

pub fn get_root() -> anyhow::Result<PathBuf> {
    let current_dir = std::env::current_dir()?;

    let args = ["rev-parse", "--show-toplevel"];

    let (root, _args) = spawn(&args, &current_dir);

    let m = run(root?, &args);

    dbg!(&m);

    m.map(|result| result.into())
}

pub fn git(args: &[&str]) -> anyhow::Result<String> {
    let root = get_root()?;
    let (child, _args) = spawn(args, &root);
    run(child?, args)
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
