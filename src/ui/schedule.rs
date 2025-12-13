use crate::app::App;
use crate::enums::{InputMode, ScheduleField};
use chrono::{Local, NaiveDate, NaiveTime, TimeZone};
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn draw_schedule_page(frame: &mut Frame, app: &App, area: Rect) {
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(7),
      Constraint::Min(0),
      Constraint::Length(3),
    ])
    .split(area);

  draw_hud(frame, app, chunks[0]);
  draw_main_deck(frame, app, chunks[1]);
  draw_footer(frame, app, chunks[2]);
}

fn draw_hud(frame: &mut Frame, app: &App, area: Rect) {
  let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage(33),
      Constraint::Percentage(34),
      Constraint::Percentage(33),
    ])
    .split(area);

  let local_now = Local::now();
  let target_dt_opt = calculate_target_dt(app);

  let left_text = vec![
    Line::from("📍 YOUR LOCATION"),
    Line::from(Span::styled(
      local_now.format("%H:%M").to_string(),
      Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
        .add_modifier(Modifier::REVERSED),
    )),
    Line::from(local_now.format("%d %b %Y").to_string()),
    Line::from(format!("Zone: {}", local_now.format("%Z (%z)"))),
  ];

  frame.render_widget(
    Paragraph::new(left_text)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .style(Style::default().fg(Color::DarkGray)),
      )
      .alignment(Alignment::Center),
    layout[0],
  );

  let diff_text = if let Some(target) = target_dt_opt {
    let now_utc = local_now.timestamp();
    let target_utc = target.timestamp();
    let diff_secs = target_utc - now_utc;

    let sign = if diff_secs >= 0 { "+" } else { "-" };
    let abs_secs = diff_secs.abs();
    let hours = abs_secs / 3600;
    let mins = (abs_secs % 3600) / 60;

    let color = if diff_secs >= 0 {
      Color::Green
    } else {
      Color::Red
    };

    vec![
      Line::from("⏱  COUNTDOWN"),
      Line::from(Span::styled(
        format!("{}{:02}h {:02}m", sign, hours, mins),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
      )),
      Line::from("from now"),
    ]
  } else {
    vec![Line::from("Calculating..."), Line::from("--")]
  };

  frame.render_widget(
    Paragraph::new(diff_text)
      .block(Block::default().borders(Borders::ALL).title(" DELTA "))
      .alignment(Alignment::Center),
    layout[1],
  );

  let right_content = if let Some(target) = target_dt_opt {
    vec![
      Line::from("🎯 TARGET DELIVERY"),
      Line::from(Span::styled(
        target.format("%H:%M").to_string(),
        Style::default()
          .fg(Color::Yellow)
          .add_modifier(Modifier::BOLD)
          .add_modifier(Modifier::REVERSED),
      )),
      Line::from(target.format("%d %b %Y").to_string()),
      Line::from(format!("Zone: {}", target.format("%Z (%z)"))),
    ]
  } else {
    vec![
      Line::from("🎯 TARGET DELIVERY"),
      Line::from("INVALID DATE/TIME"),
      Line::from("Check inputs"),
    ]
  };

  frame.render_widget(
    Paragraph::new(right_content)
      .block(
        Block::default()
          .borders(Borders::ALL)
          .style(Style::default().fg(Color::White)),
      )
      .alignment(Alignment::Center),
    layout[2],
  );
}

