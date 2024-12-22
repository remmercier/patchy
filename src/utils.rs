use crate::APP_NAME;
use anyhow::{anyhow, Context};
use rand::Rng;
use reqwest::{header::USER_AGENT, Client};

use crate::types::GitHubResponse;

pub fn with_uuid(s: &str) -> String {
    let hash: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    format!("{APP_NAME}-{s}-{hash}")
}

pub fn display_link(text: &str, url: &str) -> String {
    format!("\u{1b}]8;;{}\u{1b}\\{}\u{1b}]8;;\u{1b}\\", url, text)
}

pub async fn make_request(client: &Client, url: &str) -> anyhow::Result<GitHubResponse> {
    let request = client
        .get(url)
        .header(USER_AGENT, "{APP_NAME}")
        .send()
        .await;

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
