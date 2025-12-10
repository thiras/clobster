//! Order book depth state.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// A price level in the order book.
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

    /// Get the total value at this price level.
    pub fn value(&self) -> Decimal {
        self.price * self.size
    }
}

/// Order book depth for a single outcome/token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDepth {
    /// Market condition ID.
    pub market_id: String,
    /// Token/asset ID for this outcome.
    pub token_id: String,
    /// Order book hash (for synchronization).
    pub hash: String,
    /// Timestamp of this snapshot.
    pub timestamp: DateTime<Utc>,
    /// Bid (buy) side price levels, sorted by price descending.
    pub bids: Vec<PriceLevel>,
    /// Ask (sell) side price levels, sorted by price ascending.
    pub asks: Vec<PriceLevel>,
}

impl OrderBookDepth {
    /// Create a new empty order book.
    pub fn new(market_id: impl Into<String>, token_id: impl Into<String>) -> Self {
        Self {
            market_id: market_id.into(),
            token_id: token_id.into(),
            hash: String::new(),
            timestamp: Utc::now(),
            bids: Vec::new(),
            asks: Vec::new(),
        }
    }

    /// Get the best bid (highest buy price).
    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    /// Get the best ask (lowest sell price).
    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    /// Get the best bid price.
    pub fn best_bid_price(&self) -> Option<Decimal> {
        self.best_bid().map(|l| l.price)
    }

    /// Get the best ask price.
    pub fn best_ask_price(&self) -> Option<Decimal> {
        self.best_ask().map(|l| l.price)
    }

    /// Get the mid price.
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_bid_price(), self.best_ask_price()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::TWO),
            _ => None,
        }
    }

    /// Get the bid-ask spread.
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid_price(), self.best_ask_price()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    /// Get the spread as a percentage of the mid price.
    pub fn spread_percent(&self) -> Option<Decimal> {
        match (self.spread(), self.mid_price()) {
            (Some(spread), Some(mid)) if !mid.is_zero() => {
                Some((spread / mid) * Decimal::ONE_HUNDRED)
            }
            _ => None,
        }
    }

    /// Calculate total bid liquidity up to a certain depth.
    pub fn bid_liquidity(&self, depth: usize) -> Decimal {
        self.bids.iter().take(depth).map(|l| l.value()).sum()
    }

    /// Calculate total ask liquidity up to a certain depth.
    pub fn ask_liquidity(&self, depth: usize) -> Decimal {
        self.asks.iter().take(depth).map(|l| l.value()).sum()
    }

    /// Calculate total bid volume (size) up to a certain depth.
    pub fn bid_volume(&self, depth: usize) -> Decimal {
        self.bids.iter().take(depth).map(|l| l.size).sum()
    }

    /// Calculate total ask volume (size) up to a certain depth.
    pub fn ask_volume(&self, depth: usize) -> Decimal {
        self.asks.iter().take(depth).map(|l| l.size).sum()
    }

    /// Calculate total liquidity (bid + ask) up to a certain depth.
    pub fn total_liquidity(&self, depth: usize) -> Decimal {
        self.bid_liquidity(depth) + self.ask_liquidity(depth)
    }

    /// Get order book imbalance ratio.
    /// Positive values indicate more buy pressure, negative indicates sell pressure.
    /// Range: -1.0 to 1.0
    pub fn imbalance(&self, depth: usize) -> Option<Decimal> {
        let bid_vol = self.bid_volume(depth);
        let ask_vol = self.ask_volume(depth);
        let total = bid_vol + ask_vol;

        if total.is_zero() {
            None
        } else {
            Some((bid_vol - ask_vol) / total)
        }
    }

    /// Calculate Volume Weighted Average Price (VWAP) for buys.
    /// Returns the average price to buy `size` shares.
    pub fn vwap_buy(&self, size: Decimal) -> Option<Decimal> {
        self.calculate_vwap(&self.asks, size)
    }

    /// Calculate Volume Weighted Average Price (VWAP) for sells.
    /// Returns the average price to sell `size` shares.
    pub fn vwap_sell(&self, size: Decimal) -> Option<Decimal> {
        self.calculate_vwap(&self.bids, size)
    }

    /// Calculate estimated slippage for a market buy order.
    pub fn slippage_buy(&self, size: Decimal) -> Option<Decimal> {
        match (self.vwap_buy(size), self.best_ask_price()) {
            (Some(vwap), Some(best)) if !best.is_zero() => {
                Some(((vwap - best) / best) * Decimal::ONE_HUNDRED)
            }
            _ => None,
        }
    }

    /// Calculate estimated slippage for a market sell order.
    pub fn slippage_sell(&self, size: Decimal) -> Option<Decimal> {
        match (self.vwap_sell(size), self.best_bid_price()) {
            (Some(vwap), Some(best)) if !best.is_zero() => {
                Some(((best - vwap) / best) * Decimal::ONE_HUNDRED)
            }
            _ => None,
        }
    }

    /// Calculate VWAP walking through price levels.
    fn calculate_vwap(&self, levels: &[PriceLevel], target_size: Decimal) -> Option<Decimal> {
        if levels.is_empty() || target_size.is_zero() {
            return None;
        }

        let mut remaining = target_size;
        let mut total_value = Decimal::ZERO;
        let mut total_size = Decimal::ZERO;

        for level in levels {
            let fill_size = remaining.min(level.size);
            total_value += level.price * fill_size;
            total_size += fill_size;
            remaining -= fill_size;

            if remaining.is_zero() {
                break;
            }
        }

        if total_size.is_zero() {
            None
        } else {
            Some(total_value / total_size)
        }
    }

    /// Get the depth (number of price levels) on the bid side.
    pub fn bid_depth(&self) -> usize {
        self.bids.len()
    }

    /// Get the depth (number of price levels) on the ask side.
    pub fn ask_depth(&self) -> usize {
        self.asks.len()
    }

    /// Check if the order book is empty.
    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    /// Get cumulative bid depth at each price level.
    pub fn cumulative_bids(&self) -> Vec<(Decimal, Decimal)> {
        let mut cumulative = Decimal::ZERO;
        self.bids
            .iter()
            .map(|l| {
                cumulative += l.size;
                (l.price, cumulative)
            })
            .collect()
    }

    /// Get cumulative ask depth at each price level.
    pub fn cumulative_asks(&self) -> Vec<(Decimal, Decimal)> {
        let mut cumulative = Decimal::ZERO;
        self.asks
            .iter()
            .map(|l| {
                cumulative += l.size;
                (l.price, cumulative)
            })
            .collect()
    }
}

