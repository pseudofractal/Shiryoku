use crate::config::AppConfig;
use crate::enums::{
  ComposeField, ConfigField, CurrentPage, DashboardFocus, InputMode, Notification,
};
use crate::models::{EmailDraft, FilterOptions, LogEntry};
use crate::storage::Storage;
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub struct App {
  pub should_quit: bool,
  pub input_mode: InputMode,
  pub current_page: CurrentPage,
  pub notification: Option<Notification>,

  // Compose State
  pub compose_field: ComposeField,
  pub draft: EmailDraft,
  pub attachment_input: String,

  // Config State
  pub config_field: ConfigField,
  pub config: AppConfig,

  // Dashboard State
  pub logs: Vec<LogEntry>,
  pub dashboard_focus: DashboardFocus,
  pub filter_recipient: String,
  pub filter_country: String,
  pub filter_min_opens: String,
  pub dashboard_list_state: ratatui::widgets::TableState,
  pub selected_summary_id: Option<String>,
  pub filter_options: FilterOptions,
}

pub struct RecipientSummary {
  pub tracking_id: String,
  pub decoded_email: String,
  pub country: String,
  pub open_count: usize,
  pub last_seen_raw: DateTime<Utc>,
  pub logs: Vec<LogEntry>,
}

impl App {
  pub fn new() -> Self {
    let config = Storage::load_config().unwrap_or_default();
    let draft = Storage::load_draft().unwrap_or_default();

    let attachment_input = draft
      .attachments
      .iter()
      .map(|p| p.to_string_lossy().to_string())
      .collect::<Vec<_>>()
      .join("; ");

    Self {
      should_quit: false,
      input_mode: InputMode::Normal,
      current_page: CurrentPage::Compose,
      notification: None,
      compose_field: ComposeField::Recipient,
      draft,
      attachment_input,
      config_field: ConfigField::Name,
      config,
      logs: Vec::new(),
      dashboard_focus: DashboardFocus::FilterRecipient,
      filter_recipient: String::new(),
      filter_country: String::new(),
      filter_min_opens: String::new(),
      dashboard_list_state: ratatui::widgets::TableState::default(),
      selected_summary_id: None,
      filter_options: FilterOptions::default(),
    }
  }

  pub fn quit(&mut self) {
    self.sync_attachments();
    let _ = Storage::save_draft(&self.draft);
    self.should_quit = true;
  }
  pub fn sync_attachments(&mut self) {
    self.draft.attachments = self
      .attachment_input
      .split(';')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .map(std::path::PathBuf::from)
      .collect();
  }

  pub fn set_notification(&mut self, note: Notification) {
    self.notification = Some(note);
  }

  pub fn clear_notification(&mut self) {
    self.notification = None;
  }

  pub fn toggle_editing(&mut self) {
    self.input_mode = match self.input_mode {
      InputMode::Normal => InputMode::Editing,
      InputMode::Editing => InputMode::Normal,
    };
  }

  pub fn cycle_field(&mut self) {
    match self.current_page {
      CurrentPage::Compose => self.cycle_compose_field(),
      CurrentPage::Config => self.cycle_config_field(),
      CurrentPage::Dashboard => self.cycle_dashboard_focus(),
    }
  }

  fn cycle_compose_field(&mut self) {
    self.compose_field = match self.compose_field {
      ComposeField::Recipient => ComposeField::Subject,
      ComposeField::Subject => ComposeField::Attachments,
      ComposeField::Attachments => ComposeField::Body,
      ComposeField::Body => ComposeField::SendButton,
      ComposeField::SendButton => ComposeField::Recipient,
    };
  }

  fn cycle_config_field(&mut self) {
    self.config_field = match self.config_field {
      ConfigField::Name => ConfigField::Role,
      ConfigField::Role => ConfigField::Department,
      ConfigField::Department => ConfigField::Institution,
      ConfigField::Institution => ConfigField::Phone,
      ConfigField::Phone => ConfigField::Emails,
      ConfigField::Emails => ConfigField::FooterColor,
      ConfigField::FooterColor => ConfigField::SmtpUser,
      ConfigField::SmtpUser => ConfigField::SmtpPass,
      ConfigField::SmtpPass => ConfigField::WorkerUrl,
      ConfigField::WorkerUrl => ConfigField::ApiSecret,
      ConfigField::ApiSecret => ConfigField::Name,
    };
  }

  fn cycle_dashboard_focus(&mut self) {
    if self.selected_summary_id.is_some() {
      // Modal is already opened
      return;
    }
    self.dashboard_focus = match self.dashboard_focus {
      DashboardFocus::FilterRecipient => DashboardFocus::FilterCountry,
      DashboardFocus::FilterCountry => DashboardFocus::FilterMinOpens,
      DashboardFocus::FilterMinOpens => DashboardFocus::List,
      DashboardFocus::List => DashboardFocus::FilterRecipient,
    };
  }

