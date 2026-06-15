use crate::state::{AppState, Entry, EntryType};
use chrono::{Local, NaiveTime};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn parse_input(input: &str) -> Result<NaiveTime, String> {
    let input = input.trim();
    NaiveTime::parse_from_str(input, "%H:%M").map_err(|_| "Invalid time format. Use HH:MM".into())
}

pub fn handle_key_event(key_event: KeyEvent, s: &mut AppState) -> bool {
    if key_event.code == KeyCode::Char('d') && key_event.modifiers.contains(KeyModifiers::CONTROL) {
        return true; // Exit
    }

    match key_event.code {
        KeyCode::Char('o') | KeyCode::Char('O') => {
            if !s.options_open {
                let mut fields = vec![
                    ("Times".to_string(), "Total Time (Hours)".to_string(), vec![s.config.times.total_time_hours.to_string()]),
                    ("Times".to_string(), "Overtime Threshold (Mins)".to_string(), vec![s.config.times.overtime_threshold_minutes.to_string()]),
                    ("Notifications".to_string(), "Done Msg".to_string(), vec![s.config.notifications.done_message.clone()]),
                ];

                for (i, interval) in s.config.notifications.intervals.iter().enumerate() {
                    fields.push(("Notifications".to_string(), format!("Notification {}", i + 1), vec![interval.minutes.to_string(), interval.message.clone()]));
                }
                
                let next_idx = s.config.notifications.intervals.len() + 1;
                fields.push(("Notifications".to_string(), format!("Notification {}", next_idx), vec![String::new(), String::new()]));

                fields.extend(vec![
                    ("Database".to_string(), "DB Path".to_string(), vec![s.config.get_db_path().to_string_lossy().to_string()]),
                    ("Themes".to_string(), "Text Color".to_string(), vec![s.config.themes.text.clone()]),
                    ("Themes".to_string(), "Border Color".to_string(), vec![s.config.themes.border.clone()]),
                    ("Themes".to_string(), "Title Color".to_string(), vec![s.config.themes.title.clone()]),
                    ("Themes".to_string(), "Highlight Color".to_string(), vec![s.config.themes.highlight.clone()]),
                    ("Themes".to_string(), "In State Color".to_string(), vec![s.config.themes.in_state.clone()]),
                    ("Themes".to_string(), "Out State Color".to_string(), vec![s.config.themes.out_state.clone()]),
                    ("Themes".to_string(), "Subtext Color".to_string(), vec![s.config.themes.subtext.clone()]),
                ]);

                s.config_fields = fields;
                s.selected_field = 0;
                s.selected_col = 0;
                s.cursor_x = s.config_fields[0].2[0].len();
                s.config_scroll_y = 0;
                s.options_open = true;
                return false;
            }
        }
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
            if s.selected_entry.is_some() || !s.input_buffer.is_empty() || s.error_msg.is_some() {
                s.selected_entry = None;
                s.input_buffer.clear();
                s.error_msg = None;
            } else {
                return true; // Exit
            }
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

    false
}
