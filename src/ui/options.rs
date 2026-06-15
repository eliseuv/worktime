use crate::state::AppState;
use crate::ui::ThemeColors;
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};

pub struct OptionsWidget<'a> {
    state: &'a AppState,
}

impl<'a> OptionsWidget<'a> {
    pub fn new(state: &'a AppState) -> Self {
        Self { state }
    }
}

impl<'a> Widget for OptionsWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        if !self.state.options_open {
            return;
        }

        Clear.render(area, buf);

        let colors = ThemeColors::from(&self.state.config.themes);

        let mut lines = Vec::new();

        let start_y = self.state.config_scroll_y;
        let visible_height = (area.height.saturating_sub(2)) as usize;

        let max_label_len = self
            .state
            .config_fields
            .iter()
            .map(|(_, k, _)| k.len())
            .max()
            .unwrap_or(0);


        for (i, (section, key, values)) in self.state.config_fields.iter().enumerate().skip(start_y).take(visible_height) {
            if i < self.state.config_scroll_y { continue; }
            if lines.len() >= visible_height { break; } // max height

            if i == 0 || section != &self.state.config_fields[i - 1].0 {
                lines.push(Line::from(Span::styled(format!("--- {} ---", section), Style::default().fg(colors.subtext).add_modifier(Modifier::BOLD))));
            }

            let mut spans = vec![
                Span::styled(format!("{:>width$}: ", key, width = max_label_len), Style::default().fg(colors.title)),
            ];

            for (col_idx, value) in values.iter().enumerate() {
                spans.push(Span::raw("[ "));
                
                let mut field_fg = colors.text;
                if section == "Themes" {
                    field_fg = crate::ui::parse_hex_color(value);
                }

                if i == self.state.selected_field && col_idx == self.state.selected_col {
                    let selected_fg = if section == "Themes" { field_fg } else { colors.highlight };
                    let cursor_bg_color = if section == "Themes" { field_fg } else { colors.highlight };
                    
                    if self.state.cursor_x < value.len() {
                        spans.push(Span::styled(&value[..self.state.cursor_x], Style::default().fg(selected_fg)));
                        spans.push(Span::styled(&value[self.state.cursor_x..self.state.cursor_x+1], Style::default().bg(cursor_bg_color).fg(Color::Black)));
                        spans.push(Span::styled(&value[self.state.cursor_x+1..], Style::default().fg(selected_fg)));
                    } else {
                        spans.push(Span::styled(value.as_str(), Style::default().fg(selected_fg)));
                        spans.push(Span::styled(" ", Style::default().bg(cursor_bg_color)));
                    }
                } else {
                    spans.push(Span::styled(value.as_str(), Style::default().fg(field_fg)));
                }
                
                spans.push(Span::raw(" ]"));

                if col_idx + 1 < values.len() {
                    spans.push(Span::raw(" "));
                }
            }

            lines.push(Line::from(spans));
        }

        let mut title_spans = vec![
            Span::styled("Nav ", Style::default().fg(colors.subtext)),
            Span::styled(
                "<↑/↓>  ",
                Style::default()
                    .fg(colors.border)
                    .add_modifier(Modifier::DIM),
            ),
        ];

        if !self.state.config_fields.is_empty()
            && self.state.config_fields[self.state.selected_field]
                .1
                .starts_with("Notification ")
        {
            title_spans.push(Span::styled("Remove ", Style::default().fg(colors.subtext)));
            title_spans.push(Span::styled(
                "<Del>  ",
                Style::default()
                    .fg(colors.border)
                    .add_modifier(Modifier::DIM),
            ));
        }

        title_spans.extend(vec![
            Span::styled("Save ", Style::default().fg(colors.subtext)),
            Span::styled(
                "<C-s>  ",
                Style::default()
                    .fg(colors.border)
                    .add_modifier(Modifier::DIM),
            ),
            Span::styled("Close ", Style::default().fg(colors.subtext)),
            Span::styled(
                "<Esc> ",
                Style::default()
                    .fg(colors.border)
                    .add_modifier(Modifier::DIM),
            ),
        ]);

        Paragraph::new(lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(colors.border))
                    .title(" Options ")
                    .title_style(
                        Style::default()
                            .fg(colors.title)
                            .add_modifier(Modifier::BOLD),
                    )
                    .title(Line::from(title_spans).alignment(ratatui::layout::Alignment::Right)),
            )
            .render(area, buf);
    }
}
