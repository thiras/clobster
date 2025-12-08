//! Help panel widget.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use super::super::layout::centered_rect;

/// Help panel showing keybindings.
pub struct HelpPanel;

impl HelpPanel {
    /// Render the help panel.
    pub fn render(frame: &mut Frame, area: Rect) {
        let popup_area = centered_rect(60, 80, area);

        // Clear the area behind the popup
        frame.render_widget(Clear, popup_area);

        let help_text = vec![
            Line::from(vec![Span::styled(
                "Navigation",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  j/↓  ", Style::default().fg(Color::Cyan)),
                Span::raw("Move down"),
            ]),
            Line::from(vec![
                Span::styled("  k/↑  ", Style::default().fg(Color::Cyan)),
                Span::raw("Move up"),
            ]),
            Line::from(vec![
                Span::styled("  g    ", Style::default().fg(Color::Cyan)),
                Span::raw("Go to top"),
            ]),
            Line::from(vec![
                Span::styled("  G    ", Style::default().fg(Color::Cyan)),
                Span::raw("Go to bottom"),
            ]),
            Line::from(vec![
                Span::styled("  Tab  ", Style::default().fg(Color::Cyan)),
                Span::raw("Switch tabs"),
            ]),
            Line::from(vec![
                Span::styled("  Enter", Style::default().fg(Color::Cyan)),
                Span::raw("Select/confirm"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Views",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  1    ", Style::default().fg(Color::Cyan)),
                Span::raw("Markets view"),
            ]),
            Line::from(vec![
                Span::styled("  2    ", Style::default().fg(Color::Cyan)),
                Span::raw("Order Book view"),
            ]),
            Line::from(vec![
                Span::styled("  3    ", Style::default().fg(Color::Cyan)),
                Span::raw("Orders view"),
            ]),
            Line::from(vec![
                Span::styled("  4    ", Style::default().fg(Color::Cyan)),
                Span::raw("Positions view"),
            ]),
            Line::from(vec![
                Span::styled("  5    ", Style::default().fg(Color::Cyan)),
                Span::raw("Settings"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Order Book",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  o    ", Style::default().fg(Color::Cyan)),
                Span::raw("Toggle Yes/No outcome"),
            ]),
            Line::from(vec![
                Span::styled("  m    ", Style::default().fg(Color::Cyan)),
                Span::raw("Cycle display mode"),
            ]),
            Line::from(vec![
                Span::styled("  +/-  ", Style::default().fg(Color::Cyan)),
                Span::raw("Increase/decrease levels"),
            ]),
            Line::from(""),
            Line::from(vec![Span::styled(
                "Actions",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            )]),
            Line::from(""),
            Line::from(vec![
                Span::styled("  r    ", Style::default().fg(Color::Cyan)),
                Span::raw("Refresh data"),
            ]),
            Line::from(vec![
                Span::styled("  /    ", Style::default().fg(Color::Cyan)),
                Span::raw("Search"),
            ]),
            Line::from(vec![
                Span::styled("  c    ", Style::default().fg(Color::Cyan)),
                Span::raw("Cancel order"),
            ]),
            Line::from(vec![
                Span::styled("  ?    ", Style::default().fg(Color::Cyan)),
                Span::raw("Toggle help"),
            ]),
            Line::from(vec![
                Span::styled("  q    ", Style::default().fg(Color::Cyan)),
                Span::raw("Quit"),
            ]),
        ];

        let help = Paragraph::new(help_text)
            .block(
                Block::default()
                    .title(" Help ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .style(Style::default().fg(Color::White));

        frame.render_widget(help, popup_area);
    }
}
