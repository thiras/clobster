# Order Book Module

The order book module provides real-time market depth data for trading decisions.

## Overview

Order book data shows the current buy (bid) and sell (ask) orders at various price levels, enabling:

- **Liquidity analysis** - Understanding available volume at each price
- **Spread monitoring** - Tracking bid-ask spreads
- **Slippage estimation** - Calculating execution costs for larger orders
- **Market sentiment** - Measuring order book imbalance

## Core Types

### PriceLevel

A single price level in the order book:

```rust
pub struct PriceLevel {
    /// Price at this level
    pub price: Decimal,
    /// Total size available at this price
    pub size: Decimal,
}

impl PriceLevel {
    pub fn new(price: Decimal, size: Decimal) -> Self;
    
    /// Get the total value (price × size) at this level
    pub fn value(&self) -> Decimal;
}
```

### OrderBookDepth

Complete order book snapshot for a single token/outcome:

```rust
pub struct OrderBookDepth {
    /// Market condition ID
    pub market_id: String,
    /// Token/asset ID for this outcome
    pub token_id: String,
    /// Order book hash (for synchronization)
    pub hash: String,
    /// Timestamp of this snapshot
    pub timestamp: DateTime<Utc>,
    /// Bid (buy) side price levels, sorted by price descending
    pub bids: Vec<PriceLevel>,
    /// Ask (sell) side price levels, sorted by price ascending
    pub asks: Vec<PriceLevel>,
}
```

### Key Methods

```rust
impl OrderBookDepth {
    // Best prices
    pub fn best_bid(&self) -> Option<&PriceLevel>;
    pub fn best_ask(&self) -> Option<&PriceLevel>;
    pub fn best_bid_price(&self) -> Option<Decimal>;
    pub fn best_ask_price(&self) -> Option<Decimal>;
    
    // Spread calculations
    pub fn mid_price(&self) -> Option<Decimal>;
    pub fn spread(&self) -> Option<Decimal>;
    pub fn spread_percent(&self) -> Option<Decimal>;
    
    // Liquidity metrics
    pub fn bid_liquidity(&self, depth: usize) -> Decimal;
    pub fn ask_liquidity(&self, depth: usize) -> Decimal;
    pub fn total_liquidity(&self, depth: usize) -> Decimal;
    pub fn bid_volume(&self, depth: usize) -> Decimal;
    pub fn ask_volume(&self, depth: usize) -> Decimal;
    
    // Market analysis
    pub fn imbalance(&self, depth: usize) -> Option<Decimal>;
    pub fn vwap_buy(&self, size: Decimal) -> Option<Decimal>;
    pub fn vwap_sell(&self, size: Decimal) -> Option<Decimal>;
    pub fn slippage_buy(&self, size: Decimal) -> Option<Decimal>;
    pub fn slippage_sell(&self, size: Decimal) -> Option<Decimal>;
    
    // Cumulative depth
    pub fn cumulative_bids(&self) -> Vec<(Decimal, Decimal)>;
    pub fn cumulative_asks(&self) -> Vec<(Decimal, Decimal)>;
}
```

## Order Book Imbalance

The imbalance ratio measures buy vs sell pressure:

$$\text{Imbalance} = \frac{V_{bid} - V_{ask}}{V_{bid} + V_{ask}}$$

- **Positive** (+1.0 max): More buy pressure
- **Negative** (-1.0 min): More sell pressure
- **Zero**: Balanced order book

```rust
let book = store.orderbooks.get_book("token_yes").unwrap();
if let Some(imbalance) = book.imbalance(10) {
    if imbalance > dec!(0.5) {
        println!("Strong buy pressure detected");
    }
}
```

## VWAP Calculations

Volume Weighted Average Price (VWAP) shows the average execution price for a given order size:

```rust
let book = store.orderbooks.get_book("token_yes").unwrap();

// What price would I pay to buy 100 shares?
if let Some(vwap) = book.vwap_buy(dec!(100.0)) {
    println!("Average buy price for 100 shares: {}", vwap);
}

// What price would I receive selling 50 shares?
if let Some(vwap) = book.vwap_sell(dec!(50.0)) {
    println!("Average sell price for 50 shares: {}", vwap);
}
```

## Slippage Estimation

Estimate the cost of market impact:

```rust
// Slippage is the difference between best price and actual VWAP
if let Some(slippage) = book.slippage_buy(dec!(100.0)) {
    println!("Estimated slippage for buying 100: {}%", slippage);
}
```

