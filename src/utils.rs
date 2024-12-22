use anyhow::{anyhow, Context};
use rand::Rng;
use reqwest::{header::USER_AGENT, Client};

use crate::types::GitHubResponse;

pub fn with_uuid(s: &str) -> String {
    format!(
        "{uuid}-{s}",
        uuid = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(4)
            .map(char::from)
            .collect::<String>()
    )
}

pub fn normalize_pr_title(pr_title: &str) -> String {
    pr_title
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' {
                Some(c.to_ascii_lowercase())
            } else if c.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .collect()
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
        Ok(res) => {
            let status = res.status();
            let text = res.text().await?;

            dbg!(&text);
            dbg!(&status);

            Err(anyhow!(
                "Request failed with status {}\nResponse: {}",
                status,
                text
            ))
        }
        Err(err) => Err(anyhow!("Error sending request: {err}")),
    }
}
