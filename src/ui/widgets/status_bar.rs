//! Status bar widget.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::state::Store;

/// Status bar widget.
pub struct StatusBar;

impl StatusBar {
    /// Render the status bar.
    pub fn render(frame: &mut Frame, area: Rect, store: &Store) {
        let connection_status = if store.app.connected {
            Span::styled("● Connected", Style::default().fg(Color::Green))
        } else {
            Span::styled("○ Disconnected", Style::default().fg(Color::Red))
        };

        let mode = Span::styled(
            format!(" {:?} ", store.app.mode),
            Style::default().fg(Color::Yellow),
        );

        let loading = if store.app.loading {
            Span::styled(
                " Loading... ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            )
        } else {
            Span::raw("")
        };

        let help_hint = Span::styled(" Press ? for help ", Style::default().fg(Color::DarkGray));

        // Create the status line
        let left_content = vec![
            Span::styled(
                " 🦀 Clobster ",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" | "),
            connection_status,
            Span::raw(" | "),
            mode,
            loading,
        ];

        let status_line = Line::from(left_content);

        // Calculate padding for right-aligned help hint
        let left_len: usize = status_line.spans.iter().map(|s| s.content.len()).sum();
        let right_len = help_hint.content.len();
        let padding = area
            .width
            .saturating_sub(left_len as u16 + right_len as u16);

        let mut full_line = status_line.spans;
        full_line.push(Span::raw(" ".repeat(padding as usize)));
        full_line.push(help_hint);

        let paragraph =
            Paragraph::new(Line::from(full_line)).style(Style::default().bg(Color::DarkGray));

        frame.render_widget(paragraph, area);
    }
}
