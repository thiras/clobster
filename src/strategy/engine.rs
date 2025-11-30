//! Strategy engine - manages strategy lifecycle and execution.

use super::{RiskGuard, Signal, Strategy, StrategyConfig, StrategyContext};
use crate::error::Result;
use crate::state::{Action, OrderRequest, OrderType};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};

/// Manages multiple strategies and their execution.
pub struct StrategyEngine {
    /// Registered strategies.
    strategies: HashMap<String, StrategyHandle>,
    /// Risk management guard.
    risk_guard: RiskGuard,
    /// Action sender to dispatch trading actions.
    action_tx: mpsc::UnboundedSender<Action>,
    /// Pending signals awaiting execution.
    pending_signals: Vec<Signal>,
    /// Signal history.
    signal_history: Vec<SignalRecord>,
    /// Engine configuration.
    config: EngineConfig,
    /// Is the engine running.
    running: bool,
}

impl StrategyEngine {
    /// Create a new strategy engine.
    pub fn new(action_tx: mpsc::UnboundedSender<Action>, config: EngineConfig) -> Self {
        Self {
            strategies: HashMap::new(),
            risk_guard: RiskGuard::new(config.risk_config.clone()),
            action_tx,
            pending_signals: Vec::new(),
            signal_history: Vec::new(),
            config,
            running: false,
        }
    }

