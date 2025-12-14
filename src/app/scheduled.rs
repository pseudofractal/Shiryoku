use crate::enums::{JobStatus, ScheduledFocus};
use crate::models::ScheduledJob;
use ratatui::widgets::TableState;

pub struct ScheduledState {
  pub jobs: Vec<ScheduledJob>,
  pub focus: ScheduledFocus,
  pub filter_recipient: String,
  pub filter_status: Option<JobStatus>,
  pub list_state: TableState,
  pub selected_job_id: Option<String>,
}

impl Default for ScheduledState {
  fn default() -> Self {
    Self {
      jobs: Vec::new(),
      focus: ScheduledFocus::FilterRecipient,
      filter_recipient: String::new(),
      filter_status: None,
      list_state: TableState::default(),
      selected_job_id: None,
    }
  }
}

impl ScheduledState {
  pub fn cycle_focus(&mut self, forward: bool) {
    if self.selected_job_id.is_some() {
      return;
    }

    if forward {
      self.focus = match self.focus {
        ScheduledFocus::FilterRecipient => ScheduledFocus::FilterStatus,
        ScheduledFocus::FilterStatus => ScheduledFocus::List,
        ScheduledFocus::List => ScheduledFocus::FilterRecipient,
      };
    } else {
      self.focus = match self.focus {
        ScheduledFocus::FilterRecipient => ScheduledFocus::List,
        ScheduledFocus::FilterStatus => ScheduledFocus::FilterRecipient,
        ScheduledFocus::List => ScheduledFocus::FilterStatus,
      };
    }
  }

  pub fn handle_input(&mut self, c: char) {
    match self.focus {
      ScheduledFocus::FilterRecipient => self.filter_recipient.push(c),
      _ => {}
    }
  }

  pub fn handle_backspace(&mut self) {
    match self.focus {
      ScheduledFocus::FilterRecipient => {
        self.filter_recipient.pop();
      }
      _ => {}
    }
  }

  pub fn toggle_status_filter(&mut self) {
    self.filter_status = match self.filter_status {
      None => Some(JobStatus::Pending),
      Some(JobStatus::Pending) => Some(JobStatus::Sent),
      Some(JobStatus::Sent) => Some(JobStatus::Failed),
      Some(JobStatus::Failed) => None,
      _ => None,
    }
  }

  pub fn get_filtered_jobs(&self) -> Vec<&ScheduledJob> {
    self
      .jobs
      .iter()
      .filter(|job| {
        let matches_recipient = if self.filter_recipient.is_empty() {
          true
        } else {
          job.recipient.contains(&self.filter_recipient)
        };

        let matches_status = match self.filter_status {
          None => true,
          Some(s) => job.status == s,
        };

        matches_recipient && matches_status
      })
      .collect()
  }
}
