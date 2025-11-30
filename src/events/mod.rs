//! Event handling for Clobster.
//!
//! This module provides input event handling and an event loop
//! for processing terminal events.

mod handler;
mod input;

pub use handler::EventHandler;
pub use input::{InputEvent, Key, Modifiers};

use crate::error::Result;
use crossterm::event::{Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;
use tokio::sync::mpsc;

/// Terminal event types.
#[derive(Debug, Clone)]
pub enum Event {
    /// Terminal tick (for animations/updates).
    Tick,
    /// Key press event.
    Key(KeyEvent),
    /// Mouse event.
    Mouse(MouseEvent),
    /// Terminal resize event.
    Resize(u16, u16),
    /// Focus gained.
    FocusGained,
    /// Focus lost.
    FocusLost,
    /// Paste event.
    Paste(String),
    /// Custom application event.
    App(AppEvent),
}

/// Application-specific events.
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Data refresh completed.
    DataRefreshed,
    /// WebSocket message received.
    WsMessage(WsMessageType),
    /// Connection status changed.
    ConnectionChanged(bool),
    /// Error occurred.
    Error(String),
    /// Notification.
    Notification(crate::state::Notification),
}

/// WebSocket message types.
#[derive(Debug, Clone)]
pub enum WsMessageType {
    /// Market price update.
    PriceUpdate {
        token_id: String,
        bid: rust_decimal::Decimal,
        ask: rust_decimal::Decimal,
    },
    /// Order update.
    OrderUpdate { order_id: String, status: String },
    /// Trade executed.
    Trade {
        token_id: String,
        price: rust_decimal::Decimal,
        size: rust_decimal::Decimal,
    },
}

/// Configuration for the event handler.
#[derive(Debug, Clone)]
pub struct EventConfig {
    /// Tick rate for the event loop.
    pub tick_rate: Duration,
    /// Whether to capture mouse events.
    pub mouse_capture: bool,
    /// Whether to capture paste events.
    pub paste_capture: bool,
}

impl Default for EventConfig {
    fn default() -> Self {
        Self {
            tick_rate: Duration::from_millis(250),
            mouse_capture: true,
            paste_capture: true,
        }
    }
}

impl EventConfig {
    /// Create a new event config with the specified tick rate in milliseconds.
    pub fn with_tick_rate_ms(mut self, ms: u64) -> Self {
        self.tick_rate = Duration::from_millis(ms);
        self
    }

    /// Enable or disable mouse capture.
    pub fn with_mouse_capture(mut self, capture: bool) -> Self {
        self.mouse_capture = capture;
        self
    }
}

/// Event loop for handling terminal events.
pub struct EventLoop {
    /// Event sender.
    event_tx: mpsc::UnboundedSender<Event>,
    /// Event receiver.
    event_rx: mpsc::UnboundedReceiver<Event>,
    /// Configuration.
    config: EventConfig,
}

impl EventLoop {
    /// Create a new event loop.
    pub fn new(config: EventConfig) -> Self {
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        Self {
            event_tx,
            event_rx,
            config,
        }
    }

    /// Get a sender for sending events.
    pub fn sender(&self) -> mpsc::UnboundedSender<Event> {
        self.event_tx.clone()
    }

    /// Start the event loop.
    pub fn start(self) -> (mpsc::UnboundedReceiver<Event>, tokio::task::JoinHandle<()>) {
        let event_tx = self.event_tx;
        let tick_rate = self.config.tick_rate;

        let handle = tokio::spawn(async move {
            let mut tick_interval = tokio::time::interval(tick_rate);

            loop {
                let event = tokio::select! {
                    _ = tick_interval.tick() => Event::Tick,
                    maybe_event = Self::read_crossterm_event() => {
                        match maybe_event {
                            Ok(Some(event)) => event,
                            Ok(None) => continue,
                            Err(_) => continue,
                        }
                    }
                };

                if event_tx.send(event).is_err() {
                    break;
                }
            }
        });

        (self.event_rx, handle)
    }

    async fn read_crossterm_event() -> Result<Option<Event>> {
        if crossterm::event::poll(Duration::from_millis(10))? {
            let event = crossterm::event::read()?;
            Ok(Some(match event {
                CrosstermEvent::Key(key) => Event::Key(key),
                CrosstermEvent::Mouse(mouse) => Event::Mouse(mouse),
                CrosstermEvent::Resize(w, h) => Event::Resize(w, h),
                CrosstermEvent::FocusGained => Event::FocusGained,
                CrosstermEvent::FocusLost => Event::FocusLost,
                CrosstermEvent::Paste(s) => Event::Paste(s),
            }))
        } else {
            Ok(None)
        }
    }
}
