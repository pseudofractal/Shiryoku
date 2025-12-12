mod app;
mod compiler;
mod config;
mod models;
mod tui;
mod ui;

use anyhow::Result;
use app::{App, ComposeField, ConfigField, CurrentPage, InputMode};
use config::AppConfig;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};

#[tokio::main]
async fn main() -> Result<()> {
    // Load config or use default
    let config = AppConfig::load().unwrap_or_default();

    let mut terminal = tui::init()?;
    let mut app = App::new(config);

    let result = run_app(&mut terminal, &mut app).await;

    tui::restore()?;
    result
}

async fn run_app(terminal: &mut tui::Tui, app: &mut App) -> Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| ui::draw(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match app.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('q') => app.quit(),
                        KeyCode::Char('1') => app.current_page = CurrentPage::Compose,
                        KeyCode::Char('2') => app.current_page = CurrentPage::Config,
                        KeyCode::Tab => app.cycle_field(),
                        KeyCode::Enter => app.toggle_editing(),
                        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            if let Err(e) = app.config.save() {
                                // In a real app we would show an error popup
                                eprintln!("Failed to save config: {}", e);
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
                        KeyCode::Char(c) => handle_input(app, c),
                        KeyCode::Backspace => handle_backspace(app),
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}

fn handle_input(app: &mut App, c: char) {
    match app.current_page {
        CurrentPage::Compose => match app.compose_field {
            ComposeField::Recipient => app.draft.recipient.push(c),
            ComposeField::Subject => app.draft.subject.push(c),
            ComposeField::Body => app.draft.body.push(c),
        },
        CurrentPage::Config => match app.config_field {
            ConfigField::Name => app.config.identity.name.push(c),
            ConfigField::Role => app.config.identity.role.push(c),
            ConfigField::Department => app.config.identity.department.push(c),
            ConfigField::Institution => app.config.identity.institution.push(c),
            ConfigField::Phone => app.config.identity.phone.push(c),
            ConfigField::Emails => {
                // We edit a string in memory, later split it when using
                // For now, we need to rebuild the vector or just store a temporary string
                // But since we bound the UI directly to the vector join, we need a smarter way.
                // Simpler approach for this step: Only append to last email or create new if empty?
                // ACTUALLY: Let's simplify and assume the user edits the raw string of the FIRST email for now
                // or we add a helper to parse comma separated strings back into the vec.
                // Let's implement a quick helper here.
                let mut current = app.config.identity.emails.join(", ");
                current.push(c);
                app.config.identity.emails =
                    current.split(',').map(|s| s.trim().to_string()).collect();
            }
            ConfigField::SmtpUser => app.config.smtp_username.push(c),
            ConfigField::SmtpPass => app.config.smtp_app_password.push(c),
            ConfigField::WorkerUrl => app.config.worker_url.push(c),
        },
    }
}

fn handle_backspace(app: &mut App) {
    match app.current_page {
        CurrentPage::Compose => match app.compose_field {
            ComposeField::Recipient => {
                app.draft.recipient.pop();
            }
            ComposeField::Subject => {
                app.draft.subject.pop();
            }
            ComposeField::Body => {
                app.draft.body.pop();
            }
        },
        CurrentPage::Config => match app.config_field {
            ConfigField::Name => {
                app.config.identity.name.pop();
            }
            ConfigField::Role => {
                app.config.identity.role.pop();
            }
            ConfigField::Department => {
                app.config.identity.department.pop();
            }
            ConfigField::Institution => {
                app.config.identity.institution.pop();
            }
            ConfigField::Phone => {
                app.config.identity.phone.pop();
            }
            ConfigField::Emails => {
                let mut current = app.config.identity.emails.join(", ");
                current.pop();
                // Filter empty strings to avoid [""] when clearing
                app.config.identity.emails = if current.is_empty() {
                    Vec::new()
                } else {
                    current.split(',').map(|s| s.trim().to_string()).collect()
                };
            }
            ConfigField::SmtpUser => {
                app.config.smtp_username.pop();
            }
            ConfigField::SmtpPass => {
                app.config.smtp_app_password.pop();
            }
            ConfigField::WorkerUrl => {
                app.config.worker_url.pop();
            }
        },
    }
}
