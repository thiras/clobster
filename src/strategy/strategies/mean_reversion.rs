//! Mean reversion strategy.
//!
//! Buys when price is below the moving average and sells when above.

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

/// Mean reversion strategy.
///
/// This strategy looks for markets where the current price has deviated
/// significantly from its moving average, betting that it will revert.
#[derive(Debug)]
pub struct MeanReversionStrategy {
    /// Number of periods for moving average.
    ma_periods: usize,
    /// Standard deviation threshold for entry.
    entry_threshold: Decimal,
    /// Standard deviation threshold for exit.
    exit_threshold: Decimal,
    /// Default position size.
    position_size: Decimal,
    /// Minimum liquidity required.
    min_liquidity: Decimal,
    /// Track entered positions.
    entered_markets: HashMap<String, EntryInfo>,
}

#[derive(Debug, Clone)]
struct EntryInfo {
    #[allow(dead_code)]
    entry_price: Decimal,
    #[allow(dead_code)]
    side: OrderSide,
    ma_at_entry: Decimal,
}

impl MeanReversionStrategy {
    /// Create a new mean reversion strategy with default parameters.
    pub fn new() -> Self {
        Self {
            ma_periods: 20,
            entry_threshold: dec!(0.10), // 10% deviation
            exit_threshold: dec!(0.02),  // 2% back to mean
            position_size: dec!(10),
            min_liquidity: dec!(1000),
            entered_markets: HashMap::new(),
        }
    }

    /// Set the moving average period.
    pub fn with_ma_periods(mut self, periods: usize) -> Self {
        self.ma_periods = periods;
        self
    }

    /// Set the entry threshold (deviation from MA).
    pub fn with_entry_threshold(mut self, threshold: Decimal) -> Self {
        self.entry_threshold = threshold;
        self
    }

    /// Set the exit threshold.
    pub fn with_exit_threshold(mut self, threshold: Decimal) -> Self {
        self.exit_threshold = threshold;
        self
    }

    /// Set the position size.
    pub fn with_position_size(mut self, size: Decimal) -> Self {
        self.position_size = size;
        self
    }

    fn calculate_deviation(&self, current: Decimal, ma: Decimal) -> Decimal {
        if ma.is_zero() {
            Decimal::ZERO
        } else {
            (current - ma) / ma
        }
    }
}

