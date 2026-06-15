use crate::state::AppState;
use crate::ui::ThemeColors;
use ratatui::{
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};

pub struct LogWidget<'a> {
    state: &'a AppState,
}

impl<'a> LogWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for LogWidget<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let colors = ThemeColors::from(&self.state.config.themes);
        
        let list_items: Vec<ListItem> = self.state.app_logs.iter().map(|log| {
            ListItem::new(Line::from(vec![
                Span::styled(" ", Style::default()),
                Span::styled(log.clone(), Style::default().fg(colors.subtext)),
            ]))
        }).collect();

        // If we want it to auto-scroll, we could use a ListState, but let's just display it normally or slice the last N items.
        // Actually, List renders from top to bottom, maybe we should reverse the items or rely on List rendering.
        // Since it's a log, having the newest at the bottom or top is fine. Let's just pass them as is, but if it overflows, 
        // the top logs might hide the bottom ones. 
        // Let's take the last `area.height` items to show newest logs if we don't have scrolling.
        let height = area.height.saturating_sub(2) as usize; // account for borders
        let display_items = if list_items.len() > height {
            list_items[list_items.len() - height..].to_vec()
        } else {
            list_items
        };

        List::new(display_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.border))
                    .title(" Logs ")
                    .title_style(Style::default().fg(colors.title)),
            )
            .render(area, buf);
    }
}
