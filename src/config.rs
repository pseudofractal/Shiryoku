use crate::models::UserIdentity;
use serde::{Deserialize, Serialize};
use std::default::Default;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
  pub identity: UserIdentity,
  pub smtp_username: String,
  pub smtp_app_password: String,
  pub worker_url: String,
  pub api_secret: String,
}