    /// Register a strategy with the engine.
    pub async fn register<S: Strategy + 'static>(
        &mut self,
        strategy: S,
        config: StrategyConfig,
    ) -> Result<()> {
        let name = strategy.name().to_string();

        if self.strategies.contains_key(&name) {
            return Err(crate::Error::invalid_input(format!(
                "Strategy '{}' already registered",
                name
            )));
        }

        let handle = StrategyHandle {
            strategy: Arc::new(RwLock::new(Box::new(strategy))),
            config,
            status: StrategyStatus::Stopped,
            last_evaluated: None,
            signals_generated: 0,
            signals_executed: 0,
            errors: 0,
        };

        // Initialize the strategy
        {
            let mut strategy = handle.strategy.write().await;
            strategy.initialize(&handle.config).await?;
        }

        info!("Registered strategy: {}", name);
        self.strategies.insert(name, handle);

        Ok(())
    }

    /// Unregister a strategy.
    pub async fn unregister(&mut self, name: &str) -> Result<()> {
        if let Some(handle) = self.strategies.remove(name) {
            let mut strategy = handle.strategy.write().await;
            strategy.shutdown().await?;
            info!("Unregistered strategy: {}", name);
        }
        Ok(())
    }

    /// Start a strategy.
    pub fn start_strategy(&mut self, name: &str) -> Result<()> {
        if let Some(handle) = self.strategies.get_mut(name) {
            handle.status = StrategyStatus::Running;
            info!("Started strategy: {}", name);
            Ok(())
        } else {
            Err(crate::Error::invalid_input(format!(
                "Strategy '{}' not found",
                name
            )))
        }
    }

    /// Stop a strategy.
    pub fn stop_strategy(&mut self, name: &str) -> Result<()> {
        if let Some(handle) = self.strategies.get_mut(name) {
            handle.status = StrategyStatus::Stopped;
            info!("Stopped strategy: {}", name);
            Ok(())
        } else {
            Err(crate::Error::invalid_input(format!(
                "Strategy '{}' not found",
                name
            )))
        }
    }

    /// Pause a strategy.
    pub fn pause_strategy(&mut self, name: &str) -> Result<()> {
        if let Some(handle) = self.strategies.get_mut(name) {
            handle.status = StrategyStatus::Paused;
            info!("Paused strategy: {}", name);
            Ok(())
        } else {
            Err(crate::Error::invalid_input(format!(
                "Strategy '{}' not found",
                name
            )))
        }
    }

    /// Start the engine (enables evaluation loop).
    pub fn start(&mut self) {
        self.running = true;
        info!("Strategy engine started");
    }

    /// Stop the engine.
    pub fn stop(&mut self) {
        self.running = false;
        info!("Strategy engine stopped");
    }

    /// Evaluate all running strategies against current context.
    pub async fn evaluate(&mut self, ctx: &StrategyContext) -> Vec<Signal> {
        if !self.running {
            return vec![];
        }

        let mut all_signals = Vec::new();

        // Collect strategy names to evaluate
        let strategies_to_evaluate: Vec<String> = self
            .strategies
            .iter()
            .filter(|(_, handle)| handle.status == StrategyStatus::Running && handle.config.enabled)
            .filter(|(_, handle)| {
                if let Some(last) = handle.last_evaluated {
                    let elapsed = Utc::now().signed_duration_since(last).num_seconds() as u64;
                    elapsed >= handle.config.min_signal_interval_secs
                } else {
                    true
                }
            })
            .map(|(name, _)| name.clone())
            .collect();

        for name in strategies_to_evaluate {
            let handle = match self.strategies.get(&name) {
                Some(h) => h,
                None => continue,
            };

            // Evaluate the strategy
            let filtered_ctx = self.filter_context(ctx, &handle.config);
            let signals_result = {
                let mut strategy = handle.strategy.write().await;
                debug!("Evaluating strategy: {}", name);
                let mut signals = strategy.evaluate(&filtered_ctx);
                for signal in &mut signals {
                    signal.strategy_name = name.clone();
                }
                Ok::<Vec<Signal>, crate::Error>(signals)
            };

            match signals_result {
                Ok(signals) => {
                    if let Some(handle) = self.strategies.get_mut(&name) {
                        handle.last_evaluated = Some(Utc::now());
                        handle.signals_generated += signals.len();
                    }
                    all_signals.extend(signals);
                }
                Err(e) => {
                    error!("Strategy '{}' evaluation error: {}", name, e);
                    if let Some(handle) = self.strategies.get_mut(&name) {
                        handle.errors += 1;
                        if handle.errors >= self.config.max_strategy_errors {
                            warn!("Strategy '{}' disabled due to too many errors", name);
                            handle.status = StrategyStatus::Error;
                        }
                    }
                }
            }
        }

        // Apply risk checks to signals
        let approved_signals = self.apply_risk_checks(all_signals, ctx);

        // Store signals for potential execution
        self.pending_signals.extend(approved_signals.clone());

        approved_signals
    }

    fn filter_context(&self, ctx: &StrategyContext, config: &StrategyConfig) -> StrategyContext {
        let mut filtered = ctx.clone();

        // Filter markets if include/exclude lists are specified
        if !config.include_markets.is_empty() {
            filtered
                .markets
                .retain(|id, _| config.include_markets.contains(id));
        }

        if !config.exclude_markets.is_empty() {
            filtered
                .markets
                .retain(|id, _| !config.exclude_markets.contains(id));
        }

        filtered
    }

    fn apply_risk_checks(&self, signals: Vec<Signal>, ctx: &StrategyContext) -> Vec<Signal> {
        let mut approved = Vec::new();

        for signal in signals {
            match self.risk_guard.check_signal(&signal, ctx) {
                Ok(()) => approved.push(signal),
                Err(violation) => {
                    warn!(
                        "Signal rejected by risk guard: {} - {:?}",
                        signal.id, violation
                    );
                }
            }
        }

        approved
    }

    /// Execute pending signals that are configured for auto-execution.
    #[allow(clippy::collapsible_if)] // Intentionally avoiding let-chains for stable Rust
    pub async fn execute_pending_signals(&mut self) -> Result<Vec<String>> {
        let mut executed = Vec::new();

        // Drain pending signals
        let signals: Vec<Signal> = self.pending_signals.drain(..).collect();

        for signal in signals {
            // Check if signal is expired
            if signal.is_expired() {
                debug!("Signal {} expired, skipping", signal.id);
                continue;
            }

            // Check if strategy is configured for auto-execution
            if let Some(handle) = self.strategies.get(&signal.strategy_name) {
                if !handle.config.auto_execute {
                    debug!("Signal {} not auto-executed (disabled)", signal.id);
                    continue;
                }
            }

            // Convert signal to order request
            let order_request = self.signal_to_order(&signal)?;

            // Dispatch order action
            self.action_tx
                .send(Action::PlaceOrder(order_request))
                .map_err(|e| crate::Error::channel(e.to_string()))?;

            // Record execution
            self.record_signal(&signal, true);
            executed.push(signal.id.clone());

            // Notify strategy and update execution count
            if let Some(handle) = self.strategies.get_mut(&signal.strategy_name) {
                handle.signals_executed += 1;
                let mut strategy = handle.strategy.write().await;
                strategy.on_signal_executed(&signal, true);
            }
        }

        Ok(executed)
    }

    fn signal_to_order(&self, signal: &Signal) -> Result<OrderRequest> {
        Ok(OrderRequest {
            market_id: signal.market_id.clone(),
            token_id: signal.token_id.clone(),
            side: signal.side,
            price: signal.price.ok_or_else(|| {
                crate::Error::invalid_input("Signal must have a price for limit order")
            })?,
            size: signal.size,
            order_type: OrderType::Limit,
        })
    }

    fn record_signal(&mut self, signal: &Signal, executed: bool) {
        self.signal_history.push(SignalRecord {
            signal: signal.clone(),
            executed,
            executed_at: if executed { Some(Utc::now()) } else { None },
            result: None,
        });

        // Trim history if too long
        if self.signal_history.len() > self.config.max_signal_history {
            self.signal_history.remove(0);
        }
    }

    /// Get all pending signals.
    pub fn pending_signals(&self) -> &[Signal] {
        &self.pending_signals
    }

    /// Get signal history.
    pub fn signal_history(&self) -> &[SignalRecord] {
        &self.signal_history
    }

    /// Get strategy handles.
    pub fn strategies(&self) -> &HashMap<String, StrategyHandle> {
        &self.strategies
    }

    /// Get a mutable reference to a strategy.
    pub fn get_strategy_mut(&mut self, name: &str) -> Option<&mut StrategyHandle> {
        self.strategies.get_mut(name)
    }

    /// Update strategy configuration.
    pub fn update_config(&mut self, name: &str, config: StrategyConfig) -> Result<()> {
        if let Some(handle) = self.strategies.get_mut(name) {
            handle.config = config;
            Ok(())
        } else {
            Err(crate::Error::invalid_input(format!(
                "Strategy '{}' not found",
                name
            )))
        }
    }

    /// Notify strategies of a market update.
    pub async fn on_market_update(&mut self, ctx: &StrategyContext) {
        for handle in self.strategies.values() {
            if handle.status == StrategyStatus::Running {
                let mut strategy = handle.strategy.write().await;
                strategy.on_market_update(ctx);
            }
        }
    }

    /// Notify strategies of an order fill.
    pub async fn on_order_filled(
        &mut self,
        strategy_name: &str,
        order_id: &str,
        filled_price: rust_decimal::Decimal,
        filled_size: rust_decimal::Decimal,
    ) {
        if let Some(handle) = self.strategies.get(strategy_name) {
            let mut strategy = handle.strategy.write().await;
            strategy.on_order_filled(order_id, filled_price, filled_size);
        }
    }

    /// Clear a specific pending signal.
    pub fn clear_signal(&mut self, signal_id: &str) {
        self.pending_signals.retain(|s| s.id != signal_id);
    }

    /// Clear all pending signals.
    pub fn clear_all_signals(&mut self) {
        self.pending_signals.clear();
    }

    /// Manually execute a specific signal.
    pub async fn execute_signal(&mut self, signal_id: &str) -> Result<()> {
        let signal = self
            .pending_signals
            .iter()
            .find(|s| s.id == signal_id)
            .cloned()
            .ok_or_else(|| crate::Error::invalid_input("Signal not found"))?;

        let order_request = self.signal_to_order(&signal)?;

        self.action_tx
            .send(Action::PlaceOrder(order_request))
            .map_err(|e| crate::Error::channel(e.to_string()))?;

        self.record_signal(&signal, true);
        self.pending_signals.retain(|s| s.id != signal_id);

        if let Some(handle) = self.strategies.get_mut(&signal.strategy_name) {
            handle.signals_executed += 1;
            let mut strategy = handle.strategy.write().await;
            strategy.on_signal_executed(&signal, true);
        }

        Ok(())
    }
}

