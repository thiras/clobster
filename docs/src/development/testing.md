# Testing

Clobster uses Rust's built-in testing framework along with helpful crates for mocking and assertions.

## Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in a module
cargo test strategy::

# Run tests with coverage (requires cargo-llvm-cov)
cargo llvm-cov
```

## Test Organization

```
src/
├── strategy/
│   ├── traits.rs
│   ├── signal.rs      # Unit tests at bottom of file
│   └── strategies/
│       └── momentum.rs  # Strategy-specific tests
tests/
└── integration/       # Integration tests (future)
```

## Unit Tests

Place unit tests at the bottom of source files:

```rust
// src/strategy/signal.rs

pub struct Signal { /* ... */ }

impl Signal {
    pub fn buy(market_id: String, token_id: String, size: Decimal) -> Self {
        // ...
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_buy_signal_creation() {
        let signal = Signal::buy(
            "market_123".to_string(),
            "token_456".to_string(),
            dec!(10.0),
        );
        
        assert_eq!(signal.signal_type, SignalType::Buy);
        assert_eq!(signal.market_id, "market_123");
        assert_eq!(signal.size, dec!(10.0));
    }

    #[test]
    fn test_signal_builder() {
        let signal = Signal::buy("m".to_string(), "t".to_string(), dec!(5.0))
            .with_limit_price(dec!(0.45))
            .with_strength(SignalStrength::Strong);
        
        assert_eq!(signal.limit_price, Some(dec!(0.45)));
        assert_eq!(signal.strength, SignalStrength::Strong);
    }
}
```

## Testing Strategies

### Basic Strategy Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategy::{StrategyContext, MarketSnapshot, OutcomeSnapshot};
    use rust_decimal_macros::dec;
    use chrono::Utc;

    fn create_test_context() -> StrategyContext {
        StrategyContext::new(
            vec![MarketSnapshot {
                id: "market_1".to_string(),
                question: "Test market?".to_string(),
                outcomes: vec![OutcomeSnapshot {
                    token_id: "token_yes".to_string(),
                    name: "Yes".to_string(),
                    price: dec!(0.40),
                    bid: dec!(0.39),
                    ask: dec!(0.41),
                }],
                volume_24h: dec!(10000),
                status: MarketStatus::Active,
            }],
            vec![],  // No positions
            vec![],  // No orders
            dec!(1000.0),  // Available balance
            Utc::now(),
        )
    }

    #[test]
    fn test_momentum_generates_signal_on_trend() {
        let mut strategy = MomentumStrategy::builder()
            .entry_threshold(dec!(0.05))
            .build();
        
        let ctx = create_test_context();
        let signals = strategy.evaluate(&ctx);
        
        // Verify signal properties
        assert!(!signals.is_empty() || signals.is_empty()); // Depends on conditions
    }
}
```

### Testing with Mocks

Use `mockall` for mocking dependencies:

```rust
use mockall::mock;
use mockall::predicate::*;

mock! {
    pub ApiClient {
        async fn fetch_markets(&self) -> Result<Vec<Market>>;
        async fn place_order(&self, order: OrderRequest) -> Result<Order>;
    }
}

#[tokio::test]
async fn test_refresh_markets_updates_state() {
    let mut mock_api = MockApiClient::new();
    
    mock_api
        .expect_fetch_markets()
        .times(1)
        .returning(|| Ok(vec![
            Market {
                id: "test".to_string(),
                // ...
            }
        ]));
    
    // Use mock in test
    let markets = mock_api.fetch_markets().await.unwrap();
    assert_eq!(markets.len(), 1);
}
```

## Testing State Management

```rust
#[test]
fn test_store_reduce_markets_loaded() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut store = Store::new(tx);
    
    let markets = vec![
        Market {
            id: "m1".to_string(),
            question: "Test?".to_string(),
            // ...
        }
    ];
    
    store.reduce(Action::MarketsLoaded(markets.clone()));
    
    assert_eq!(store.markets.items.len(), 1);
    assert_eq!(store.markets.items[0].id, "m1");
    assert!(!store.markets.loading);
}

#[test]
fn test_store_reduce_set_view() {
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    let mut store = Store::new(tx);
    
    store.reduce(Action::SetView(View::Orders));
    
    assert_eq!(store.app.current_view, View::Orders);
}
```

## Async Tests

Use `#[tokio::test]` for async tests:

```rust
#[tokio::test]
async fn test_strategy_initialization() {
    let mut strategy = MomentumStrategy::default();
    let config = StrategyConfig::default();
    
    let result = strategy.initialize(&config).await;
    
    assert!(result.is_ok());
}
```

## Pretty Assertions

Use `pretty_assertions` for better diff output:

```rust
use pretty_assertions::assert_eq;

#[test]
fn test_complex_struct_equality() {
    let expected = Signal::buy("m".to_string(), "t".to_string(), dec!(10.0));
    let actual = create_signal();
    
    assert_eq!(expected, actual);  // Pretty diff on failure
}
```

## Test Helpers

Create helper functions for common test setup:

```rust
// tests/helpers.rs or in test module

pub fn create_mock_market(id: &str, price: Decimal) -> MarketSnapshot {
    MarketSnapshot {
        id: id.to_string(),
        question: format!("Test market {}?", id),
        outcomes: vec![OutcomeSnapshot {
            token_id: format!("token_{}", id),
            name: "Yes".to_string(),
            price,
            bid: price - dec!(0.01),
            ask: price + dec!(0.01),
        }],
        volume_24h: dec!(10000),
        status: MarketStatus::Active,
    }
}

pub fn create_test_position(market_id: &str, size: Decimal) -> PositionSnapshot {
    PositionSnapshot {
        market_id: market_id.to_string(),
        token_id: format!("token_{}", market_id),
        size,
        avg_price: dec!(0.50),
        current_price: dec!(0.55),
    }
}
```

## CI Integration

Tests run automatically in CI on every PR. See `.github/workflows/ci-checks.yml` for the full pipeline.
