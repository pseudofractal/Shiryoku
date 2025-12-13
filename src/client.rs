use crate::compiler::CompiledEmail;
use crate::models::{FilterOptions, LogEntry};
use anyhow::{Context, Result};
use reqwest::Client;
use reqwest::multipart::{Form, Part};
use tokio::fs;

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

pub async fn fetch_filters(worker_url: &str, api_secret: &str) -> Result<FilterOptions> {
  let client = Client::new();
  let url = format!("{}/api/filters?secret={}", worker_url, api_secret);
  let response = client
    .get(&url)
    .send()
    .await
    .context("Failed to send request to worker")?;
  if !response.status().is_success() {
    return Err(anyhow::anyhow!("Worker error: {}", response.status()));
  }

  let filters: FilterOptions = response.json().await.context("Failed to parse filters")?;
  Ok(filters)
}

pub async fn delete_recipient_logs(
  worker_url: &str,
  api_secret: &str,
  tracking_id: &str,
) -> Result<()> {
  let client = Client::new();
  let url = format!(
    "{}/api/logs?secret={}&tracking_id={}",
    worker_url, api_secret, tracking_id
  );

  let response = client
    .delete(&url)
    .send()
    .await
    .context("Failed to send delete request")?;

  if !response.status().is_success() {
    return Err(anyhow::anyhow!("Worker error: {}", response.status()));
  }

  Ok(())
}

pub async fn schedule_email(
  worker_url: &str,
  api_secret: &str,
  compiled: CompiledEmail,
  subject: String,
  recipient: String,
  scheduled_at: chrono::DateTime<chrono::Utc>,
  smtp_username: String,
  smtp_password: String,
) -> Result<()> {
  let client = Client::new();
  let url = format!("{}/api/schedule?secret={}", worker_url, api_secret);

  let mut form = Form::new()
    .text("recipient", recipient)
    .text("subject", subject)
    .text("html_body", compiled.html_body)
    .text("plain_body", compiled.plain_body)
    .text("scheduled_at", scheduled_at.to_rfc3339())
    .text("smtp_username", smtp_username)
    .text("smtp_password", smtp_password);

  for path in compiled.attachments {
    if let Ok(bytes) = fs::read(&path).await {
      let filename = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "attachment.bin".to_string());

      let mime = mime_guess::from_path(&path).first_or_octet_stream();

      let part = Part::bytes(bytes)
        .file_name(filename)
        .mime_str(mime.as_ref())
        .unwrap();

      form = form.part("attachments", part);
    }
  }

  for img in compiled.inline_images {
    if let Ok(bytes) = fs::read(&img.path).await {
      let cid = img.cid.clone();
      let mime = mime_guess::from_path(&img.path).first_or_octet_stream();

      let part = Part::bytes(bytes)
        .file_name(cid)
        .mime_str(mime.as_ref())
        .unwrap();

      form = form.part("inline_images", part);
    }
  }

  let response = client
    .post(&url)
    .multipart(form)
    .send()
    .await
    .context("Failed to send schedule request")?;

  if !response.status().is_success() {
    let text = response.text().await.unwrap_or_default();
    return Err(anyhow::anyhow!("Worker rejected schedule: {}", text));
  }

  Ok(())
}
