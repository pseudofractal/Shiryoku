use anyhow::Result;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};

pub type Tui = Terminal<CrosstermBackend<Stdout>>;

pub fn init() -> Result<Tui> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let backend = CrosstermBackend::new(io::stdout());
    Terminal::new(backend).map_err(Into::into)
}

pub fn restore() -> Result<()> {
    execute!(io::stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
