use crate::state::AppState;
use crate::ui::ThemeColors;
use ratatui::{
    style::{Modifier, Style},
    text::Span,
    widgets::{Paragraph, Widget},
};

pub struct ErrorWidget<'a> {
    state: &'a AppState,
}

impl<'a> ErrorWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for ErrorWidget<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let colors = ThemeColors::from(&self.state.config.themes);
        let err_text = if let Some(err) = &self.state.error_msg {
            Span::styled(
                format!("Error: {}", err),
                Style::default()
                    .fg(colors.out_state)
                    .add_modifier(Modifier::BOLD),
            )
        } else {
            Span::raw("")
        };
        Paragraph::new(err_text).render(area, buf);
    }
}