/// Order book summary statistics.
#[derive(Debug, Clone, Default)]
pub struct OrderBookStats {
    /// Best bid price.
    pub best_bid: Option<Decimal>,
    /// Best ask price.
    pub best_ask: Option<Decimal>,
    /// Mid price.
    pub mid_price: Option<Decimal>,
    /// Bid-ask spread.
    pub spread: Option<Decimal>,
    /// Spread as percentage.
    pub spread_percent: Option<Decimal>,
    /// Total bid liquidity (value).
    pub bid_liquidity: Decimal,
    /// Total ask liquidity (value).
    pub ask_liquidity: Decimal,
    /// Order book imbalance (-1 to 1).
    pub imbalance: Option<Decimal>,
    /// Number of bid levels.
    pub bid_depth: usize,
    /// Number of ask levels.
    pub ask_depth: usize,
}

impl OrderBookStats {
    /// Create stats from an order book.
    pub fn from_orderbook(book: &OrderBookDepth, depth: usize) -> Self {
        Self {
            best_bid: book.best_bid_price(),
            best_ask: book.best_ask_price(),
            mid_price: book.mid_price(),
            spread: book.spread(),
            spread_percent: book.spread_percent(),
            bid_liquidity: book.bid_liquidity(depth),
            ask_liquidity: book.ask_liquidity(depth),
            imbalance: book.imbalance(depth),
            bid_depth: book.bid_depth(),
            ask_depth: book.ask_depth(),
        }
    }
}

/// State for order book data.
#[derive(Debug, Default)]
pub struct OrderBookState {
    /// Order books by token ID.
    pub books: std::collections::HashMap<String, OrderBookDepth>,
    /// Currently selected token ID for detailed view.
    pub selected_token_id: Option<String>,
    /// Whether order books are currently loading.
    pub loading: bool,
    /// Last update timestamp.
    pub last_updated: Option<DateTime<Utc>>,
    /// Display depth (number of levels to show).
    pub display_depth: usize,
    /// Error message if loading failed.
    pub error: Option<String>,
}

impl OrderBookState {
    /// Create a new order book state.
    pub fn new() -> Self {
        Self {
            books: std::collections::HashMap::new(),
            selected_token_id: None,
            loading: false,
            last_updated: None,
            display_depth: 10,
            error: None,
        }
    }

    /// Get order book for a specific token.
    pub fn get_book(&self, token_id: &str) -> Option<&OrderBookDepth> {
        self.books.get(token_id)
    }

    /// Get the currently selected order book.
    pub fn selected_book(&self) -> Option<&OrderBookDepth> {
        self.selected_token_id
            .as_ref()
            .and_then(|id| self.books.get(id))
    }

    /// Update an order book.
    pub fn update_book(&mut self, book: OrderBookDepth) {
        self.books.insert(book.token_id.clone(), book);
        self.last_updated = Some(Utc::now());
    }

    /// Remove an order book.
    pub fn remove_book(&mut self, token_id: &str) {
        self.books.remove(token_id);
    }

    /// Clear all order books.
    pub fn clear(&mut self) {
        self.books.clear();
        self.selected_token_id = None;
    }

    /// Get stats for a specific token.
    pub fn get_stats(&self, token_id: &str) -> Option<OrderBookStats> {
        self.get_book(token_id)
            .map(|book| OrderBookStats::from_orderbook(book, self.display_depth))
    }

    /// Get all token IDs with order book data.
    pub fn token_ids(&self) -> Vec<&String> {
        self.books.keys().collect()
    }
}
