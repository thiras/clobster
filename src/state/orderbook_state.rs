//! Orderbook state for tracking market depth and liquidity.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A price level in the orderbook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price at this level.
    pub price: Decimal,
    /// Total size available at this price.
    pub size: Decimal,
}

impl PriceLevel {
    /// Create a new price level.
    pub fn new(price: Decimal, size: Decimal) -> Self {
        Self { price, size }
    }

    /// Calculate the total value (price * size) at this level.
    pub fn total_value(&self) -> Decimal {
        self.price * self.size
    }
}

/// Orderbook for a single token/outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// Token ID this orderbook is for.
    pub token_id: String,
    /// Market/condition ID.
    pub market_id: String,
    /// Asset ID.
    pub asset_id: String,
    /// Buy orders (bids) - sorted by price descending (best bid first).
    pub bids: Vec<PriceLevel>,
    /// Sell orders (asks) - sorted by price ascending (best ask first).
    pub asks: Vec<PriceLevel>,
    /// Last trade price (if available).
    pub last_trade_price: Option<Decimal>,
    /// Orderbook hash for tracking updates.
    pub hash: String,
    /// Timestamp of the snapshot.
    pub timestamp: DateTime<Utc>,
}

impl OrderBook {
    /// Get the best bid (highest buy price).
    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    /// Get the best ask (lowest sell price).
    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    /// Get the mid price (average of best bid and best ask).
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid.price + ask.price) / Decimal::TWO),
            (Some(bid), None) => Some(bid.price),
            (None, Some(ask)) => Some(ask.price),
            (None, None) => self.last_trade_price,
        }
    }

    /// Get the spread (difference between best ask and best bid).
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    /// Get the spread as a percentage of the mid price.
    pub fn spread_percent(&self) -> Option<Decimal> {
        let spread = self.spread()?;
        let mid = self.mid_price()?;
        if mid.is_zero() {
            return None;
        }
        Some((spread / mid) * Decimal::ONE_HUNDRED)
    }

    /// Get total bid depth (sum of all bid sizes).
    pub fn total_bid_depth(&self) -> Decimal {
        self.bids.iter().map(|l| l.size).sum()
    }

    /// Get total ask depth (sum of all ask sizes).
    pub fn total_ask_depth(&self) -> Decimal {
        self.asks.iter().map(|l| l.size).sum()
    }

    /// Get total bid value (sum of price * size for all bids).
    pub fn total_bid_value(&self) -> Decimal {
        self.bids.iter().map(|l| l.total_value()).sum()
    }

    /// Get total ask value (sum of price * size for all asks).
    pub fn total_ask_value(&self) -> Decimal {
        self.asks.iter().map(|l| l.total_value()).sum()
    }

    /// Calculate cumulative depth at each price level for bids.
    pub fn cumulative_bid_depth(&self) -> Vec<(Decimal, Decimal)> {
        let mut cumulative = Decimal::ZERO;
        self.bids
            .iter()
            .map(|level| {
                cumulative += level.size;
                (level.price, cumulative)
            })
            .collect()
    }

    /// Calculate cumulative depth at each price level for asks.
    pub fn cumulative_ask_depth(&self) -> Vec<(Decimal, Decimal)> {
        let mut cumulative = Decimal::ZERO;
        self.asks
            .iter()
            .map(|level| {
                cumulative += level.size;
                (level.price, cumulative)
            })
            .collect()
    }

    /// Get the price impact of buying a certain size.
    /// Returns (average_price, total_cost) or None if insufficient liquidity.
    pub fn buy_price_impact(&self, size: Decimal) -> Option<(Decimal, Decimal)> {
        let mut remaining = size;
        let mut total_cost = Decimal::ZERO;

        for level in &self.asks {
            if remaining.is_zero() {
                break;
            }
            let filled = remaining.min(level.size);
            total_cost += filled * level.price;
            remaining -= filled;
        }

        if remaining.is_zero() {
            let avg_price = total_cost / size;
            Some((avg_price, total_cost))
        } else {
            None // Insufficient liquidity
        }
    }

    /// Get the price impact of selling a certain size.
    /// Returns (average_price, total_proceeds) or None if insufficient liquidity.
    pub fn sell_price_impact(&self, size: Decimal) -> Option<(Decimal, Decimal)> {
        let mut remaining = size;
        let mut total_proceeds = Decimal::ZERO;

        for level in &self.bids {
            if remaining.is_zero() {
                break;
            }
            let filled = remaining.min(level.size);
            total_proceeds += filled * level.price;
            remaining -= filled;
        }

        if remaining.is_zero() {
            let avg_price = total_proceeds / size;
            Some((avg_price, total_proceeds))
        } else {
            None // Insufficient liquidity
        }
    }

    /// Get depth within a certain price range from the mid.
    pub fn depth_within_range(&self, percent_from_mid: Decimal) -> (Decimal, Decimal) {
        let mid = match self.mid_price() {
            Some(m) => m,
            None => return (Decimal::ZERO, Decimal::ZERO),
        };

        let range = mid * percent_from_mid / Decimal::ONE_HUNDRED;
        let lower_bound = mid - range;
        let upper_bound = mid + range;

        let bid_depth: Decimal = self
            .bids
            .iter()
            .filter(|l| l.price >= lower_bound)
            .map(|l| l.size)
            .sum();

        let ask_depth: Decimal = self
            .asks
            .iter()
            .filter(|l| l.price <= upper_bound)
            .map(|l| l.size)
            .sum();

        (bid_depth, ask_depth)
    }

    /// Check if there's sufficient liquidity for a trade.
    pub fn has_liquidity_for(&self, size: Decimal, is_buy: bool) -> bool {
        if is_buy {
            self.buy_price_impact(size).is_some()
        } else {
            self.sell_price_impact(size).is_some()
        }
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        Self {
            token_id: String::new(),
            market_id: String::new(),
            asset_id: String::new(),
            bids: Vec::new(),
            asks: Vec::new(),
            last_trade_price: None,
            hash: String::new(),
            timestamp: Utc::now(),
        }
    }
}

