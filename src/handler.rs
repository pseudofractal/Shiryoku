use crate::app::App;
use crate::client;
use crate::enums::{ComposeField, CurrentPage, DashboardFocus, InputMode, Notification};
use crate::mailer;
use crate::models::{FilterOptions, LogEntry};
use crate::storage::Storage;
use crossterm::event::{KeyCode, KeyEvent};
use crossterm::{
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io::{self, Write};
use std::process::Command;
use tokio::sync::mpsc;

pub enum Action {
  RenderTick,
  EmailSent,
  EmailFailed(String),
  LogsFetched(Vec<LogEntry>),
  LogsFailed(String),
  FiltersFetched(FilterOptions),
  FiltersFailed(String),
}

/// Main entry point for key event handling.
/// Delegates to specific handlers based on the current page.
/// Returns true if the screen needs to be re-rendered/cleared.
pub async fn handle_key_events(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  if app.current_page == CurrentPage::Dashboard {
    handle_dashboard_events(key, app, tx).await
  } else {
    handle_standard_events(key, app, tx).await
  }
}

// DASHBOARD EVENT HANDLERS

async fn handle_dashboard_events(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  // Popup Interaction
  if app.selected_summary_id.is_some() {
    if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
      app.selected_summary_id = None;
      return true;
    }
    return false;
  }

  match app.input_mode {
    InputMode::Normal => handle_dashboard_normal_input(key, app, tx),
    InputMode::Editing => handle_dashboard_editing_input(key, app),
  }
}

fn handle_dashboard_normal_input(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  match key.code {
    KeyCode::Char('q') => {
      app.quit();
      false
    }
    KeyCode::Tab => {
      app.cycle_field();
      true
    }
    KeyCode::Char('1') => {
      app.current_page = CurrentPage::Compose;
      false
    }
    KeyCode::Char('2') => {
      app.current_page = CurrentPage::Config;
      false
    }
    KeyCode::Char('3') => {
      trigger_fetch(app, tx);
      false
    }
    KeyCode::Char('r') => {
      trigger_fetch(app, tx);
      false
    }
    KeyCode::Enter => {
      if app.dashboard_focus == DashboardFocus::List {
        if let Some(idx) = app.dashboard_list_state.selected() {
          let summaries = app.get_aggregated_logs();
          if let Some(s) = summaries.get(idx) {
            app.selected_summary_id = Some(s.tracking_id.clone());
          }
        }
      } else {
        app.toggle_editing();
      }
      true
    }
    KeyCode::Down => {
      if app.dashboard_focus == DashboardFocus::List {
        // [TODO] Logic to move selection down. Gotta be a better way.
        let i = match app.dashboard_list_state.selected() {
          Some(i) => {
            if i >= app.get_aggregated_logs().len().saturating_sub(1) {
              0
            } else {
              i + 1
            }
          }
          None => 0,
        };
        app.dashboard_list_state.select(Some(i));
        return true;
      }
      false
    }
    KeyCode::Up => {
      if app.dashboard_focus == DashboardFocus::List {
        // [TODO] Same as above but up
        let i = match app.dashboard_list_state.selected() {
          Some(i) => {
            if i == 0 {
              app.get_aggregated_logs().len().saturating_sub(1)
            } else {
              i - 1
            }
          }
          None => 0,
        };
        app.dashboard_list_state.select(Some(i));
        return true;
      }
      false
    }
    _ => false,
  }
}

fn handle_dashboard_editing_input(key: KeyEvent, app: &mut App) -> bool {
  match key.code {
    KeyCode::Esc | KeyCode::Enter => {
      app.toggle_editing();
    }
    KeyCode::Char(c) => match app.dashboard_focus {
      DashboardFocus::FilterRecipient => app.filter_recipient.push(c),
      DashboardFocus::FilterCountry => app.filter_country.push(c),
      DashboardFocus::FilterMinOpens => {
        if c.is_numeric() {
          app.filter_min_opens.push(c)
        }
      }
      _ => {}
    },
    KeyCode::Backspace => match app.dashboard_focus {
      DashboardFocus::FilterRecipient => {
        app.filter_recipient.pop();
      }
      DashboardFocus::FilterCountry => {
        app.filter_country.pop();
      }
      DashboardFocus::FilterMinOpens => {
        app.filter_min_opens.pop();
      }
      _ => {}
    },
    _ => {}
  }
  true
}

// STANDARD PAGE EVENT HANDLERS (Compose & Config)

async fn handle_standard_events(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  match app.input_mode {
    InputMode::Normal => handle_standard_normal_input(key, app, tx).await,
    InputMode::Editing => handle_standard_editing_input(key, app),
  }
}

