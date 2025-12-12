use crate::app::App;
use crate::enums::{ComposeField, ConfigField, CurrentPage, InputMode, Notification};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
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
            Constraint::Length(3), // To
            Constraint::Length(3), // Subject
            Constraint::Min(5),    // Body
            Constraint::Length(3), // Send Button
        ])
        .split(area);

    let styles = get_field_styles(
        app,
        &[
            ComposeField::Recipient,
            ComposeField::Subject,
            ComposeField::Body,
            ComposeField::SendButton,
        ],
        |f| app.compose_field == *f,
    );

    let recipient = Paragraph::new(app.draft.recipient.as_str())
        .block(Block::default().borders(Borders::ALL).title("To"))
        .style(styles[0]);

    let subject = Paragraph::new(app.draft.subject.as_str())
        .block(Block::default().borders(Borders::ALL).title("Subject"))
        .style(styles[1]);

    let body_content = if app.draft.body.is_empty() {
        "Press <Enter> to open external editor...".to_string()
    } else {
        app.draft.body.clone()
    };

    let body = Paragraph::new(body_content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Body (Markdown) - Press Enter to Edit"),
        )
        .style(styles[2]);

    let button_text = if app.compose_field == ComposeField::SendButton {
        "> [ SEND EMAIL ] <"
    } else {
        "  [ SEND EMAIL ]  "
    };

    let send_btn = Paragraph::new(button_text)
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL))
        .style(styles[3]);

    frame.render_widget(recipient, layout[0]);
    frame.render_widget(subject, layout[1]);
    frame.render_widget(body, layout[2]);
    frame.render_widget(send_btn, layout[3]);
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
            Constraint::Length(3), // Color
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
        ConfigField::FooterColor,
        ConfigField::SmtpUser,
        ConfigField::SmtpPass,
        ConfigField::WorkerUrl,
    ];

    let styles = get_field_styles(app, &fields, |f| app.config_field == *f);

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

    let email_str = app.config.identity.emails.join(", ");
    render_input(5, "Emails", &email_str, false);

    render_input(
        6,
        "Footer Color (Hex)",
        &app.config.identity.footer_color,
        false,
    );

    render_input(7, "SMTP Email", &app.config.smtp_username, false);
    render_input(8, "SMTP App Password", &app.config.smtp_app_password, true);
    render_input(9, "Worker URL", &app.config.worker_url, false);
}

fn draw_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let (text, color) = if let Some(note) = &app.notification {
        match note {
            Notification::Info(msg) => (format!(" ℹ️  {} ", msg), Color::Blue),
            Notification::Success(msg) => (format!(" ✅ {} ", msg), Color::Green),
            Notification::Error(msg) => (format!(" ❌ {} ", msg), Color::Red),
        }
    } else {
        match app.input_mode {
            InputMode::Normal => (
                " [Tab] Next | [Enter] Edit | [Ctrl+Enter] Send Email | [1/2] Switch Page | [q] Quit".to_string(), 
                Color::DarkGray
            ),
            InputMode::Editing => (
                " Editing Mode: [Esc] Finish Editing".to_string(), 
                Color::Yellow
            ),
        }
    };

    let status =
        Paragraph::new(text).style(Style::default().fg(color).add_modifier(Modifier::BOLD));

    frame.render_widget(status, area);
}

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
