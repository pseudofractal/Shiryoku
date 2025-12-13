use crate::app::{App, RecipientSummary};
use crate::enums::{
  ComposeField, ConfigField, CurrentPage, DashboardFocus, InputMode, Notification,
};
use base64::Engine;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap},
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
    CurrentPage::Dashboard => draw_dashboard(frame, app, chunks[1]),
  }

  draw_status_bar(frame, app, chunks[2]);
}

fn draw_tabs(frame: &mut Frame, app: &App, area: Rect) {
  let titles = vec![" [1] Compose ", " [2] Config ", " [3] Dashboard "];
  let tabs = Tabs::new(titles)
    .block(Block::default().borders(Borders::ALL).title(" Shiryoku "))
    .select(match app.current_page {
      CurrentPage::Compose => 0,
      CurrentPage::Config => 1,
      CurrentPage::Dashboard => 2,
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
      Constraint::Length(3), // Attachments
      Constraint::Min(5),    // Body
      Constraint::Length(3), // Send Button
    ])
    .split(area);

  let styles = get_field_styles(
    app,
    &[
      ComposeField::Recipient,
      ComposeField::Subject,
      ComposeField::Attachments,
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

  let attach_widget = Paragraph::new(app.attachment_input.as_str())
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Attachments (semicolon separated paths)"),
    )
    .style(styles[2]);

  let body = Paragraph::new(body_content)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title("Body (Markdown) - Press Enter to Edit"),
    )
    .style(styles[3]);

  let button_text = if app.compose_field == ComposeField::SendButton {
    "> [SEND EMAIL] <"
  } else {
    "  [SEND EMAIL]  "
  };

  let send_btn = Paragraph::new(button_text)
    .alignment(ratatui::layout::Alignment::Center)
    .block(Block::default().borders(Borders::ALL))
    .style(styles[4]);

  frame.render_widget(recipient, layout[0]);
  frame.render_widget(subject, layout[1]);
  frame.render_widget(attach_widget, layout[2]);
  frame.render_widget(body, layout[3]);
  frame.render_widget(send_btn, layout[4]);
}

fn draw_config(frame: &mut Frame, app: &App, area: Rect) {
  struct ConfigItem<'a> {
    field: ConfigField,
    title: &'a str,
    value: String,
    secure: bool,
  }

  let email_str = app.config.identity.emails.join(", ");

  let items = vec![
    ConfigItem {
      field: ConfigField::Name,
      title: "Full Name",
      value: app.config.identity.name.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Role,
      title: "Role",
      value: app.config.identity.role.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Department,
      title: "Department",
      value: app.config.identity.department.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Institution,
      title: "Institution",
      value: app.config.identity.institution.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::Phone,
      title: "Phone",
      value: app.config.identity.phone.clone(),
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
      value: app.config.identity.footer_color.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::SmtpUser,
      title: "SMTP Email",
      value: app.config.smtp_username.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::SmtpPass,
      title: "SMTP App Password",
      value: app.config.smtp_app_password.clone(),
      secure: true,
    },
    ConfigItem {
      field: ConfigField::WorkerUrl,
      title: "Worker URL",
      value: app.config.worker_url.clone(),
      secure: false,
    },
    ConfigItem {
      field: ConfigField::ApiSecret,
      title: "API Secret",
      value: app.config.api_secret.clone(),
      secure: true,
    },
  ];

  let item_height = 3;
  let total_items = items.len();
  let max_visible_items = (area.height / item_height).max(1) as usize;
  let selected_idx = items
    .iter()
    .position(|i| i.field == app.config_field)
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

    let style = if app.config_field == item.field {
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

fn draw_dashboard(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Filter Bar
      Constraint::Min(0),    // List
    ])
    .split(area);

  draw_filter_bar(frame, app, chunks[0]);
  draw_summary_list(frame, app, chunks[1]);

  // Popup Logic
  if let Some(selected_id) = &app.selected_summary_id {
    let summaries = app.get_aggregated_logs();
    if let Some(summary) = summaries.iter().find(|s| s.tracking_id == *selected_id) {
      draw_detail_popup(frame, summary, area);
    }
  }
}

fn draw_filter_bar(frame: &mut Frame, app: &App, area: Rect) {
  let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage(40),
      Constraint::Percentage(30),
      Constraint::Percentage(30),
    ])
    .split(area);

  let get_style = |f: DashboardFocus| {
    if app.dashboard_focus == f {
      Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::default()
    }
  };

  let mut r_text = app.filter_recipient.clone();
  if app.input_mode == InputMode::Editing
    && app.dashboard_focus == DashboardFocus::FilterRecipient
    && !r_text.is_empty()
  {
    if let Some(suggestion) = app
      .filter_options
      .recipients
      .iter()
      .find(|r| r.contains(&r_text))
    {
      let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(suggestion)
        .unwrap_or_default();
      let hint = String::from_utf8_lossy(&decoded);
      r_text = format!("{} ({})", r_text, hint);
    }
  }

  let r_input = Paragraph::new(r_text)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(" Filter Email "),
    )
    .style(get_style(DashboardFocus::FilterRecipient));

  let c_input = Paragraph::new(app.filter_country.as_str())
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(" Filter Country "),
    )
    .style(get_style(DashboardFocus::FilterCountry));

  let m_input = Paragraph::new(app.filter_min_opens.as_str())
    .block(Block::default().borders(Borders::ALL).title(" Min Opens "))
    .style(get_style(DashboardFocus::FilterMinOpens));

  frame.render_widget(r_input, layout[0]);
  frame.render_widget(c_input, layout[1]);
  frame.render_widget(m_input, layout[2]);
}

