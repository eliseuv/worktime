use crate::state::{AppState, EntryType};
use crossterm::event::{KeyCode, KeyEvent};

pub fn handle_key_event(key_event: KeyEvent, s: &mut AppState) {
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
}
