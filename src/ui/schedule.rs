use crate::app::App;
use crate::enums::{InputMode, ScheduleField};
use chrono::{Local, NaiveDate, NaiveTime, TimeZone, Utc};
use chrono_tz::OffsetComponents; // Fixed: Required for base_utc_offset
use ratatui::{
  Frame,
  layout::{Alignment, Constraint, Direction, Layout, Rect},
  style::{Color, Modifier, Style},
  text::{Line, Span},
  widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn draw_schedule_page(frame: &mut Frame, app: &App, area: Rect) {
  // 1. MAIN LAYOUT (Header -> Body -> Footer)
  let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(7), // HUD (Stats)
      Constraint::Min(0),    // Main Controls (Split Left/Right)
      Constraint::Length(3), // Footer (Lock Button)
    ])
    .split(area);

  draw_hud(frame, app, chunks[0]);
  draw_main_deck(frame, app, chunks[1]);
  draw_footer(frame, app, chunks[2]);
}

// --- TOP HUD: Time Comparison ---
fn draw_hud(frame: &mut Frame, app: &App, area: Rect) {
  let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage(33),
      Constraint::Percentage(34), // Middle is slightly wider for "Diff" arrow
      Constraint::Percentage(33),
    ])
    .split(area);

  let system_tz = Local::now().format("%Z").to_string();

  // Attempt to calculate the target time to show stats
  let (target_dt_opt, offset_str) = calculate_target(app);

  // 1. LEFT: YOUR LOCATION
  let local_now = Local::now();
  let left_text = vec![
    Line::from("📍 YOUR LOCATION"),
    Line::from(Span::styled(
      local_now.format("%H:%M").to_string(),
      Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD)
        // Fixed: Upper case REVERSED
        .add_modifier(Modifier::REVERSED),
    )),
    Line::from(local_now.format("%d %b %Y").to_string()),
    Line::from(format!("Timezone: {}", system_tz)),
  ];
  let left_block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().fg(Color::DarkGray));
  frame.render_widget(
    Paragraph::new(left_text)
      .block(left_block)
      .alignment(Alignment::Center),
    layout[0],
  );

  // 2. MIDDLE: THE GAP (Time Difference)
  let diff_text = if let Some(target_dt) = target_dt_opt {
    // Calculate difference relative to the Target's timezone interpretation of "now"
    let diff = target_dt.signed_duration_since(local_now.with_timezone(&target_dt.timezone()));
    let hours = diff.num_hours();
    let mins = (diff.num_minutes() % 60).abs(); // abs() to ensure minutes don't show negative if hours are negative

    let color = if hours >= 0 { Color::Green } else { Color::Red };
    let sign = if hours >= 0 { "+" } else { "" };

    vec![
      Line::from("⏱ TIME DIFFERENCE"),
      Line::from(Span::styled(
        format!("{}{:02}h {:02}m", sign, hours, mins),
        Style::default().fg(color).add_modifier(Modifier::BOLD),
      )),
      Line::from("from now"),
    ]
  } else {
    vec![Line::from("Calculating..."), Line::from("--")]
  };

  let mid_block = Block::default().borders(Borders::ALL).title(" DELTA ");
  frame.render_widget(
    Paragraph::new(diff_text)
      .block(mid_block)
      .alignment(Alignment::Center),
    layout[1],
  );

  // 3. RIGHT: TARGET LOCATION
  let right_content = if let Some(target_dt) = target_dt_opt {
    vec![
      Line::from("🎯 TARGET DELIVERY"),
      Line::from(Span::styled(
        target_dt.format("%H:%M").to_string(),
        Style::default()
          .fg(Color::Yellow)
          .add_modifier(Modifier::BOLD)
          // Fixed: Upper case REVERSED
          .add_modifier(Modifier::REVERSED),
      )),
      Line::from(target_dt.format("%d %b %Y").to_string()),
      Line::from(format!("Offset: {}", offset_str)),
    ]
  } else {
    vec![
      Line::from("🎯 TARGET DELIVERY"),
      Line::from("INVALID DATE/TIME"),
      Line::from("Check inputs"),
    ]
  };

  let right_block = Block::default()
    .borders(Borders::ALL)
    .style(Style::default().fg(Color::White));
  frame.render_widget(
    Paragraph::new(right_content)
      .block(right_block)
      .alignment(Alignment::Center),
    layout[2],
  );
}

