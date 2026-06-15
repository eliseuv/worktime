use crate::state::{AppState, EntryType};
use crate::ui::ThemeColors;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct InputWidget<'a> {
    state: &'a AppState,
}

impl<'a> InputWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for InputWidget<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let colors = ThemeColors::from(&self.state.config.theme);
        let formatted = match self.state.input_buffer.len() {
            3 => format!(
                "{}:{}",
                &self.state.input_buffer[..1],
                &self.state.input_buffer[1..]
            ),
            4 => format!(
                "{}:{}",
                &self.state.input_buffer[..2],
                &self.state.input_buffer[2..]
            ),
            _ => self.state.input_buffer.clone(),
        };

        let next_type = if let Some(idx) = self.state.selected_entry {
            match self.state.entries[idx].entry_type {
                EntryType::In => "In",
                EntryType::Out => "Out",
            }
        } else {
            match self.state.entries.last() {
                Some(entry) => match entry.entry_type {
                    EntryType::In => "Out",
                    EntryType::Out => "In",
                },
                None => "In",
            }
        };

        let next_color = if next_type == "In" {
            colors.in_state
        } else {
            colors.out_state
        };

        let input_text = Line::from(vec![
            Span::styled(
                format!("({}) ", next_type),
                Style::default().fg(next_color).add_modifier(Modifier::BOLD),
            ),
            Span::styled("> ", Style::default().fg(colors.subtext)),
            Span::styled(
                formatted,
                Style::default()
                    .fg(colors.highlight)
                    .add_modifier(Modifier::BOLD),
            ),
        ]);

        Paragraph::new(input_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.border))
                    .title(" Input (HH:MM) ")
                    .title(
                        ratatui::text::Line::from(vec![
                            ratatui::text::Span::styled("Remove ", Style::default().fg(colors.subtext)),
                            ratatui::text::Span::styled("<Del>  ", Style::default().fg(colors.border).add_modifier(Modifier::DIM)),
                            ratatui::text::Span::styled("Exit ", Style::default().fg(colors.subtext)),
                            ratatui::text::Span::styled("<C-d> ", Style::default().fg(colors.border).add_modifier(Modifier::DIM)),
                        ]).alignment(ratatui::layout::Alignment::Right)
                    )
                    .title_style(Style::default().fg(colors.title)),
            )
            .render(area, buf);
    }
}
