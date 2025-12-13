use crate::app::App;
use crate::enums::{InputMode, Notification, ScheduleField};
use crate::handler::Action;
use crate::storage::Storage;
use crate::{client, compiler};
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

pub async fn handle_schedule_input(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  match app.input_mode {
    InputMode::Normal => handle_normal_mode(key, app, tx).await,
    InputMode::Editing => handle_editing_mode(key, app).await,
  }
}

async fn handle_normal_mode(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
  match key.code {
    KeyCode::Char('q') => {
      app.quit();
      false
    }
    KeyCode::Esc => {
      app.sync_schedule_to_draft();
      let _ = Storage::save_draft(&app.compose.draft);
      app.current_page = crate::enums::CurrentPage::Compose;
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
    KeyCode::Enter => {
      if app.schedule.active_field == ScheduleField::Submit {
        app.sync_schedule_to_draft();
        let _ = Storage::save_draft(&app.compose.draft);

        if let Some(utc_target) = app.schedule.calculate_utc_target() {
          app.set_notification(Notification::Info("Scheduling...".to_string()));

          let draft_clone = app.compose.draft.clone();
          let identity_clone = app.config.data.identity.clone();
          let worker_url = app.config.data.worker_url.clone();
          let api_secret = app.config.data.api_secret.clone();
          let smtp_user = app.config.data.smtp_username.clone();
          let smtp_pass = app.config.data.smtp_app_password.clone();

          let tx_sched = tx.clone();

          tokio::spawn(async move {
            let compiled = compiler::compile(&draft_clone, &identity_clone, &worker_url);

            match client::schedule_email(
              &worker_url,
              &api_secret,
              compiled,
              draft_clone.subject,
              draft_clone.recipient,
              utc_target,
              smtp_user,
              smtp_pass,
              identity_clone.name,
            )
            .await
            {
              Ok(_) => {
                tx_sched.send(Action::EmailSent).await.unwrap();
              }
              Err(e) => {
                tx_sched
                  .send(Action::EmailFailed(format!("Schedule Error: {}", e)))
                  .await
                  .unwrap();
              }
            }
          });

          app.current_page = crate::enums::CurrentPage::Dashboard;
        } else {
          app.set_notification(Notification::Error("Invalid Date/Time".to_string()));
        }
      } else {
        app.schedule.clear_current_field();
        app.toggle_editing();
      }
      false
    }
    KeyCode::Down if app.schedule.active_field == ScheduleField::Timezone => {
      if !app.schedule.filtered_timezones.is_empty()
        && app.schedule.selected_timezone_idx
          < app.schedule.filtered_timezones.len().saturating_sub(1)
      {
        app.schedule.selected_timezone_idx += 1;
      }
      false
    }
    KeyCode::Up if app.schedule.active_field == ScheduleField::Timezone => {
      if app.schedule.selected_timezone_idx > 0 {
        app.schedule.selected_timezone_idx -= 1;
      }
      false
    }
    _ => false,
  }
}

async fn handle_editing_mode(key: KeyEvent, app: &mut App) -> bool {
  match key.code {
    KeyCode::Esc | KeyCode::Enter => {
      app.toggle_editing();
      if app.schedule.active_field == ScheduleField::Timezone {
        if let Some(tz) = app
          .schedule
          .filtered_timezones
          .get(app.schedule.selected_timezone_idx)
        {
          app.schedule.timezone_input = tz.clone();
        }
      }
      false
    }
    KeyCode::Tab => {
      app.toggle_editing();
      app.cycle_field();
      false
    }
    KeyCode::BackTab => {
      app.toggle_editing();
      app.cycle_field_backward();
      false
    }
    KeyCode::Char(c) => {
      app.push_input(c);
      false
    }
    KeyCode::Backspace => {
      app.pop_input();
      false
    }
    _ => false,
  }
}
