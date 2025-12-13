use crate::config::AppConfig;
use crate::enums::{ComposeField, ConfigField, CurrentPage, InputMode, Notification};
use crate::models::{EmailDraft, LogEntry};
use crate::storage::Storage;

pub struct App {
  pub should_quit: bool,
  pub input_mode: InputMode,
  pub current_page: CurrentPage,
  pub notification: Option<Notification>,

  // Compose State
  pub compose_field: ComposeField,
  pub draft: EmailDraft,

  // Config State
  pub config_field: ConfigField,
  pub config: AppConfig,

  // Logs State
  pub logs: Vec<LogEntry>,
}

impl App {
  pub fn new() -> Self {
    let config = Storage::load_config().unwrap_or_default();
    let draft = Storage::load_draft().unwrap_or_default();

    Self {
      should_quit: false,
      input_mode: InputMode::Normal,
      current_page: CurrentPage::Compose,
      notification: None,
      compose_field: ComposeField::Recipient,
      draft,
      config_field: ConfigField::Name,
      config,
      logs: Vec::new(),
    }
  }

  pub fn quit(&mut self) {
    let _ = Storage::save_draft(&self.draft);
    self.should_quit = true;
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

  pub fn switch_page(&mut self) {
    self.current_page = match self.current_page {
      CurrentPage::Compose => CurrentPage::Config,
      CurrentPage::Config => CurrentPage::Compose,
      CurrentPage::Dashboard => CurrentPage::Dashboard,
    };
    self.input_mode = InputMode::Normal;
    self.clear_notification();
  }

  pub fn cycle_field(&mut self) {
    match self.current_page {
      CurrentPage::Compose => self.cycle_compose_field(),
      CurrentPage::Config => self.cycle_config_field(),
      CurrentPage::Dashboard => todo!("No fields in this yet."),
    }
  }

  fn cycle_compose_field(&mut self) {
    self.compose_field = match self.compose_field {
      ComposeField::Recipient => ComposeField::Subject,
      ComposeField::Subject => ComposeField::Body,
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
