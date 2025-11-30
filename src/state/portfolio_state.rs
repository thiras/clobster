//! Portfolio and position state.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A balance entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Asset type (e.g., "USDC").
    pub asset: String,
    /// Total balance.
    pub total: Decimal,
    /// Available balance (not in orders).
    pub available: Decimal,
    /// Locked in orders.
    pub locked: Decimal,
}

impl Balance {
    /// Create a new balance entry.
    pub fn new(asset: impl Into<String>, total: Decimal, available: Decimal) -> Self {
        Self {
            asset: asset.into(),
            total,
            available,
            locked: total - available,
        }
    }
}

/// A position in a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Market ID.
    pub market_id: String,
    /// Market question (for display).
    pub market_question: String,
    /// Token ID.
    pub token_id: String,
    /// Outcome name (e.g., "Yes", "No").
    pub outcome_name: String,
    /// Position size.
    pub size: Decimal,
    /// Average entry price.
    pub avg_price: Decimal,
    /// Current market price.
    pub current_price: Decimal,
    /// Unrealized PnL.
    pub unrealized_pnl: Decimal,
    /// Unrealized PnL percentage.
    pub unrealized_pnl_percent: Decimal,
    /// Realized PnL.
    pub realized_pnl: Decimal,
    /// Total cost basis.
    pub cost_basis: Decimal,
    /// Current market value.
    pub market_value: Decimal,
}

impl Position {
    /// Calculate unrealized PnL.
    pub fn calculate_pnl(&mut self) {
        self.market_value = self.size * self.current_price;
        self.cost_basis = self.size * self.avg_price;
        self.unrealized_pnl = self.market_value - self.cost_basis;

        if !self.cost_basis.is_zero() {
            self.unrealized_pnl_percent =
                (self.unrealized_pnl / self.cost_basis) * Decimal::ONE_HUNDRED;
        }
    }

    /// Check if position is profitable.
    pub fn is_profitable(&self) -> bool {
        self.unrealized_pnl > Decimal::ZERO
    }
}

/// Portfolio state.
#[derive(Debug, Default, Clone)]
pub struct PortfolioState {
    /// Account balances.
    pub balances: Vec<Balance>,
    /// Open positions.
    pub positions: Vec<Position>,
    /// Total portfolio value.
    pub total_value: Decimal,
    /// Total unrealized PnL.
    pub total_unrealized_pnl: Decimal,
    /// Total realized PnL.
    pub total_realized_pnl: Decimal,
    /// Currently selected position index.
    pub selected_position: Option<usize>,
    /// Whether portfolio is loading.
    pub loading: bool,
    /// Last update timestamp.
    pub last_updated: Option<DateTime<Utc>>,
    /// Scroll offset for display.
    pub scroll_offset: usize,
}

impl PortfolioState {
    /// Get the currently selected position.
    pub fn selected_position(&self) -> Option<&Position> {
        self.selected_position.and_then(|i| self.positions.get(i))
    }

    /// Get available USDC balance.
    pub fn available_usdc(&self) -> Decimal {
        self.balances
            .iter()
            .find(|b| b.asset == "USDC")
            .map(|b| b.available)
            .unwrap_or_default()
    }

    /// Calculate totals from positions.
    pub fn calculate_totals(&mut self) {
        self.total_unrealized_pnl = self.positions.iter().map(|p| p.unrealized_pnl).sum();
        self.total_realized_pnl = self.positions.iter().map(|p| p.realized_pnl).sum();

        let positions_value: Decimal = self.positions.iter().map(|p| p.market_value).sum();
        let balances_value: Decimal = self.balances.iter().map(|b| b.total).sum();

        self.total_value = positions_value + balances_value;
    }

    /// Get profitable positions.
    pub fn profitable_positions(&self) -> Vec<&Position> {
        self.positions
            .iter()
            .filter(|p| p.is_profitable())
            .collect()
    }

    /// Get losing positions.
    pub fn losing_positions(&self) -> Vec<&Position> {
        self.positions
            .iter()
            .filter(|p| !p.is_profitable())
            .collect()
    }
}
