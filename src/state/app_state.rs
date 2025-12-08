//! Application-level state.

use super::Notification;

/// The current view/screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum View {
    #[default]
    Markets,
    MarketDetail,
    OrderBook,
    Orders,
    OrderEntry,
    Positions,
    Portfolio,
    Settings,
}

/// Input mode for the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Normal,
    Insert,
    Command,
    Search,
}

/// Application mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Browse,
    Trade,
    ViewOnly,
}

/// Global application state.
#[derive(Debug, Default)]
pub struct AppState {
    /// Current view.
    pub current_view: View,
    /// Current input mode.
    pub input_mode: InputMode,
    /// Application mode.
    pub mode: AppMode,
    /// Whether to show help overlay.
    pub show_help: bool,
    /// Current notification.
    pub notification: Option<Notification>,
    /// Current error message.
    pub error: Option<String>,
    /// Whether the app is loading data.
    pub loading: bool,
    /// Whether connected to the API.
    pub connected: bool,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Current search/command input.
    pub input_buffer: String,
    /// Cursor position in input buffer.
    pub cursor_position: usize,
}

impl AppState {
    /// Create a new application state.
    pub fn new() -> Self {
        Self {
            current_view: View::Markets,
            input_mode: InputMode::Normal,
            mode: AppMode::Browse,
            connected: false,
            ..Default::default()
        }
    }

    /// Check if in an input mode.
    pub fn is_editing(&self) -> bool {
        matches!(
            self.input_mode,
            InputMode::Insert | InputMode::Command | InputMode::Search
        )
    }

    /// Clear the input buffer.
    pub fn clear_input(&mut self) {
        self.input_buffer.clear();
        self.cursor_position = 0;
    }

    /// Add a character to the input buffer.
    pub fn push_char(&mut self, c: char) {
        self.input_buffer.insert(self.cursor_position, c);
        self.cursor_position += 1;
    }

    /// Remove the character before the cursor.
    pub fn pop_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Move cursor left.
    pub fn cursor_left(&mut self) {
        self.cursor_position = self.cursor_position.saturating_sub(1);
    }

    /// Move cursor right.
    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }
}
