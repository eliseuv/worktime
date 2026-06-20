pub mod background;
pub mod events;

use crate::config::AppConfig;
use crate::db;
use crate::state::AppState;
use crate::ui::{
    ConfirmDeleteWidget, EntriesWidget, ErrorWidget, HeaderWidget, HistoryWidget, InputWidget,
    LogWidget, OptionsWidget,
};

use chrono::Local;
use crossterm::{
    event::{self, Event, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    Terminal,
};
use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration as StdDuration,
};

pub struct App {
    state: Arc<Mutex<AppState>>,
}

impl App {
    pub fn new() -> io::Result<Self> {
        let config = AppConfig::load_or_default();
        let db_path = config.get_db_path();
        let history = db::load_history(&db_path).unwrap_or_default();
        let today = chrono::Local::now().date_naive();
        let today_entries = db::load_entries_for_date(&db_path, today).unwrap_or_default();
        let mut state_data = AppState::new(config, history);
        state_data.entries = today_entries;
        state_data.current_date = today;
        state_data.add_log("Config loaded successfully.".to_string());
        state_data.add_log(format!("Loaded history from {}", db_path.display()));
        state_data.add_log(format!("Loaded {} entries for today.", state_data.entries.len()));

        let state = Arc::new(Mutex::new(state_data));

        Ok(Self { state })
    }

    pub fn main_loop(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        background::start_background_thread(Arc::clone(&self.state));

        loop {
            let now = Local::now();
            let mut s = self.state.lock().unwrap();

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
                f.render_widget(OptionsWidget::new(&s), f.area());
            })?;

            // Handle events
            if event::poll(StdDuration::from_millis(100))? {
                if let Event::Key(key_event) = event::read()? {
                    if key_event.kind == KeyEventKind::Press {
                        let should_exit = events::handle_key_event(key_event, &mut s);
                        if should_exit {
                            break;
                        }
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.show_cursor()?;

        Ok(())
    }

    pub fn shutdown(&self) -> io::Result<()> {
        let final_state = self.state.lock().unwrap();
        let db_path = final_state.config.get_db_path();
        if let Err(e) = db::save_entries(&db_path, final_state.current_date, &final_state.entries) {
            eprintln!("Failed to save entries to database: {}", e);
        }
        Ok(())
    }
}
