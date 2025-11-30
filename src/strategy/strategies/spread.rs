//! Spread strategy.
//!
//! Market making strategy that places orders on both sides of the spread.

use crate::error::Result;
use crate::state::OrderSide;
use crate::strategy::{
    ParameterDef, ParameterType, ParameterValue, Signal, SignalStrength, SignalType, Strategy,
    StrategyConfig, StrategyContext, StrategyMetadata,
};
use async_trait::async_trait;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::HashMap;

/// Spread/market-making strategy.
///
/// This strategy provides liquidity by placing limit orders on both
/// sides of the market, profiting from the bid-ask spread.
#[derive(Debug)]
pub struct SpreadStrategy {
    /// Minimum spread to participate (as decimal).
    min_spread: Decimal,
    /// Offset from mid-price for bids.
    bid_offset: Decimal,
    /// Offset from mid-price for asks.
    ask_offset: Decimal,
    /// Order size.
    order_size: Decimal,
    /// Minimum liquidity required.
    min_liquidity: Decimal,
    /// Maximum inventory imbalance.
    max_inventory_imbalance: Decimal,
    /// Current inventory per market.
    inventory: HashMap<String, Decimal>,
    /// Active order pairs.
    #[allow(dead_code)]
    active_orders: HashMap<String, OrderPair>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct OrderPair {
    bid_id: Option<String>,
    ask_id: Option<String>,
    mid_price: Decimal,
}

impl SpreadStrategy {
    /// Create a new spread strategy with default parameters.
    pub fn new() -> Self {
        Self {
            min_spread: dec!(0.02),            // 2% minimum spread
            bid_offset: dec!(0.01),            // 1% below mid
            ask_offset: dec!(0.01),            // 1% above mid
            order_size: dec!(5),               // 5 USDC per side
            min_liquidity: dec!(1000),         // Minimum 1000 liquidity
            max_inventory_imbalance: dec!(50), // Max 50 units imbalance
            inventory: HashMap::new(),
            active_orders: HashMap::new(),
        }
    }

    /// Set the minimum spread.
    pub fn with_min_spread(mut self, spread: Decimal) -> Self {
        self.min_spread = spread;
        self
    }

    /// Set the bid offset.
    pub fn with_bid_offset(mut self, offset: Decimal) -> Self {
        self.bid_offset = offset;
        self
    }

    /// Set the ask offset.
    pub fn with_ask_offset(mut self, offset: Decimal) -> Self {
        self.ask_offset = offset;
        self
    }

    /// Set the order size.
    pub fn with_order_size(mut self, size: Decimal) -> Self {
        self.order_size = size;
        self
    }

    fn calculate_mid_price(&self, yes_price: Decimal) -> Decimal {
        // For binary markets, mid = yes_price (since no = 1 - yes)
        yes_price
    }

    fn get_inventory(&self, market_id: &str) -> Decimal {
        self.inventory
            .get(market_id)
            .copied()
            .unwrap_or(Decimal::ZERO)
    }

