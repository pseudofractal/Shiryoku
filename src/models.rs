use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDraft {
    pub recipient: String,
    pub subject: String,
    pub body: String,
    pub attachments: Vec<PathBuf>,
    pub scheduled_at: Option<DateTime<Utc>>,
}

impl Default for EmailDraft {
    fn default() -> Self {
        Self {
            recipient: String::new(),
            subject: String::new(),
            body: String::new(),
            attachments: Vec::new(),
            scheduled_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdentity {
    pub name: String,
    pub role: String,
    pub department: String,
    pub institution: String,
    pub phone: String,
    pub emails: Vec<String>,
}
