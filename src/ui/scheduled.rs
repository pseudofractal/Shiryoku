use crate::app::App;
use crate::enums::{JobStatus, ScheduledFocus};
use crate::models::ScheduledJob;
use chrono::Local;
use chrono_tz::Tz;
use ratatui::{
  Frame,
  layout::{Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Wrap},
};

pub fn draw_scheduled(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Filters
      Constraint::Min(0),    // Table
    ])
    .split(area);

  draw_filters(frame, app, chunks[0]);
  draw_table(frame, app, chunks[1]);

  if let Some(selected_id) = &app.scheduled.selected_job_id {
    if let Some(job) = app.scheduled.jobs.iter().find(|j| j.id == *selected_id) {
      draw_detail_popup(frame, job, area);
    }
  }
}

fn draw_filters(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
    .split(area);

  let style = |f: ScheduledFocus| {
    if app.scheduled.focus == f {
      Style::default()
        .fg(Color::Yellow)
        .add_modifier(Modifier::BOLD)
    } else {
      Style::default()
    }
  };

  let recip_widget = Paragraph::new(app.scheduled.filter_recipient.as_str())
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(" Filter Recipient "),
    )
    .style(style(ScheduledFocus::FilterRecipient));

  let status_str = match app.scheduled.filter_status {
    None => "ALL",
    Some(JobStatus::Pending) => "PENDING",
    Some(JobStatus::Sent) => "SENT",
    Some(JobStatus::Failed) => "FAILED",
    _ => "OTHER",
  };

  let status_widget = Paragraph::new(format!("< {} >", status_str))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(" Status (Enter to toggle) "),
    )
    .style(style(ScheduledFocus::FilterStatus));

  frame.render_widget(recip_widget, chunks[0]);
  frame.render_widget(status_widget, chunks[1]);
}

fn draw_table(frame: &mut Frame, app: &App, area: Rect) {
  let jobs = app.scheduled.get_filtered_jobs();

  let rows: Vec<Row> = jobs
    .iter()
    .map(|job| {
      // 1. To
      let to = job.recipient.clone();

      // 2. Recipient Local Time
      let tz: Tz = job.recipient_timezone.parse().unwrap_or(chrono_tz::UTC);
      let recip_time = job
        .scheduled_at
        .with_timezone(&tz)
        .format("%Y-%m-%d %H:%M")
        .to_string();

      // 3. User Local Time
      let user_time = job
        .scheduled_at
        .with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string();

      // 4. Status
      let status = format!("{:?}", job.status);
      let status_style = match job.status {
        JobStatus::Pending => Style::default().fg(Color::Yellow),
        JobStatus::Sent => Style::default().fg(Color::Green),
        JobStatus::Failed => Style::default().fg(Color::Red),
        _ => Style::default(),
      };

      Row::new(vec![
        Cell::from(to),
        Cell::from(recip_time),
        Cell::from(user_time),
        Cell::from(status).style(status_style),
      ])
    })
    .collect();

  let table = Table::new(
    rows,
    [
      Constraint::Percentage(30),
      Constraint::Percentage(25),
      Constraint::Percentage(25),
      Constraint::Percentage(20),
    ],
  )
  .header(
    Row::new(vec!["To", "Recipient Time", "Your Time", "Status"]).style(
      Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD),
    ),
  )
  .block(
    Block::default()
      .borders(Borders::ALL)
      .title(" Scheduled Jobs ")
      .border_style(if app.scheduled.focus == ScheduledFocus::List {
        Style::default().fg(Color::Yellow)
      } else {
        Style::default()
      }),
  )
  .row_highlight_style(Style::default().bg(Color::DarkGray));

  let mut state = app.scheduled.list_state.clone();
  frame.render_stateful_widget(table, area, &mut state);
}

fn draw_detail_popup(frame: &mut Frame, job: &ScheduledJob, area: Rect) {
  let popup_area = crate::ui::centered_rect(70, 70, area);
  frame.render_widget(Clear, popup_area);

  let block = Block::default()
    .borders(Borders::ALL)
    .title(format!(" Details: {} ", job.subject))
    .title_bottom("[D] Download | [X] Cancel (if pending) | [Esc] Close");

  let inner = block.inner(popup_area);
  frame.render_widget(block, popup_area);

  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(1), // Recipient
      Constraint::Length(1), // Time
      Constraint::Min(0),    // Body
    ])
    .split(inner);

  frame.render_widget(Paragraph::new(format!("To: {}", job.recipient)), chunks[0]);
  frame.render_widget(
    Paragraph::new(format!("Scheduled: {} (UTC)", job.scheduled_at)),
    chunks[1],
  );

  let body_block = Block::default().borders(Borders::TOP).title(" Body ");
  frame.render_widget(
    Paragraph::new(job.body.clone())
      .block(body_block)
      .wrap(Wrap { trim: false }),
    chunks[2],
  );
}
