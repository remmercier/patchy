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

/// Converts a commit message to only contain lowercase characters, underscores and dashes
pub fn normalize_commit_msg(commit_msg: &str) -> String {
    commit_msg
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() {
                '_'
            } else {
                '-'
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
                serde_json::from_str(&out).context(format!("Could not parse response.\n{out}"))?;

            Ok(response)
        }
        Ok(res) => {
            let status = res.status();
            let text = res.text().await?;

            Err(anyhow!(
                "Request failed with status: {status}\nRequested URL: {url}\nResponse: {text}",
            ))
        }
        Err(err) => Err(anyhow!("Error sending request: {err}")),
    }
}

#[macro_export]
macro_rules! success {
    ($($arg:tt)*) => {{
        println!("{INDENT}{}{}", "✓ ".bright_green().bold(), format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! fail {
    ($($arg:tt)*) => {{
        eprintln!("{INDENT}{}{}", "✗ ".bright_red().bold(), format!($($arg)*))
    }};
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {{
        if *IS_VERBOSE {
            eprintln!("{INDENT}{}{}", "--verbose: ".bright_yellow().bold(), format!($($arg)*))
        }
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {{
        eprintln!("{INDENT}{}{}", "ⓘ ".bright_blue().bold(), format!($($arg)*))
    }};
}

/// Interact with the user to get a yes or a no answer
#[macro_export]
macro_rules! confirm_prompt {
    ($($arg:tt)*) => {{
        Confirm::new()
            .with_prompt(format!(
                "\n{INDENT}{} {}",
                "»".bright_black(),
                format!($($arg)*)
            ))
            .interact()
            .unwrap()
    }};
}
