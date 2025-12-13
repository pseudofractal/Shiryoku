use crate::app::App;
use crate::enums::{InputMode, Notification, ScheduleField};
use crate::handler::Action;
use crate::storage::Storage;
use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc;

pub async fn handle_schedule_input(
  key: KeyEvent,
  app: &mut App,
  _tx: mpsc::Sender<Action>,
) -> bool {
  match app.input_mode {
    InputMode::Normal => handle_normal_mode(key, app).await,
    InputMode::Editing => handle_editing_mode(key, app).await,
  }
}

async fn handle_normal_mode(key: KeyEvent, app: &mut App) -> bool {
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
        app.set_notification(Notification::Info(
          "Scheduling logic pending...".to_string(),
        ));
        app.current_page = crate::enums::CurrentPage::Compose;
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
