use crate::app::App;
use crate::enums::ComposeField;
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Paragraph},
};

pub fn draw_compose(frame: &mut Frame, app: &App, area: Rect) {
  let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // To
      Constraint::Length(3), // Subject
      Constraint::Length(3), // Attachments
      Constraint::Min(5),    // Body
      Constraint::Length(3), // Send Button
      Constraint::Length(1), // Spacer
      Constraint::Length(1), // Schedule Hint
    ])
    .split(area);

  let styles = super::get_field_styles(
    app,
    &[
      ComposeField::Recipient,
      ComposeField::Subject,
      ComposeField::Attachments,
      ComposeField::Body,
      ComposeField::SendButton,
    ],
    |f| app.compose.field == *f,
  );

  // 1. Recipient
  let recipient = Paragraph::new(app.compose.draft.recipient.as_str())
    .block(Block::default().borders(Borders::ALL).title("To"))
    .style(styles[0]);
  frame.render_widget(recipient, layout[0]);

  // 2. Subject
  let subject = Paragraph::new(app.compose.draft.subject.as_str())
    .block(Block::default().borders(Borders::ALL).title("Subject"))
    .style(styles[1]);
  frame.render_widget(subject, layout[1]);

  // 3. Attachments
  let attach_widget = Paragraph::new(app.compose.attachment_input.as_str())
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Attachments (semicolon separated paths)"),
    )
    .style(styles[2]);
  frame.render_widget(attach_widget, layout[2]);

  // 4. Body
  let body_content = if app.compose.draft.body.is_empty() {
    "Press <Enter> to open external editor...".to_string()
  } else {
    app.compose.draft.body.clone()
  };
  let body = Paragraph::new(body_content)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Body (Markdown) - Press Enter to Edit"),
    )
    .style(styles[3]);
  frame.render_widget(body, layout[3]);

  // 5. Send Button
  let button_text = if app.compose.field == ComposeField::SendButton {
    "> [SEND EMAIL] <"
  } else {
    "  [SEND EMAIL]  "
  };
  let send_btn = Paragraph::new(button_text)
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::ALL))
    .style(styles[4]);
  frame.render_widget(send_btn, layout[4]);

  // 6. Schedule Hint
  let schedule_hint = Paragraph::new("Tip: Press [Ctrl + s] to Schedule Send")
    .style(
      Style::default()
        .fg(Color::DarkGray)
        .add_modifier(Modifier::ITALIC),
    )
    .alignment(ratatui::layout::Alignment::Right);
  frame.render_widget(schedule_hint, layout[6]);
}
