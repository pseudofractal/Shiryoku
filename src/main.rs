mod app;
mod models;
mod tui;
mod ui;

use anyhow::Result;
use app::{App, ComposeField, InputMode};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};

#[tokio::main]
async fn main() -> Result<()> {
    let mut terminal = tui::init()?;
    let mut app = App::new();

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
                        KeyCode::Tab => app.cycle_field(),
                        KeyCode::Enter => app.toggle_editing(),
                        _ => {}
                    },
                    InputMode::Editing => match key.code {
                        KeyCode::Esc => app.toggle_editing(),
                        KeyCode::Char(c) => handle_input(app, c),
                        KeyCode::Backspace => handle_backspace(app),
                        KeyCode::Enter => {
                            if app.current_field == ComposeField::Body {
                                handle_input(app, '\n');
                            } else {
                                app.toggle_editing();
                                app.cycle_field();
                            }
                        }
                        _ => {}
                    },
                }
            }
        }
    }
    Ok(())
}

fn handle_input(app: &mut App, c: char) {
    match app.current_field {
        ComposeField::Recipient => app.draft.recipient.push(c),
        ComposeField::Subject => app.draft.subject.push(c),
        ComposeField::Body => app.draft.body.push(c),
    }
}

fn handle_backspace(app: &mut App) {
    match app.current_field {
        ComposeField::Recipient => {
            app.draft.recipient.pop();
        }
        ComposeField::Subject => {
            app.draft.subject.pop();
        }
        ComposeField::Body => {
            app.draft.body.pop();
        }
    }
}
