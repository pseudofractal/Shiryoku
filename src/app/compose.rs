use crate::enums::ComposeField;
use crate::models::EmailDraft;
use std::path::PathBuf;

pub struct ComposeState {
  pub field: ComposeField,
  pub draft: EmailDraft,
  pub attachment_input: String,
}

impl ComposeState {
  pub fn new(draft: EmailDraft) -> Self {
    let attachment_input = draft
      .attachments
      .iter()
      .map(|p| p.to_string_lossy().to_string())
      .collect::<Vec<_>>()
      .join("; ");

    Self {
      field: ComposeField::Recipient,
      draft,
      attachment_input,
    }
  }

  pub fn sync_attachments(&mut self) {
    self.draft.attachments = self
      .attachment_input
      .split(';')
      .map(|s| s.trim())
      .filter(|s| !s.is_empty())
      .map(PathBuf::from)
      .collect();
  }

  pub fn handle_input(&mut self, c: char) {
    match self.field {
      ComposeField::Recipient => self.draft.recipient.push(c),
      ComposeField::Subject => self.draft.subject.push(c),
      ComposeField::Attachments => self.attachment_input.push(c),
      ComposeField::Body => {}
      ComposeField::SendButton => {}
    }
  }

  pub fn handle_backspace(&mut self) {
    match self.field {
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

  pub fn cycle_field(&mut self, forward: bool) {
    if forward {
      self.field = match self.field {
        ComposeField::Recipient => ComposeField::Subject,
        ComposeField::Subject => ComposeField::Attachments,
        ComposeField::Attachments => ComposeField::Body,
        ComposeField::Body => ComposeField::SendButton,
        ComposeField::SendButton => ComposeField::Recipient,
      };
    } else {
      self.field = match self.field {
        ComposeField::Recipient => ComposeField::SendButton,
        ComposeField::Subject => ComposeField::Recipient,
        ComposeField::Attachments => ComposeField::Subject,
        ComposeField::Body => ComposeField::Attachments,
        ComposeField::SendButton => ComposeField::Body,
      };
    }
  }
}