async fn handle_standard_normal_input(
  key: KeyEvent,
  app: &mut App,
  tx: mpsc::Sender<Action>,
) -> bool {
  match key.code {
    KeyCode::Char('q') => {
      app.quit();
      false
    }
    KeyCode::Char('1') => {
      app.current_page = CurrentPage::Compose;
      false
    }
    KeyCode::Char('2') => {
      app.current_page = CurrentPage::Config;
      false
    }
    KeyCode::Char('3') => {
      app.current_page = CurrentPage::Dashboard;
      trigger_fetch(app, tx.clone());
      false
    }
    KeyCode::Tab => {
      app.cycle_field();
      false
    }
    KeyCode::Enter => handle_enter_action(app, tx).await,
    KeyCode::Char('s')
      if key
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL) =>
    {
      if let Err(e) = Storage::save_config(&app.config) {
        app.set_notification(Notification::Error(e.to_string()));
      } else {
        app.set_notification(Notification::Success("Config saved".to_string()));
      }
      false
    }
    _ => false,
  }
}

async fn handle_enter_action(app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  if app.current_page == CurrentPage::Config {
    app.toggle_editing();
    return false;
  }
  if app.current_page == CurrentPage::Dashboard {
    return false;
  }

  match app.compose_field {
    ComposeField::Body => {
      let _ = execute!(io::stdout(), LeaveAlternateScreen);
      let _ = disable_raw_mode();

      let new_body = open_external_editor(&app.draft.body);

      let _ = enable_raw_mode();
      let _ = execute!(io::stdout(), EnterAlternateScreen);

      match new_body {
        Ok(content) => {
          app.draft.body = content;
          let _ = Storage::save_draft(&app.draft);
        }
        Err(e) => {
          app.set_notification(Notification::Error(format!("Editor error: {}", e)));
        }
      }
      true
    }
    ComposeField::SendButton => {
      app.set_notification(Notification::Info("Sending email...".to_string()));
      let draft = app.draft.clone();
      let config = app.config.clone();
      let tx_clone = tx.clone();

      tokio::spawn(async move {
        match mailer::send_email(config, draft).await {
          Ok(_) => tx_clone.send(Action::EmailSent).await.unwrap(),
          Err(e) => tx_clone
            .send(Action::EmailFailed(e.to_string()))
            .await
            .unwrap(),
        }
      });
      false
    }
    _ => {
      app.toggle_editing();
      false
    }
  }
}

fn handle_standard_editing_input(key: KeyEvent, app: &mut App) -> bool {
  match key.code {
    KeyCode::Esc => {
      app.toggle_editing();
      if app.current_page == CurrentPage::Compose {
        let _ = Storage::save_draft(&app.draft);
      }
    }
    KeyCode::Enter => {
      app.toggle_editing();
      app.cycle_field();
      if app.current_page == CurrentPage::Compose {
        let _ = Storage::save_draft(&app.draft);
      }
    }
    KeyCode::Char(c) => app.push_input(c),
    KeyCode::Backspace => app.pop_input(),
    _ => {}
  }
  false
}

// HELPER FUNCTIONS

fn trigger_fetch(app: &mut App, tx: mpsc::Sender<Action>) {
  app.set_notification(Notification::Info("Fetching logs...".to_string()));
  let url = app.config.worker_url.clone();
  let secret = app.config.api_secret.clone();
  let tx_logs = tx.clone();

  tokio::spawn(async move {
    match client::fetch_logs(&url, &secret).await {
      Ok(logs) => tx_logs.send(Action::LogsFetched(logs)).await.unwrap(),
      Err(e) => tx_logs
        .send(Action::LogsFailed(e.to_string()))
        .await
        .unwrap(),
    }
  });

  let url2 = app.config.worker_url.clone();
  let sec2 = app.config.api_secret.clone();

  let tx_filters = tx.clone();

  tokio::spawn(async move {
    match client::fetch_filters(&url2, &sec2).await {
      Ok(f) => tx_filters.send(Action::FiltersFetched(f)).await.unwrap(),
      Err(e) => tx_filters
        .send(Action::FiltersFailed(e.to_string()))
        .await
        .unwrap(),
    }
  });
}

fn open_external_editor(initial_text: &str) -> io::Result<String> {
  let mut temp_file = tempfile::Builder::new().suffix(".md").tempfile()?;
  write!(temp_file, "{}", initial_text)?;
  let mut candidates = Vec::new();
  if let Ok(v) = std::env::var("VISUAL") {
    candidates.push(v);
  }
  if let Ok(e) = std::env::var("EDITOR") {
    candidates.push(e);
  }
  if cfg!(target_os = "windows") {
    candidates.push("notepad.exe".to_string());
    candidates.push("code.cmd".to_string());
  } else {
    candidates.push("nano".to_string());
    candidates.push("vim".to_string());
    candidates.push("vi".to_string());
    candidates.push("editor".to_string());
    candidates.push("xdg-open".to_string());
  }

  for editor in candidates {
    let result = Command::new(&editor).arg(temp_file.path()).status();

    match result {
      Ok(status) => {
        if status.success() {
          let content = std::fs::read_to_string(temp_file.path())?;
          return Ok(content);
        } else {
          continue;
        }
      }
      Err(e) => {
        if e.kind() == io::ErrorKind::NotFound {
          continue;
        }
        continue;
      }
    }
  }

  Err(io::Error::new(
    io::ErrorKind::NotFound,
    "No suitable editor found (tried $EDITOR, nano, vim, vi, notepad).",
  ))
}
