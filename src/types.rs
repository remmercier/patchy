use std::collections::HashSet;

use indexmap::IndexSet;
use serde::{Deserialize, Serialize};

pub type CommandArgs = IndexSet<String>;

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Configuration {
    pub local_branch: String,
    pub patches: Option<HashSet<String>>,
    pub pull_requests: Vec<String>,
    pub remote_branch: String,
    pub repo: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GitHubResponse {
    pub head: Head,
    pub title: String,
    pub html_url: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Head {
    pub repo: Repo,
    pub r#ref: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Repo {
    pub clone_url: String,
}

pub struct Branch {
    pub local_name: String,
    pub remote_name: String,
}

pub struct Remote {
    pub local_name: String,
    pub remote_name: String,
}

pub struct BranchAndRemote {
    pub branch: Branch,
    pub remote: Remote,
}

impl BranchAndRemote {
    pub fn new(
        local_branch: &str,
        remote_branch: &str,
        local_remote: &str,
        remote_remote: &str,
    ) -> Self {
        let branch = Branch {
            local_name: local_branch.into(),
            remote_name: remote_branch.into(),
        };
        let remote = Remote {
            local_name: local_remote.into(),
            remote_name: remote_remote.into(),
        };
        Self { branch, remote }
    }
}