// --- MAIN DECK: Inputs & List ---
fn draw_main_deck(frame: &mut Frame, app: &App, area: Rect) {
  let layout = Layout::default()
    .direction(Direction::Horizontal)
    .constraints([
      Constraint::Percentage(40), // Inputs
      Constraint::Percentage(60), // Search/List
    ])
    .split(area);

  // -- LEFT COL: Date/Time Inputs --
  let col1 = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Label
      Constraint::Length(3), // Day/Month/Year
      Constraint::Length(1), // Spacer
      Constraint::Length(3), // Label
      Constraint::Length(3), // Hr/Min/Sec
      Constraint::Min(0),    // Empty
    ])
    .split(layout[0]);

  // Date Header
  frame.render_widget(
    Paragraph::new("📅  SET DATE").style(Style::default().add_modifier(Modifier::BOLD)),
    col1[0],
  );

  // Date Row (Split 3 ways)
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

  // Time Header
  frame.render_widget(
    Paragraph::new("⏰  SET TIME").style(Style::default().add_modifier(Modifier::BOLD)),
    col1[3],
  );

  // Time Row (Split 3 ways)
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

  // -- RIGHT COL: Timezone Search --
  let col2 = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
      Constraint::Length(3), // Search Input
      Constraint::Min(0),    // List
    ])
    .split(layout[1]);

  // Search Box
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

  // List
  let items: Vec<ListItem> = app
    .schedule
    .filtered_timezones
    .iter()
    .take(50) // Show more items since we have space
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
      // Add a little arrow for the selected one
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
    .constraints([
      Constraint::Percentage(70), // Verification Text
      Constraint::Percentage(30), // Lock Button
    ])
    .split(area);

  let (target_opt, _) = calculate_target(app);
  let verify_text = if let Some(t) = target_opt {
    let utc = t.with_timezone(&Utc);
    format!(" ✅ Verified: {} (UTC)", utc.format("%Y-%m-%d %H:%M:%S"))
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

// --- HELPER FUNCTIONS ---

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

// Logic to try and parse the messy UI state into a real DateTime
fn calculate_target(app: &App) -> (Option<chrono::DateTime<chrono_tz::Tz>>, String) {
  let day = app.schedule.day.parse::<u32>().unwrap_or(0);
  let month = app.schedule.month.parse::<u32>().unwrap_or(0);
  let year = app.schedule.year.parse::<i32>().unwrap_or(0);
  let hour = app.schedule.hour.parse::<u32>().unwrap_or(0);
  let min = app.schedule.minute.parse::<u32>().unwrap_or(0);
  let sec = app.schedule.second.parse::<u32>().unwrap_or(0);

  let naive_date = NaiveDate::from_ymd_opt(year, month, day);
  let naive_time = NaiveTime::from_hms_opt(hour, min, sec);

  // Attempt to parse the timezone string into a Tz struct
  let parsed_tz = app.schedule.timezone_input.parse::<chrono_tz::Tz>().ok();

  if let (Some(d), Some(t), Some(tz)) = (naive_date, naive_time, parsed_tz) {
    let local_dt = d.and_time(t);
    let target = match tz.from_local_datetime(&local_dt) {
      chrono::LocalResult::Single(dt) => Some(dt),
      chrono::LocalResult::Ambiguous(dt1, _) => Some(dt1),
      _ => None,
    };

    if let Some(t) = target {
      // Fixed: use dst_offset() or base_utc_offset() via OffsetComponents trait
      let offset_secs = t.offset().base_utc_offset().num_seconds();
      let sign = if offset_secs >= 0 { "+" } else { "-" };
      let abs = offset_secs.abs();
      let h = abs / 3600;
      let m = (abs % 3600) / 60;
      return (Some(t), format!("UTC{}{:02}:{:02}", sign, h, m));
    }
  }

  (None, "UTC?".to_string())
}
