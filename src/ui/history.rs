use crate::state::AppState;
use crate::ui::ThemeColors;
use chrono::DateTime;
use ratatui::{
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};
use std::collections::HashMap;

pub struct HistoryWidget<'a> {
    state: &'a AppState,
}

impl<'a> HistoryWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for HistoryWidget<'a> {
    fn render(self, area: ratatui::layout::Rect, buf: &mut ratatui::buffer::Buffer) {
        let colors = ThemeColors::from(&self.state.config.theme);

        let mut daily_totals: HashMap<String, i64> = HashMap::new();
        let mut last_in = None;
        for entry in &self.state.history {
            let date = entry.time_str.split('T').next().unwrap_or("").to_string();
            if let Ok(dt) = DateTime::parse_from_rfc3339(&entry.time_str) {
                if entry.entry_type == "IN" {
                    last_in = Some((date, dt));
                } else if entry.entry_type == "OUT" {
                    if let Some((in_date, in_dt)) = last_in {
                        let duration = dt.signed_duration_since(in_dt).num_minutes();
                        *daily_totals.entry(in_date).or_insert(0) += duration;
                    }
                    last_in = None;
                }
            }
        }

        let mut list_items = Vec::new();
        let mut current_date = String::new();

        for entry in &self.state.history {
            let parts: Vec<&str> = entry.time_str.split('T').collect();
            let date = parts.first().copied().unwrap_or("");
            let time = parts
                .get(1)
                .map(|t| if t.len() >= 5 { &t[..5] } else { *t })
                .unwrap_or("");

            if date != current_date {
                current_date = date.to_string();
                let total_mins = daily_totals.get(date).copied().unwrap_or(0);
                let hours = total_mins / 60;
                let mins = total_mins % 60;
                let total_str = format!("  Total: {:02}:{:02} ", hours, mins);

                let target_mins = (self.state.config.total_time_hours * 60.0) as i64;
                let overtime = total_mins - target_mins;

                let mut spans = vec![
                    Span::styled(
                        format!(" [ {} ]", date),
                        Style::default()
                            .fg(colors.title)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(total_str, Style::default().fg(colors.highlight)),
                ];

                if overtime > self.state.config.overtime_threshold_minutes {
                    spans.push(Span::styled(
                        format!("(+{}m) ", overtime),
                        Style::default()
                            .fg(colors.out_state)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else if overtime < -self.state.config.overtime_threshold_minutes {
                    spans.push(Span::styled(
                        format!("({}m) ", overtime),
                        Style::default()
                            .fg(colors.out_state)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                list_items.push(ListItem::new(Line::from(spans)));
            }

            let type_str = if entry.entry_type == "IN" {
                " IN "
            } else {
                " OUT"
            };
            let color = if entry.entry_type == "IN" {
                colors.in_state
            } else {
                colors.out_state
            };

            list_items.push(ListItem::new(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(
                    type_str,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" at ", Style::default().fg(colors.subtext)),
                Span::styled(time, Style::default().fg(colors.text)),
            ])));
        }

        List::new(list_items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.border))
                    .title(" DB History ")
                    .title_style(Style::default().fg(colors.title)),
            )
            .render(area, buf);
    }
}
