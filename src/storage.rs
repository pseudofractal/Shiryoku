use crate::config::AppConfig;
use crate::models::EmailDraft;
use anyhow::{Context, Result};
use directories::ProjectDirs;
use std::fs;
use std::path::Path;

pub struct Storage;

impl Storage {
  // Uses standard paths:
  // Linux:   ~/.config/shiryoku  &  ~/.local/share/shiryoku
  // Windows: %APPDATA%\shiryoku  &  %LOCALAPPDATA%\shiryoku
  // macOS:   ~/Library/Application Support/shiryoku
  fn get_proj_dirs() -> Result<ProjectDirs> {
    ProjectDirs::from("", "", "shiryoku").context("Could not determine home directory")
  }

  fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
      fs::create_dir_all(path)?;
    }
    Ok(())
  }

  pub fn load_config() -> Result<AppConfig> {
    let dirs = Self::get_proj_dirs()?;
    let config_dir = dirs.config_dir();
    let path = config_dir.join("config.json");

    if !path.exists() {
      return Ok(AppConfig::default());
    }

    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
  }

  pub fn save_config(config: &AppConfig) -> Result<()> {
    let dirs = Self::get_proj_dirs()?;
    let config_dir = dirs.config_dir();
    Self::ensure_dir(config_dir)?;

    let path = config_dir.join("config.json");
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
  }

  pub fn load_draft() -> Result<EmailDraft> {
    let dirs = Self::get_proj_dirs()?;
    let data_dir = dirs.data_dir();
    let path = data_dir.join("draft.json");

    if !path.exists() {
      return Ok(EmailDraft::default());
    }

    let content = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&content).unwrap_or_default())
  }

  pub fn save_draft(draft: &EmailDraft) -> Result<()> {
    let dirs = Self::get_proj_dirs()?;
    let data_dir = dirs.data_dir();
    Self::ensure_dir(data_dir)?;

    let path = data_dir.join("draft.json");
    let content = serde_json::to_string_pretty(draft)?;
    fs::write(path, content)?;
    Ok(())
  }
}
