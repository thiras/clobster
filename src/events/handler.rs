//! Event handler for processing input events.

use crate::config::KeyBindings;
use crate::error::Result;
use crate::state::{Action, InputMode, Store, View};
use crossterm::event::{
    self, Event as CrosstermEvent, KeyCode, KeyEvent, KeyEventKind, MouseEvent, MouseEventKind,
};
use std::time::Duration;
use tokio::sync::mpsc;

/// Handles input events and produces actions.
pub struct EventHandler {
    /// Action sender (for future async dispatch).
    #[allow(dead_code)]
    action_tx: mpsc::UnboundedSender<Action>,
    /// Key bindings.
    keybindings: KeyBindings,
    /// Store reference for state-aware handling.
    store_snapshot: Option<StoreSnapshot>,
}

/// Snapshot of relevant store state for event handling.
#[derive(Clone)]
struct StoreSnapshot {
    input_mode: InputMode,
    current_view: View,
    selected_order_id: Option<String>,
    selected_order_can_cancel: bool,
}

impl EventHandler {
    /// Create a new event handler with the given action sender.
    pub fn new(action_tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            action_tx,
            keybindings: KeyBindings::default(),
            store_snapshot: None,
        }
    }

    /// Update the store snapshot for state-aware event handling.
    pub fn update_store_snapshot(&mut self, store: &Store) {
        let selected_order = store.orders.selected_order();
        self.store_snapshot = Some(StoreSnapshot {
            input_mode: store.app.input_mode,
            current_view: store.app.current_view,
            selected_order_id: selected_order.map(|o| o.id.clone()),
            selected_order_can_cancel: selected_order.map(|o| o.can_cancel()).unwrap_or(false),
        });
    }

    /// Get the next action from user input.
    pub async fn next(&mut self) -> Result<Option<Action>> {
        if event::poll(Duration::from_millis(100))? {
            let event = event::read()?;
            match event {
                CrosstermEvent::Key(key) => {
                    if let Some(action) = self.handle_key(key) {
                        return Ok(Some(action));
                    }
                }
                CrosstermEvent::Mouse(mouse) => {
                    if let Some(action) = self.handle_mouse(mouse) {
                        return Ok(Some(action));
                    }
                }
                CrosstermEvent::Resize(_, _) => {
                    // Terminal will automatically redraw
                }
                _ => {}
            }
        }
        Ok(None)
    }

    /// Handle a key event and return an optional action.
    fn handle_key(&self, key: KeyEvent) -> Option<Action> {
        // Only process key press events
        if key.kind != KeyEventKind::Press {
            return None;
        }

        let snapshot = self.store_snapshot.as_ref()?;

        // Handle based on current input mode
        match snapshot.input_mode {
            InputMode::Normal => self.handle_normal_mode(key, snapshot),
            InputMode::Insert => self.handle_insert_mode(key),
            InputMode::Command => self.handle_command_mode(key),
            InputMode::Search => self.handle_search_mode(key),
        }
    }

    /// Handle a mouse event and return an optional action.
    fn handle_mouse(&self, mouse: MouseEvent) -> Option<Action> {
        match mouse.kind {
            MouseEventKind::ScrollUp => Some(Action::ScrollUp),
            MouseEventKind::ScrollDown => Some(Action::ScrollDown),
            _ => None,
        }
    }

    fn handle_normal_mode(&self, key: KeyEvent, snapshot: &StoreSnapshot) -> Option<Action> {
        let input = super::InputEvent::from(key);

        // Global shortcuts
        if input.matches(&self.keybindings.quit) {
            return Some(Action::Quit);
        }

        if input.matches(&self.keybindings.help) {
            return Some(Action::ToggleHelp);
        }

        if input.matches(&self.keybindings.refresh) {
            return Some(Action::RefreshAll);
        }

        // View switching
        if input.matches(&self.keybindings.markets) {
            return Some(Action::SetView(View::Markets));
        }
        if input.matches(&self.keybindings.orderbook) {
            return Some(Action::SetView(View::OrderBook));
        }
        if input.matches(&self.keybindings.orders) {
            return Some(Action::SetView(View::Orders));
        }
        if input.matches(&self.keybindings.positions) {
            return Some(Action::SetView(View::Positions));
        }
        if input.matches(&self.keybindings.portfolio) {
            return Some(Action::SetView(View::Portfolio));
        }

        // Navigation
        if input.matches(&self.keybindings.up) || key.code == KeyCode::Up {
            return Some(Action::ScrollUp);
        }
        if input.matches(&self.keybindings.down) || key.code == KeyCode::Down {
            return Some(Action::ScrollDown);
        }

        // Page navigation
        if key.code == KeyCode::PageUp {
            return Some(Action::PageUp);
        }
        if key.code == KeyCode::PageDown {
            return Some(Action::PageDown);
        }
        if key.code == KeyCode::Home {
            return Some(Action::GoToTop);
        }
        if key.code == KeyCode::End {
            return Some(Action::GoToBottom);
        }

        // Search mode
        if input.matches(&self.keybindings.search) {
            return Some(Action::SetInputMode(InputMode::Search));
        }

        // View-specific actions
        match snapshot.current_view {
            View::Markets | View::MarketDetail => self.handle_markets_view(key),
            View::OrderBook => self.handle_orderbook_view(key),
            View::Orders | View::OrderEntry => self.handle_orders_view(key, snapshot),
            View::Positions | View::Portfolio => self.handle_positions_view(key),
            View::Settings => None,
        }
    }

    fn handle_markets_view(&self, key: KeyEvent) -> Option<Action> {
        let input = super::InputEvent::from(key);

        if input.matches(&self.keybindings.select) {
            return Some(Action::SetView(View::MarketDetail));
        }

        if input.matches(&self.keybindings.place_order) {
            return Some(Action::SetView(View::OrderEntry));
        }

        // 'b' to jump to orderbook for selected market
        if key.code == KeyCode::Char('b') {
            return Some(Action::SetView(View::OrderBook));
        }

        None
    }

    fn handle_orderbook_view(&self, key: KeyEvent) -> Option<Action> {
        match key.code {
            // Toggle outcome (Yes/No)
            KeyCode::Char('o') | KeyCode::Char('O') => Some(Action::ToggleOrderBookOutcome),
            // Cycle display mode
            KeyCode::Char('m') | KeyCode::Char('M') => Some(Action::CycleOrderBookDisplayMode),
            // Increase levels
            KeyCode::Char('+') | KeyCode::Char('=') => Some(Action::IncreaseOrderBookLevels),
            // Decrease levels
            KeyCode::Char('-') | KeyCode::Char('_') => Some(Action::DecreaseOrderBookLevels),
            // Back to markets
            KeyCode::Backspace | KeyCode::Esc => Some(Action::SetView(View::Markets)),
            _ => None,
        }
    }

    fn handle_orders_view(&self, key: KeyEvent, snapshot: &StoreSnapshot) -> Option<Action> {
        let input = super::InputEvent::from(key);

        if input.matches(&self.keybindings.cancel_order)
            && snapshot.selected_order_can_cancel
            && let Some(order_id) = &snapshot.selected_order_id
        {
            return Some(Action::CancelOrder(order_id.clone()));
        }

        None
    }

    fn handle_positions_view(&self, key: KeyEvent) -> Option<Action> {
        let input = super::InputEvent::from(key);

        if input.matches(&self.keybindings.select) {
            return Some(Action::SetView(View::MarketDetail));
        }

        None
    }

    fn handle_insert_mode(&self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => Some(Action::SetInputMode(InputMode::Normal)),
            KeyCode::Enter => {
                // Submit the input
                Some(Action::SetInputMode(InputMode::Normal))
            }
            _ => None, // Character input handled separately
        }
    }

    fn handle_command_mode(&self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => Some(Action::SetInputMode(InputMode::Normal)),
            KeyCode::Enter => {
                // Execute command
                Some(Action::SetInputMode(InputMode::Normal))
            }
            _ => None,
        }
    }

    fn handle_search_mode(&self, key: KeyEvent) -> Option<Action> {
        match key.code {
            KeyCode::Esc => {
                // Cancel search and clear
                Some(Action::SetInputMode(InputMode::Normal))
            }
            KeyCode::Enter => {
                // Execute search
                Some(Action::SetInputMode(InputMode::Normal))
            }
            _ => None,
        }
    }
}
