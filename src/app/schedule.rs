use crate::enums::ScheduleField;
use chrono_tz::TZ_VARIANTS;

pub struct ScheduleState {
  pub day: String,
  pub month: String,
  pub year: String,
  pub hour: String,
  pub minute: String,
  pub second: String,
  pub timezone_input: String,
  pub available_timezones: Vec<String>,
  pub filtered_timezones: Vec<String>,
  pub selected_timezone_idx: usize,
  pub active_field: ScheduleField,
  pub is_open: bool,
}

impl Default for ScheduleState {
  fn default() -> Self {
    let tzs: Vec<String> = TZ_VARIANTS.iter().map(|tz| tz.name().to_string()).collect();
    Self {
      day: String::new(),
      month: String::new(),
      year: String::new(),
      hour: String::new(),
      minute: String::new(),
      second: String::new(),
      timezone_input: String::new(),
      filtered_timezones: tzs.clone(),
      available_timezones: tzs,
      selected_timezone_idx: 0,
      active_field: ScheduleField::Day,
      is_open: false,
    }
  }
}

impl ScheduleState {
  pub fn update_timezone_filter(&mut self) {
    let query = self.timezone_input.trim().to_lowercase();
    if query.is_empty() {
      self.filtered_timezones = self.available_timezones.clone();
    } else {
      self.filtered_timezones = self
        .available_timezones
        .iter()
        .filter(|tz| tz.to_lowercase().contains(&query))
        .cloned()
        .collect();
    }
    self.selected_timezone_idx = 0;
  }

  pub fn reset_defaults_if_empty(&mut self) {
    if self.day.is_empty() || self.year.is_empty() {
      let now = chrono::Utc::now();
      self.day = now.format("%d").to_string();
      self.month = now.format("%m").to_string();
      self.year = now.format("%Y").to_string();
      let future = now + chrono::Duration::minutes(30);
      self.hour = future.format("%H").to_string();
      self.minute = future.format("%M").to_string();
      self.second = "00".to_string();
    }
    self.is_open = true;
    self.active_field = ScheduleField::Day;
  }

  pub fn clear_current_field(&mut self) {
    match self.active_field {
      ScheduleField::Day => self.day.clear(),
      ScheduleField::Month => self.month.clear(),
      ScheduleField::Year => self.year.clear(),
      ScheduleField::Hour => self.hour.clear(),
      ScheduleField::Minute => self.minute.clear(),
      ScheduleField::Second => self.second.clear(),
      ScheduleField::Timezone => {
        self.timezone_input.clear();
        self.update_timezone_filter();
      }
      ScheduleField::Submit => {}
    }
  }

  pub fn handle_input(&mut self, c: char) {
    match self.active_field {
      ScheduleField::Day => {
        if self.day.len() < 2 {
          self.day.push(c)
        }
      }
      ScheduleField::Month => {
        if self.month.len() < 2 {
          self.month.push(c)
        }
      }
      ScheduleField::Year => {
        if self.year.len() < 4 {
          self.year.push(c)
        }
      }
      ScheduleField::Hour => {
        if self.hour.len() < 2 {
          self.hour.push(c)
        }
      }
      ScheduleField::Minute => {
        if self.minute.len() < 2 {
          self.minute.push(c)
        }
      }
      ScheduleField::Second => {
        if self.second.len() < 2 {
          self.second.push(c)
        }
      }
      ScheduleField::Timezone => {
        self.timezone_input.push(c);
        self.update_timezone_filter();
      }
      ScheduleField::Submit => {}
    }
  }

  pub fn handle_backspace(&mut self) {
    match self.active_field {
      ScheduleField::Day => {
        self.day.pop();
      }
      ScheduleField::Month => {
        self.month.pop();
      }
      ScheduleField::Year => {
        self.year.pop();
      }
      ScheduleField::Hour => {
        self.hour.pop();
      }
      ScheduleField::Minute => {
        self.minute.pop();
      }
      ScheduleField::Second => {
        self.second.pop();
      }
      ScheduleField::Timezone => {
        self.timezone_input.pop();
        self.update_timezone_filter();
      }
      ScheduleField::Submit => {}
    }
  }

  pub fn cycle_field(&mut self, forward: bool) {
    if forward {
      self.active_field = match self.active_field {
        ScheduleField::Day => ScheduleField::Month,
        ScheduleField::Month => ScheduleField::Year,
        ScheduleField::Year => ScheduleField::Hour,
        ScheduleField::Hour => ScheduleField::Minute,
        ScheduleField::Minute => ScheduleField::Second,
        ScheduleField::Second => ScheduleField::Timezone,
        ScheduleField::Timezone => ScheduleField::Submit,
        ScheduleField::Submit => ScheduleField::Day,
      };
    } else {
      self.active_field = match self.active_field {
        ScheduleField::Day => ScheduleField::Submit,
        ScheduleField::Month => ScheduleField::Day,
        ScheduleField::Year => ScheduleField::Month,
        ScheduleField::Hour => ScheduleField::Year,
        ScheduleField::Minute => ScheduleField::Hour,
        ScheduleField::Second => ScheduleField::Minute,
        ScheduleField::Timezone => ScheduleField::Second,
        ScheduleField::Submit => ScheduleField::Timezone,
      };
    }
  }
}
