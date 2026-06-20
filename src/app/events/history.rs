use crate::state::{AppState, Focus};
use crossterm::event::{KeyCode, KeyEvent};
use crate::db;

pub fn handle_key_event(key_event: KeyEvent, s: &mut AppState) -> bool {
    match key_event.code {
        KeyCode::Tab => {
            s.focus = Focus::Main;
            s.history_selected_date = None;
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::Char('K') => {
            if !s.history_dates.is_empty() {
                let new_idx = match s.history_selected_date {
                    Some(idx) => idx.saturating_sub(1),
                    None => s.history_dates.len() - 1,
                };
                s.history_selected_date = Some(new_idx);
            }
        }
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Char('J') => {
            if let Some(idx) = s.history_selected_date {
                if idx + 1 < s.history_dates.len() {
                    s.history_selected_date = Some(idx + 1);
                }
            } else if !s.history_dates.is_empty() {
                s.history_selected_date = Some(0);
            }
        }
        KeyCode::Enter => {
            if let Some(idx) = s.history_selected_date {
                let date_str = &s.history_dates[idx];
                if let Ok(target_date) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    // Save current date's entries
                    let db_path = s.config.get_db_path();
                    if let Err(e) = db::save_entries(&db_path, s.current_date, &s.entries) {
                        s.error_msg = Some(format!("Failed to save entries: {}", e));
                    } else {
                        // Load new date's entries
                        match db::load_entries_for_date(&db_path, target_date) {
                            Ok(entries) => {
                                s.entries = entries;
                                s.current_date = target_date;
                                s.focus = Focus::Main;
                                s.history_selected_date = None;
                                s.selected_entry = None;
                                s.add_log(format!("Loaded entries for {}", target_date));
                                
                                if let Ok(new_history) = db::load_history(&db_path) {
                                    s.history = new_history;
                                    let mut history_dates = Vec::new();
                                    let mut current_date_str = String::new();
                                    for entry in &s.history {
                                        if let Some(date) = entry.time_str.split('T').next() {
                                            if date != current_date_str {
                                                history_dates.push(date.to_string());
                                                current_date_str = date.to_string();
                                            }
                                        }
                                    }
                                    s.history_dates = history_dates;
                                }
                            }
                            Err(e) => {
                                s.error_msg = Some(format!("Failed to load entries: {}", e));
                            }
                        }
                    }
                }
            }
        }
        KeyCode::Esc => {
            s.focus = Focus::Main;
            s.history_selected_date = None;
        }
        KeyCode::Char('q') | KeyCode::Char('Q') => {
            return true;
        }
        _ => {}
    }
    false
}
