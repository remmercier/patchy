use std::collections::HashSet;

use serde::{Deserialize, Serialize};

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
