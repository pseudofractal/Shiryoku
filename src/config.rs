use crate::models::UserIdentity;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub identity: UserIdentity,
    pub smtp_username: String,
    pub smtp_app_password: String,
    pub worker_url: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            identity: UserIdentity {
                name: String::new(),
                role: String::new(),
                department: String::new(),
                institution: String::new(),
                phone: String::new(),
                emails: Vec::new(),
            },
            smtp_username: String::new(),
            smtp_app_password: String::new(),
            worker_url: String::new(),
        }
    }
}

impl AppConfig {
    fn get_config_path() -> Result<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "shiryoku", "shiryoku")
            .context("Could not determine config directory")?;

        let config_dir = proj_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }

        Ok(config_dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::get_config_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;

        let config =
            serde_json::from_str(&content).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}