    fn adjust_size_for_inventory(
        &self,
        base_size: Decimal,
        inventory: Decimal,
        side: OrderSide,
    ) -> Decimal {
        // Avoid division by zero if max_inventory_imbalance is zero
        if self.max_inventory_imbalance.is_zero() {
            return base_size;
        }

        // Reduce size when inventory is imbalanced in the direction of the trade
        let imbalance_ratio = inventory.abs() / self.max_inventory_imbalance;

        match side {
            OrderSide::Buy if inventory > Decimal::ZERO => {
                // Already long, reduce buy size
                base_size * (Decimal::ONE - imbalance_ratio.min(dec!(0.8)))
            }
            OrderSide::Sell if inventory < Decimal::ZERO => {
                // Already short, reduce sell size
                base_size * (Decimal::ONE - imbalance_ratio.min(dec!(0.8)))
            }
            _ => base_size,
        }
    }
}

impl Default for SpreadStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Strategy for SpreadStrategy {
    fn name(&self) -> &str {
        "spread"
    }

    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Spread/Market Making".to_string(),
            description: "Provides liquidity by placing orders on both sides of the spread"
                .to_string(),
            version: "1.0.0".to_string(),
            author: Some("Clobster".to_string()),
            tags: vec![
                "market-making".to_string(),
                "spread".to_string(),
                "liquidity".to_string(),
            ],
        }
    }

    async fn initialize(&mut self, config: &StrategyConfig) -> Result<()> {
        if let Some(n) = config.parameters.get("min_spread").and_then(|v| v.as_f64()) {
            self.min_spread = Decimal::try_from(n).unwrap_or(self.min_spread);
        }
        if let Some(n) = config.parameters.get("bid_offset").and_then(|v| v.as_f64()) {
            self.bid_offset = Decimal::try_from(n).unwrap_or(self.bid_offset);
        }
        if let Some(n) = config.parameters.get("ask_offset").and_then(|v| v.as_f64()) {
            self.ask_offset = Decimal::try_from(n).unwrap_or(self.ask_offset);
        }
        if let Some(n) = config.parameters.get("order_size").and_then(|v| v.as_f64()) {
            self.order_size = Decimal::try_from(n).unwrap_or(self.order_size);
        }

        Ok(())
    }

    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        let mut signals = Vec::new();

        for market in ctx.active_markets() {
            // Skip low liquidity markets
            if market.liquidity < self.min_liquidity {
                continue;
            }

            let Some(yes_price) = market.yes_price() else {
                continue;
            };

            let token_id = market.token_ids.first().cloned().unwrap_or_default();
            let mid_price = self.calculate_mid_price(yes_price);

            // Calculate spread
            let implied_spread = if let Some(spread) = market.spread {
                spread
            } else {
                // Estimate spread from price proximity to extremes
                let dist_from_half = (mid_price - dec!(0.5)).abs();
                dec!(0.02) + dist_from_half * dec!(0.1) // Wider spread near extremes
            };

            // Skip if spread is too tight
            if implied_spread < self.min_spread {
                continue;
            }

            // Calculate bid and ask prices
            let bid_price = mid_price - self.bid_offset;
            let ask_price = mid_price + self.ask_offset;

            // Validate prices are in valid range
            if bid_price <= Decimal::ZERO || ask_price >= Decimal::ONE {
                continue;
            }

            // Check inventory
            let inventory = self.get_inventory(&market.condition_id);

            // Skip if at max inventory
            if inventory.abs() >= self.max_inventory_imbalance {
                continue;
            }

            // Generate bid signal (buy order below mid)
            let bid_size =
                self.adjust_size_for_inventory(self.order_size, inventory, OrderSide::Buy);
            if bid_size > dec!(0.1) {
                let signal = Signal::buy(market.condition_id.clone(), token_id.clone(), bid_size)
                    .with_strategy(self.name())
                    .with_type(SignalType::Entry)
                    .with_strength(SignalStrength::Weak)
                    .with_price(bid_price)
                    .with_ttl(300) // 5 minute TTL for limit orders
                    .with_reason(format!(
                        "Spread bid: {:.4} (mid: {:.4}, spread: {:.2}%)",
                        bid_price,
                        mid_price,
                        implied_spread * dec!(100)
                    ));

                signals.push(signal);
            }

            // Generate ask signal (sell order above mid)
            let ask_size =
                self.adjust_size_for_inventory(self.order_size, inventory, OrderSide::Sell);
            if ask_size > dec!(0.1) {
                let signal = Signal::sell(market.condition_id.clone(), token_id.clone(), ask_size)
                    .with_strategy(self.name())
                    .with_type(SignalType::Entry)
                    .with_strength(SignalStrength::Weak)
                    .with_price(ask_price)
                    .with_ttl(300)
                    .with_reason(format!(
                        "Spread ask: {:.4} (mid: {:.4}, spread: {:.2}%)",
                        ask_price,
                        mid_price,
                        implied_spread * dec!(100)
                    ));

                signals.push(signal);
            }
        }

        signals
    }

    fn on_order_filled(&mut self, _order_id: &str, filled_price: Decimal, filled_size: Decimal) {
        // This would need the market_id to properly track inventory
        // For now, this is a simplified implementation
        // In production, you'd track which market/token the order belongs to
        tracing::debug!(
            "Spread order filled: price={}, size={}",
            filled_price,
            filled_size
        );
    }

    fn on_signal_executed(&mut self, signal: &Signal, success: bool) {
        if !success {
            return;
        }

        // Update inventory tracking
        let delta = match signal.side {
            OrderSide::Buy => signal.size,
            OrderSide::Sell => -signal.size,
        };

        let inventory = self
            .inventory
            .entry(signal.market_id.clone())
            .or_insert(Decimal::ZERO);
        *inventory += delta;

        tracing::debug!(
            "Spread inventory updated for {}: {} (delta: {})",
            signal.market_id,
            inventory,
            delta
        );
    }

    fn parameters(&self) -> HashMap<String, ParameterDef> {
        let mut params = HashMap::new();

        params.insert(
            "min_spread".to_string(),
            ParameterDef {
                name: "min_spread".to_string(),
                description: "Minimum spread required to place orders".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.02),
                min: Some(ParameterValue::Float(0.005)),
                max: Some(ParameterValue::Float(0.20)),
                allowed_values: None,
            },
        );

        params.insert(
            "bid_offset".to_string(),
            ParameterDef {
                name: "bid_offset".to_string(),
                description: "Offset below mid-price for bid orders".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.01),
                min: Some(ParameterValue::Float(0.001)),
                max: Some(ParameterValue::Float(0.10)),
                allowed_values: None,
            },
        );

        params.insert(
            "ask_offset".to_string(),
            ParameterDef {
                name: "ask_offset".to_string(),
                description: "Offset above mid-price for ask orders".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.01),
                min: Some(ParameterValue::Float(0.001)),
                max: Some(ParameterValue::Float(0.10)),
                allowed_values: None,
            },
        );

        params.insert(
            "order_size".to_string(),
            ParameterDef {
                name: "order_size".to_string(),
                description: "Size per order in USDC".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(5.0),
                min: Some(ParameterValue::Float(1.0)),
                max: Some(ParameterValue::Float(100.0)),
                allowed_values: None,
            },
        );

        params.insert(
            "max_inventory_imbalance".to_string(),
            ParameterDef {
                name: "max_inventory_imbalance".to_string(),
                description: "Maximum inventory imbalance allowed".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(50.0),
                min: Some(ParameterValue::Float(10.0)),
                max: Some(ParameterValue::Float(500.0)),
                allowed_values: None,
            },
        );

        params
    }

    fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
        match name {
            "min_spread" => {
                self.min_spread = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "bid_offset" => {
                self.bid_offset = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "ask_offset" => {
                self.ask_offset = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "order_size" => {
                self.order_size = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "max_inventory_imbalance" => {
                self.max_inventory_imbalance = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            _ => return Err(crate::Error::invalid_input("Unknown parameter")),
        }
        Ok(())
    }
}
