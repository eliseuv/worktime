use crate::config::{AppConfig, NotificationInterval};
use crate::state::AppState;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub fn handle_key_event(key_event: KeyEvent, s: &mut AppState) {
    match key_event.code {
        KeyCode::Esc => {
            s.options_open = false;
        }
        KeyCode::Char('s') | KeyCode::Char('S') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
            let mut new_config = s.config.clone();
            new_config.notifications.intervals.clear();

            let mut has_error = false;
            for (_section, key, values) in &s.config_fields {
                let value = values.get(0).cloned().unwrap_or_default();
                match key.as_str() {
                    "Total Time (Hours)" => { if let Ok(val) = value.parse::<f64>() { new_config.times.total_time_hours = val; } }
                    "Overtime Threshold (Mins)" => { if let Ok(val) = value.parse::<i64>() { new_config.times.overtime_threshold_minutes = val; } }
                    "Done Msg" => new_config.notifications.done_message = value.clone(),
                    "DB Path" => {
                        let db_val = value.trim();
                        new_config.database.path = if db_val.is_empty() { None } else { Some(db_val.to_string()) };
                    }
                    "Text Color" => new_config.themes.text = value.clone(),
                    "Border Color" => new_config.themes.border = value.clone(),
                    "Title Color" => new_config.themes.title = value.clone(),
                    "Highlight Color" => new_config.themes.highlight = value.clone(),
                    "In State Color" => new_config.themes.in_state = value.clone(),
                    "Out State Color" => new_config.themes.out_state = value.clone(),
                    "Subtext Color" => new_config.themes.subtext = value.clone(),
                    k if k.starts_with("Notification ") => {
                        let m1 = values.get(0).map(|s| s.trim().to_string()).unwrap_or_default();
                        let m2 = values.get(1).map(|s| s.trim().to_string()).unwrap_or_default();
                        
                        if !m1.is_empty() && !m2.is_empty() {
                            if let Ok(mins) = m1.parse::<i64>() {
                                new_config.notifications.intervals.push(NotificationInterval {
                                    minutes: mins,
                                    message: m2,
                                });
                            }
                        } else if (!m1.is_empty() && m2.is_empty()) || (m1.is_empty() && !m2.is_empty()) {
                            s.error_msg = Some("Both Minutes and Message are required for an interval.".to_string());
                            has_error = true;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            
            if !has_error {
                if let Some(path) = AppConfig::get_config_path() {
                    if let Ok(yaml) = serde_yaml::to_string(&new_config) {
                        if let Err(e) = std::fs::write(&path, &yaml) {
                            s.error_msg = Some(format!("Failed to save config: {}", e));
                        } else {
                            s.config = new_config;
                            s.add_log("Config saved and applied.".to_string());
                            s.options_open = false;
                        }
                    }
                }
            }
        }
        KeyCode::Up => {
            if s.selected_field > 0 { s.selected_field -= 1; }
            s.selected_col = s.selected_col.min(s.config_fields[s.selected_field].2.len().saturating_sub(1));
            s.cursor_x = s.cursor_x.min(s.config_fields[s.selected_field].2[s.selected_col].len());
            if s.selected_field < s.config_scroll_y { s.config_scroll_y = s.selected_field; }
        }
        KeyCode::Down => {
            if s.selected_field + 1 < s.config_fields.len() { s.selected_field += 1; }
            s.selected_col = s.selected_col.min(s.config_fields[s.selected_field].2.len().saturating_sub(1));
            s.cursor_x = s.cursor_x.min(s.config_fields[s.selected_field].2[s.selected_col].len());
        }
        KeyCode::Left => {
            if s.cursor_x > 0 {
                s.cursor_x -= 1;
            } else if s.selected_col > 0 {
                s.selected_col -= 1;
                s.cursor_x = s.config_fields[s.selected_field].2[s.selected_col].len();
            }
        }
        KeyCode::Right => {
            let sf = s.selected_field;
            let sc = s.selected_col;
            if s.cursor_x < s.config_fields[sf].2[sc].len() {
                s.cursor_x += 1;
            } else if sc + 1 < s.config_fields[sf].2.len() {
                s.selected_col += 1;
                s.cursor_x = 0;
            }
        }
        KeyCode::Backspace => {
            if s.cursor_x > 0 {
                let sf = s.selected_field;
                let sc = s.selected_col;
                let cx = s.cursor_x;
                s.config_fields[sf].2[sc].remove(cx - 1);
                s.cursor_x -= 1;
            }
        }
        KeyCode::Delete => {
            let sf = s.selected_field;
            let sc = s.selected_col;
            if s.config_fields[sf].1.starts_with("Notification ") {
                s.config_fields.remove(sf);
                
                let mut interval_idx = 1;
                for (_, key, _) in s.config_fields.iter_mut() {
                    if key.starts_with("Notification ") {
                        *key = format!("Notification {}", interval_idx);
                        interval_idx += 1;
                    }
                }

                if s.selected_field >= s.config_fields.len() {
                    s.selected_field = s.config_fields.len().saturating_sub(1);
                }
                s.selected_col = 0;
                let sf_new = s.selected_field;
                s.cursor_x = s.cursor_x.min(s.config_fields[sf_new].2[s.selected_col].len());
            } else {
                let cx = s.cursor_x;
                if cx < s.config_fields[sf].2[sc].len() {
                    s.config_fields[sf].2[sc].remove(cx);
                }
            }
        }
        KeyCode::Enter => {
            let sf = s.selected_field;
            let sc = s.selected_col;
            if sc + 1 < s.config_fields[sf].2.len() {
                s.selected_col += 1;
                s.cursor_x = s.config_fields[sf].2[s.selected_col].len();
            } else if s.selected_field + 1 < s.config_fields.len() {
                s.selected_field += 1;
                s.selected_col = 0;
                s.cursor_x = s.config_fields[s.selected_field].2[0].len();
            }
        }
        KeyCode::Char(c) => {
            if !key_event.modifiers.contains(KeyModifiers::CONTROL) {
                let sf = s.selected_field;
                let sc = s.selected_col;
                let cx = s.cursor_x;
                s.config_fields[sf].2[sc].insert(cx, c);
                s.cursor_x += 1;
            }
        }
        _ => {}
    }
}
