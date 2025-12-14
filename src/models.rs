use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
pub use crate::enums::JobStatus;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmailDraft {
  pub recipient: String,
  pub subject: String,
  pub body: String,
  pub attachments: Vec<PathBuf>,
  pub scheduled_at: Option<DateTime<Utc>>,
  #[serde(default)]
  pub schedule: ScheduleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
  pub name: String,
  pub role: String,
  pub department: String,
  pub institution: String,
  pub phone: String,
  pub emails: Vec<String>,
  #[serde(default = "default_color")]
  pub footer_color: String,
}

fn default_color() -> String {
  "#179299".to_string()
}

impl Default for UserIdentity {
  fn default() -> Self {
    Self {
      name: String::new(),
      role: String::new(),
      department: String::new(),
      institution: String::new(),
      phone: String::new(),
      emails: Vec::new(),
      footer_color: default_color(),
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FilterOptions {
  pub recipients: Vec<String>,
  pub countries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
  pub id: u32,
  pub tracking_id: String,
  pub timestamp: String,
  pub ip: String,
  pub country: String,
  pub city: String,
  pub user_agent: String,
  pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScheduleMetadata {
  pub day: String,
  pub month: String,
  pub year: String,
  pub hour: String,
  pub minute: String,
  pub second: String,
  pub timezone: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentStub {
  pub filename: String,
  pub content: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledJob {
  pub id: String,
  pub recipient: String,
  pub subject: String,
  pub body: String,
  pub scheduled_at: DateTime<Utc>,
  pub recipient_timezone: String,
  pub status: JobStatus,
  pub attachments: Vec<AttachmentStub>,
}
