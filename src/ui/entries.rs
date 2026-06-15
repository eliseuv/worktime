use crate::state::{AppState, EntryType};
use crate::ui::ThemeColors;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

pub struct EntriesWidget<'a> {
    state: &'a AppState,
}

impl<'a> EntriesWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for EntriesWidget<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let colors = ThemeColors::from(&self.state.config.theme);
        let mut list_items = Vec::new();
        for (i, entry) in self.state.entries.iter().enumerate() {
            let type_str = format!(" {:<3}", entry.entry_type);
            let color = match entry.entry_type {
                EntryType::In => colors.in_state,
                EntryType::Out => colors.out_state,
            };
            
            let is_selected = self.state.selected_entry == Some(i);
            
            let mut line = Line::from(vec![
                Span::styled(
                    type_str,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" at ", Style::default().fg(colors.subtext)),
                Span::styled(
                    entry.time.format("%H:%M").to_string(),
                    Style::default().fg(colors.text),
                ),
            ]);

            if is_selected {
                line = line.style(Style::default().add_modifier(Modifier::REVERSED));
            }

            list_items.push(ListItem::new(line));
        }

        List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.border))
                    .title(format!(" Today [{}, {}] ", chrono::Local::now().format("%A"), chrono::Local::now().format("%Y-%m-%d")))
                    .title_style(Style::default().fg(colors.title)),
            )
            .render(area, buf);
    }
}
