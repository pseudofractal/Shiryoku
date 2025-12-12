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
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ComposeField {
    #[default]
    Recipient,
    Subject,
    Body,
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
    SmtpUser,
    SmtpPass,
    WorkerUrl,
}

#[derive(Debug, Clone)]
pub enum Notification {
    Info(String),
    Success(String),
    Error(String),
}
