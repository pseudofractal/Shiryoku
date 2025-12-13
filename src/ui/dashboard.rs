use crate::app::App;
use crate::app::dashboard::RecipientSummary;
use crate::enums::{DashboardFocus, InputMode};
use base64::Engine;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::Line,
  widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

pub fn draw_dashboard(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Filter Bar
      Constraint::Min(0),    // List
    ])
    .split(area);

  draw_filter_bar(frame, app, chunks[0]);
  draw_summary_list(frame, app, chunks[1]);

  if let Some(selected_id) = &app.dashboard.selected_summary_id {
    let summaries = app.dashboard.get_aggregated_logs();
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
    if app.dashboard.focus == f {
      Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::default()
    }
  };

  let mut r_text = app.dashboard.filter_recipient.clone();
  // Autocomplete Logic
  if app.input_mode == InputMode::Editing
    && app.dashboard.focus == DashboardFocus::FilterRecipient
    && !r_text.is_empty()
  {
    if let Some(suggestion) = app
      .dashboard
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

  let c_input = Paragraph::new(app.dashboard.filter_country.as_str())
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(" Filter Country "),
    )
    .style(get_style(DashboardFocus::FilterCountry));

  let m_input = Paragraph::new(app.dashboard.filter_min_opens.as_str())
    .block(Block::default().borders(Borders::ALL).title(" Min Opens "))
    .style(get_style(DashboardFocus::FilterMinOpens));

  frame.render_widget(r_input, layout[0]);
  frame.render_widget(c_input, layout[1]);
  frame.render_widget(m_input, layout[2]);
}

fn draw_summary_list(frame: &mut Frame, app: &App, area: Rect) {
  let summaries = app.dashboard.get_aggregated_logs();
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
      .border_style(if app.dashboard.focus == DashboardFocus::List {
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

  let mut state = app.dashboard.list_state.clone();
  frame.render_stateful_widget(table, area, &mut state);
}

fn draw_detail_popup(frame: &mut Frame, summary: &RecipientSummary, area: Rect) {
  let popup_area = crate::ui::centered_rect(60, 60, area);
  frame.render_widget(Clear, popup_area);

  let block = Block::default()
    .borders(Borders::ALL)
    .title(format!(" Activity: {} ", summary.decoded_email))
    .title_bottom(Line::from("[Ctrl+d] Delete History").right_aligned());

  let inner = block.inner(popup_area);
  frame.render_widget(block, popup_area);

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(3)])
    .split(inner);

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
