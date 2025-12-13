use crate::enums::DashboardFocus;
use crate::models::{FilterOptions, LogEntry};
use base64::{Engine as _, engine::general_purpose};
use chrono::{DateTime, Utc};
use ratatui::widgets::TableState;
use std::collections::HashMap;

pub struct RecipientSummary {
  pub tracking_id: String,
  pub decoded_email: String,
  pub country: String,
  pub open_count: usize,
  pub last_seen_raw: DateTime<Utc>,
  pub logs: Vec<LogEntry>,
}

pub struct DashboardState {
  pub logs: Vec<LogEntry>,
  pub focus: DashboardFocus,
  pub filter_recipient: String,
  pub filter_country: String,
  pub filter_min_opens: String,
  pub list_state: TableState,
  pub selected_summary_id: Option<String>,
  pub filter_options: FilterOptions,
}

impl Default for DashboardState {
  fn default() -> Self {
    Self {
      logs: Vec::new(),
      focus: DashboardFocus::FilterRecipient,
      filter_recipient: String::new(),
      filter_country: String::new(),
      filter_min_opens: String::new(),
      list_state: TableState::default(),
      selected_summary_id: None,
      filter_options: FilterOptions::default(),
    }
  }
}

impl DashboardState {
  pub fn cycle_focus(&mut self, forward: bool) {
    if self.selected_summary_id.is_some() {
      return;
    }
    if forward {
      self.focus = match self.focus {
        DashboardFocus::FilterRecipient => DashboardFocus::FilterCountry,
        DashboardFocus::FilterCountry => DashboardFocus::FilterMinOpens,
        DashboardFocus::FilterMinOpens => DashboardFocus::List,
        DashboardFocus::List => DashboardFocus::FilterRecipient,
      };
    } else {
      self.focus = match self.focus {
        DashboardFocus::FilterRecipient => DashboardFocus::List,
        DashboardFocus::FilterCountry => DashboardFocus::FilterRecipient,
        DashboardFocus::FilterMinOpens => DashboardFocus::FilterCountry,
        DashboardFocus::List => DashboardFocus::FilterMinOpens,
      };
    }
  }

  pub fn handle_input(&mut self, c: char) {
    match self.focus {
      DashboardFocus::FilterRecipient => self.filter_recipient.push(c),
      DashboardFocus::FilterCountry => self.filter_country.push(c),
      DashboardFocus::FilterMinOpens => {
        if c.is_numeric() {
          self.filter_min_opens.push(c)
        }
      }
      _ => {}
    }
  }

  pub fn handle_backspace(&mut self) {
    match self.focus {
      DashboardFocus::FilterRecipient => {
        self.filter_recipient.pop();
      }
      DashboardFocus::FilterCountry => {
        self.filter_country.pop();
      }
      DashboardFocus::FilterMinOpens => {
        self.filter_min_opens.pop();
      }
      _ => {}
    }
  }

  pub fn get_aggregated_logs(&self) -> Vec<RecipientSummary> {
    let mut groups: HashMap<String, Vec<LogEntry>> = HashMap::new();
    for log in &self.logs {
      groups
        .entry(log.tracking_id.clone())
        .or_default()
        .push(log.clone());
    }
    let mut summaries = Vec::new();
    let min_opens = self.filter_min_opens.parse::<usize>().unwrap_or(0);

    for (id, mut entries) in groups {
      let decoded_email = general_purpose::URL_SAFE_NO_PAD
        .decode(&id)
        .map(|b| String::from_utf8_lossy(&b).to_string())
        .unwrap_or_else(|_| id.clone());
      entries.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

      if entries.len() < min_opens {
        continue;
      }
      if !self.filter_recipient.is_empty() && !decoded_email.contains(&self.filter_recipient) {
        continue;
      }
      let country = entries
        .first()
        .map(|l| l.country.clone())
        .unwrap_or_default();
      if !self.filter_country.is_empty()
        && !country
          .to_lowercase()
          .contains(&self.filter_country.to_lowercase())
      {
        continue;
      }

      let last_seen_str = entries.first().unwrap().timestamp.clone();
      let last_seen_raw = DateTime::parse_from_rfc3339(&last_seen_str)
        .map(|dt| dt.with_timezone(&Utc))
        .unwrap_or_default();
      summaries.push(RecipientSummary {
        tracking_id: id,
        decoded_email,
        country,
        open_count: entries.len(),
        last_seen_raw,
        logs: entries,
      });
    }
    summaries.sort_by(|a, b| b.last_seen_raw.cmp(&a.last_seen_raw));
    summaries
  }
}
