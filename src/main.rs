mod app;
mod models;
mod tui;

use anyhow::Result;
use app::App;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

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
        terminal.draw(|frame| ui(frame, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => app.quit(),
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn ui(frame: &mut Frame, _app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)])
        .split(frame.area());

    let placeholder = Paragraph::new("Shiryoku Initialized. Press 'q' to quit.")
        .block(Block::default().title("Status").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));

    frame.render_widget(placeholder, chunks[0]);
}
