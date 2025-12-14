pub mod dashboard;
pub mod helper;
pub mod schedule;
pub mod scheduled;
pub mod standard;

use crate::app::App;
use crate::enums::CurrentPage;
use crate::models::{FilterOptions, LogEntry, ScheduledJob};
use crossterm::event::KeyEvent;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Action {
  RenderTick,
  EmailSent,
  EmailFailed(String),
  LogsFetched(Vec<LogEntry>),
  LogsFailed(String),
  FiltersFetched(FilterOptions),
  FiltersFailed(String),
  LogsDeleted(String),
  JobsFetched(Vec<ScheduledJob>),
  JobsFailed(String),
  JobCancelled(String),
  JobActionFailed(String),
}

/// Main entry point for key event handling.
pub async fn handle_key_events(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  match app.current_page {
    CurrentPage::Schedule => schedule::handle_schedule_input(key, app, tx).await,
    CurrentPage::Dashboard => dashboard::handle_dashboard_events(key, app, tx).await,
    CurrentPage::Scheduled => scheduled::handle_scheduled_events(key, app, tx).await,
    _ => standard::handle_standard_events(key, app, tx).await,
  }
}
