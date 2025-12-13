#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
  #[default]
  Normal,
  Editing,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum CurrentPage {
  #[default]
  Compose,
  Config,
  Dashboard,
  Schedule,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ComposeField {
  #[default]
  Recipient,
  Subject,
  Body,
  Attachments,
  SendButton,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ScheduleField {
  #[default]
  Day,
  Month,
  Year,
  Hour,
  Minute,
  Second,
  Timezone,
  Submit,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
  #[default]
  Name,
  Role,
  Department,
  Institution,
  Phone,
  Emails,
  FooterColor,
  SmtpUser,
  SmtpPass,
  WorkerUrl,
  ApiSecret,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum DashboardFocus {
  #[default]
  FilterRecipient,
  FilterCountry,
  FilterMinOpens,
  List,
}

#[derive(Debug, Clone)]
pub enum Notification {
  Info(String),
  Success(String),
  Error(String),
}