  pub fn get_aggregated_logs(&self) -> Vec<RecipientSummary> {
    let mut groups: HashMap<String, Vec<LogEntry>> = HashMap::new();
    for log in &self.logs {
      groups
        .entry(log.tracking_id.clone())
        .or_default()
        .push(log.clone());
    }
    let mut summaries = Vec::new();
    let min_opens = self.filter_min_opens.parse::<usize>().unwrap_or(0);

    for (id, mut entries) in groups {
      let decoded_email = general_purpose::URL_SAFE_NO_PAD
        .decode(&id)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_else(|_| id.clone());
      entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

      if entries.len() < min_opens {
        continue;
      }
      if !self.filter_recipient.is_empty() && !decoded_email.contains(&self.filter_recipient) {
        continue;
      }
      let country = entries
        .first()
        .map(|l| l.country.clone())
        .unwrap_or_default();
      if !self.filter_country.is_empty()
        && !country
          .to_lowercase()
          .contains(&self.filter_country.to_lowercase())
      {
        continue;
      }

      let last_seen_str = entries.first().unwrap().timestamp.clone();
      let last_seen_raw = DateTime::parse_from_rfc3339(&last_seen_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_default();
      summaries.push(RecipientSummary {
        tracking_id: id,
        decoded_email,
        country,
        open_count: entries.len(),
        last_seen_raw,
        logs: entries,
      });
    }
    summaries.sort_by(|a, b| b.last_seen_raw.cmp(&a.last_seen_raw));
    summaries
  }

  pub fn push_input(&mut self, c: char) {
    match self.current_page {
      CurrentPage::Compose => self.handle_compose_input(c),
      CurrentPage::Config => self.handle_config_input(c),
      CurrentPage::Dashboard => {}
    }
  }

  pub fn pop_input(&mut self) {
    match self.current_page {
      CurrentPage::Compose => self.handle_compose_backspace(),
      CurrentPage::Config => self.handle_config_backspace(),
      CurrentPage::Dashboard => {}
    }
  }

  fn handle_compose_input(&mut self, c: char) {
    match self.compose_field {
      ComposeField::Recipient => self.draft.recipient.push(c),
      ComposeField::Subject => self.draft.subject.push(c),
      ComposeField::Attachments => self.attachment_input.push(c),
      ComposeField::Body => {}
      ComposeField::SendButton => {}
    }
  }

  fn handle_compose_backspace(&mut self) {
    match self.compose_field {
      ComposeField::Recipient => {
        self.draft.recipient.pop();
      }
      ComposeField::Subject => {
        self.draft.subject.pop();
      }
      ComposeField::Attachments => {
        self.attachment_input.pop();
      }
      ComposeField::Body => {}
      ComposeField::SendButton => {}
    }
  }

  fn handle_config_input(&mut self, c: char) {
    match self.config_field {
      ConfigField::Name => self.config.identity.name.push(c),
      ConfigField::Role => self.config.identity.role.push(c),
      ConfigField::Department => self.config.identity.department.push(c),
      ConfigField::Institution => self.config.identity.institution.push(c),
      ConfigField::Phone => self.config.identity.phone.push(c),
      ConfigField::Emails => self.modify_emails(c, false),
      ConfigField::FooterColor => self.config.identity.footer_color.push(c),
      ConfigField::SmtpUser => self.config.smtp_username.push(c),
      ConfigField::SmtpPass => self.config.smtp_app_password.push(c),
      ConfigField::WorkerUrl => self.config.worker_url.push(c),
      ConfigField::ApiSecret => self.config.api_secret.push(c),
    }
  }

  fn handle_config_backspace(&mut self) {
    match self.config_field {
      ConfigField::Name => {
        self.config.identity.name.pop();
      }
      ConfigField::Role => {
        self.config.identity.role.pop();
      }
      ConfigField::Department => {
        self.config.identity.department.pop();
      }
      ConfigField::Institution => {
        self.config.identity.institution.pop();
      }
      ConfigField::Phone => {
        self.config.identity.phone.pop();
      }
      ConfigField::Emails => self.modify_emails(' ', true),
      ConfigField::FooterColor => {
        self.config.identity.footer_color.pop();
      }
      ConfigField::SmtpUser => {
        self.config.smtp_username.pop();
      }
      ConfigField::SmtpPass => {
        self.config.smtp_app_password.pop();
      }
      ConfigField::WorkerUrl => {
        self.config.worker_url.pop();
      }
      ConfigField::ApiSecret => {
        self.config.api_secret.pop();
      }
    }
  }

  fn modify_emails(&mut self, c: char, is_backspace: bool) {
    let mut str_rep = self.config.identity.emails.join(", ");

    if is_backspace {
      str_rep.pop();
    } else {
      str_rep.push(c);
    }

    self.config.identity.emails = if str_rep.is_empty() {
      Vec::new()
    } else {
      str_rep.split(',').map(|s| s.trim().to_string()).collect()
    };
  }
}
