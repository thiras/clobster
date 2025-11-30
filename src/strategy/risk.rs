//! Risk management for strategies.

use super::{Signal, StrategyContext};
use crate::state::OrderSide;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Risk management guard that validates signals.
#[derive(Debug, Clone)]
pub struct RiskGuard {
    config: RiskConfig,
}

impl RiskGuard {
    /// Create a new risk guard with the given configuration.
    pub fn new(config: RiskConfig) -> Self {
        Self { config }
    }

    /// Check if a signal passes all risk rules.
    pub fn check_signal(
        &self,
        signal: &Signal,
        ctx: &StrategyContext,
    ) -> Result<(), RiskViolation> {
        // Check if trading is enabled
        if !self.config.enabled {
            return Err(RiskViolation::TradingDisabled);
        }

        // Check market whitelist/blacklist
        self.check_market_allowed(signal)?;

        // Check position size limits
        self.check_position_size(signal)?;

        // Check total exposure
        self.check_total_exposure(signal, ctx)?;

        // Check maximum positions
        self.check_max_positions(signal, ctx)?;

        // Check market-specific limits
        self.check_market_exposure(signal, ctx)?;

        // Check daily limits (not yet implemented)
        self.check_daily_limits()?;

        // Check price bounds
        self.check_price_bounds(signal)?;

        Ok(())
    }

    fn check_market_allowed(&self, signal: &Signal) -> Result<(), RiskViolation> {
        // Check blacklist first
        if self.config.blacklisted_markets.contains(&signal.market_id) {
            return Err(RiskViolation::MarketBlacklisted {
                market_id: signal.market_id.clone(),
            });
        }

        // Check whitelist (if non-empty, only whitelisted markets are allowed)
        if !self.config.whitelisted_markets.is_empty()
            && !self.config.whitelisted_markets.contains(&signal.market_id)
        {
            return Err(RiskViolation::MarketNotWhitelisted {
                market_id: signal.market_id.clone(),
            });
        }

        Ok(())
    }

    #[allow(clippy::collapsible_if)] // Intentionally avoiding let-chains for stable Rust
    fn check_position_size(&self, signal: &Signal) -> Result<(), RiskViolation> {
        if let Some(max_size) = self.config.max_position_size {
            if signal.size > max_size {
                return Err(RiskViolation::PositionSizeExceeded {
                    requested: signal.size,
                    max: max_size,
                });
            }
        }

        if let Some(min_size) = self.config.min_position_size {
            if signal.size < min_size {
                return Err(RiskViolation::PositionSizeTooSmall {
                    requested: signal.size,
                    min: min_size,
                });
            }
        }

        Ok(())
    }

    fn check_total_exposure(
        &self,
        signal: &Signal,
        ctx: &StrategyContext,
    ) -> Result<(), RiskViolation> {
        if let Some(max_exposure) = self.config.max_total_exposure {
            let current_exposure = ctx.total_exposure();
            let signal_value = signal.size * signal.price.unwrap_or(Decimal::ONE);

            // Sell signals reduce exposure, buy signals increase it
            let new_exposure = match signal.side {
                OrderSide::Buy => current_exposure + signal_value,
                OrderSide::Sell => current_exposure.saturating_sub(signal_value),
            };

            // Only check limit for buy signals that increase exposure
            if signal.side == OrderSide::Buy && new_exposure > max_exposure {
                return Err(RiskViolation::TotalExposureExceeded {
                    current: current_exposure,
                    requested: signal_value,
                    max: max_exposure,
                });
            }
        }

        Ok(())
    }

    fn check_max_positions(
        &self,
        signal: &Signal,
        ctx: &StrategyContext,
    ) -> Result<(), RiskViolation> {
        if let Some(max_positions) = self.config.max_positions {
            // Check for new positions (buy signals without existing position)
            if signal.side == OrderSide::Buy {
                let existing = ctx.get_position(&signal.token_id);
                if existing.is_none() {
                    let current_positions = ctx.positions().len();
                    if current_positions >= max_positions {
                        return Err(RiskViolation::MaxPositionsReached {
                            current: current_positions,
                            max: max_positions,
                        });
                    }
                }
            }
        }

        Ok(())
    }

    fn check_market_exposure(
        &self,
        signal: &Signal,
        ctx: &StrategyContext,
    ) -> Result<(), RiskViolation> {
        if let Some(max_per_market) = self.config.max_exposure_per_market {
            let mut market_exposure = Decimal::ZERO;

            // Sum existing exposure in this market
            for position in ctx.positions() {
                if position.market_id == signal.market_id {
                    market_exposure += position.current_value;
                }
            }

            let signal_value = signal.size * signal.price.unwrap_or(Decimal::ONE);
            let new_exposure = market_exposure + signal_value;

            if new_exposure > max_per_market {
                return Err(RiskViolation::MarketExposureExceeded {
                    market_id: signal.market_id.clone(),
                    current: market_exposure,
                    requested: signal_value,
                    max: max_per_market,
                });
            }
        }

        Ok(())
    }

    fn check_daily_limits(&self) -> Result<(), RiskViolation> {
        // NOTE: Daily limit tracking is not yet implemented.
        // This requires persistent state to track daily volume/trade count.
        // For now, this check always passes.
        Ok(())
    }

    fn check_price_bounds(&self, signal: &Signal) -> Result<(), RiskViolation> {
        if let Some(price) = signal.price {
            // Prices should be between 0 and 1 for Polymarket
            if price < Decimal::ZERO || price > Decimal::ONE {
                return Err(RiskViolation::InvalidPrice { price });
            }
        }

        Ok(())
    }

