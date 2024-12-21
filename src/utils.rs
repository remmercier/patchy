use crate::APP_NAME;
use anyhow::{anyhow, Context};
use rand::Rng;
use reqwest::{Error, Response};

use crate::types::GitHubResponse;

pub fn with_uuid(s: &str) -> String {
    let hash: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(30)
        .map(char::from)
        .collect();

    format!("{APP_NAME}-{s}-{hash}")
}

pub async fn handle_request(request: Result<Response, Error>) -> anyhow::Result<GitHubResponse> {
    match request {
        Ok(res) if res.status().is_success() => {
            let out = res.text().await?;

            let response: GitHubResponse =
                serde_json::from_str(&out).context("Could not parse response.\n{out}")?;

            Ok(response)
        }
        Ok(res) => Err(anyhow!(
            "Request failed with status {}\nResponse: {}",
            res.status(),
            res.text().await?
        )),
        Err(err) => Err(anyhow!("Error sending request: {err}")),
    }
}
