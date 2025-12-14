use crate::app::App;
use crate::client;
use crate::enums::{InputMode, Notification, ScheduledFocus};
use crate::handler::Action;
use crossterm::event::{KeyCode, KeyEvent};
use directories::UserDirs;
use std::fs;
use tokio::sync::mpsc;

pub async fn handle_scheduled_events(
  key: KeyEvent,
  app: &mut App,
  tx: mpsc::Sender<Action>,
) -> bool {
  if let Some(selected_id) = app.scheduled.selected_job_id.clone() {
    // Detail View Mode
    match key.code {
      KeyCode::Esc | KeyCode::Char('q') => {
        app.scheduled.selected_job_id = None;
        return true;
      }
      KeyCode::Char('x') => {
        if let Some(_job) = app.scheduled.jobs.iter().find(|j| j.id == selected_id) {
          let url = app.config.data.worker_url.clone();
          let secret = app.config.data.api_secret.clone();
          let id = selected_id.clone();
          let tx_cancel = tx.clone();

          app.set_notification(Notification::Info("Deleting Job...".to_string()));

          tokio::spawn(async move {
            match client::cancel_scheduled_job(&url, &secret, &id).await {
              Ok(_) => tx_cancel.send(Action::JobCancelled(id)).await.unwrap(),
              Err(e) => tx_cancel
                .send(Action::JobActionFailed(e.to_string()))
                .await
                .unwrap(),
            }
          });
          app.scheduled.selected_job_id = None; // Close popup
        }
      }
      KeyCode::Char('d') => {
        if let Some(job) = app.scheduled.jobs.iter().find(|j| j.id == selected_id) {
          if let Some(user_dirs) = UserDirs::new() {
            if let Some(dl_dir) = user_dirs.download_dir() {
              let safe_recipient = job.recipient.replace(|c: char| !c.is_alphanumeric(), "_");
              let target_dir = dl_dir.join(&safe_recipient);

              if let Err(e) = fs::create_dir_all(&target_dir) {
                app.set_notification(Notification::Error(format!("FS Error: {}", e)));
                return false;
              }

              // Metadata
              let meta_path = target_dir.join("metadata.json");
              let meta_content = serde_json::to_string_pretty(job).unwrap_or_default();
              let _ = fs::write(meta_path, meta_content);

              // Body
              let body_path = target_dir.join("body.txt");
              let _ = fs::write(body_path, &job.body);

              // Attachments
              for att in &job.attachments {
                let att_path = target_dir.join(&att.filename);
                let _ = fs::write(att_path, &att.content);
              }

              app.set_notification(Notification::Success(format!(
                "Downloaded to Downloads/{}",
                safe_recipient
              )));
            }
          }
        }
      }
      _ => {}
    }
    return true;
  }

  // Normal List/Filter Mode
  match app.input_mode {
    InputMode::Normal => match key.code {
      KeyCode::Char('q') => {
        app.quit();
        false
      }
      KeyCode::Tab => {
        app.scheduled.cycle_focus(true);
        true
      }
      KeyCode::BackTab => {
        app.scheduled.cycle_focus(false);
        true
      }
      KeyCode::Enter => {
        match app.scheduled.focus {
          ScheduledFocus::List => {
            if let Some(idx) = app.scheduled.list_state.selected() {
              let filtered = app.scheduled.get_filtered_jobs();
              if let Some(job) = filtered.get(idx) {
                app.scheduled.selected_job_id = Some(job.id.clone());
              }
            }
          }
          ScheduledFocus::FilterStatus => {
            app.scheduled.toggle_status_filter();
          }
          ScheduledFocus::FilterRecipient => {
            app.toggle_editing();
          }
        }
        true
      }
      KeyCode::Down if app.scheduled.focus == ScheduledFocus::List => {
        let i = match app.scheduled.list_state.selected() {
          Some(i) => {
            if i >= app.scheduled.get_filtered_jobs().len().saturating_sub(1) {
              0
            } else {
              i + 1
            }
          }
          None => 0,
        };
        app.scheduled.list_state.select(Some(i));
        true
      }
      KeyCode::Up if app.scheduled.focus == ScheduledFocus::List => {
        let i = match app.scheduled.list_state.selected() {
          Some(i) => {
            if i == 0 {
              app.scheduled.get_filtered_jobs().len().saturating_sub(1)
            } else {
              i - 1
            }
          }
          None => 0,
        };
        app.scheduled.list_state.select(Some(i));
        true
      }
      // Global Nav
      KeyCode::Char('1') => {
        app.current_page = crate::enums::CurrentPage::Compose;
        false
      }
      KeyCode::Char('2') => {
        app.current_page = crate::enums::CurrentPage::Config;
        false
      }
      KeyCode::Char('3') => {
        app.current_page = crate::enums::CurrentPage::Dashboard;
        false
      }
      KeyCode::Char('4') => {
        app.current_page = crate::enums::CurrentPage::Scheduled;
        false
      } // Current Schedule page
      KeyCode::Char('r') => {
        crate::handler::helper::trigger_fetch_jobs(app, tx);
        false
      }
      _ => false,
    },
    InputMode::Editing => match key.code {
      KeyCode::Esc | KeyCode::Enter => {
        app.toggle_editing();
        true
      }
      KeyCode::Char(c) => {
        app.scheduled.handle_input(c);
        true
      }
      KeyCode::Backspace => {
        app.scheduled.handle_backspace();
        true
      }
      _ => false,
    },
  }
}
