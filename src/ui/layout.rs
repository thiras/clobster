//! Layout management for the TUI.

use ratatui::layout::{Constraint, Direction, Layout as RatatuiLayout, Rect};

/// UI layout areas.
pub struct Layout {
    /// Status bar area (top).
    pub status_area: Rect,
    /// Tab bar area.
    pub tab_area: Rect,
    /// Main content area.
    pub main_area: Rect,
    /// Notification area (overlaid).
    pub notification_area: Rect,
}

impl Layout {
    /// Create a new layout from the terminal area.
    pub fn new(area: Rect) -> Self {
        let chunks = RatatuiLayout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1), // Status bar
                Constraint::Length(1), // Tab bar
                Constraint::Min(0),    // Main content
            ])
            .split(area);

        // Notification area is centered in the main area
        let notification_area = Rect {
            x: area.width / 4,
            y: area.height / 2 - 2,
            width: area.width / 2,
            height: 4,
        };

        Self {
            status_area: chunks[0],
            tab_area: chunks[1],
            main_area: chunks[2],
            notification_area,
        }
    }
}

/// Create a centered popup area.
pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = RatatuiLayout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    RatatuiLayout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
