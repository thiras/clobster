//! UI rendering using ratatui.
//!
//! This module contains all TUI components and rendering logic.

mod layout;
mod widgets;

pub use layout::Layout;
pub use widgets::{HelpPanel, MarketList, OrderList, PositionList, StatusBar, TabBar};

use crate::state::Store;
use ratatui::Frame;

/// Main UI renderer.
pub struct Ui;

impl Ui {
    /// Render the entire UI.
    pub fn render(frame: &mut Frame, store: &Store) {
        let layout = Layout::new(frame.area());

        // Render status bar
        StatusBar::render(frame, layout.status_area, store);

        // Render tab bar
        TabBar::render(frame, layout.tab_area, store);

        // Render main content based on current view
        match store.app.current_view {
            crate::state::View::Markets | crate::state::View::MarketDetail => {
                MarketList::render(frame, layout.main_area, store);
            }
            crate::state::View::Orders | crate::state::View::OrderEntry => {
                OrderList::render(frame, layout.main_area, store);
            }
            crate::state::View::Positions | crate::state::View::Portfolio => {
                PositionList::render(frame, layout.main_area, store);
            }
            crate::state::View::Settings => {
                // TODO: Settings view - render placeholder for now
                let block = ratatui::widgets::Block::default()
                    .title(" Settings ")
                    .borders(ratatui::widgets::Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
                frame.render_widget(block, layout.main_area);
            }
        }

        // Render help panel if visible
        if store.app.show_help {
            HelpPanel::render(frame, frame.area());
        }

        // Render notification if present
        if let Some(notification) = &store.app.notification {
            widgets::render_notification(frame, layout.notification_area, notification);
        }

        // Render error if present
        if let Some(error) = &store.app.error {
            widgets::render_error(frame, layout.notification_area, error);
        }
    }
}
