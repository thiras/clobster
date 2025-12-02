//! Core strategy traits and types.

use super::{Signal, StrategyContext};
use crate::error::Result;
use async_trait::async_trait;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

/// Core trait that all trading strategies must implement.
///
/// Strategies are evaluated periodically against the current market state
/// and can generate trading signals based on their internal logic.
#[async_trait]
pub trait Strategy: Send + Sync + Debug {
    /// Returns the unique name/identifier of this strategy.
    fn name(&self) -> &str;

    /// Returns metadata about this strategy.
    fn metadata(&self) -> StrategyMetadata {
        StrategyMetadata {
            name: self.name().to_string(),
            description: String::new(),
            version: "1.0.0".to_string(),
            author: None,
            tags: vec![],
        }
    }

    /// Initialize the strategy with configuration.
    ///
    /// Called once when the strategy is loaded. Use this to set up any
    /// initial state or validate configuration.
    async fn initialize(&mut self, _config: &StrategyConfig) -> Result<()> {
        Ok(())
    }

    /// Evaluate the strategy against current market conditions.
    ///
    /// This is the core method where strategy logic lives. It receives
    /// a snapshot of current market state and returns any trading signals.
    ///
    /// # Arguments
    /// * `ctx` - Current market context with prices, positions, and orders
    ///
    /// # Returns
    /// A vector of signals (can be empty if no action needed)
    fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal>;

    /// Called when a signal from this strategy was executed.
    ///
    /// Use this to update internal state, track performance, etc.
    fn on_signal_executed(&mut self, _signal: &Signal, _success: bool) {}

    /// Called when market data is updated.
    ///
    /// Optional hook for strategies that need to respond to new market data.
    /// The provided `StrategyContext` contains a snapshot of the current market state,
    /// including prices, positions, and orders. Strategies that need to track price
    /// changes should compare the current snapshot to previous ones to detect changes
    /// and update internal indicators or state as needed.
    fn on_market_update(&mut self, _ctx: &StrategyContext) {}

    /// Called when an order from this strategy is filled.
    fn on_order_filled(&mut self, _order_id: &str, _filled_price: Decimal, _filled_size: Decimal) {}

    /// Called when an order from this strategy is cancelled.
    fn on_order_cancelled(&mut self, _order_id: &str) {}

    /// Clean up any resources when the strategy is stopped.
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }

    /// Returns the current state for persistence/debugging.
    fn state(&self) -> Option<Box<dyn Any + Send + Sync>> {
        None
    }

    /// Restore state from a previous session.
    fn restore_state(&mut self, _state: Box<dyn Any + Send + Sync>) -> Result<()> {
        Ok(())
    }

    /// Validate that the strategy is properly configured.
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Returns parameters that can be tuned for optimization.
    fn parameters(&self) -> HashMap<String, ParameterDef> {
        HashMap::new()
    }

    /// Update a parameter value.
    fn set_parameter(&mut self, _name: &str, _value: ParameterValue) -> Result<()> {
        Err(crate::Error::invalid_input("Parameter not found"))
    }
}

/// Metadata about a strategy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyMetadata {
    /// Strategy name.
    pub name: String,
    /// Human-readable description.
    pub description: String,
    /// Version string.
    pub version: String,
    /// Author/creator.
    pub author: Option<String>,
    /// Tags for categorization.
    pub tags: Vec<String>,
}

/// Configuration for a strategy instance.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StrategyConfig {
    /// Whether the strategy is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Whether to auto-execute signals or just generate them.
    #[serde(default)]
    pub auto_execute: bool,

    /// Maximum position size per market (in USDC).
    #[serde(default)]
    pub max_position_size: Option<Decimal>,

    /// Maximum total exposure across all positions.
    #[serde(default)]
    pub max_total_exposure: Option<Decimal>,

    /// Minimum time between signal executions (seconds).
    #[serde(default)]
    pub min_signal_interval_secs: u64,

    /// Markets to include (empty = all markets).
    #[serde(default)]
    pub include_markets: Vec<String>,

    /// Markets to exclude.
    #[serde(default)]
    pub exclude_markets: Vec<String>,

    /// Custom parameters for the strategy.
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
}

fn default_true() -> bool {
    true
}

/// Definition of a tunable parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDef {
    /// Parameter name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Parameter type.
    pub param_type: ParameterType,
    /// Default value.
    pub default: ParameterValue,
    /// Minimum value (for numeric types).
    pub min: Option<ParameterValue>,
    /// Maximum value (for numeric types).
    pub max: Option<ParameterValue>,
    /// Allowed values (for enum types).
    pub allowed_values: Option<Vec<ParameterValue>>,
}

/// Types of strategy parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParameterType {
    Integer,
    Float,
    Decimal,
    Boolean,
    String,
    Enum,
}

/// A parameter value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParameterValue {
    Integer(i64),
    Float(f64),
    Decimal(Decimal),
    Boolean(bool),
    String(String),
}

impl ParameterValue {
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(v) => Some(*v),
            Self::Float(v) => Some(*v as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(v) => Some(*v),
            Self::Integer(v) => Some(*v as f64),
            Self::Decimal(v) => v.to_string().parse().ok(),
            _ => None,
        }
    }

    pub fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Self::Decimal(v) => Some(*v),
            Self::Float(v) => Decimal::try_from(*v).ok(),
            Self::Integer(v) => Some(Decimal::from(*v)),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Boolean(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(v) => Some(v),
            _ => None,
        }
    }
}
