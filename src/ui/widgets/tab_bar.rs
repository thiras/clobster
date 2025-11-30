//! Tab bar widget.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::state::{Store, View};

/// Tab bar widget.
pub struct TabBar;

impl TabBar {
    /// Render the tab bar.
    pub fn render(frame: &mut Frame, area: Rect, store: &Store) {
        let tabs = vec![
            ("1", "Markets", View::Markets),
            ("2", "Orders", View::Orders),
            ("3", "Positions", View::Positions),
            ("4", "Settings", View::Settings),
        ];

        let mut spans = vec![Span::raw(" ")];

        for (key, name, view) in tabs {
            let is_selected = store.app.current_view == view;

            let key_style = Style::default().fg(Color::DarkGray);
            let name_style = if is_selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(Color::White)
            };

            spans.push(Span::styled(format!("[{}] ", key), key_style));
            spans.push(Span::styled(name, name_style));
            spans.push(Span::raw("  "));
        }

        let tab_line = Line::from(spans);
        let paragraph = Paragraph::new(tab_line);

        frame.render_widget(paragraph, area);
    }
}
