//! Error types for the Clobster application.

use thiserror::Error;

/// The main error type for Clobster.
#[derive(Error, Debug)]
pub enum Error {
    /// IO errors (file operations, terminal, etc.)
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Terminal/TUI related errors
    #[error("Terminal error: {0}")]
    Terminal(String),

    /// Polymarket API errors
    #[error("API error: {0}")]
    Api(#[from] polymarket_rs::Error),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Channel communication errors
    #[error("Channel error: {0}")]
    Channel(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Wallet/signing errors
    #[error("Wallet error: {0}")]
    Wallet(String),

    /// Invalid input or state
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Network connectivity errors
    #[error("Network error: {0}")]
    Network(String),

    /// Rate limiting errors
    #[error("Rate limited: retry after {0} seconds")]
    RateLimited(u64),

    /// Generic application error
    #[error("{0}")]
    Application(String),
}

/// Alias for Result with our Error type.
pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Create a new terminal error.
    pub fn terminal(msg: impl Into<String>) -> Self {
        Self::Terminal(msg.into())
    }

    /// Create a new config error.
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a new channel error.
    pub fn channel(msg: impl Into<String>) -> Self {
        Self::Channel(msg.into())
    }

    /// Create a new auth error.
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth(msg.into())
    }

    /// Create a new wallet error.
    pub fn wallet(msg: impl Into<String>) -> Self {
        Self::Wallet(msg.into())
    }

    /// Create a new invalid input error.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Create a new network error.
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network(msg.into())
    }

    /// Create a new application error.
    pub fn application(msg: impl Into<String>) -> Self {
        Self::Application(msg.into())
    }

    /// Check if this error is recoverable (user can retry).
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Self::Network(_) | Self::RateLimited(_) | Self::Channel(_)
        )
    }
}