fn draw_summary_list(frame: &mut Frame, app: &App, area: Rect) {
  let summaries = app.get_aggregated_logs();
  let rows: Vec<Row> = summaries
    .iter()
    .map(|s| {
      let now = chrono::Utc::now();
      let diff = now.signed_duration_since(s.last_seen_raw);
      let time_ago = if diff.num_minutes() < 60 {
        format!("{}m ago", diff.num_minutes())
      } else if diff.num_hours() < 24 {
        format!("{}h ago", diff.num_hours())
      } else {
        format!("{}d ago", diff.num_days())
      };

      // Fallback safely if logs is empty (shouldn't be, but safe)
      let tz: Tz = s
        .logs
        .first()
        .map(|l| l.timezone.parse().unwrap_or(chrono_tz::UTC))
        .unwrap_or(chrono_tz::UTC);

      let current_time = now.with_timezone(&tz).format("%H:%M %p").to_string();

      Row::new(vec![
        Cell::from(s.decoded_email.clone()),
        Cell::from(s.open_count.to_string()),
        Cell::from(time_ago),
        Cell::from(current_time),
        Cell::from(s.country.clone()),
      ])
    })
    .collect();

  let table = Table::new(
    rows,
    [
      Constraint::Percentage(40),
      Constraint::Length(10),
      Constraint::Length(15),
      Constraint::Length(15),
      Constraint::Percentage(20),
    ],
  )
  .header(
    Row::new(vec![
      "Recipient",
      "Opens",
      "Last Seen",
      "Current Time",
      "Country",
    ])
    .style(
      Style::default()
        .add_modifier(Modifier::BOLD)
        .fg(Color::Cyan),
    ),
  )
  .block(
    Block::default()
      .borders(Borders::ALL)
      .title(" Recipients ")
      .border_style(if app.dashboard_focus == DashboardFocus::List {
        Style::default().fg(Color::Yellow)
      } else {
        Style::default()
      }),
  )
  .row_highlight_style(
    Style::default()
      .bg(Color::DarkGray)
      .add_modifier(Modifier::BOLD),
  );

  let mut state = app.dashboard_list_state.clone();
  frame.render_stateful_widget(table, area, &mut state);
}

fn draw_detail_popup(frame: &mut Frame, summary: &RecipientSummary, area: Rect) {
  let popup_area = centered_rect(60, 60, area);
  frame.render_widget(Clear, popup_area);

  let block = Block::default()
    .borders(Borders::ALL)
    .title(format!(" Activity: {} ", summary.decoded_email));
  let inner = block.inner(popup_area);
  frame.render_widget(block, popup_area);

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(3)])
    .split(inner);

  // Activity Table
  let rows: Vec<Row> = summary
    .logs
    .iter()
    .take(10)
    .map(|log| {
      let utc = DateTime::parse_from_rfc3339(&log.timestamp)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_default();
      let tz: Tz = log.timezone.parse().ok().unwrap_or(chrono_tz::UTC);
      let local = utc.with_timezone(&tz);

      Row::new(vec![Cell::from(
        local.format("%Y-%m-%d %H:%M:%S").to_string(),
      )])
    })
    .collect();

  let table = Table::new(rows, [Constraint::Percentage(100)])
    .header(Row::new(vec!["Local Timestamp"]).style(Style::default().fg(Color::Cyan)));

  frame.render_widget(table, chunks[0]);

  let now = chrono::Utc::now();
  let latest_log = summary.logs.first();
  let tz: Tz = latest_log
    .and_then(|l| l.timezone.parse().ok())
    .unwrap_or(chrono_tz::UTC);
  let current = now
    .with_timezone(&tz)
    .format("%Y-%m-%d %H:%M:%S %Z")
    .to_string();
  let ua = latest_log.map(|l| l.user_agent.clone()).unwrap_or_default();

  let info = Paragraph::new(format!("Current User Time: {}\nDevice: {}", current, ua))
    .wrap(Wrap { trim: true })
    .style(Style::default().fg(Color::Green));

  frame.render_widget(info, chunks[1]);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
  let popup_layout = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Percentage((100 - percent_y) / 2),
      Constraint::Percentage(percent_y),
      Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

  Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage((100 - percent_x) / 2),
      Constraint::Percentage(percent_x),
      Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
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
      InputMode::Normal => {
        let base_help = " [Tab] Next | [Enter] Edit | [Ctrl+Enter] Send | [1/2/3] Nav | [q] Quit";
        let extra_help = if app.current_page == CurrentPage::Compose
          && app.compose_field == ComposeField::Attachments
        {
          " | [Ctrl+o] Browse Files | [Ctrl+x] Clear Attachments"
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
