use crate::models::LogEntry;
use anyhow::{Context, Result};
use reqwest::Client;

pub async fn fetch_logs(worker_url: &str, api_secret: &str) -> Result<Vec<LogEntry>> {
  let client = Client::new();

  let url = format!("{}/api/logs?secret={}", worker_url, api_secret);

  let response = client
    .get(&url)
    .send()
    .await
    .context("Failed to send request to worker")?;

  if !response.status().is_success() {
    return Err(anyhow::anyhow!(
      "Worker returned error: {}",
      response.status()
    ));
  }

  let logs: Vec<LogEntry> = response.json().await.context("Failed to parse logs JSON")?;

  Ok(logs)
}
