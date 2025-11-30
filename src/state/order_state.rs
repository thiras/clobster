//! Order-related state.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Order status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum OrderStatus {
    #[default]
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Expired,
    Failed,
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Open => write!(f, "Open"),
            Self::PartiallyFilled => write!(f, "Partial"),
            Self::Filled => write!(f, "Filled"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Expired => write!(f, "Expired"),
            Self::Failed => write!(f, "Failed"),
        }
    }
}

/// An order on Polymarket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Order ID.
    pub id: String,
    /// Market ID.
    pub market_id: String,
    /// Market question (for display).
    pub market_question: String,
    /// Token ID.
    pub token_id: String,
    /// Outcome name (e.g., "Yes", "No").
    pub outcome_name: String,
    /// Order side.
    pub side: super::OrderSide,
    /// Order type.
    pub order_type: super::OrderType,
    /// Order price.
    pub price: Decimal,
    /// Original size.
    pub original_size: Decimal,
    /// Remaining size.
    pub remaining_size: Decimal,
    /// Filled size.
    pub filled_size: Decimal,
    /// Order status.
    pub status: OrderStatus,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
    /// Expiration timestamp (if any).
    pub expires_at: Option<DateTime<Utc>>,
}

impl Order {
    /// Get the fill percentage.
    pub fn fill_percent(&self) -> Decimal {
        if self.original_size.is_zero() {
            Decimal::ZERO
        } else {
            (self.filled_size / self.original_size) * Decimal::ONE_HUNDRED
        }
    }

    /// Check if the order is active (can still be filled).
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }

    /// Check if the order is complete (no longer active).
    pub fn is_complete(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled
                | OrderStatus::Cancelled
                | OrderStatus::Expired
                | OrderStatus::Failed
        )
    }

    /// Check if the order can be cancelled.
    pub fn can_cancel(&self) -> bool {
        self.is_active()
    }

    /// Get the total value of the order.
    pub fn total_value(&self) -> Decimal {
        self.price * self.original_size
    }

    /// Get the filled value.
    pub fn filled_value(&self) -> Decimal {
        self.price * self.filled_size
    }
}

/// State for order-related data.
#[derive(Debug, Default)]
pub struct OrderState {
    /// All orders.
    pub orders: Vec<Order>,
    /// Currently selected order index.
    pub selected_index: Option<usize>,
    /// Status filter.
    pub status_filter: Option<OrderStatus>,
    /// Whether orders are currently loading.
    pub loading: bool,
    /// Last update timestamp.
    pub last_updated: Option<DateTime<Utc>>,
    /// Scroll offset for display.
    pub scroll_offset: usize,
}

impl OrderState {
    /// Get the currently selected order.
    pub fn selected_order(&self) -> Option<&Order> {
        self.selected_index.and_then(|i| self.orders.get(i))
    }

    /// Get open orders.
    pub fn open_orders(&self) -> Vec<&Order> {
        self.orders.iter().filter(|o| o.is_active()).collect()
    }

    /// Get filled orders.
    pub fn filled_orders(&self) -> Vec<&Order> {
        self.orders
            .iter()
            .filter(|o| o.status == OrderStatus::Filled)
            .collect()
    }

    /// Get order history (completed orders).
    pub fn order_history(&self) -> Vec<&Order> {
        self.orders.iter().filter(|o| o.is_complete()).collect()
    }

    /// Get filtered orders based on status filter.
    pub fn filtered_orders(&self) -> Vec<&Order> {
        if let Some(status) = &self.status_filter {
            self.orders.iter().filter(|o| o.status == *status).collect()
        } else {
            self.orders.iter().collect()
        }
    }

    /// Get the count of open orders.
    pub fn open_count(&self) -> usize {
        self.open_orders().len()
    }
}
