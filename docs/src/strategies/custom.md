# Custom Strategies

Build your own trading strategies by implementing the `Strategy` trait.

## Basic Structure

```rust
use clobster::strategy::{Strategy, StrategyContext, Signal, StrategyConfig};
use clobster::error::Result;
use async_trait::async_trait;

#[derive(Debug)]
pub struct MyStrategy {
    name: String,
    threshold: f64,
    // Your custom state
}

#[async_trait]
impl Strategy for MyStrategy {
    fn name(&self) -> &str {
        &self.name
    }

    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        let mut signals = vec![];
        
        for market in ctx.markets() {
            // Your logic here
            if self.should_buy(market) {
                signals.push(
                    Signal::buy(
                        market.id.clone(),
                        market.outcomes[0].token_id.clone(),
                        0.10,
                    )
                );
            }
        }
        
        signals
    }
}
```

## Full Implementation Example

Here's a complete strategy that buys when a market's "Yes" outcome drops below a threshold:

```rust
use clobster::strategy::{
    Strategy, StrategyContext, StrategyConfig, StrategyMetadata,
    Signal, SignalStrength, MarketSnapshot,
};
use clobster::error::Result;
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ValueStrategy {
    /// Minimum probability to trigger a buy
    threshold: Decimal,
    /// Position size as fraction of balance
    position_size: Decimal,
    /// Markets we've already bought
    bought_markets: HashMap<String, Decimal>,
}

impl ValueStrategy {
    pub fn new(threshold: Decimal, position_size: Decimal) -> Self {
        Self {
            threshold,
            position_size,
            bought_markets: HashMap::new(),
        }
    }
    
    pub fn builder() -> ValueStrategyBuilder {
        ValueStrategyBuilder::default()
    }
}

#[async_trait]
impl Strategy for ValueStrategy {
    fn name(&self) -> &str {
        "value_strategy"
    }
    
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Value Strategy".to_string(),
            description: "Buys undervalued outcomes".to_string(),
            version: "1.0.0".to_string(),
            author: Some("Your Name".to_string()),
            tags: vec!["value".to_string(), "simple".to_string()],
        }
    }
    
    async fn initialize(&mut self, config: &StrategyConfig) -> Result<()> {
        if let Some(threshold) = config.get_decimal("threshold") {
            self.threshold = threshold;
        }
        if let Some(size) = config.get_decimal("position_size") {
            self.position_size = size;
        }
        Ok(())
    }

    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        let mut signals = vec![];
        
        for market in ctx.markets() {
            // Skip if we already have a position
            if self.bought_markets.contains_key(&market.id) {
                continue;
            }
            
            // Check each outcome
            for outcome in &market.outcomes {
                if outcome.price < self.threshold {
                    let size = ctx.available_balance * self.position_size;
                    
                    signals.push(
                        Signal::buy(
                            market.id.clone(),
                            outcome.token_id.clone(),
                            size,
                        )
                        .with_limit_price(outcome.price + dec!(0.01))
                        .with_strength(SignalStrength::Medium)
                        .with_reason(format!(
                            "Price {} below threshold {}",
                            outcome.price, self.threshold
                        ))
                    );
                }
            }
        }
        
        signals
    }
    
    fn on_signal_executed(&mut self, signal: &Signal, success: bool) {
        if success {
            self.bought_markets.insert(
                signal.market_id.clone(),
                signal.size,
            );
        }
    }
}

// Builder pattern for ergonomic construction
#[derive(Default)]
pub struct ValueStrategyBuilder {
    threshold: Option<Decimal>,
    position_size: Option<Decimal>,
}

impl ValueStrategyBuilder {
    pub fn threshold(mut self, threshold: Decimal) -> Self {
        self.threshold = Some(threshold);
        self
    }
    
    pub fn position_size(mut self, size: Decimal) -> Self {
        self.position_size = Some(size);
        self
    }
    
    pub fn build(self) -> ValueStrategy {
        ValueStrategy::new(
            self.threshold.unwrap_or(dec!(0.30)),
            self.position_size.unwrap_or(dec!(0.10)),
        )
    }
}
```

## Accessing Context Data

The `StrategyContext` provides:

```rust
fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
    // All available markets
    let markets = ctx.markets();
    
    // Current positions
    let positions = ctx.positions();
    
    // Open orders
    let orders = ctx.orders();
    
    // Available balance for trading
    let balance = ctx.available_balance;
    
    // Current timestamp
    let now = ctx.timestamp;
    
    // ...
}
```

## Signal Builder

Use the builder pattern for signals:

```rust
Signal::buy(market_id, token_id, size)
    .with_limit_price(price)           // Limit order price
    .with_strength(SignalStrength::Strong)  // Signal confidence
    .with_reason("Explanation")        // For logging/debugging
    .with_expiry(Duration::from_secs(60))  // Auto-cancel after
```

## Lifecycle Hooks

Override these methods for additional functionality:

```rust
// Called when the strategy is loaded
async fn initialize(&mut self, config: &StrategyConfig) -> Result<()>;

// Called on every market data update
fn on_market_update(&mut self, ctx: &StrategyContext);

// Called when your signal was executed
fn on_signal_executed(&mut self, signal: &Signal, success: bool);

// Called when your order fills
fn on_order_filled(&mut self, order_id: &str, price: Decimal, size: Decimal);

// Called when your order is cancelled
fn on_order_cancelled(&mut self, order_id: &str);

// Called when strategy is stopped
async fn shutdown(&mut self) -> Result<()>;
```

## Testing Your Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use clobster::strategy::StrategyContext;
    
    #[test]
    fn test_value_strategy_generates_buy_signal() {
        let mut strategy = ValueStrategy::builder()
            .threshold(dec!(0.30))
            .build();
        
        let ctx = create_test_context_with_low_prices();
        let signals = strategy.evaluate(&ctx);
        
        assert!(!signals.is_empty());
        assert!(signals[0].is_buy());
    }
}
```

## Best Practices

1. **Use `Decimal`** for all financial calculations
2. **Track state** to avoid duplicate signals
3. **Implement `on_signal_executed`** to update internal state
4. **Add logging** for debugging in production
5. **Validate configuration** in `initialize()`
6. **Handle edge cases** (empty markets, zero balance, etc.)
