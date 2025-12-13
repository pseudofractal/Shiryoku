pub mod compose;
pub mod config;
pub mod dashboard;
pub mod schedule;
pub mod statusbar;
pub mod tabs;

use crate::app::App;
use crate::enums::{CurrentPage, InputMode};
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
};

pub fn draw(frame: &mut Frame, app: &App) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Tabs/Title
      Constraint::Min(0),    // Main Content
      Constraint::Length(3), // Status Bar
    ])
    .split(frame.area());

  tabs::draw_tabs(frame, app, chunks[0]);

  match app.current_page {
    CurrentPage::Compose => compose::draw_compose(frame, app, chunks[1]),
    CurrentPage::Config => config::draw_config(frame, app, chunks[1]),
    CurrentPage::Dashboard => dashboard::draw_dashboard(frame, app, chunks[1]),
    CurrentPage::Schedule => schedule::draw_schedule_page(frame, app, chunks[1]), // CHANGED
  }

  statusbar::draw_status_bar(frame, app, chunks[2]);
}

pub(crate) fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
  let v_constraints = if percent_y < 100 {
    vec![
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ]
  } else {
    let gap = r.height.saturating_sub(percent_y) / 2;
    vec![
      Constraint::Length(gap),
      Constraint::Length(percent_y),
      Constraint::Length(gap),
    ]
  };

  let vertical = Layout::default()
    .direction(Direction::Vertical)
    .constraints(v_constraints)
    .split(r);

  let h_constraints = if percent_x < 100 {
    vec![
      Constraint::Percentage((100 - percent_x) / 2),
      Constraint::Percentage(percent_x),
      Constraint::Percentage((100 - percent_x) / 2),
    ]
  } else {
    let gap = r.width.saturating_sub(percent_x) / 2;
    vec![
      Constraint::Length(gap),
      Constraint::Length(percent_x),
      Constraint::Length(gap),
    ]
  };

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints(h_constraints)
    .split(vertical[1])[1]
}

pub(crate) fn get_field_styles<T, F>(app: &App, fields: &[T], is_current: F) -> Vec<Style>
where
  F: Fn(&T) -> bool,
{
  fields
    .iter()
    .map(|f| {
      if is_current(f) {
        match app.input_mode {
          InputMode::Editing => Style::default().fg(Color::Yellow),
          InputMode::Normal => Style::default().fg(Color::Green),
        }
      } else {
        Style::default().fg(Color::White)
      }
    })
    .collect()
}