/// Aggregated orderbook data for a market (both outcomes).
#[derive(Debug, Clone, Default)]
pub struct MarketOrderBook {
    /// Orderbooks keyed by token_id.
    pub books: HashMap<String, OrderBook>,
    /// Whether orderbook data is currently loading.
    pub loading: bool,
    /// Last update timestamp.
    pub last_updated: Option<DateTime<Utc>>,
}

impl MarketOrderBook {
    /// Get the orderbook for a specific token/outcome.
    pub fn get(&self, token_id: &str) -> Option<&OrderBook> {
        self.books.get(token_id)
    }

    /// Insert or update an orderbook.
    pub fn insert(&mut self, book: OrderBook) {
        self.books.insert(book.token_id.clone(), book);
        self.last_updated = Some(Utc::now());
    }

    /// Get total liquidity across all outcomes.
    pub fn total_liquidity(&self) -> Decimal {
        self.books
            .values()
            .map(|b| b.total_bid_value() + b.total_ask_value())
            .sum()
    }
}

/// State for orderbook data across all markets.
#[derive(Debug, Default)]
pub struct OrderBookState {
    /// Orderbooks keyed by market_id.
    pub orderbooks: HashMap<String, MarketOrderBook>,
    /// Currently selected market for orderbook display.
    pub selected_market_id: Option<String>,
    /// Currently selected outcome/token index (0 = Yes, 1 = No typically).
    pub selected_outcome_index: usize,
    /// Display mode for the orderbook.
    pub display_mode: OrderBookDisplayMode,
    /// Number of price levels to show.
    pub display_levels: usize,
    /// Whether aggregated view is enabled (combine similar price levels).
    pub aggregated: bool,
    /// Aggregation tick size (for grouping price levels).
    pub aggregation_tick: Decimal,
}

/// Display modes for the orderbook widget.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OrderBookDisplayMode {
    /// Show both bids and asks side by side.
    #[default]
    Split,
    /// Show combined view with asks above, bids below.
    Combined,
    /// Show only bids.
    BidsOnly,
    /// Show only asks.
    AsksOnly,
}

impl OrderBookState {
    /// Create a new orderbook state with sensible defaults.
    pub fn new() -> Self {
        Self {
            orderbooks: HashMap::new(),
            selected_market_id: None,
            selected_outcome_index: 0,
            display_mode: OrderBookDisplayMode::Split,
            display_levels: 10,
            aggregated: false,
            aggregation_tick: Decimal::new(1, 2), // 0.01 default
        }
    }

    /// Get the orderbook for the currently selected market and outcome.
    pub fn selected_orderbook(&self) -> Option<&OrderBook> {
        let market_id = self.selected_market_id.as_ref()?;
        let market_book = self.orderbooks.get(market_id)?;
        
        // Find the book for the selected outcome index
        let token_ids: Vec<_> = market_book.books.keys().collect();
        token_ids.get(self.selected_outcome_index)
            .and_then(|tid| market_book.books.get(*tid))
    }

    /// Get the market orderbook for the selected market.
    pub fn selected_market_orderbook(&self) -> Option<&MarketOrderBook> {
        self.selected_market_id
            .as_ref()
            .and_then(|id| self.orderbooks.get(id))
    }

    /// Insert or update a market's orderbook.
    pub fn update_orderbook(&mut self, market_id: String, book: OrderBook) {
        self.orderbooks
            .entry(market_id)
            .or_default()
            .insert(book);
    }

    /// Set loading state for a market.
    pub fn set_loading(&mut self, market_id: &str, loading: bool) {
        if let Some(market_book) = self.orderbooks.get_mut(market_id) {
            market_book.loading = loading;
        }
    }
    
    /// Toggle display mode.
    pub fn cycle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            OrderBookDisplayMode::Split => OrderBookDisplayMode::Combined,
            OrderBookDisplayMode::Combined => OrderBookDisplayMode::BidsOnly,
            OrderBookDisplayMode::BidsOnly => OrderBookDisplayMode::AsksOnly,
            OrderBookDisplayMode::AsksOnly => OrderBookDisplayMode::Split,
        };
    }

    /// Toggle outcome selection (Yes/No).
    pub fn toggle_outcome(&mut self) {
        self.selected_outcome_index = if self.selected_outcome_index == 0 { 1 } else { 0 };
    }

    /// Increase display levels.
    pub fn increase_levels(&mut self) {
        self.display_levels = (self.display_levels + 5).min(50);
    }

    /// Decrease display levels.
    pub fn decrease_levels(&mut self) {
        self.display_levels = self.display_levels.saturating_sub(5).max(5);
    }
}
