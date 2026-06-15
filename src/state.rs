use crate::config::AppConfig;
use chrono::{DateTime, Duration, Local};
use std::collections::HashSet;
use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum EntryType {
    In,
    Out,
}

impl fmt::Display for EntryType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntryType::In => write!(f, "In"),
            EntryType::Out => write!(f, "Out"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub entry_type: EntryType,
    pub time: DateTime<Local>,
}

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub entries: Vec<Entry>,
    pub input_buffer: String,
    pub notified_done: bool,
    pub notified_intervals: HashSet<i64>,
    pub error_msg: Option<String>,
    pub confirm_delete: bool,
    pub history: Vec<crate::db::DbEntry>,
    pub selected_entry: Option<usize>,
    pub app_logs: Vec<String>,
}

impl AppState {
    pub fn new(config: AppConfig, history: Vec<crate::db::DbEntry>) -> Self {
        Self {
            config,
            entries: Vec::new(),
            input_buffer: String::new(),
            notified_done: false,
            notified_intervals: HashSet::new(),
            error_msg: None,
            confirm_delete: false,
            history,
            selected_entry: None,
            app_logs: Vec::new(),
        }
    }

    pub fn add_log(&mut self, msg: String) {
        let ts = chrono::Local::now().format("%H:%M:%S").to_string();
        self.app_logs.push(format!("[{}] {}", ts, msg));
    }

    pub fn calculate_worked_time(&self, now: DateTime<Local>) -> Duration {
        let mut total = Duration::zero();
        let mut last_in: Option<DateTime<Local>> = None;

        for entry in &self.entries {
            match entry.entry_type {
                EntryType::In => {
                    last_in = Some(entry.time);
                }
                EntryType::Out => {
                    if let Some(in_time) = last_in {
                        total += entry.time - in_time;
                        last_in = None;
                    }
                }
            }
        }

        if let Some(in_time) = last_in {
            total += now - in_time;
        }

        total
    }
}