/// Handle to a registered strategy.
pub struct StrategyHandle {
    /// The strategy instance.
    pub strategy: Arc<RwLock<Box<dyn Strategy>>>,
    /// Strategy configuration.
    pub config: StrategyConfig,
    /// Current status.
    pub status: StrategyStatus,
    /// Last evaluation timestamp.
    pub last_evaluated: Option<DateTime<Utc>>,
    /// Number of signals generated.
    pub signals_generated: usize,
    /// Number of signals executed.
    pub signals_executed: usize,
    /// Number of errors.
    pub errors: usize,
}

/// Status of a strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrategyStatus {
    /// Strategy is stopped.
    Stopped,
    /// Strategy is running and evaluating.
    Running,
    /// Strategy is paused (won't evaluate).
    Paused,
    /// Strategy encountered an error.
    Error,
}

impl std::fmt::Display for StrategyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stopped => write!(f, "⏹ Stopped"),
            Self::Running => write!(f, "▶ Running"),
            Self::Paused => write!(f, "⏸ Paused"),
            Self::Error => write!(f, "⚠ Error"),
        }
    }
}

/// Record of a signal.
#[derive(Debug, Clone)]
pub struct SignalRecord {
    /// The signal.
    pub signal: Signal,
    /// Whether it was executed.
    pub executed: bool,
    /// When it was executed.
    pub executed_at: Option<DateTime<Utc>>,
    /// Execution result.
    pub result: Option<SignalResult>,
}

/// Result of signal execution.
#[derive(Debug, Clone)]
pub enum SignalResult {
    /// Order was placed successfully.
    OrderPlaced { order_id: String },
    /// Order was filled.
    Filled {
        order_id: String,
        filled_price: rust_decimal::Decimal,
    },
    /// Order was rejected.
    Rejected { reason: String },
    /// Order was cancelled.
    Cancelled,
}

/// Configuration for the strategy engine.
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Risk configuration.
    pub risk_config: super::RiskConfig,
    /// Maximum errors before disabling a strategy.
    pub max_strategy_errors: usize,
    /// Maximum signal history to keep.
    pub max_signal_history: usize,
    /// Evaluation interval in milliseconds.
    pub evaluation_interval_ms: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            risk_config: super::RiskConfig::default(),
            max_strategy_errors: 5,
            max_signal_history: 1000,
            evaluation_interval_ms: 1000,
        }
    }
}
