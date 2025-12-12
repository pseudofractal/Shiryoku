use crate::models::EmailDraft;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    #[default]
    Normal,
    Editing,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ComposeField {
    #[default]
    Recipient,
    Subject,
    Body,
}

pub struct App {
    pub should_quit: bool,
    pub input_mode: InputMode,
    pub current_field: ComposeField,
    pub draft: EmailDraft,
}

impl App {
    pub fn new() -> Self {
        Self {
            should_quit: false,
            input_mode: InputMode::Normal,
            current_field: ComposeField::Recipient,
            draft: EmailDraft::default(),
        }
    }

    pub fn toggle_editing(&mut self) {
        if self.input_mode == InputMode::Normal {
            self.input_mode = InputMode::Editing;
        } else {
            self.input_mode = InputMode::Normal;
        }
    }

    pub fn cycle_field(&mut self) {
        self.current_field = match self.current_field {
            ComposeField::Recipient => ComposeField::Subject,
            ComposeField::Subject => ComposeField::Body,
            ComposeField::Body => ComposeField::Recipient,
        };
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
