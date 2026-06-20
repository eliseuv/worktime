use crate::state::{AppState, EntryType};
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
        let colors = ThemeColors::from(&self.state.config.themes);

        let mut daily_totals: HashMap<String, i64> = HashMap::new();
        let mut daily_incomplete: HashMap<String, bool> = HashMap::new();
        let mut last_in = None;
        for entry in &self.state.history {
            let date = entry.time_str.split('T').next().unwrap_or("").to_string();
            if let Ok(dt) = DateTime::parse_from_rfc3339(&entry.time_str) {
                if entry.entry_type == EntryType::In {
                    last_in = Some((date.clone(), dt));
                    daily_incomplete.insert(date, true);
                } else if entry.entry_type == EntryType::Out {
                    if let Some((in_date, in_dt)) = last_in {
                        let duration = dt.signed_duration_since(in_dt).num_minutes();
                        *daily_totals.entry(in_date.clone()).or_insert(0) += duration;
                        daily_incomplete.insert(in_date, false);
                    }
                    last_in = None;
                }
            }
        }

        let mut list_items = Vec::new();
        let mut current_date = String::new();
        let mut date_indices = Vec::new(); // to map s.history_dates index to list item index
        let mut current_date_idx: Option<usize> = None;

        for entry in &self.state.history {
            let parts: Vec<&str> = entry.time_str.split('T').collect();
            let date = parts.first().copied().unwrap_or("");
            let time = parts
                .get(1)
                .map(|t| if t.len() >= 5 { &t[..5] } else { *t })
                .unwrap_or("");

            if date != current_date {
                current_date = date.to_string();
                current_date_idx = Some(date_indices.len());
                date_indices.push(list_items.len()); // This list item corresponds to a new date

                let total_mins = daily_totals.get(date).copied().unwrap_or(0);
                let hours = total_mins / 60;
                let mins = total_mins % 60;
                let total_str = format!("  Total: {:02}:{:02} ", hours, mins);

                let target_mins = (self.state.config.times.total_time_hours * 60.0) as i64;
                let overtime = total_mins - target_mins;

                let display_date = if let Ok(dt) = DateTime::parse_from_rfc3339(&entry.time_str) {
                    format!(" [{}, {}] ", dt.format("%A"), date)
                } else {
                    format!(" [ {} ]", date)
                };

                let mut spans = vec![
                    Span::styled(
                        display_date,
                        Style::default()
                            .fg(colors.title)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(total_str, Style::default().fg(colors.highlight)),
                ];

                let incomplete = daily_incomplete.get(date).copied().unwrap_or(false);
                if overtime > self.state.config.times.overtime_threshold_minutes {
                    spans.push(Span::styled(
                        format!("(+{}m) ", overtime),
                        Style::default()
                            .fg(colors.out_state)
                            .add_modifier(Modifier::BOLD),
                    ));
                } else if overtime < -self.state.config.times.overtime_threshold_minutes
                    && !incomplete
                {
                    spans.push(Span::styled(
                        format!("({}m) ", overtime),
                        Style::default()
                            .fg(colors.out_state)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                let is_selected_day = self.state.focus == crate::state::Focus::History
                    && self.state.history_selected_date == current_date_idx;

                let mut item = ListItem::new(Line::from(spans));
                if is_selected_day {
                    item = item.style(Style::default().bg(colors.border));
                }
                list_items.push(item);
            }

            let is_in = entry.entry_type == EntryType::In;
            let type_str = format!(" {:<3}", entry.entry_type);
            let color = if is_in {
                colors.in_state
            } else {
                colors.out_state
            };

            let is_selected_day = self.state.focus == crate::state::Focus::History
                && self.state.history_selected_date == current_date_idx;

            let mut item = ListItem::new(Line::from(vec![
                Span::styled("   ", Style::default()),
                Span::styled(
                    type_str,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ),
                Span::styled(" at ", Style::default().fg(colors.subtext)),
                Span::styled(time, Style::default().fg(colors.text)),
            ]));

            if is_selected_day {
                item = item.style(Style::default().bg(colors.border));
            }
            list_items.push(item);
        }

        let title = if self.state.focus == crate::state::Focus::History {
            Line::from(vec![
                Span::styled(
                    " History ",
                    Style::default()
                        .fg(colors.title)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    " <Enter> Edit  <Esc> Back ",
                    Style::default().fg(colors.subtext),
                ),
            ])
        } else {
            Line::from(Span::styled(" History ", Style::default().fg(colors.title)))
        };

        let list = List::new(list_items).block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(if self.state.focus == crate::state::Focus::History {
                    Style::default().fg(colors.highlight)
                } else {
                    Style::default().fg(colors.border)
                })
                .title(title),
        );

        let mut state = ratatui::widgets::ListState::default();
        if self.state.focus == crate::state::Focus::History {
            if let Some(idx) = self.state.history_selected_date {
                if let Some(list_idx) = date_indices.get(idx) {
                    state.select(Some(*list_idx));
                }
            }
        }

        ratatui::widgets::StatefulWidget::render(list, area, buf, &mut state);
    }
}
