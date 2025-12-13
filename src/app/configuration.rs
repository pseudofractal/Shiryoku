use crate::config::AppConfig;
use crate::enums::ConfigField;

pub struct ConfigState {
  pub field: ConfigField,
  pub data: AppConfig,
}

impl ConfigState {
  pub fn new(data: AppConfig) -> Self {
    Self {
      field: ConfigField::Name,
      data,
    }
  }

  pub fn handle_input(&mut self, c: char) {
    match self.field {
      ConfigField::Name => self.data.identity.name.push(c),
      ConfigField::Role => self.data.identity.role.push(c),
      ConfigField::Department => self.data.identity.department.push(c),
      ConfigField::Institution => self.data.identity.institution.push(c),
      ConfigField::Phone => self.data.identity.phone.push(c),
      ConfigField::Emails => self.modify_emails(c, false),
      ConfigField::FooterColor => self.data.identity.footer_color.push(c),
      ConfigField::SmtpUser => self.data.smtp_username.push(c),
      ConfigField::SmtpPass => self.data.smtp_app_password.push(c),
      ConfigField::WorkerUrl => self.data.worker_url.push(c),
      ConfigField::ApiSecret => self.data.api_secret.push(c),
    }
  }

  pub fn handle_backspace(&mut self) {
    match self.field {
      ConfigField::Name => {
        self.data.identity.name.pop();
      }
      ConfigField::Role => {
        self.data.identity.role.pop();
      }
      ConfigField::Department => {
        self.data.identity.department.pop();
      }
      ConfigField::Institution => {
        self.data.identity.institution.pop();
      }
      ConfigField::Phone => {
        self.data.identity.phone.pop();
      }
      ConfigField::Emails => self.modify_emails(' ', true),
      ConfigField::FooterColor => {
        self.data.identity.footer_color.pop();
      }
      ConfigField::SmtpUser => {
        self.data.smtp_username.pop();
      }
      ConfigField::SmtpPass => {
        self.data.smtp_app_password.pop();
      }
      ConfigField::WorkerUrl => {
        self.data.worker_url.pop();
      }
      ConfigField::ApiSecret => {
        self.data.api_secret.pop();
      }
    }
  }

  fn modify_emails(&mut self, c: char, is_backspace: bool) {
    let mut str_rep = self.data.identity.emails.join(", ");
    if is_backspace {
      str_rep.pop();
    } else {
      str_rep.push(c);
    }
    self.data.identity.emails = if str_rep.is_empty() {
      Vec::new()
    } else {
      str_rep.split(',').map(|s| s.trim().to_string()).collect()
    };
  }

  pub fn cycle_field(&mut self, forward: bool) {
    if forward {
      self.field = match self.field {
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
    } else {
      self.field = match self.field {
        ConfigField::Name => ConfigField::ApiSecret,
        ConfigField::Role => ConfigField::Name,
        ConfigField::Department => ConfigField::Role,
        ConfigField::Institution => ConfigField::Department,
        ConfigField::Phone => ConfigField::Institution,
        ConfigField::Emails => ConfigField::Phone,
        ConfigField::FooterColor => ConfigField::Emails,
        ConfigField::SmtpUser => ConfigField::FooterColor,
        ConfigField::SmtpPass => ConfigField::SmtpUser,
        ConfigField::WorkerUrl => ConfigField::SmtpPass,
        ConfigField::ApiSecret => ConfigField::WorkerUrl,
      };
    }
  }
}
