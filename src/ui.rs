use crate::app::{App, ComposeField, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Recipient
            Constraint::Length(3), // Subject
            Constraint::Min(5),    // Body
            Constraint::Length(3), // Status/Help
        ])
        .split(frame.area());

    let (recipient_style, subject_style, body_style) = match app.input_mode {
        InputMode::Normal => (
            get_style(app.current_field == ComposeField::Recipient, false),
            get_style(app.current_field == ComposeField::Subject, false),
            get_style(app.current_field == ComposeField::Body, false),
        ),
        InputMode::Editing => (
            get_style(app.current_field == ComposeField::Recipient, true),
            get_style(app.current_field == ComposeField::Subject, true),
            get_style(app.current_field == ComposeField::Body, true),
        ),
    };

    let recipient = Paragraph::new(app.draft.recipient.as_str())
        .block(Block::default().borders(Borders::ALL).title("To"))
        .style(recipient_style);

    let subject = Paragraph::new(app.draft.subject.as_str())
        .block(Block::default().borders(Borders::ALL).title("Subject"))
        .style(subject_style);

    let body = Paragraph::new(app.draft.body.as_str())
        .block(Block::default().borders(Borders::ALL).title("Body"))
        .style(body_style);

    let help_text = match app.input_mode {
        InputMode::Normal => "Normal Mode: [Tab] Cycle Fields | [Enter] Edit | [q] Quit",
        InputMode::Editing => "Editing Mode: [Esc] Stop Editing | [Type] Insert Text",
    };

    let footer = Paragraph::new(help_text)
        .block(Block::default().borders(Borders::ALL).title("Status"))
        .style(Style::default().fg(Color::DarkGray));

    frame.render_widget(recipient, layout[0]);
    frame.render_widget(subject, layout[1]);
    frame.render_widget(body, layout[2]);
    frame.render_widget(footer, layout[3]);
}

fn get_style(is_selected: bool, is_editing: bool) -> Style {
    if is_editing && is_selected {
        Style::default().fg(Color::Yellow)
    } else if is_selected {
        Style::default().fg(Color::Green)
    } else {
        Style::default().fg(Color::White)
    }
}
