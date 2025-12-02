//! State management for Clobster.
//!
//! This module provides centralized state management with a unidirectional
//! data flow pattern inspired by Redux/Elm architecture.

mod app_state;
mod market_state;
mod order_state;
mod portfolio_state;

pub use app_state::{AppMode, AppState, InputMode, View};
pub use market_state::{Market, MarketState, MarketStatus, Outcome};
pub use order_state::{Order, OrderState, OrderStatus};
pub use portfolio_state::{Balance, PortfolioState, Position};

use crate::error::Result;
use tokio::sync::mpsc;

/// Actions that can be dispatched to modify state.
#[derive(Debug, Clone)]
pub enum Action {
    // Navigation
    SetView(View),
    SetInputMode(InputMode),
    SetAppMode(AppMode),

    // Market actions
    LoadMarkets,
    MarketsLoaded(Vec<Market>),
    SelectMarket(usize),
    SearchMarkets(String),
    FilterMarkets(MarketStatus),
    ClearMarketFilter,

    // Order actions
    LoadOrders,
    OrdersLoaded(Vec<Order>),
    SelectOrder(usize),
    PlaceOrder(OrderRequest),
    CancelOrder(String),
    OrderPlaced(Order),
    OrderCancelled(String),

    // Portfolio actions
    LoadPortfolio,
    PortfolioLoaded(PortfolioState),
    LoadPositions,
    PositionsLoaded(Vec<Position>),

    // UI actions
    ScrollUp,
    ScrollDown,
    PageUp,
    PageDown,
    GoToTop,
    GoToBottom,
    ToggleHelp,
    ShowNotification(Notification),
    DismissNotification,

    // Data refresh
    RefreshAll,
    RefreshMarkets,
    RefreshOrders,
    RefreshPortfolio,

    // Error handling
    SetError(String),
    ClearError,

    // Connection status
    SetConnected(bool),
    SetLoading(bool),

    // Quit
    Quit,
}

/// Request to place an order.
#[derive(Debug, Clone)]
pub struct OrderRequest {
    pub market_id: String,
    pub token_id: String,
    pub side: OrderSide,
    /// Price for limit orders. None for market orders.
    pub price: Option<rust_decimal::Decimal>,
    pub size: rust_decimal::Decimal,
    pub order_type: OrderType,
}

/// Order side (buy/sell).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

/// Order type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum OrderType {
    Limit,
    Market,
}

/// A notification to display to the user.
#[derive(Debug, Clone)]
pub struct Notification {
    pub message: String,
    pub level: NotificationLevel,
    pub duration_secs: u64,
}

/// Notification severity level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl Notification {
    pub fn info(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Info,
            duration_secs: 3,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Success,
            duration_secs: 3,
        }
    }

    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Warning,
            duration_secs: 5,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            level: NotificationLevel::Error,
            duration_secs: 10,
        }
    }
}

/// The global state store.
#[derive(Debug)]
pub struct Store {
    /// Application state.
    pub app: AppState,
    /// Market state.
    pub markets: MarketState,
    /// Order state.
    pub orders: OrderState,
    /// Portfolio state.
    pub portfolio: PortfolioState,
    /// Action sender for dispatching actions.
    action_tx: mpsc::UnboundedSender<Action>,
}

impl Store {
    /// Create a new store with the given action sender.
    pub fn new(action_tx: mpsc::UnboundedSender<Action>) -> Self {
        Self {
            app: AppState::default(),
            markets: MarketState::default(),
            orders: OrderState::default(),
            portfolio: PortfolioState::default(),
            action_tx,
        }
    }

    /// Dispatch an action to the store.
    pub fn dispatch(&self, action: Action) -> Result<()> {
        self.action_tx
            .send(action)
            .map_err(|e| crate::Error::channel(e.to_string()))
    }

