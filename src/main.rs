mod app;
mod client;
mod compiler;
mod config;
mod enums;
mod handler;
mod mailer;
mod models;
mod storage;
mod tui;
mod ui;

use anyhow::Result;
use app::App;
use crossterm::event::{self, Event, KeyEventKind};
use enums::Notification;
use handler::Action;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
  let mut terminal = tui::init()?;
  let mut app = App::new();

  let (tx, mut rx) = mpsc::channel(10);

  let tick_rate = std::time::Duration::from_millis(250);
  let tx_tick = tx.clone();
  tokio::spawn(async move {
    loop {
      tokio::time::sleep(tick_rate).await;
      if tx_tick.send(Action::RenderTick).await.is_err() {
        break;
      }
    }
  });

  loop {
    terminal.draw(|frame| ui::draw(frame, &app))?;
    if let Ok(action) = rx.try_recv() {
      match action {
        Action::RenderTick => {}
        Action::EmailSent => {
          app.set_notification(Notification::Success(
            "Email sent successfully!".to_string(),
          ));
        }
        Action::EmailFailed(err) => {
          app.set_notification(Notification::Error(format!("Sending failed: {}", err)));
        }
        Action::LogsFetched(logs) => {
          app.logs = logs;
          app.set_notification(Notification::Success("Logs updated".to_string()));
        }
        Action::LogsFailed(err) => {
          app.set_notification(Notification::Error(format!("Fetch failed: {}", err)));
        }
        Action::FiltersFetched(filters) => {
          app.filter_options = filters;
        }
        Action::FiltersFailed(err) => {
          app.set_notification(Notification::Error(format!("Filters failed: {}", err)));
        }
      }
    }
    if event::poll(std::time::Duration::from_millis(10))? {
      if let Event::Key(key) = event::read()? {
        if key.kind == KeyEventKind::Press {
          if app.notification.is_some() {
            app.clear_notification();
          }

          let should_clear = handler::handle_key_events(key, &mut app, tx.clone()).await;

          if should_clear {
            terminal.clear()?;
          }

          if app.should_quit {
            break;
          }
        }
      }
    }
  }

  tui::restore()?;
  Ok(())
}
