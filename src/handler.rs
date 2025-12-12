use crate::app::App;
use crate::enums::{ComposeField, CurrentPage, InputMode, Notification};
use crate::mailer;
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
}

pub async fn handle_key_events(key: KeyEvent, app: &mut App, tx: mpsc::Sender<Action>) -> bool {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => {
                app.quit();
                false
            }
            KeyCode::Char('1') => {
                app.switch_page();
                false
            }
            KeyCode::Char('2') => {
                app.switch_page();
                false
            }
            KeyCode::Tab => {
                app.cycle_field();
                false
            }

            KeyCode::Enter => {
                if app.current_page == CurrentPage::Config {
                    app.toggle_editing();
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
                                app.set_notification(Notification::Error(format!(
                                    "Editor error: {}",
                                    e
                                )));
                            }
                        }
                        true // Return true to signal screen clear
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
        },
        InputMode::Editing => {
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
    }
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
