use crate::app::App;
use crate::enums::{ComposeField, CurrentPage, InputMode, Notification};
use ratatui::{
  Frame,
  layout::Rect,
  style::{Color, Modifier, Style},
  widgets::Paragraph,
};

pub fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
  let (text, color) = if let Some(note) = &app.notification {
    match note {
      Notification::Info(msg) => (format!(" ℹ️  {} ", msg), Color::Blue),
      Notification::Success(msg) => (format!(" ✅ {} ", msg), Color::Green),
      Notification::Error(msg) => (format!(" ❌ {} ", msg), Color::Red),
    }
  } else {
    match app.input_mode {
      InputMode::Normal => {
        let base_help = " [Tab] Next | [Enter] Edit | [Ctrl+Enter] Send | [1/2/3] Nav | [q] Quit";
        let extra_help = if app.current_page == CurrentPage::Compose
          && app.compose.field == ComposeField::Attachments
        {
          " | [Ctrl+o] Browse Files | [Ctrl+x] Clear"
        } else {
          ""
        };
        (format!("{}{}", base_help, extra_help), Color::DarkGray)
      }
      InputMode::Editing => (
        " Editing Mode: [Esc] Finish Editing".to_string(),
        Color::Yellow,
      ),
    }
  };

  let status = Paragraph::new(text).style(Style::default().fg(color).add_modifier(Modifier::BOLD));
  frame.render_widget(status, area);
}
