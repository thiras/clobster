//! Notification rendering.

use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

use crate::state::{Notification, NotificationLevel};

/// Render a notification popup.
pub fn render_notification(frame: &mut Frame, area: Rect, notification: &Notification) {
    frame.render_widget(Clear, area);

    let (border_color, icon) = match notification.level {
        NotificationLevel::Info => (Color::Cyan, "ℹ"),
        NotificationLevel::Success => (Color::Green, "✓"),
        NotificationLevel::Warning => (Color::Yellow, "⚠"),
        NotificationLevel::Error => (Color::Red, "✗"),
    };

    let content = Line::from(vec![
        Span::styled(format!("{} ", icon), Style::default().fg(border_color)),
        Span::raw(&notification.message),
    ]);

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}

/// Render an error popup.
pub fn render_error(frame: &mut Frame, area: Rect, error: &str) {
    frame.render_widget(Clear, area);

    let content = Line::from(vec![
        Span::styled(
            "✗ Error: ",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ),
        Span::raw(error),
    ]);

    let paragraph = Paragraph::new(content)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Red)),
        )
        .style(Style::default().fg(Color::White));

    frame.render_widget(paragraph, area);
}
