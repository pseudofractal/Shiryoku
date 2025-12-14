use crate::app::App;
use crate::client;
use crate::enums::Notification;
use crate::handler::Action;
use std::io::{self, Write};
use std::process::Command;
use tokio::sync::mpsc;

pub fn trigger_fetch(app: &mut App, tx: mpsc::Sender<Action>) {
  app.set_notification(Notification::Info("Fetching logs...".to_string()));
  let url = app.config.data.worker_url.clone();
  let secret = app.config.data.api_secret.clone();
  let tx_logs = tx.clone();

  tokio::spawn(async move {
    match client::fetch_logs(&url, &secret).await {
      Ok(logs) => tx_logs.send(Action::LogsFetched(logs)).await.unwrap(),
      Err(e) => tx_logs
        .send(Action::LogsFailed(e.to_string()))
        .await
        .unwrap(),
    }
  });

  let url2 = app.config.data.worker_url.clone();
  let sec2 = app.config.data.api_secret.clone();
  let tx_filters = tx.clone();

  tokio::spawn(async move {
    match client::fetch_filters(&url2, &sec2).await {
      Ok(f) => tx_filters.send(Action::FiltersFetched(f)).await.unwrap(),
      Err(e) => tx_filters
        .send(Action::FiltersFailed(e.to_string()))
        .await
        .unwrap(),
    }
  });
}

pub fn trigger_fetch_jobs(app: &mut App, tx: mpsc::Sender<Action>) {
  app.current_page = crate::enums::CurrentPage::Scheduled;
  app.set_notification(Notification::Info("Fetching scheduled jobs...".to_string()));
  let url = app.config.data.worker_url.clone();
  let secret = app.config.data.api_secret.clone();
  let tx_jobs = tx.clone();

  tokio::spawn(async move {
    match client::fetch_scheduled_jobs(&url, &secret).await {
      Ok(jobs) => tx_jobs.send(Action::JobsFetched(jobs)).await.unwrap(),
      Err(e) => tx_jobs
        .send(Action::JobsFailed(e.to_string()))
        .await
        .unwrap(),
    }
  });
}

pub fn open_external_editor(initial_text: &str) -> io::Result<String> {
  let mut temp_file = tempfile::Builder::new().suffix(".md").tempfile()?;
  write!(temp_file, "{}", initial_text)?;
  let mut candidates = Vec::new();
  if let Ok(v) = std::env::var("VISUAL") {
    candidates.push(v);
  }
  if let Ok(e) = std::env::var("EDITOR") {
    candidates.push(e);
  }
  if cfg!(target_os = "windows") {
    candidates.push("notepad.exe".to_string());
    candidates.push("code.cmd".to_string());
  } else {
    candidates.push("nano".to_string());
    candidates.push("vim".to_string());
    candidates.push("vi".to_string());
    candidates.push("editor".to_string());
    candidates.push("xdg-open".to_string());
  }

  for editor in candidates {
    let result = Command::new(&editor).arg(temp_file.path()).status();

    match result {
      Ok(status) => {
        if status.success() {
          let content = std::fs::read_to_string(temp_file.path())?;
          return Ok(content);
        } else {
          continue;
        }
      }
      Err(e) => {
        if e.kind() == io::ErrorKind::NotFound {
          continue;
        }
        continue;
      }
    }
  }

  Err(io::Error::new(
    io::ErrorKind::NotFound,
    "No suitable editor found (tried $EDITOR, nano, vim, vi, notepad).",
  ))
}
