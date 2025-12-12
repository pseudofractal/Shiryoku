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
