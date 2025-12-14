use crate::app::App;
use crate::enums::CurrentPage;
use ratatui::{
  Frame,
  layout::Rect,
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Tabs},
};

pub fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
  let titles = vec![
    " [1] Compose ",
    " [2] Config ",
    " [3] Dashboard ",
    " [4] Scheduled ",
  ];
  let tabs = Tabs::new(titles)
    .block(Block::default().borders(Borders::ALL).title(" Shiryoku "))
    .select(match app.current_page {
      CurrentPage::Compose => 0,
      CurrentPage::Config => 1,
      CurrentPage::Dashboard => 2,
      CurrentPage::Schedule => 0,
      CurrentPage::Scheduled => 3,
    })
    .highlight_style(
      Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD),
    );

  frame.render_widget(tabs, area);
}
