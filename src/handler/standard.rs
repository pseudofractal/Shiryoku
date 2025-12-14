use crate::app::App;
use crate::enums::{ComposeField, CurrentPage, InputMode, Notification};
use crate::handler::{Action, helper};
use crate::mailer;
use crate::storage::Storage;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use crossterm::{
  execute,
  terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use std::io;
use tokio::sync::mpsc;

pub async fn handle_standard_events(
  key: KeyEvent,
  app: &mut App,
  tx: mpsc::Sender<Action>,
) -> bool {
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
      helper::trigger_fetch(app, tx.clone());
      false
    }
    KeyCode::Char('4') => {
      helper::trigger_fetch_jobs(app, tx.clone());
      false
    }
    KeyCode::Tab => {
      app.cycle_field();
      false
    }
    KeyCode::BackTab => {
      app.cycle_field_backward();
      false
    }
    KeyCode::Enter => handle_enter_action(app, tx).await,

    KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      if app.current_page == CurrentPage::Compose && app.compose.field == ComposeField::Attachments
      {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let files = rfd::FileDialog::new()
          .set_title("Select Attachments")
          .pick_files();
        let _ = enable_raw_mode();
        let _ = execute!(io::stdout(), EnterAlternateScreen);
        if let Some(paths) = files {
          let paths_vec: Vec<String> = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();

          if !app.compose.attachment_input.is_empty() {
            app.compose.attachment_input.push_str("; ");
          }
          app.compose.attachment_input.push_str(&paths_vec.join("; "));
          app.set_notification(Notification::Success(format!(
            "Added {} files",
            paths.len()
          )));
        }
        true
      } else {
        false
      }
    }

    KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      if app.current_page == CurrentPage::Compose && app.compose.field == ComposeField::Attachments
      {
        app.compose.attachment_input.clear();
        app.compose.sync_attachments();
        let _ = Storage::save_draft(&app.compose.draft);
        app.set_notification(Notification::Info("Attachments cleared".to_string()));
        true
      } else {
        false
      }
    }

    KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
      if app.current_page == CurrentPage::Compose {
        app.reset_schedule_modal();
        app.current_page = CurrentPage::Schedule;
        true
      } else {
        if let Err(e) = Storage::save_config(&app.config.data) {
          app.set_notification(Notification::Error(e.to_string()));
        } else {
          app.set_notification(Notification::Success("Config saved".to_string()));
        }
        false
      }
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
  if app.current_page == CurrentPage::Scheduled {
    return false;
  }

  match app.compose.field {
    ComposeField::Body => {
      let _ = execute!(io::stdout(), LeaveAlternateScreen);
      let _ = disable_raw_mode();

      let new_body = helper::open_external_editor(&app.compose.draft.body);

      let _ = enable_raw_mode();
      let _ = execute!(io::stdout(), EnterAlternateScreen);

      match new_body {
        Ok(content) => {
          app.compose.draft.body = content;
          let _ = Storage::save_draft(&app.compose.draft);
        }
        Err(e) => {
          app.set_notification(Notification::Error(format!("Editor error: {}", e)));
        }
      }
      true
    }
    ComposeField::SendButton => {
      app.set_notification(Notification::Info("Sending email...".to_string()));
      let mut draft = app.compose.draft.clone();
      draft.attachments = app
        .compose
        .attachment_input
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(std::path::PathBuf::from)
        .collect();
      let config = app.config.data.clone();
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
        app.compose.sync_attachments();
        let _ = Storage::save_draft(&app.compose.draft);
      }
    }
    KeyCode::Enter => {
      app.toggle_editing();
      app.cycle_field();
      if app.current_page == CurrentPage::Compose {
        app.compose.sync_attachments();
        let _ = Storage::save_draft(&app.compose.draft);
      }
    }
    KeyCode::BackTab => {
      app.toggle_editing();
      app.cycle_field_backward();
    }
    KeyCode::Tab => {
      app.toggle_editing();
      app.cycle_field();
    }
    KeyCode::Char(c) => app.push_input(c),
    KeyCode::Backspace => app.pop_input(),
    _ => {}
  }
  false
}
