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
        let colors = ThemeColors::from(&self.state.config.themes);
        let mut list_items = Vec::new();
        for (i, entry) in self.state.entries.iter().enumerate() {
            let type_str = format!(" {:<3}", entry.entry_type);
            let color = match entry.entry_type {
                EntryType::In => colors.in_state,
                EntryType::Out => colors.out_state,
            };
            
            let is_selected = self.state.selected_entry == Some(i);
            
            let mut line = Line::from(vec![
                Span::styled("   ", Style::default()),
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

        let title_text = if self.state.current_date == chrono::Local::now().date_naive() {
            format!(" Today [{}, {}] ", self.state.current_date.format("%A"), self.state.current_date.format("%Y-%m-%d"))
        } else {
            format!(" {} [{}, {}] ", self.state.current_date.format("%Y-%m-%d"), self.state.current_date.format("%A"), self.state.current_date.format("%Y-%m-%d"))
        };

        let title = if self.state.focus == crate::state::Focus::Main {
            Line::from(vec![
                Span::styled(title_text, Style::default().fg(colors.title).add_modifier(Modifier::BOLD)),
                Span::styled(" <Tab> History ", Style::default().fg(colors.subtext)),
            ])
        } else {
            Line::from(Span::styled(title_text, Style::default().fg(colors.title)))
        };

        List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(if self.state.focus == crate::state::Focus::Main {
                        Style::default().fg(colors.highlight)
                    } else {
                        Style::default().fg(colors.border)
                    })
                    .title(title),
            )
            .render(area, buf);
    }
}
