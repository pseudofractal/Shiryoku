use crate::app::App;
use crate::enums::{CurrentPage, InputMode, Notification};
use crate::mailer;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use tokio::sync::mpsc;

pub enum Action {
    RenderTick,
    EmailSent,
    EmailFailed(String),
}

pub async fn handle_key_events(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => app.quit(),
            KeyCode::Char('1') => app.switch_page(),
            KeyCode::Char('2') => app.switch_page(),
            KeyCode::Tab => app.cycle_field(),

            // CTRL+Enter: Send Email
            KeyCode::Enter if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if app.current_page == CurrentPage::Compose {
                    app.set_notification(Notification::Info("Sending email...".to_string()));

                    let draft = app.draft.clone();
                    let config = app.config.clone();

                    tokio::spawn(async move {
                        match mailer::send_email(config, draft).await {
                            Ok(_) => tx.send(Action::EmailSent).await.unwrap(),
                            Err(e) => tx.send(Action::EmailFailed(e.to_string())).await.unwrap(),
                        }
                    });
                }
            }

            // Normal Enter: Edit Mode
            KeyCode::Enter => app.toggle_editing(),

            // CTRL+S: Save Config
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                if let Err(e) = app.config.save() {
                    app.set_notification(Notification::Error(e.to_string()));
                } else {
                    app.set_notification(Notification::Success("Config saved".to_string()));
                }
            }
            _ => {}
        },
        InputMode::Editing => match key.code {
            KeyCode::Esc => app.toggle_editing(),
            KeyCode::Enter => {
                app.toggle_editing();
                app.cycle_field();
            }
            KeyCode::Char(c) => app.push_input(c),
            KeyCode::Backspace => app.pop_input(),
            _ => {}
        },
    }
}