fn draw_main_deck(frame: &mut Frame, app: &App, area: Rect) {
  let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
    .split(area);

  let col1 = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3),
      Constraint::Length(3),
      Constraint::Length(1),
      Constraint::Length(3),
      Constraint::Length(3),
      Constraint::Min(0),
    ])
    .split(layout[0]);

  frame.render_widget(
    Paragraph::new("📅  SET DATE").style(Style::default().add_modifier(Modifier::BOLD)),
    col1[0],
  );

  let date_row = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Ratio(1, 3),
      Constraint::Ratio(1, 3),
      Constraint::Ratio(1, 3),
    ])
    .split(col1[1]);

  render_input(
    frame,
    app,
    app.schedule.day.as_str(),
    "Day",
    ScheduleField::Day,
    date_row[0],
  );
  render_input(
    frame,
    app,
    app.schedule.month.as_str(),
    "Month",
    ScheduleField::Month,
    date_row[1],
  );
  render_input(
    frame,
    app,
    app.schedule.year.as_str(),
    "Year",
    ScheduleField::Year,
    date_row[2],
  );

  frame.render_widget(
    Paragraph::new("⏰  SET TIME").style(Style::default().add_modifier(Modifier::BOLD)),
    col1[3],
  );

  let time_row = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Ratio(1, 3),
      Constraint::Ratio(1, 3),
      Constraint::Ratio(1, 3),
    ])
    .split(col1[4]);

  render_input(
    frame,
    app,
    app.schedule.hour.as_str(),
    "Hour",
    ScheduleField::Hour,
    time_row[0],
  );
  render_input(
    frame,
    app,
    app.schedule.minute.as_str(),
    "Min",
    ScheduleField::Minute,
    time_row[1],
  );
  render_input(
    frame,
    app,
    app.schedule.second.as_str(),
    "Sec",
    ScheduleField::Second,
    time_row[2],
  );

  let col2 = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)])
    .split(layout[1]);

  let tz_active = app.schedule.active_field == ScheduleField::Timezone;
  let tz_style = get_style(app, tz_active);
  frame.render_widget(
    Paragraph::new(app.schedule.timezone_input.as_str())
      .block(
        Block::default()
          .borders(Borders::ALL)
          .title(" 🔍 Search Region "),
      )
      .style(tz_style),
    col2[0],
  );

  let items: Vec<ListItem> = app
    .schedule
    .filtered_timezones
    .iter()
    .take(50)
    .enumerate()
    .map(|(i, tz)| {
      let is_selected = i == app.schedule.selected_timezone_idx;
      let style = if is_selected {
        Style::default()
          .bg(Color::Blue)
          .fg(Color::Black)
          .add_modifier(Modifier::BOLD)
      } else {
        Style::default().fg(Color::DarkGray)
      };
      let prefix = if is_selected { "> " } else { "  " };
      ListItem::new(format!("{}{}", prefix, tz)).style(style)
    })
    .collect();

  frame.render_widget(
    List::new(items).block(
      Block::default()
        .borders(Borders::ALL)
        .title(" Available Regions "),
    ),
    col2[1],
  );
}

fn draw_footer(frame: &mut Frame, app: &App, area: Rect) {
  let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
    .split(area);

  let target_opt = calculate_target_dt(app);
  let verify_text = if let Some(t) = target_opt {
    format!(
      " ✅ Verified: {} ({})",
      t.format("%Y-%m-%d %H:%M:%S"),
      t.format("%Z")
    )
  } else {
    " ❌ Incomplete or Invalid Configuration".to_string()
  };

  frame.render_widget(
    Paragraph::new(verify_text)
      .block(Block::default().borders(Borders::ALL))
      .style(Style::default().fg(Color::DarkGray)),
    layout[0],
  );

  let btn_active = app.schedule.active_field == ScheduleField::Submit;
  let btn_text = if btn_active {
    " > LOCK SCHEDULE < "
  } else {
    " [ LOCK SCHEDULE ] "
  };

  frame.render_widget(
    Paragraph::new(btn_text)
      .block(Block::default().borders(Borders::ALL))
      .alignment(Alignment::Center)
      .style(get_style(app, btn_active).add_modifier(Modifier::BOLD)),
    layout[1],
  );
}

fn render_input(
  frame: &mut Frame,
  app: &App,
  text: &str,
  title: &str,
  field: ScheduleField,
  area: Rect,
) {
  let is_active = app.schedule.active_field == field;
  let style = get_style(app, is_active);

  frame.render_widget(
    Paragraph::new(text)
      .block(Block::default().borders(Borders::ALL).title(title))
      .alignment(Alignment::Center)
      .style(style),
    area,
  );
}

fn get_style(app: &App, is_active: bool) -> Style {
  if is_active {
    match app.input_mode {
      InputMode::Editing => Style::default().fg(Color::Yellow),
      InputMode::Normal => Style::default().fg(Color::Green),
    }
  } else {
    Style::default().fg(Color::White)
  }
}

fn calculate_target_dt(app: &App) -> Option<chrono::DateTime<chrono_tz::Tz>> {
  let day = app.schedule.day.parse::<u32>().ok()?;
  let month = app.schedule.month.parse::<u32>().ok()?;
  let year = app.schedule.year.parse::<i32>().ok()?;
  let hour = app.schedule.hour.parse::<u32>().ok()?;
  let min = app.schedule.minute.parse::<u32>().ok()?;
  let sec = app.schedule.second.parse::<u32>().ok()?;

  let naive_date = NaiveDate::from_ymd_opt(year, month, day)?;
  let naive_time = NaiveTime::from_hms_opt(hour, min, sec)?;

  let parsed_tz = app.schedule.timezone_input.parse::<chrono_tz::Tz>().ok()?;

  let local_dt = naive_date.and_time(naive_time);

  match parsed_tz.from_local_datetime(&local_dt) {
    chrono::LocalResult::Single(dt) => Some(dt),
    chrono::LocalResult::Ambiguous(dt1, _) => Some(dt1),
    _ => None,
  }
}