    /// Update the risk configuration.
    pub fn update_config(&mut self, config: RiskConfig) {
        self.config = config;
    }

    /// Get the current configuration.
    pub fn config(&self) -> &RiskConfig {
        &self.config
    }
}

/// Risk management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Whether risk checks are enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum size for a single position.
    pub max_position_size: Option<Decimal>,

    /// Minimum size for a position.
    pub min_position_size: Option<Decimal>,

    /// Maximum total exposure across all positions.
    pub max_total_exposure: Option<Decimal>,

    /// Maximum number of open positions.
    pub max_positions: Option<usize>,

    /// Maximum exposure per market.
    pub max_exposure_per_market: Option<Decimal>,

    /// Maximum daily trading volume.
    pub max_daily_volume: Option<Decimal>,

    /// Maximum number of trades per day.
    pub max_daily_trades: Option<usize>,

    /// Maximum loss before circuit breaker.
    pub max_daily_loss: Option<Decimal>,

    /// Required minimum balance.
    pub min_balance: Option<Decimal>,

    /// Cooldown period after loss (seconds).
    pub loss_cooldown_secs: Option<u64>,

    /// Markets that are blacklisted.
    #[serde(default)]
    pub blacklisted_markets: Vec<String>,

    /// Only trade whitelisted markets (if non-empty).
    #[serde(default)]
    pub whitelisted_markets: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for RiskConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_position_size: Some(Decimal::from(100)),
            min_position_size: Some(Decimal::from(1)),
            max_total_exposure: Some(Decimal::from(1000)),
            max_positions: Some(10),
            max_exposure_per_market: Some(Decimal::from(200)),
            max_daily_volume: None,
            max_daily_trades: None,
            max_daily_loss: None,
            min_balance: Some(Decimal::from(10)),
            loss_cooldown_secs: None,
            blacklisted_markets: vec![],
            whitelisted_markets: vec![],
        }
    }
}

/// A risk rule violation.
#[derive(Debug, Clone)]
pub enum RiskViolation {
    /// Trading is globally disabled.
    TradingDisabled,

    /// Position size exceeds maximum.
    PositionSizeExceeded { requested: Decimal, max: Decimal },

    /// Position size is below minimum.
    PositionSizeTooSmall { requested: Decimal, min: Decimal },

    /// Total portfolio exposure would exceed limit.
    TotalExposureExceeded {
        current: Decimal,
        requested: Decimal,
        max: Decimal,
    },

    /// Maximum number of positions reached.
    MaxPositionsReached { current: usize, max: usize },

    /// Exposure in this market would exceed limit.
    MarketExposureExceeded {
        market_id: String,
        current: Decimal,
        requested: Decimal,
        max: Decimal,
    },

    /// Daily volume limit exceeded.
    DailyVolumeExceeded { current: Decimal, max: Decimal },

    /// Daily trade count exceeded.
    DailyTradesExceeded { current: usize, max: usize },

    /// Daily loss limit exceeded.
    DailyLossExceeded { current: Decimal, max: Decimal },

    /// Insufficient balance.
    InsufficientBalance {
        available: Decimal,
        required: Decimal,
    },

    /// Market is blacklisted.
    MarketBlacklisted { market_id: String },

    /// Market is not whitelisted.
    MarketNotWhitelisted { market_id: String },

    /// Invalid price.
    InvalidPrice { price: Decimal },

    /// In cooldown period.
    CooldownActive { remaining_secs: u64 },
}

impl std::fmt::Display for RiskViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TradingDisabled => write!(f, "Trading is disabled"),
            Self::PositionSizeExceeded { requested, max } => {
                write!(f, "Position size {} exceeds max {}", requested, max)
            }
            Self::PositionSizeTooSmall { requested, min } => {
                write!(f, "Position size {} below min {}", requested, min)
            }
            Self::TotalExposureExceeded {
                current,
                requested,
                max,
            } => {
                write!(
                    f,
                    "Total exposure {} + {} would exceed max {}",
                    current, requested, max
                )
            }
            Self::MaxPositionsReached { current, max } => {
                write!(f, "Max positions {} reached (current: {})", max, current)
            }
            Self::MarketExposureExceeded {
                market_id,
                current,
                requested,
                max,
            } => {
                write!(
                    f,
                    "Market {} exposure {} + {} would exceed max {}",
                    market_id, current, requested, max
                )
            }
            Self::DailyVolumeExceeded { current, max } => {
                write!(f, "Daily volume {} exceeds max {}", current, max)
            }
            Self::DailyTradesExceeded { current, max } => {
                write!(f, "Daily trades {} exceeds max {}", current, max)
            }
            Self::DailyLossExceeded { current, max } => {
                write!(f, "Daily loss {} exceeds max {}", current, max)
            }
            Self::InsufficientBalance {
                available,
                required,
            } => {
                write!(
                    f,
                    "Insufficient balance: {} available, {} required",
                    available, required
                )
            }
            Self::MarketBlacklisted { market_id } => {
                write!(f, "Market {} is blacklisted", market_id)
            }
            Self::MarketNotWhitelisted { market_id } => {
                write!(f, "Market {} is not whitelisted", market_id)
            }
            Self::InvalidPrice { price } => {
                write!(f, "Invalid price: {}", price)
            }
            Self::CooldownActive { remaining_secs } => {
                write!(f, "Cooldown active: {} seconds remaining", remaining_secs)
            }
        }
    }
}

impl std::error::Error for RiskViolation {}
