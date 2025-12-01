//! Strategy context - market data and state provided to strategies.

use crate::state::{Market, MarketStatus, Order, OrderStatus, Position};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::collections::HashMap;

/// Context provided to strategies during evaluation.
///
/// Contains snapshots of current market data, positions, orders,
/// and account state that strategies can use to make decisions.
#[derive(Debug, Clone)]
pub struct StrategyContext {
    /// Current timestamp.
    pub timestamp: DateTime<Utc>,
    /// Market snapshots indexed by condition ID.
    pub markets: HashMap<String, MarketSnapshot>,
    /// Current positions indexed by token ID.
    pub positions: HashMap<String, PositionSnapshot>,
    /// Open orders indexed by order ID.
    pub orders: HashMap<String, OrderSnapshot>,
    /// Available balance.
    pub available_balance: Decimal,
    /// Total portfolio value.
    pub total_value: Decimal,
    /// Price history for markets (condition_id -> prices).
    pub price_history: HashMap<String, Vec<PricePoint>>,
}

impl StrategyContext {
    /// Create a new empty context.
    pub fn new() -> Self {
        Self {
            timestamp: Utc::now(),
            markets: HashMap::new(),
            positions: HashMap::new(),
            orders: HashMap::new(),
            available_balance: Decimal::ZERO,
            total_value: Decimal::ZERO,
            price_history: HashMap::new(),
        }
    }

    /// Get all markets as a vector.
    pub fn markets(&self) -> Vec<&MarketSnapshot> {
        self.markets.values().collect()
    }

    /// Get active markets only.
    pub fn active_markets(&self) -> Vec<&MarketSnapshot> {
        self.markets
            .values()
            .filter(|m| m.status == MarketStatus::Active)
            .collect()
    }

    /// Get a market by condition ID.
    pub fn get_market(&self, condition_id: &str) -> Option<&MarketSnapshot> {
        self.markets.get(condition_id)
    }

    /// Get all positions as a vector.
    pub fn positions(&self) -> Vec<&PositionSnapshot> {
        self.positions.values().collect()
    }

    /// Get a position by token ID.
    pub fn get_position(&self, token_id: &str) -> Option<&PositionSnapshot> {
        self.positions.get(token_id)
    }

    /// Check if we have a position in a market.
    pub fn has_position_in_market(&self, condition_id: &str) -> bool {
        self.positions
            .values()
            .any(|p| p.market_id == condition_id && p.size > Decimal::ZERO)
    }

    /// Get total exposure (sum of position values).
    pub fn total_exposure(&self) -> Decimal {
        self.positions.values().map(|p| p.current_value).sum()
    }

    /// Get all open orders.
    pub fn open_orders(&self) -> Vec<&OrderSnapshot> {
        self.orders.values().filter(|o| o.is_open()).collect()
    }

    /// Get orders for a specific market.
    pub fn orders_for_market(&self, condition_id: &str) -> Vec<&OrderSnapshot> {
        self.orders
            .values()
            .filter(|o| o.market_id == condition_id)
            .collect()
    }

    /// Get price history for a market.
    pub fn get_price_history(&self, condition_id: &str) -> Option<&Vec<PricePoint>> {
        self.price_history.get(condition_id)
    }

    /// Calculate simple moving average for a market.
    pub fn sma(&self, condition_id: &str, periods: usize) -> Option<Decimal> {
        let history = self.price_history.get(condition_id)?;
        if history.len() < periods {
            return None;
        }

        let sum: Decimal = history.iter().rev().take(periods).map(|p| p.price).sum();
        Some(sum / Decimal::from(periods))
    }

    /// Calculate exponential moving average for a market.
    ///
    /// Computes EMA by first calculating SMA of the first `periods` points,
    /// then applying the EMA formula over the remaining history.
    pub fn ema(&self, condition_id: &str, periods: usize) -> Option<Decimal> {
        let history = self.price_history.get(condition_id)?;
        if history.len() < periods {
            return None;
        }

        let multiplier = Decimal::from(2) / Decimal::from(periods + 1);

        // Initialize EMA with SMA of first `periods` points
        let sma_sum: Decimal = history.iter().take(periods).map(|p| p.price).sum();
        let mut ema = sma_sum / Decimal::from(periods);

        // Apply EMA formula over the rest of the history
        for point in history.iter().skip(periods) {
            ema = (point.price - ema) * multiplier + ema;
        }

        Some(ema)
    }

    /// Get the latest price for a market token.
    pub fn latest_price(&self, condition_id: &str, token_index: usize) -> Option<Decimal> {
        self.markets
            .get(condition_id)
            .and_then(|m| m.token_prices.get(token_index))
            .copied()
    }

    /// Calculate price change over N periods.
    pub fn price_change(&self, condition_id: &str, periods: usize) -> Option<Decimal> {
        let history = self.price_history.get(condition_id)?;
        if history.len() <= periods {
            return None;
        }

        let current = history.last()?.price;
        let past = history.get(history.len() - periods)?.price;

        if past.is_zero() {
            return None;
        }

        Some((current - past) / past)
    }
}

