use crate::app::{App, ComposeField, ConfigField, CurrentPage, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs},
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

    draw_tabs(frame, app, chunks[0]);

    match app.current_page {
        CurrentPage::Compose => draw_compose(frame, app, chunks[1]),
        CurrentPage::Config => draw_config(frame, app, chunks[1]),
    }

    draw_status_bar(frame, app, chunks[2]);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec![" [1] Compose ", " [2] Config "];
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" Shiryoku "))
        .select(match app.current_page {
            CurrentPage::Compose => 0,
            CurrentPage::Config => 1,
        })
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

fn draw_compose(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(5),
        ])
        .split(area);

    let styles = get_field_styles(
        app,
        &[
            ComposeField::Recipient,
            ComposeField::Subject,
            ComposeField::Body,
        ],
        |f| app.compose_field == *f,
    );
    let recipient = Paragraph::new(app.draft.recipient.as_str())
        .block(Block::default().borders(Borders::ALL).title("To"))
        .style(styles[0]);

    let subject = Paragraph::new(app.draft.subject.as_str())
        .block(Block::default().borders(Borders::ALL).title("Subject"))
        .style(styles[1]);

    let body = Paragraph::new(app.draft.body.as_str())
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Body (Markdown)"),
        )
        .style(styles[2]);

    frame.render_widget(recipient, layout[0]);
    frame.render_widget(subject, layout[1]);
    frame.render_widget(body, layout[2]);
}

fn draw_config(frame: &mut Frame, app: &App, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Name
            Constraint::Length(3), // Role
            Constraint::Length(3), // Dept
            Constraint::Length(3), // Institution
            Constraint::Length(3), // Phone
            Constraint::Length(3), // Emails
            Constraint::Length(3), // SMTP User
            Constraint::Length(3), // SMTP Pass
            Constraint::Length(3), // Worker URL
        ])
        .split(area);

    let fields = [
        ConfigField::Name,
        ConfigField::Role,
        ConfigField::Department,
        ConfigField::Institution,
        ConfigField::Phone,
        ConfigField::Emails,
        ConfigField::SmtpUser,
        ConfigField::SmtpPass,
        ConfigField::WorkerUrl,
    ];

    let styles = get_field_styles(app, &fields, |f| app.config_field == *f);

    // Helper to render input fields
    let mut render_input = |idx: usize, title: &str, val: &str, secure: bool| {
        let content = if secure && !val.is_empty() {
            "*".repeat(val.len())
        } else {
            val.to_string()
        };
        let widget = Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(title))
            .style(styles[idx]);
        frame.render_widget(widget, layout[idx]);
    };

    render_input(0, "Full Name", &app.config.identity.name, false);
    render_input(1, "Role", &app.config.identity.role, false);
    render_input(2, "Department", &app.config.identity.department, false);
    render_input(3, "Institution", &app.config.identity.institution, false);
    render_input(4, "Phone", &app.config.identity.phone, false);

    // Join emails for display
    let email_str = app.config.identity.emails.join(", ");
    render_input(5, "Emails (comma separated)", &email_str, false);

    render_input(6, "SMTP Email", &app.config.smtp_username, false);
    render_input(7, "SMTP App Password", &app.config.smtp_app_password, true);
    render_input(8, "Worker URL", &app.config.worker_url, false);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let help_text = match app.input_mode {
        InputMode::Normal => {
            "Normal: [Tab] Next Field | [Enter] Edit | [1/2] Switch Page | [Ctrl+s] Save Config | [q] Quit"
        }
        InputMode::Editing => "Editing: [Esc] Stop | [Enter] Next Field",
    };

    let status = Paragraph::new(help_text).style(Style::default().fg(Color::DarkGray));
    frame.render_widget(status, area);
}

// Helper to calculate styles for a list of fields based on current selection
fn get_field_styles<T, F>(app: &App, fields: &[T], is_current: F) -> Vec<Style>
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
