use crate::state::AppState;
use crate::ui::ThemeColors;
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub struct ConfirmDeleteWidget<'a> {
    state: &'a AppState,
}

impl<'a> ConfirmDeleteWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for ConfirmDeleteWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.state.confirm_delete {
            return;
        }

        let colors = ThemeColors::from(&self.state.config.themes);

        let centered_rect = |w: u16, h: u16, r: Rect| {
            let popup_layout = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(r.height.saturating_sub(h) / 2),
                    Constraint::Length(h),
                    Constraint::Min(0),
                ])
                .split(r);

            Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(popup_layout[1].width.saturating_sub(w) / 2),
                    Constraint::Length(w),
                    Constraint::Min(0),
                ])
                .split(popup_layout[1])[1]
        };

        let popup_area = centered_rect(42, 3, area);

        Clear.render(popup_area, buf);

        let popup_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.out_state))
            .title(" Delete Entry? ")
            .title_style(Style::default().fg(colors.out_state));

        let text = Line::from(vec![
            Span::styled("Confirm ", Style::default().fg(colors.subtext)),
            Span::styled(
                "<Y>  ",
                Style::default()
                    .fg(colors.border)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled("Cancel ", Style::default().fg(colors.subtext)),
            Span::styled(
                "<N/Esc>",
                Style::default()
                    .fg(colors.border)
                    .add_modifier(Modifier::DIM),
            ),
        ]);

        Paragraph::new(text)
            .block(popup_block)
            .alignment(Alignment::Center)
            .render(popup_area, buf);
    }
}
