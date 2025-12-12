use crate::config::AppConfig;
use crate::models::EmailDraft;

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

// Navigation for Compose Page
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ComposeField {
    #[default]
    Recipient,
    Subject,
    Body,
}

// Navigation for Config Page
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    #[default]
    Name,
    Role,
    Department,
    Institution,
    Phone,
    Emails, // Comma separated for simplicity in UI
    SmtpUser,
    SmtpPass,
    WorkerUrl,
}

pub struct App {
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub current_page: CurrentPage,

    // State for Compose Page
    pub compose_field: ComposeField,
    pub draft: EmailDraft,

    // State for Config Page
    pub config_field: ConfigField,
    pub config: AppConfig,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        Self {
            should_quit: false,
            input_mode: InputMode::Normal,
            current_page: CurrentPage::Compose,
            compose_field: ComposeField::Recipient,
            draft: EmailDraft::default(),
            config_field: ConfigField::Name,
            config,
        }
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
        };
        // Reset mode when switching
        self.input_mode = InputMode::Normal;
    }

    pub fn cycle_field(&mut self) {
        match self.current_page {
            CurrentPage::Compose => {
                self.compose_field = match self.compose_field {
                    ComposeField::Recipient => ComposeField::Subject,
                    ComposeField::Subject => ComposeField::Body,
                    ComposeField::Body => ComposeField::Recipient,
                };
            }
            CurrentPage::Config => {
                self.config_field = match self.config_field {
                    ConfigField::Name => ConfigField::Role,
                    ConfigField::Role => ConfigField::Department,
                    ConfigField::Department => ConfigField::Institution,
                    ConfigField::Institution => ConfigField::Phone,
                    ConfigField::Phone => ConfigField::Emails,
                    ConfigField::Emails => ConfigField::SmtpUser,
                    ConfigField::SmtpUser => ConfigField::SmtpPass,
                    ConfigField::SmtpPass => ConfigField::WorkerUrl,
                    ConfigField::WorkerUrl => ConfigField::Name,
                };
            }
        }
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
