use crate::state::AppState;
use crate::ui::ThemeColors;
use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::Style,
    widgets::{Block, Borders, Widget},
};
use tui_widgets::big_text::{BigTextBuilder, PixelSize};

pub struct HeaderWidget<'a> {
    state: &'a AppState,
    now: DateTime<Local>,
}

impl<'a> HeaderWidget<'a> {
    pub fn new(state: &'a AppState, now: DateTime<Local>) -> Self {
        Self { state, now }
    }
}

impl<'a> Widget for HeaderWidget<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let colors = ThemeColors::from(&self.state.config.themes);
        let target_seconds = (self.state.config.times.total_time_hours * 3600.0) as i64;
        let worked_time = self.state.calculate_worked_time(self.now);
        let worked_seconds = worked_time.num_seconds();
        let remaining_seconds = target_seconds - worked_seconds;

        let mut finish_time = self.now + chrono::Duration::try_seconds(remaining_seconds.max(0)).unwrap_or_default();
        let considers_lunch = !self.state.has_taken_break() && self.state.config.times.expected_lunch_time_minutes > 0;
        if considers_lunch {
            finish_time += chrono::Duration::try_minutes(self.state.config.times.expected_lunch_time_minutes).unwrap_or_default();
        }
        let finish_str = finish_time.format("%H:%M").to_string();

        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(colors.border))
            .title(ratatui::text::Line::from(" WorkTime ").alignment(Alignment::Left));

        let inner_area = block.inner(area);
        block.render(area, buf);

        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(inner_area);

        let remaining_color = if remaining_seconds <= 0 {
            colors.out_state
        } else {
            colors.highlight // Most important color
        };

        let current_time_str = self.now.format("%H:%M:%S").to_string();
        
        let (sign, abs_seconds) = if remaining_seconds < 0 {
            ("-", -remaining_seconds)
        } else {
            ("", remaining_seconds)
        };
        let hours = abs_seconds / 3600;
        let minutes = (abs_seconds % 3600) / 60;
        let seconds = abs_seconds % 60;
        let total_time_str = format!("{}{:02}:{:02}:{:02}", sign, hours, minutes, seconds);

        let center_vertically = |r: ratatui::layout::Rect, h: u16| {
            if r.height <= h {
                return r;
            }
            let top_margin = (r.height - h) / 2;
            Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(top_margin),
                    Constraint::Length(h),
                    Constraint::Min(0),
                ])
                .split(r)[1]
        };

        // Remaining Time
        let left_block = Block::default()
            .title(ratatui::text::Line::from(ratatui::text::Span::styled(" Remaining Time ", Style::default().fg(remaining_color))).alignment(Alignment::Center));
        let left_inner = left_block.inner(chunks[0]);
        left_block.render(chunks[0], buf);

        let left_centered = center_vertically(left_inner, 4);
        BigTextBuilder::default()
            .pixel_size(PixelSize::HalfHeight)
            .alignment(Alignment::Center)
            .lines(vec![total_time_str.into()])
            .style(Style::default().fg(remaining_color))
            .build()
            .render(left_centered, buf);

        // Estimated Finish Time
        let title_str = if considers_lunch { " Estimated Finish (+Lunch) " } else { " Estimated Finish " };
        let middle_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(colors.border))
            .title(ratatui::text::Line::from(ratatui::text::Span::styled(title_str, Style::default().fg(colors.in_state))).alignment(Alignment::Center));
        let middle_inner = middle_block.inner(chunks[1]);
        middle_block.render(chunks[1], buf);

        let middle_centered = center_vertically(middle_inner, 4);
        BigTextBuilder::default()
            .pixel_size(PixelSize::HalfHeight)
            .alignment(Alignment::Center)
            .lines(vec![finish_str.into()])
            .style(Style::default().fg(colors.in_state))
            .build()
            .render(middle_centered, buf);

        // Current Time
        let right_block = Block::default()
            .borders(Borders::LEFT)
            .border_style(Style::default().fg(colors.border))
            .title(ratatui::text::Line::from(ratatui::text::Span::styled(" Current Time ", Style::default().fg(colors.subtext))).alignment(Alignment::Center));
        let right_inner = right_block.inner(chunks[2]);
        right_block.render(chunks[2], buf);

        let right_centered = center_vertically(right_inner, 4);
        BigTextBuilder::default()
            .pixel_size(PixelSize::HalfHeight)
            .alignment(Alignment::Center)
            .lines(vec![current_time_str.into()])
            .style(Style::default().fg(colors.subtext))
            .build()
            .render(right_centered, buf);
    }
}
