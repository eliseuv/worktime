pub mod confirm;
pub mod history;
pub mod main_screen;
pub mod options;

use crate::state::AppState;
use crossterm::event::KeyEvent;

pub fn handle_key_event(key_event: KeyEvent, s: &mut AppState) -> bool {
    if s.options_open {
        options::handle_key_event(key_event, s);
        return false;
    }

    if s.confirm_delete {
        confirm::handle_key_event(key_event, s);
        return false;
    }

    match s.focus {
        crate::state::Focus::Main => main_screen::handle_key_event(key_event, s),
        crate::state::Focus::History => history::handle_key_event(key_event, s),
    }
}