impl Default for StrategyContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Build a StrategyContext from current state.
impl StrategyContext {
    /// Build context from state components.
    pub fn from_state(
        markets: &[Market],
        positions: &[Position],
        orders: &[Order],
        balance: Decimal,
    ) -> Self {
        let mut ctx = Self::new();

        ctx.available_balance = balance;

        // Convert markets
        for market in markets {
            let snapshot = MarketSnapshot::from_market(market);
            ctx.markets.insert(market.id.clone(), snapshot);
        }

        // Convert positions
        for position in positions {
            let snapshot = PositionSnapshot::from_position(position);
            ctx.total_value += snapshot.current_value;
            ctx.positions.insert(position.token_id.clone(), snapshot);
        }
        ctx.total_value += balance;

        // Convert orders
        for order in orders {
            let snapshot = OrderSnapshot::from_order(order);
            ctx.orders.insert(order.id.clone(), snapshot);
        }

        ctx
    }
}

/// Snapshot of market state for strategy evaluation.
#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    /// Market condition ID.
    pub condition_id: String,
    /// Market question/title.
    pub question: String,
    /// Market status.
    pub status: MarketStatus,
    /// Token IDs for outcomes.
    pub token_ids: Vec<String>,
    /// Token names/labels.
    pub token_names: Vec<String>,
    /// Current token prices.
    pub token_prices: Vec<Decimal>,
    /// 24h volume.
    pub volume_24h: Decimal,
    /// Total liquidity.
    pub liquidity: Decimal,
    /// Spread (difference between best bid and ask).
    pub spread: Option<Decimal>,
    /// End date if applicable.
    pub end_date: Option<DateTime<Utc>>,
}

impl MarketSnapshot {
    /// Create a snapshot from a Market.
    pub fn from_market(market: &Market) -> Self {
        let token_prices: Vec<Decimal> = market.outcomes.iter().map(|o| o.mid_price()).collect();
        let token_ids: Vec<String> = market.outcomes.iter().map(|o| o.token_id.clone()).collect();
        let token_names: Vec<String> = market.outcomes.iter().map(|o| o.name.clone()).collect();

        Self {
            condition_id: market.id.clone(),
            question: market.question.clone(),
            status: market.status,
            token_ids,
            token_names,
            token_prices,
            volume_24h: market.volume,
            liquidity: market.liquidity,
            spread: None,
            end_date: market.end_date,
        }
    }

    /// Get the "Yes" price (first outcome).
    pub fn yes_price(&self) -> Option<Decimal> {
        self.token_prices.first().copied()
    }

    /// Get the "No" price (second outcome).
    pub fn no_price(&self) -> Option<Decimal> {
        self.token_prices.get(1).copied()
    }

    /// Check if the market is tradeable.
    pub fn is_tradeable(&self) -> bool {
        self.status == MarketStatus::Active
    }

    /// Get the implied probability for the first outcome.
    pub fn implied_probability(&self) -> Option<Decimal> {
        self.yes_price()
    }
}

/// Snapshot of a position.
#[derive(Debug, Clone)]
pub struct PositionSnapshot {
    /// Market condition ID.
    pub market_id: String,
    /// Token ID.
    pub token_id: String,
    /// Position size.
    pub size: Decimal,
    /// Average entry price.
    pub avg_price: Decimal,
    /// Current market price.
    pub current_price: Decimal,
    /// Current position value.
    pub current_value: Decimal,
    /// Unrealized P&L.
    pub unrealized_pnl: Decimal,
    /// P&L percentage.
    pub pnl_percent: Decimal,
}

impl PositionSnapshot {
    /// Create a snapshot from a Position.
    pub fn from_position(position: &Position) -> Self {
        Self {
            market_id: position.market_id.clone(),
            token_id: position.token_id.clone(),
            size: position.size,
            avg_price: position.avg_price,
            current_price: position.current_price,
            current_value: position.market_value,
            unrealized_pnl: position.unrealized_pnl,
            pnl_percent: position.unrealized_pnl_percent,
        }
    }

    /// Check if this is a winning position.
    pub fn is_profitable(&self) -> bool {
        self.unrealized_pnl > Decimal::ZERO
    }
}

/// Snapshot of an order.
#[derive(Debug, Clone)]
pub struct OrderSnapshot {
    /// Order ID.
    pub order_id: String,
    /// Market condition ID.
    pub market_id: String,
    /// Token ID.
    pub token_id: String,
    /// Order side (buy/sell).
    pub side: crate::state::OrderSide,
    /// Limit price.
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
}

impl OrderSnapshot {
    /// Create a snapshot from an Order.
    pub fn from_order(order: &Order) -> Self {
        Self {
            order_id: order.id.clone(),
            market_id: order.market_id.clone(),
            token_id: order.token_id.clone(),
            side: order.side,
            price: order.price,
            original_size: order.original_size,
            remaining_size: order.remaining_size,
            filled_size: order.filled_size,
            status: order.status,
            created_at: order.created_at,
        }
    }

    /// Check if the order is still open.
    pub fn is_open(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }

    /// Get the fill percentage.
    pub fn fill_percent(&self) -> Decimal {
        if self.original_size.is_zero() {
            Decimal::ZERO
        } else {
            (self.filled_size / self.original_size) * Decimal::from(100)
        }
    }
}

/// A price point in history.
#[derive(Debug, Clone)]
pub struct PricePoint {
    /// Timestamp.
    pub timestamp: DateTime<Utc>,
    /// Price.
    pub price: Decimal,
    /// Volume at this point.
    pub volume: Option<Decimal>,
}
