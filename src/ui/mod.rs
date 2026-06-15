pub mod confirm_delete;
pub mod error;
pub mod header;
pub mod history;
pub mod entries;
pub mod input;
pub mod log;

pub use confirm_delete::ConfirmDeleteWidget;
pub use error::ErrorWidget;
pub use header::HeaderWidget;
pub use history::HistoryWidget;
pub use entries::EntriesWidget;
pub use input::InputWidget;
pub use log::LogWidget;

use crate::config::ThemeConfig;
use ratatui::style::Color;

pub fn parse_hex_color(hex: &str) -> Color {
    let hex = hex.trim_start_matches('#');
    if hex.len() == 6 {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(255);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);
        Color::Rgb(r, g, b)
    } else {
        Color::White
    }
}

pub struct ThemeColors {
    pub text: Color,
    pub border: Color,
    pub title: Color,
    pub highlight: Color,
    pub in_state: Color,
    pub out_state: Color,
    pub subtext: Color,
}

impl From<&ThemeConfig> for ThemeColors {
    fn from(config: &ThemeConfig) -> Self {
        Self {
            text: parse_hex_color(&config.text),
            border: parse_hex_color(&config.border),
            title: parse_hex_color(&config.title),
            highlight: parse_hex_color(&config.highlight),
            in_state: parse_hex_color(&config.in_state),
            out_state: parse_hex_color(&config.out_state),
            subtext: parse_hex_color(&config.subtext),
        }
    }
}
