use crate::app::App;
use crate::client;
use crate::enums::{CurrentPage, DashboardFocus, InputMode, Notification};
use crate::handler::{Action, helper};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

pub async fn handle_dashboard_events(
  key: KeyEvent,
  app: &mut App,
  tx: mpsc::Sender<Action>,
) -> bool {
  if let Some(selected_id) = &app.dashboard.selected_summary_id {
    if key.code == KeyCode::Char('d')
      && key
        .modifiers
        .contains(crossterm::event::KeyModifiers::CONTROL)
    {
      let tracking_id = selected_id.clone();
      let url = app.config.data.worker_url.clone();
      let secret = app.config.data.api_secret.clone();
      let tx_delete = tx.clone();
      app.set_notification(Notification::Info("Deleting logs...".to_string()));
      tokio::spawn(async move {
        match client::delete_recipient_logs(&url, &secret, &tracking_id).await {
          Ok(_) => tx_delete
            .send(Action::LogsDeleted(tracking_id))
            .await
            .unwrap(),
          Err(e) => tx_delete
            .send(Action::LogsFailed(e.to_string()))
            .await
            .unwrap(),
        }
      });
      return false;
    }
    if matches!(key.code, KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q')) {
      app.dashboard.selected_summary_id = None;
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
    KeyCode::BackTab => {
      app.cycle_field_backward();
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
      helper::trigger_fetch(app, tx);
      false
    }
    KeyCode::Char('4') => {
      helper::trigger_fetch_jobs(app, tx);
      false
    }
    KeyCode::Char('r') => {
      helper::trigger_fetch(app, tx);
      false
    }
    KeyCode::Enter => {
      if app.dashboard.focus == DashboardFocus::List {
        if let Some(idx) = app.dashboard.list_state.selected() {
          let summaries = app.dashboard.get_aggregated_logs();
          if let Some(s) = summaries.get(idx) {
            app.dashboard.selected_summary_id = Some(s.tracking_id.clone());
          }
        }
      } else {
        app.toggle_editing();
      }
      true
    }
    KeyCode::Down => {
      if app.dashboard.focus == DashboardFocus::List {
        let i = match app.dashboard.list_state.selected() {
          Some(i) => {
            if i >= app.dashboard.get_aggregated_logs().len().saturating_sub(1) {
              0
            } else {
              i + 1
            }
          }
          None => 0,
        };
        app.dashboard.list_state.select(Some(i));
        return true;
      }
      false
    }
    KeyCode::Up => {
      if app.dashboard.focus == DashboardFocus::List {
        let i = match app.dashboard.list_state.selected() {
          Some(i) => {
            if i == 0 {
              app.dashboard.get_aggregated_logs().len().saturating_sub(1)
            } else {
              i - 1
            }
          }
          None => 0,
        };
        app.dashboard.list_state.select(Some(i));
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
    KeyCode::Tab => {
      app.toggle_editing();
      app.cycle_field();
    }
    KeyCode::BackTab => {
      app.toggle_editing();
      app.cycle_field_backward();
    }
    KeyCode::Char(c) => app.push_input(c),
    KeyCode::Backspace => app.pop_input(),
    _ => {}
  }
  true
}
