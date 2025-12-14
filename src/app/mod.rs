pub mod compose;
pub mod configuration;
pub mod dashboard;
pub mod schedule;
pub mod scheduled;

use crate::enums::{CurrentPage, InputMode, Notification};
use crate::storage::Storage;
use compose::ComposeState;
use configuration::ConfigState;
use dashboard::DashboardState;
use schedule::ScheduleState;
use scheduled::ScheduledState;

pub struct App {
  pub should_quit: bool,
  pub input_mode: InputMode,
  pub current_page: CurrentPage,
  pub notification: Option<Notification>,

  // Sub-States
  pub compose: ComposeState,
  pub config: ConfigState,
  pub dashboard: DashboardState,
  pub schedule: ScheduleState,
  pub scheduled: ScheduledState,
}

impl App {
  pub fn new() -> Self {
    let loaded_config = Storage::load_config().unwrap_or_default();
    let loaded_draft = Storage::load_draft().unwrap_or_default();

    // Hydrate Schedule
    let mut schedule = ScheduleState::default();
    if !loaded_draft.schedule.day.is_empty() {
      schedule.day = loaded_draft.schedule.day.clone();
      schedule.month = loaded_draft.schedule.month.clone();
      schedule.year = loaded_draft.schedule.year.clone();
      schedule.hour = loaded_draft.schedule.hour.clone();
      schedule.minute = loaded_draft.schedule.minute.clone();
      schedule.second = loaded_draft.schedule.second.clone();
      schedule.timezone_input = loaded_draft.schedule.timezone.clone();
      schedule.update_timezone_filter();
    }

    Self {
      should_quit: false,
      input_mode: InputMode::Normal,
      current_page: CurrentPage::Compose,
      notification: None,
      compose: ComposeState::new(loaded_draft),
      config: ConfigState::new(loaded_config),
      dashboard: DashboardState::default(),
      schedule,
      scheduled: ScheduledState::default(),
    }
  }

  pub fn reset_schedule_modal(&mut self) {
    self.schedule.reset_defaults_if_empty();
  }

  pub fn quit(&mut self) {
    self.compose.sync_attachments();
    self.sync_schedule_to_draft();
    let _ = Storage::save_draft(&self.compose.draft);
    self.should_quit = true;
  }

  pub fn sync_schedule_to_draft(&mut self) {
    self.compose.draft.schedule.day = self.schedule.day.clone();
    self.compose.draft.schedule.month = self.schedule.month.clone();
    self.compose.draft.schedule.year = self.schedule.year.clone();
    self.compose.draft.schedule.hour = self.schedule.hour.clone();
    self.compose.draft.schedule.minute = self.schedule.minute.clone();
    self.compose.draft.schedule.second = self.schedule.second.clone();
    self.compose.draft.schedule.timezone = self.schedule.timezone_input.clone();
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
    self.cycle_generic(true);
  }

  pub fn cycle_field_backward(&mut self) {
    self.cycle_generic(false);
  }

  fn cycle_generic(&mut self, forward: bool) {
    match self.current_page {
      CurrentPage::Compose => self.compose.cycle_field(forward),
      CurrentPage::Config => self.config.cycle_field(forward),
      CurrentPage::Dashboard => self.dashboard.cycle_focus(forward),
      CurrentPage::Schedule => self.schedule.cycle_field(forward),
      CurrentPage::Scheduled => self.scheduled.cycle_focus(forward),
    }
  }

  pub fn push_input(&mut self, c: char) {
    match self.current_page {
      CurrentPage::Compose => self.compose.handle_input(c),
      CurrentPage::Config => self.config.handle_input(c),
      CurrentPage::Dashboard => self.dashboard.handle_input(c),
      CurrentPage::Schedule => self.schedule.handle_input(c),
      CurrentPage::Scheduled => self.scheduled.handle_input(c),
    }
  }

  pub fn pop_input(&mut self) {
    match self.current_page {
      CurrentPage::Compose => self.compose.handle_backspace(),
      CurrentPage::Config => self.config.handle_backspace(),
      CurrentPage::Dashboard => self.dashboard.handle_backspace(),
      CurrentPage::Schedule => self.schedule.handle_backspace(),
      CurrentPage::Scheduled => self.scheduled.handle_backspace(),
    }
  }
}
