use crate::app::App;
use crate::enums::{ConfigField, InputMode};
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Style},
  widgets::{Block, Borders, Paragraph},
};

pub fn draw_config(frame: &mut Frame, app: &App, area: Rect) {
  struct ConfigItem<'a> {
    field: ConfigField,
    title: &'a str,
    value: String,
    secure: bool,
  }

  let email_str = app.config.data.identity.emails.join(", ");
  let items = vec![
    ConfigItem {
      field: ConfigField::Name,
      title: "Full Name",
      value: app.config.data.identity.name.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Role,
      title: "Role",
      value: app.config.data.identity.role.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Department,
      title: "Department",
      value: app.config.data.identity.department.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Institution,
      title: "Institution",
      value: app.config.data.identity.institution.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Phone,
      title: "Phone",
      value: app.config.data.identity.phone.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Emails,
      title: "Emails",
      value: email_str,
      secure: false,
    },
    ConfigItem {
      field: ConfigField::FooterColor,
      title: "Footer Color (Hex)",
      value: app.config.data.identity.footer_color.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::SmtpUser,
      title: "SMTP Email",
      value: app.config.data.smtp_username.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::SmtpPass,
      title: "SMTP App Password",
      value: app.config.data.smtp_app_password.clone(),
      secure: true,
    },
    ConfigItem {
      field: ConfigField::WorkerUrl,
      title: "Worker URL",
      value: app.config.data.worker_url.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::ApiSecret,
      title: "API Secret",
      value: app.config.data.api_secret.clone(),
      secure: true,
    },
  ];

  let item_height = 3;
  let total_items = items.len();
  let max_visible_items = (area.height / item_height).max(1) as usize;

  let selected_idx = items
    .iter()
    .position(|i| i.field == app.config.field)
    .unwrap_or(0);

  let mut start_idx = if selected_idx < max_visible_items / 2 {
    0
  } else {
    selected_idx.saturating_sub(max_visible_items / 2)
  };

  if start_idx + max_visible_items > total_items {
    start_idx = total_items.saturating_sub(max_visible_items);
  }

  let end_idx = (start_idx + max_visible_items).min(total_items);
  let visible_items = &items[start_idx..end_idx];

  let constraints: Vec<Constraint> = visible_items
    .iter()
    .map(|_| Constraint::Length(item_height))
    .collect();

  let layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints(constraints)
    .split(area);

  for (i, item) in visible_items.iter().enumerate() {
    let content = if item.secure && !item.value.is_empty() {
      "*".repeat(item.value.len())
    } else {
      item.value.clone()
    };

    let style = if app.config.field == item.field {
      match app.input_mode {
        InputMode::Editing => Style::default().fg(Color::Yellow),
        InputMode::Normal => Style::default().fg(Color::Green),
      }
    } else {
      Style::default().fg(Color::White)
    };

    let widget = Paragraph::new(content)
      .block(Block::default().borders(Borders::ALL).title(item.title))
      .style(style);

    frame.render_widget(widget, layout[i]);
  }
}