    /// Apply an action to update state.
    pub fn reduce(&mut self, action: Action) {
        match action {
            // Navigation
            Action::SetView(view) => self.app.current_view = view,
            Action::SetInputMode(mode) => self.app.input_mode = mode,
            Action::SetAppMode(mode) => self.app.mode = mode,

            // Market actions
            Action::LoadMarkets => self.markets.loading = true,
            Action::MarketsLoaded(markets) => {
                self.markets.markets = markets;
                self.markets.loading = false;
                self.markets.last_updated = Some(chrono::Utc::now());
            }
            Action::SelectMarket(index) => {
                if index < self.markets.markets.len() {
                    self.markets.selected_index = Some(index);
                }
            }
            Action::SearchMarkets(query) => {
                self.markets.search_query = Some(query);
            }
            Action::FilterMarkets(status) => {
                self.markets.status_filter = Some(status);
            }
            Action::ClearMarketFilter => {
                self.markets.search_query = None;
                self.markets.status_filter = None;
            }

            // Order actions
            Action::LoadOrders => self.orders.loading = true,
            Action::OrdersLoaded(orders) => {
                self.orders.orders = orders;
                self.orders.loading = false;
                self.orders.last_updated = Some(chrono::Utc::now());
            }
            Action::SelectOrder(index) => {
                if index < self.orders.orders.len() {
                    self.orders.selected_index = Some(index);
                }
            }
            Action::PlaceOrder(_) => self.orders.loading = true,
            Action::CancelOrder(_) => self.orders.loading = true,
            Action::OrderPlaced(order) => {
                self.orders.orders.push(order);
                self.orders.loading = false;
            }
            Action::OrderCancelled(id) => {
                self.orders.orders.retain(|o| o.id != id);
                self.orders.loading = false;
            }

            // Portfolio actions
            Action::LoadPortfolio => self.portfolio.loading = true,
            Action::PortfolioLoaded(portfolio) => {
                self.portfolio = portfolio;
                self.portfolio.loading = false;
            }
            Action::LoadPositions => self.portfolio.loading = true,
            Action::PositionsLoaded(positions) => {
                self.portfolio.positions = positions;
                self.portfolio.loading = false;
            }

            // UI actions
            Action::ScrollUp => self.scroll(-1),
            Action::ScrollDown => self.scroll(1),
            Action::PageUp => self.scroll(-10),
            Action::PageDown => self.scroll(10),
            Action::GoToTop => self.go_to_top(),
            Action::GoToBottom => self.go_to_bottom(),
            Action::ToggleHelp => self.app.show_help = !self.app.show_help,
            Action::ShowNotification(notification) => {
                self.app.notification = Some(notification);
            }
            Action::DismissNotification => {
                self.app.notification = None;
            }

            // Data refresh
            Action::RefreshAll
            | Action::RefreshMarkets
            | Action::RefreshOrders
            | Action::RefreshPortfolio => {
                self.app.loading = true;
            }

            // Error handling
            Action::SetError(error) => {
                self.app.error = Some(error);
                self.app.loading = false;
            }
            Action::ClearError => {
                self.app.error = None;
            }

            // Connection status
            Action::SetConnected(connected) => {
                self.app.connected = connected;
            }
            Action::SetLoading(loading) => {
                self.app.loading = loading;
            }

            // Quit
            Action::Quit => {
                self.app.should_quit = true;
            }
        }
    }

    fn scroll(&mut self, delta: i32) {
        match self.app.current_view {
            View::Markets => {
                let current = self.markets.selected_index.unwrap_or(0) as i32;
                let new_index = (current + delta).max(0) as usize;
                let max_index = self.markets.filtered_markets().len().saturating_sub(1);
                self.markets.selected_index = Some(new_index.min(max_index));
            }
            View::Orders => {
                let current = self.orders.selected_index.unwrap_or(0) as i32;
                let new_index = (current + delta).max(0) as usize;
                let max_index = self.orders.orders.len().saturating_sub(1);
                self.orders.selected_index = Some(new_index.min(max_index));
            }
            View::Positions => {
                let current = self.portfolio.selected_position.unwrap_or(0) as i32;
                let new_index = (current + delta).max(0) as usize;
                let max_index = self.portfolio.positions.len().saturating_sub(1);
                self.portfolio.selected_position = Some(new_index.min(max_index));
            }
            _ => {}
        }
    }

    fn go_to_top(&mut self) {
        match self.app.current_view {
            View::Markets => self.markets.selected_index = Some(0),
            View::Orders => self.orders.selected_index = Some(0),
            View::Positions => self.portfolio.selected_position = Some(0),
            _ => {}
        }
    }

    fn go_to_bottom(&mut self) {
        match self.app.current_view {
            View::Markets => {
                let max = self.markets.filtered_markets().len().saturating_sub(1);
                self.markets.selected_index = Some(max);
            }
            View::Orders => {
                let max = self.orders.orders.len().saturating_sub(1);
                self.orders.selected_index = Some(max);
            }
            View::Positions => {
                let max = self.portfolio.positions.len().saturating_sub(1);
                self.portfolio.selected_position = Some(max);
            }
            _ => {}
        }
    }
}
