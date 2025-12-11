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
    ///
    /// Returns the average price to buy `size` shares.
    ///
    /// Returns `None` if there is no available liquidity. If the requested
    /// `size` exceeds available ask liquidity, returns the VWAP for the
    /// maximum available size (partial fill).
    pub fn vwap_buy(&self, size: Decimal) -> Option<Decimal> {
        self.calculate_vwap(&self.asks, size)
    }

    /// Calculate Volume Weighted Average Price (VWAP) for sells.
    ///
    /// Returns the average price to sell `size` shares.
    ///
    /// Returns `None` if there is no available liquidity. If the requested
    /// `size` exceeds available bid liquidity, returns the VWAP for the
    /// maximum available size (partial fill).
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

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn create_test_orderbook() -> OrderBookDepth {
        let mut book = OrderBookDepth::new("market_1", "token_1");
        // Bids: descending price order (best bid first)
        book.bids = vec![
            PriceLevel::new(dec!(0.50), dec!(100.0)),
            PriceLevel::new(dec!(0.49), dec!(200.0)),
            PriceLevel::new(dec!(0.48), dec!(150.0)),
        ];
        // Asks: ascending price order (best ask first)
        book.asks = vec![
            PriceLevel::new(dec!(0.52), dec!(80.0)),
            PriceLevel::new(dec!(0.53), dec!(120.0)),
            PriceLevel::new(dec!(0.54), dec!(100.0)),
        ];
        book
    }

    #[test]
    fn test_price_level_value() {
        let level = PriceLevel::new(dec!(0.50), dec!(100.0));
        assert_eq!(level.value(), dec!(50.0));
    }

    #[test]
    fn test_best_bid_ask() {
        let book = create_test_orderbook();
        assert_eq!(book.best_bid_price(), Some(dec!(0.50)));
        assert_eq!(book.best_ask_price(), Some(dec!(0.52)));
    }

    #[test]
    fn test_mid_price() {
        let book = create_test_orderbook();
        // (0.50 + 0.52) / 2 = 0.51
        assert_eq!(book.mid_price(), Some(dec!(0.51)));
    }

    #[test]
    fn test_spread() {
        let book = create_test_orderbook();
        // 0.52 - 0.50 = 0.02
        assert_eq!(book.spread(), Some(dec!(0.02)));
    }

    #[test]
    fn test_spread_percent() {
        let book = create_test_orderbook();
        // spread = 0.02, mid = 0.51
        // spread_percent = (0.02 / 0.51) * 100 ≈ 3.92%
        let spread_pct = book.spread_percent().unwrap();
        assert!(spread_pct > dec!(3.9) && spread_pct < dec!(4.0));
    }

    #[test]
    fn test_bid_volume() {
        let book = create_test_orderbook();
        // All bids: 100 + 200 + 150 = 450
        assert_eq!(book.bid_volume(10), dec!(450.0));
        // First 2 levels: 100 + 200 = 300
        assert_eq!(book.bid_volume(2), dec!(300.0));
    }

    #[test]
    fn test_ask_volume() {
        let book = create_test_orderbook();
        // All asks: 80 + 120 + 100 = 300
        assert_eq!(book.ask_volume(10), dec!(300.0));
    }

    #[test]
    fn test_imbalance() {
        let book = create_test_orderbook();
        // bid_vol = 450, ask_vol = 300
        // imbalance = (450 - 300) / (450 + 300) = 150 / 750 = 0.2
        assert_eq!(book.imbalance(10), Some(dec!(0.2)));
    }

    #[test]
    fn test_vwap_buy_single_level() {
        let book = create_test_orderbook();
        // Buy 50 shares - fits in first ask level at 0.52
        assert_eq!(book.vwap_buy(dec!(50.0)), Some(dec!(0.52)));
    }

    #[test]
    fn test_vwap_buy_multiple_levels() {
        let book = create_test_orderbook();
        // Buy 100 shares:
        // 80 @ 0.52 = 41.6
        // 20 @ 0.53 = 10.6
        // Total: 52.2 / 100 = 0.522
        assert_eq!(book.vwap_buy(dec!(100.0)), Some(dec!(0.522)));
    }

    #[test]
    fn test_vwap_buy_partial_fill() {
        let book = create_test_orderbook();
        // Try to buy 500 shares but only 300 available
        // Should return VWAP for partial fill (300 shares)
        // 80 @ 0.52 + 120 @ 0.53 + 100 @ 0.54 = 41.6 + 63.6 + 54 = 159.2
        // 159.2 / 300 = 0.5306666...
        let vwap = book.vwap_buy(dec!(500.0)).unwrap();
        assert!(vwap > dec!(0.53) && vwap < dec!(0.532));
    }

    #[test]
    fn test_vwap_sell_single_level() {
        let book = create_test_orderbook();
        // Sell 50 shares - fits in first bid level at 0.50
        assert_eq!(book.vwap_sell(dec!(50.0)), Some(dec!(0.50)));
    }

    #[test]
    fn test_vwap_empty_book() {
        let book = OrderBookDepth::new("market_1", "token_1");
        assert_eq!(book.vwap_buy(dec!(100.0)), None);
        assert_eq!(book.vwap_sell(dec!(100.0)), None);
    }

    #[test]
    fn test_slippage_buy() {
        let book = create_test_orderbook();
        // Buy 100 shares: VWAP = 0.522, best ask = 0.52
        // Slippage = ((0.522 - 0.52) / 0.52) * 100 ≈ 0.38%
        let slippage = book.slippage_buy(dec!(100.0)).unwrap();
        assert!(slippage > dec!(0.3) && slippage < dec!(0.5));
    }

    #[test]
    fn test_slippage_sell() {
        let book = create_test_orderbook();
        // Sell 150 shares: 100 @ 0.50 + 50 @ 0.49
        // VWAP = (50 + 24.5) / 150 = 74.5 / 150 = 0.4966...
        // Best bid = 0.50
        // Slippage = ((0.50 - 0.4966) / 0.50) * 100 ≈ 0.66%
        let slippage = book.slippage_sell(dec!(150.0)).unwrap();
        assert!(slippage > dec!(0.5) && slippage < dec!(0.8));
    }

    #[test]
    fn test_cumulative_bids() {
        let book = create_test_orderbook();
        let cumulative = book.cumulative_bids();
        assert_eq!(cumulative.len(), 3);
        assert_eq!(cumulative[0], (dec!(0.50), dec!(100.0)));
        assert_eq!(cumulative[1], (dec!(0.49), dec!(300.0)));
        assert_eq!(cumulative[2], (dec!(0.48), dec!(450.0)));
    }

    #[test]
    fn test_orderbook_state() {
        let mut state = OrderBookState::new();
        assert!(state.books.is_empty());
        assert_eq!(state.display_depth, 10);

        let book = create_test_orderbook();
        state.update_book(book);

        assert_eq!(state.books.len(), 1);
        assert!(state.get_book("token_1").is_some());
        assert!(state.last_updated.is_some());

        state.selected_token_id = Some("token_1".to_string());
        assert!(state.selected_book().is_some());

        state.remove_book("token_1");
        assert!(state.books.is_empty());
    }

    #[test]
    fn test_orderbook_stats() {
        let book = create_test_orderbook();
        let stats = OrderBookStats::from_orderbook(&book, 10);

        assert_eq!(stats.best_bid, Some(dec!(0.50)));
        assert_eq!(stats.best_ask, Some(dec!(0.52)));
        assert_eq!(stats.mid_price, Some(dec!(0.51)));
        assert_eq!(stats.bid_depth, 3);
        assert_eq!(stats.ask_depth, 3);
    }

    #[test]
    fn test_empty_orderbook() {
        let book = OrderBookDepth::new("market_1", "token_1");
        assert!(book.is_empty());
        assert!(book.best_bid().is_none());
        assert!(book.best_ask().is_none());
        assert_eq!(book.mid_price(), None);
        assert_eq!(book.spread(), None);
        assert_eq!(book.imbalance(10), None);
    }
}
