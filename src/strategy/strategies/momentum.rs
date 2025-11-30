//! Momentum strategy.
//!
//! Follows price trends, buying when price is rising and selling when falling.

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

/// Momentum strategy.
///
/// This strategy identifies markets with strong price momentum and
/// trades in the direction of the trend.
#[derive(Debug)]
pub struct MomentumStrategy {
    /// Short-term EMA period.
    short_ema_periods: usize,
    /// Long-term EMA period.
    long_ema_periods: usize,
    /// Momentum threshold for entry.
    momentum_threshold: Decimal,
    /// Default position size.
    position_size: Decimal,
    /// Minimum volume required.
    min_volume: Decimal,
    /// Stop loss percentage.
    stop_loss_pct: Decimal,
    /// Take profit percentage.
    take_profit_pct: Decimal,
    /// Tracked positions.
    positions: HashMap<String, MomentumPosition>,
}

#[derive(Debug, Clone)]
struct MomentumPosition {
    entry_price: Decimal,
    side: OrderSide,
    stop_loss: Decimal,
    take_profit: Decimal,
}

impl MomentumStrategy {
    /// Create a new momentum strategy with default parameters.
    pub fn new() -> Self {
        Self {
            short_ema_periods: 9,
            long_ema_periods: 21,
            momentum_threshold: dec!(0.05), // 5% momentum
            position_size: dec!(10),
            min_volume: dec!(500),
            stop_loss_pct: dec!(0.10),   // 10% stop loss
            take_profit_pct: dec!(0.20), // 20% take profit
            positions: HashMap::new(),
        }
    }

    /// Set the short EMA period.
    pub fn with_short_ema(mut self, periods: usize) -> Self {
        self.short_ema_periods = periods;
        self
    }

    /// Set the long EMA period.
    pub fn with_long_ema(mut self, periods: usize) -> Self {
        self.long_ema_periods = periods;
        self
    }

    /// Set the momentum threshold.
    pub fn with_momentum_threshold(mut self, threshold: Decimal) -> Self {
        self.momentum_threshold = threshold;
        self
    }

    /// Set the position size.
    pub fn with_position_size(mut self, size: Decimal) -> Self {
        self.position_size = size;
        self
    }

    /// Set the stop loss percentage.
    pub fn with_stop_loss(mut self, pct: Decimal) -> Self {
        self.stop_loss_pct = pct;
        self
    }

    /// Set the take profit percentage.
    pub fn with_take_profit(mut self, pct: Decimal) -> Self {
        self.take_profit_pct = pct;
        self
    }

    fn calculate_momentum(&self, ctx: &StrategyContext, condition_id: &str) -> Option<Decimal> {
        let short_ema = ctx.ema(condition_id, self.short_ema_periods)?;
        let long_ema = ctx.ema(condition_id, self.long_ema_periods)?;

        if long_ema.is_zero() {
            return None;
        }

        Some((short_ema - long_ema) / long_ema)
    }

    fn check_stop_loss_take_profit(
        &self,
        position: &MomentumPosition,
        current_price: Decimal,
    ) -> Option<SignalType> {
        match position.side {
            OrderSide::Buy => {
                if current_price <= position.stop_loss {
                    Some(SignalType::StopLoss)
                } else if current_price >= position.take_profit {
                    Some(SignalType::TakeProfit)
                } else {
                    None
                }
            }
            OrderSide::Sell => {
                // For short positions, stop loss is above entry, take profit is below
                if current_price >= position.stop_loss {
                    Some(SignalType::StopLoss)
                } else if current_price <= position.take_profit {
                    Some(SignalType::TakeProfit)
                } else {
                    None
                }
            }
        }
    }
}