## OrderBookStats

Summary statistics for quick analysis:

```rust
pub struct OrderBookStats {
    pub best_bid: Option<Decimal>,
    pub best_ask: Option<Decimal>,
    pub mid_price: Option<Decimal>,
    pub spread: Option<Decimal>,
    pub spread_percent: Option<Decimal>,
    pub bid_liquidity: Decimal,
    pub ask_liquidity: Decimal,
    pub imbalance: Option<Decimal>,
    pub bid_depth: usize,
    pub ask_depth: usize,
}

// Create stats from an order book
let stats = OrderBookStats::from_orderbook(&book, 10);
```

## OrderBookState

Container for managing multiple order books:

```rust
pub struct OrderBookState {
    /// Order books by token ID
    pub books: HashMap<String, OrderBookDepth>,
    /// Currently selected token ID for detailed view
    pub selected_token_id: Option<String>,
    /// Whether order books are currently loading
    pub loading: bool,
    /// Last update timestamp
    pub last_updated: Option<DateTime<Utc>>,
    /// Display depth (number of levels to show)
    pub display_depth: usize,
    /// Last error encountered when updating order books
    pub error: Option<String>,
}

impl OrderBookState {
    pub fn get_book(&self, token_id: &str) -> Option<&OrderBookDepth>;
    pub fn selected_book(&self) -> Option<&OrderBookDepth>;
    pub fn update_book(&mut self, book: OrderBookDepth);
    pub fn remove_book(&mut self, token_id: &str);
    pub fn clear(&mut self);
    pub fn get_stats(&self, token_id: &str) -> Option<OrderBookStats>;
}
```

## API Client Methods

Fetch order book data from the Polymarket API:

```rust
// Fetch single order book
let book = api.fetch_orderbook("token_yes_id").await?;

// Fetch multiple order books
use polymarket_rs::types::Side;
let books = api.fetch_orderbooks(&[
    ("token_1".to_string(), Side::Buy),
    ("token_2".to_string(), Side::Sell),
]).await?;
```

## Actions

Order book-related actions for state management:

```rust
// Load order book for a token
store.dispatch(Action::LoadOrderBook("token_id".to_string()))?;

// When loaded, the store receives:
store.reduce(Action::OrderBookLoaded(order_book_depth));

// Select a book for detailed view
store.reduce(Action::SelectOrderBook("token_id".to_string()));

// Clear order books
store.reduce(Action::ClearOrderBook("token_id".to_string()));
store.reduce(Action::ClearAllOrderBooks);

// Adjust display depth (1-100 levels)
store.reduce(Action::SetOrderBookDepth(20));

// Refresh order book data
store.dispatch(Action::RefreshOrderBook("token_id".to_string()))?;
```

## Usage Example

```rust
use clobster::state::{Store, Action};
use rust_decimal_macros::dec;

// Fetch order book
store.dispatch(Action::LoadOrderBook("token_yes".to_string()))?;

// Later, analyze the data
if let Some(book) = store.orderbooks.get_book("token_yes") {
    // Check spread
    if let Some(spread_pct) = book.spread_percent() {
        if spread_pct > dec!(5.0) {
            println!("Wide spread: {}%", spread_pct);
        }
    }
    
    // Check liquidity
    let total = book.total_liquidity(10);
    println!("Top 10 levels liquidity: ${}", total);
    
    // Estimate execution cost
    let order_size = dec!(50.0);
    if let Some(slippage) = book.slippage_buy(order_size) {
        println!("Slippage for {} shares: {}%", order_size, slippage);
    }
    
    // Check market sentiment
    if let Some(imbalance) = book.imbalance(10) {
        let sentiment = if imbalance > dec!(0.3) {
            "bullish"
        } else if imbalance < dec!(-0.3) {
            "bearish"
        } else {
            "neutral"
        };
        println!("Market sentiment: {}", sentiment);
    }
}
```

## Integration with Strategies

Order book data can inform trading strategies:

```rust
impl Strategy for MyStrategy {
    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        // Access order book from context (when available)
        // Use spread, imbalance, and slippage to make decisions
        
        let mut signals = vec![];
        
        // Example: Only trade when spread is tight
        for market in ctx.markets() {
            // Your order book-aware logic here
        }
        
        signals
    }
}
```
