pub mod config;
pub mod db;
pub mod state;
pub mod ui;

use chrono::{Local, NaiveTime};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use notify_rust::Notification;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
};
use std::{
    io,
    sync::{Arc, Mutex},
    thread,
    time::Duration as StdDuration,
};

use crate::config::AppConfig;
use crate::state::{AppState, Entry, EntryType};
use crate::ui::{
    ConfirmDeleteWidget, EntriesWidget, ErrorWidget, HeaderWidget, HistoryWidget, InputWidget, LogWidget,
};

fn parse_input(input: &str) -> Result<NaiveTime, String> {
    let input = input.trim();
    NaiveTime::parse_from_str(input, "%H:%M").map_err(|_| "Invalid time format. Use HH:MM".into())
}

fn main() -> io::Result<()> {
    let config = AppConfig::load_or_default();
    let db_path = config.get_db_path();
    let history = db::load_history(&db_path).unwrap_or_default();
    let today_entries = db::load_today_entries(&db_path).unwrap_or_default();
    let mut state_data = AppState::new(config, history);
    state_data.entries = today_entries;
    state_data.add_log("Config loaded successfully.".to_string());
    state_data.add_log(format!("Loaded history from {}", db_path.display()));
    state_data.add_log(format!("Loaded {} entries for today.", state_data.entries.len()));
    let state = Arc::new(Mutex::new(state_data));

    // Background thread to check for intervals and total time
    let state_clone = Arc::clone(&state);
    thread::spawn(move || {
        loop {
            thread::sleep(StdDuration::from_secs(1));
            let now = Local::now();
            let mut s = state_clone.lock().unwrap();

            let worked = s.calculate_worked_time(now);
            let worked_minutes = worked.num_minutes();
            let total_time_minutes = (s.config.total_time_hours * 60.0) as i64;
            let remaining_minutes = total_time_minutes - worked_minutes;

            if remaining_minutes <= 0 && !s.notified_done {
                let body = &s.config.notifications.done_message;
                let _ = Notification::new()
                    .summary("WorkTime Alert")
                    .body(body)
                    .show();
                s.notified_done = true;
            }

            let intervals = s.config.notifications.intervals.clone();
            for interval in intervals {
                if remaining_minutes <= interval.minutes
                    && remaining_minutes > 0
                    && !s.notified_intervals.contains(&interval.minutes)
                {
                    let body = interval.message.replace("{mins}", &interval.minutes.to_string());
                    let _ = Notification::new()
                        .summary("WorkTime Alert")
                        .body(&body)
                        .show();
                    s.notified_intervals.insert(interval.minutes);
                }
            }
        }
    });

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    loop {
        let now = Local::now();
        let mut s = state.lock().unwrap();

        terminal.draw(|f| {
            let outer_chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(10), Constraint::Min(0)])
                .split(f.area());

            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(outer_chunks[1]);

            let error_len = if s.error_msg.is_some() { 2 } else { 0 };

            let left_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Percentage(50),    // Entries
                    Constraint::Length(error_len), // Error
                    Constraint::Length(3),         // Input
                    Constraint::Min(5),            // Program Logs
                ])
                .split(main_chunks[0]);

            f.render_widget(HeaderWidget::new(&s, now), outer_chunks[0]);
            f.render_widget(EntriesWidget::new(&s), left_chunks[0]);
            f.render_widget(ErrorWidget::new(&s), left_chunks[1]);
            f.render_widget(InputWidget::new(&s), left_chunks[2]);
            f.render_widget(LogWidget::new(&s), left_chunks[3]);
            f.render_widget(HistoryWidget::new(&s), main_chunks[1]);
            f.render_widget(ConfirmDeleteWidget::new(&s), f.area());
        })?;

        // Handle events
        if event::poll(StdDuration::from_millis(100))?
            && let Event::Key(key_event) = event::read()?
            && key_event.kind == KeyEventKind::Press
        {
            if s.confirm_delete {
                match key_event.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        if let Some(idx) = s.selected_entry {
                            let deleted_time = s.entries[idx].time;
                            s.entries.remove(idx);
                            s.add_log(format!("Deleted entry at {}", deleted_time.format("%H:%M")));
                            s.selected_entry = None;
                            s.input_buffer.clear();
                            
                            let mut current = EntryType::In;
                            for entry in &mut s.entries {
                                entry.entry_type = current.clone();
                                current = match current {
                                    EntryType::In => EntryType::Out,
                                    EntryType::Out => EntryType::In,
                                };
                            }
                        } else if !s.entries.is_empty() {
                            let deleted = s.entries.pop().unwrap();
                            s.add_log(format!("Deleted entry at {}", deleted.time.format("%H:%M")));
                        }
                        s.confirm_delete = false;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        s.confirm_delete = false;
                    }
                    _ => {}
                }
                continue;
            }

            if key_event.code == KeyCode::Char('d')
                && key_event.modifiers.contains(KeyModifiers::CONTROL)
            {
                break;
            }
            match key_event.code {
                KeyCode::Up => {
                    if !s.entries.is_empty() {
                        let new_idx = match s.selected_entry {
                            Some(idx) => idx.saturating_sub(1),
                            None => s.entries.len() - 1,
                        };
                        s.selected_entry = Some(new_idx);
                        s.input_buffer = s.entries[new_idx].time.format("%H%M").to_string();
                    }
                }
                KeyCode::Down => {
                    if let Some(idx) = s.selected_entry {
                        if idx + 1 < s.entries.len() {
                            s.selected_entry = Some(idx + 1);
                            s.input_buffer = s.entries[idx + 1].time.format("%H%M").to_string();
                        } else {
                            s.selected_entry = None;
                            s.input_buffer.clear();
                        }
                    } else if !s.entries.is_empty() {
                        s.selected_entry = Some(0);
                        s.input_buffer = s.entries[0].time.format("%H%M").to_string();
                    }
                }
                KeyCode::Esc => {
                    s.selected_entry = None;
                    s.input_buffer.clear();
                    s.error_msg = None;
                }
                KeyCode::Char(c) => {
                    if c.is_ascii_digit() && s.input_buffer.len() < 4 {
                        s.input_buffer.push(c);
                    }
                }
                KeyCode::Delete => {
                    if !s.entries.is_empty() {
                        s.confirm_delete = true;
                    }
                }
                KeyCode::Backspace => {
                    s.input_buffer.pop();
                }
                KeyCode::Enter => {
                    let raw_input = s.input_buffer.clone();
                    s.input_buffer.clear();
                    s.error_msg = None;

                    let input_to_parse = match raw_input.len() {
                        3 => format!("0{}:{}", &raw_input[..1], &raw_input[1..]),
                        4 => format!("{}:{}", &raw_input[..2], &raw_input[2..]),
                        _ => raw_input.clone(),
                    };

                    let input_trimmed = input_to_parse.trim();

                    if !input_trimmed.is_empty() {
                        match parse_input(input_trimmed) {
                            Ok(nt) => {
                                if let Some(idx) = s.selected_entry {
                                    let orig_time = s.entries[idx].time;
                                    let mut valid_dt = None;
                                    
                                    for offset in [0, -1, 1] {
                                        let test_date = orig_time.date_naive() + chrono::Duration::try_days(offset).unwrap_or_default();
                                        if let Some(dt) = test_date.and_time(nt).and_local_timezone(Local::now().timezone()).single() {
                                            let after_prev = if idx > 0 { dt >= s.entries[idx - 1].time } else { true };
                                            let before_next = if idx + 1 < s.entries.len() { dt <= s.entries[idx + 1].time } else { true };
                                            if after_prev && before_next {
                                                valid_dt = Some(dt);
                                                break;
                                            }
                                        }
                                    }
                                    
                                    if let Some(dt) = valid_dt {
                                        s.entries[idx].time = dt;
                                        s.add_log(format!("Edited entry from {} to {}", orig_time.format("%H:%M"), dt.format("%H:%M")));
                                        s.selected_entry = None;
                                    } else {
                                        s.error_msg = Some("Time must be between previous and next entry".into());
                                        s.input_buffer = raw_input;
                                    }
                                } else {
                                    let mut target_date = Local::now().date_naive();
                                    if let Some(last_entry) = s.entries.last() {
                                        if nt < last_entry.time.time() {
                                            target_date = last_entry.time.date_naive() + chrono::Duration::try_days(1).unwrap_or_default();
                                        } else {
                                            target_date = last_entry.time.date_naive();
                                        }
                                    }

                                    if let Some(dt) = target_date
                                        .and_time(nt)
                                        .and_local_timezone(Local::now().timezone())
                                        .single()
                                    {
                                        s.entries.push(Entry {
                                            entry_type: EntryType::In,
                                            time: dt,
                                        });
                                        s.add_log(format!("Added new entry at {}", dt.format("%H:%M")));
                                        s.entries.sort_by_key(|e| e.time);
                                        let mut current = EntryType::In;
                                        for entry in &mut s.entries {
                                            entry.entry_type = current.clone();
                                            current = match current {
                                                EntryType::In => EntryType::Out,
                                                EntryType::Out => EntryType::In,
                                            };
                                        }
                                    } else {
                                        s.error_msg = Some("Could not resolve local time.".into());
                                    }
                                }
                            }
                            Err(e) => {
                                s.error_msg = Some(e);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    let final_state = state.lock().unwrap();
    let db_path = final_state.config.get_db_path();
    if let Err(e) = db::save_entries(&db_path, &final_state.entries) {
        eprintln!("Failed to save entries to database: {}", e);
    }

    Ok(())
}