impl Default for MomentumStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Strategy for MomentumStrategy {
    fn name(&self) -> &str {
        "momentum"
    }

    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Momentum".to_string(),
            description: "Trend-following strategy using EMA crossovers to identify momentum"
                .to_string(),
            version: "1.0.0".to_string(),
            author: Some("Clobster".to_string()),
            tags: vec!["momentum".to_string(), "trend-following".to_string()],
        }
    }

    async fn initialize(&mut self, config: &StrategyConfig) -> Result<()> {
        if let Some(n) = config
            .parameters
            .get("short_ema_periods")
            .and_then(|v| v.as_u64())
        {
            self.short_ema_periods = n as usize;
        }
        if let Some(n) = config
            .parameters
            .get("long_ema_periods")
            .and_then(|v| v.as_u64())
        {
            self.long_ema_periods = n as usize;
        }
        if let Some(n) = config
            .parameters
            .get("momentum_threshold")
            .and_then(|v| v.as_f64())
        {
            self.momentum_threshold = Decimal::try_from(n).unwrap_or(self.momentum_threshold);
        }
        if let Some(n) = config
            .parameters
            .get("position_size")
            .and_then(|v| v.as_f64())
        {
            self.position_size = Decimal::try_from(n).unwrap_or(self.position_size);
        }
        if let Some(n) = config
            .parameters
            .get("stop_loss_pct")
            .and_then(|v| v.as_f64())
        {
            self.stop_loss_pct = Decimal::try_from(n).unwrap_or(self.stop_loss_pct);
        }
        if let Some(n) = config
            .parameters
            .get("take_profit_pct")
            .and_then(|v| v.as_f64())
        {
            self.take_profit_pct = Decimal::try_from(n).unwrap_or(self.take_profit_pct);
        }
        if let Some(n) = config.parameters.get("min_volume").and_then(|v| v.as_f64()) {
            self.min_volume = Decimal::try_from(n).unwrap_or(self.min_volume);
        }

        Ok(())
    }

    #[allow(clippy::collapsible_if)] // Intentionally avoiding let-chains for stable Rust
    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        let mut signals = Vec::new();

        for market in ctx.active_markets() {
            // Skip low volume markets
            if market.volume_24h < self.min_volume {
                continue;
            }

            let Some(current_price) = market.yes_price() else {
                continue;
            };

            let token_id = market.token_ids.first().cloned().unwrap_or_default();

            // Check existing position for stop loss / take profit
            if let Some(position) = self.positions.get(&market.condition_id) {
                if let Some(exit_type) = self.check_stop_loss_take_profit(position, current_price) {
                    let strength = match exit_type {
                        SignalType::StopLoss => SignalStrength::VeryStrong,
                        SignalType::TakeProfit => SignalStrength::Strong,
                        _ => SignalStrength::Medium,
                    };

                    let reason = match exit_type {
                        SignalType::StopLoss => format!(
                            "Stop loss triggered at {:.4} (entry: {:.4})",
                            current_price, position.entry_price
                        ),
                        SignalType::TakeProfit => format!(
                            "Take profit triggered at {:.4} (entry: {:.4})",
                            current_price, position.entry_price
                        ),
                        _ => String::new(),
                    };

                    // Exit with opposite side of entry position
                    let exit_side = match position.side {
                        OrderSide::Buy => OrderSide::Sell,
                        OrderSide::Sell => OrderSide::Buy,
                    };

                    let signal = match exit_side {
                        OrderSide::Buy => Signal::buy(
                            market.condition_id.clone(),
                            token_id.clone(),
                            self.position_size,
                        ),
                        OrderSide::Sell => Signal::sell(
                            market.condition_id.clone(),
                            token_id.clone(),
                            self.position_size,
                        ),
                    }
                    .with_strategy(self.name())
                    .with_type(exit_type)
                    .with_strength(strength)
                    .with_price(current_price)
                    .with_reason(reason);

                    signals.push(signal);
                    continue;
                }
            }

            // Calculate momentum for new entries
            let Some(momentum) = self.calculate_momentum(ctx, &market.condition_id) else {
                continue;
            };

            // Skip if we already have a position
            if self.positions.contains_key(&market.condition_id) {
                continue;
            }

            // Check for entry signals
            if momentum > self.momentum_threshold {
                // Bullish momentum - buy
                let stop_loss = current_price * (Decimal::ONE - self.stop_loss_pct);
                let take_profit = current_price * (Decimal::ONE + self.take_profit_pct);

                let strength = if momentum > self.momentum_threshold * dec!(2) {
                    SignalStrength::Strong
                } else {
                    SignalStrength::Medium
                };

                let signal = Signal::buy(
                    market.condition_id.clone(),
                    token_id.clone(),
                    self.position_size,
                )
                .with_strategy(self.name())
                .with_type(SignalType::Entry)
                .with_strength(strength)
                .with_price(current_price)
                .with_stop_loss(stop_loss)
                .with_take_profit(take_profit)
                .with_reason(format!(
                    "Bullish momentum: {:.2}% (threshold: {:.2}%)",
                    momentum * dec!(100),
                    self.momentum_threshold * dec!(100)
                ));

                signals.push(signal);
            } else if momentum < -self.momentum_threshold {
                // Bearish momentum - could short or avoid
                // For now, we'll generate a weak sell signal for existing holders
                let strength = if momentum < -self.momentum_threshold * dec!(2) {
                    SignalStrength::Strong
                } else {
                    SignalStrength::Medium
                };

                // Only signal if we have a position in this market
                if ctx.has_position_in_market(&market.condition_id) {
                    let signal = Signal::sell(
                        market.condition_id.clone(),
                        token_id.clone(),
                        self.position_size,
                    )
                    .with_strategy(self.name())
                    .with_type(SignalType::Exit)
                    .with_strength(strength)
                    .with_price(current_price)
                    .with_reason(format!("Bearish momentum: {:.2}%", momentum * dec!(100)));

                    signals.push(signal);
                }
            }
        }

        signals
    }

    fn on_signal_executed(&mut self, signal: &Signal, success: bool) {
        if !success {
            return;
        }

        match signal.signal_type {
            SignalType::Entry => {
                let entry_price = signal.price.unwrap_or(Decimal::ZERO);
                let (stop_loss, take_profit) = match signal.side {
                    OrderSide::Buy => (
                        entry_price * (Decimal::ONE - self.stop_loss_pct),
                        entry_price * (Decimal::ONE + self.take_profit_pct),
                    ),
                    OrderSide::Sell => (
                        entry_price * (Decimal::ONE + self.stop_loss_pct),
                        entry_price * (Decimal::ONE - self.take_profit_pct),
                    ),
                };

                self.positions.insert(
                    signal.market_id.clone(),
                    MomentumPosition {
                        entry_price,
                        side: signal.side,
                        stop_loss,
                        take_profit,
                    },
                );
            }
            SignalType::Exit | SignalType::StopLoss | SignalType::TakeProfit => {
                self.positions.remove(&signal.market_id);
            }
            _ => {}
        }
    }

    fn parameters(&self) -> HashMap<String, ParameterDef> {
        let mut params = HashMap::new();

        params.insert(
            "short_ema_periods".to_string(),
            ParameterDef {
                name: "short_ema_periods".to_string(),
                description: "Periods for short-term EMA".to_string(),
                param_type: ParameterType::Integer,
                default: ParameterValue::Integer(9),
                min: Some(ParameterValue::Integer(3)),
                max: Some(ParameterValue::Integer(50)),
                allowed_values: None,
            },
        );

        params.insert(
            "long_ema_periods".to_string(),
            ParameterDef {
                name: "long_ema_periods".to_string(),
                description: "Periods for long-term EMA".to_string(),
                param_type: ParameterType::Integer,
                default: ParameterValue::Integer(21),
                min: Some(ParameterValue::Integer(10)),
                max: Some(ParameterValue::Integer(100)),
                allowed_values: None,
            },
        );

        params.insert(
            "momentum_threshold".to_string(),
            ParameterDef {
                name: "momentum_threshold".to_string(),
                description: "Minimum momentum for entry signal".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.05),
                min: Some(ParameterValue::Float(0.01)),
                max: Some(ParameterValue::Float(0.30)),
                allowed_values: None,
            },
        );

        params.insert(
            "stop_loss_pct".to_string(),
            ParameterDef {
                name: "stop_loss_pct".to_string(),
                description: "Stop loss percentage".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.10),
                min: Some(ParameterValue::Float(0.02)),
                max: Some(ParameterValue::Float(0.50)),
                allowed_values: None,
            },
        );

        params.insert(
            "take_profit_pct".to_string(),
            ParameterDef {
                name: "take_profit_pct".to_string(),
                description: "Take profit percentage".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.20),
                min: Some(ParameterValue::Float(0.05)),
                max: Some(ParameterValue::Float(1.0)),
                allowed_values: None,
            },
        );

        params.insert(
            "position_size".to_string(),
            ParameterDef {
                name: "position_size".to_string(),
                description: "Default position size in USDC".to_string(),
                param_type: ParameterType::Decimal,
                default: ParameterValue::Decimal(Decimal::from(10)),
                min: Some(ParameterValue::Decimal(Decimal::from(1))),
                max: Some(ParameterValue::Decimal(Decimal::from(1000))),
                allowed_values: None,
            },
        );

        params.insert(
            "min_volume".to_string(),
            ParameterDef {
                name: "min_volume".to_string(),
                description: "Minimum market volume required for trading".to_string(),
                param_type: ParameterType::Decimal,
                default: ParameterValue::Decimal(Decimal::from(500)),
                min: Some(ParameterValue::Decimal(Decimal::from(0))),
                max: Some(ParameterValue::Decimal(Decimal::from(100000))),
                allowed_values: None,
            },
        );

        params
    }

    fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
        match name {
            "short_ema_periods" => {
                self.short_ema_periods = value
                    .as_i64()
                    .ok_or_else(|| crate::Error::invalid_input("Expected integer"))?
                    as usize;
            }
            "long_ema_periods" => {
                self.long_ema_periods = value
                    .as_i64()
                    .ok_or_else(|| crate::Error::invalid_input("Expected integer"))?
                    as usize;
            }
            "momentum_threshold" => {
                self.momentum_threshold = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "stop_loss_pct" => {
                self.stop_loss_pct = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "take_profit_pct" => {
                self.take_profit_pct = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "position_size" => {
                self.position_size = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "min_volume" => {
                self.min_volume = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            _ => return Err(crate::Error::invalid_input("Unknown parameter")),
        }
        Ok(())
    }
}