impl Default for MeanReversionStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Strategy for MeanReversionStrategy {
    fn name(&self) -> &str {
        "mean_reversion"
    }

    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: "Mean Reversion".to_string(),
            description:
                "Trades price deviations from moving average, betting on reversion to mean"
                    .to_string(),
            version: "1.0.0".to_string(),
            author: Some("Clobster".to_string()),
            tags: vec!["mean-reversion".to_string(), "statistical".to_string()],
        }
    }

    async fn initialize(&mut self, config: &StrategyConfig) -> Result<()> {
        // Load parameters from config
        if let Some(n) = config.parameters.get("ma_periods").and_then(|v| v.as_u64()) {
            self.ma_periods = n as usize;
        }
        if let Some(n) = config
            .parameters
            .get("entry_threshold")
            .and_then(|v| v.as_f64())
        {
            self.entry_threshold = Decimal::try_from(n).unwrap_or(self.entry_threshold);
        }
        if let Some(n) = config
            .parameters
            .get("exit_threshold")
            .and_then(|v| v.as_f64())
        {
            self.exit_threshold = Decimal::try_from(n).unwrap_or(self.exit_threshold);
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
            .get("min_liquidity")
            .and_then(|v| v.as_f64())
        {
            self.min_liquidity = Decimal::try_from(n).unwrap_or(self.min_liquidity);
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

            // Get the Yes token price (first outcome)
            let Some(current_price) = market.yes_price() else {
                continue;
            };

            // Calculate moving average
            let Some(ma) = ctx.sma(&market.condition_id, self.ma_periods) else {
                continue;
            };

            let deviation = self.calculate_deviation(current_price, ma);
            let token_id = market.token_ids.first().cloned().unwrap_or_default();

            // Check if we have an existing position
            if let Some(entry) = self.entered_markets.get(&market.condition_id) {
                // Check for exit condition
                let exit_deviation = self.calculate_deviation(current_price, entry.ma_at_entry);

                if exit_deviation.abs() < self.exit_threshold {
                    // Price reverted to mean - exit with opposite side of entry
                    let exit_side = match entry.side {
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
                    .with_type(SignalType::Exit)
                    .with_strength(SignalStrength::Medium)
                    .with_price(current_price)
                    .with_reason(format!(
                        "Mean reversion exit: deviation {:.2}% (threshold {:.2}%)",
                        exit_deviation * dec!(100),
                        self.exit_threshold * dec!(100)
                    ));

                    signals.push(signal);
                }
            } else {
                // Look for entry
                if deviation.abs() > self.entry_threshold {
                    let (side, signal_reason) = if deviation < Decimal::ZERO {
                        // Price below MA - buy (expect reversion up)
                        (
                            OrderSide::Buy,
                            format!(
                                "Mean reversion entry: price {:.2}% below MA",
                                deviation.abs() * dec!(100)
                            ),
                        )
                    } else {
                        // Price above MA - sell (expect reversion down)
                        (
                            OrderSide::Sell,
                            format!(
                                "Mean reversion entry: price {:.2}% above MA",
                                deviation * dec!(100)
                            ),
                        )
                    };

                    let strength = if deviation.abs() > self.entry_threshold * dec!(2) {
                        SignalStrength::Strong
                    } else {
                        SignalStrength::Medium
                    };

                    let mut signal = if side == OrderSide::Buy {
                        Signal::buy(
                            market.condition_id.clone(),
                            token_id.clone(),
                            self.position_size,
                        )
                    } else {
                        Signal::sell(
                            market.condition_id.clone(),
                            token_id.clone(),
                            self.position_size,
                        )
                    };

                    signal = signal
                        .with_strategy(self.name())
                        .with_type(SignalType::Entry)
                        .with_strength(strength)
                        .with_price(current_price)
                        .with_reason(signal_reason);

                    // Store MA at entry in indicators for later retrieval
                    signal.metadata.indicators.insert(
                        "ma_at_entry".to_string(),
                        ma.to_string().parse().unwrap_or(0.0),
                    );

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
                // Extract MA from signal indicators, fallback to entry price
                let ma_at_entry = signal
                    .metadata
                    .indicators
                    .get("ma_at_entry")
                    .and_then(|v| Decimal::try_from(*v).ok())
                    .unwrap_or_else(|| signal.price.unwrap_or(Decimal::ZERO));

                self.entered_markets.insert(
                    signal.market_id.clone(),
                    EntryInfo {
                        entry_price: signal.price.unwrap_or(Decimal::ZERO),
                        side: signal.side,
                        ma_at_entry,
                    },
                );
            }
            SignalType::Exit => {
                self.entered_markets.remove(&signal.market_id);
            }
            _ => {}
        }
    }

    fn parameters(&self) -> HashMap<String, ParameterDef> {
        let mut params = HashMap::new();

        params.insert(
            "ma_periods".to_string(),
            ParameterDef {
                name: "ma_periods".to_string(),
                description: "Number of periods for moving average".to_string(),
                param_type: ParameterType::Integer,
                default: ParameterValue::Integer(20),
                min: Some(ParameterValue::Integer(5)),
                max: Some(ParameterValue::Integer(100)),
                allowed_values: None,
            },
        );

        params.insert(
            "entry_threshold".to_string(),
            ParameterDef {
                name: "entry_threshold".to_string(),
                description: "Deviation from MA required for entry (as decimal)".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.10),
                min: Some(ParameterValue::Float(0.01)),
                max: Some(ParameterValue::Float(0.50)),
                allowed_values: None,
            },
        );

        params.insert(
            "exit_threshold".to_string(),
            ParameterDef {
                name: "exit_threshold".to_string(),
                description: "Deviation from MA for exit (as decimal)".to_string(),
                param_type: ParameterType::Float,
                default: ParameterValue::Float(0.02),
                min: Some(ParameterValue::Float(0.005)),
                max: Some(ParameterValue::Float(0.10)),
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
            "min_liquidity".to_string(),
            ParameterDef {
                name: "min_liquidity".to_string(),
                description: "Minimum market liquidity required for trading".to_string(),
                param_type: ParameterType::Decimal,
                default: ParameterValue::Decimal(Decimal::from(1000)),
                min: Some(ParameterValue::Decimal(Decimal::from(0))),
                max: Some(ParameterValue::Decimal(Decimal::from(1000000))),
                allowed_values: None,
            },
        );

        params
    }

    fn set_parameter(&mut self, name: &str, value: ParameterValue) -> Result<()> {
        match name {
            "ma_periods" => {
                self.ma_periods = value
                    .as_i64()
                    .ok_or_else(|| crate::Error::invalid_input("Expected integer"))?
                    as usize;
            }
            "entry_threshold" => {
                self.entry_threshold = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "exit_threshold" => {
                self.exit_threshold = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "position_size" => {
                self.position_size = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            "min_liquidity" => {
                self.min_liquidity = value
                    .as_decimal()
                    .ok_or_else(|| crate::Error::invalid_input("Expected decimal"))?;
            }
            _ => return Err(crate::Error::invalid_input("Unknown parameter")),
        }
        Ok(())
    }
}
