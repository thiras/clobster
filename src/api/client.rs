//! Polymarket API client wrapper.

use crate::config::ApiConfig;
use crate::error::{Error, Result};
use crate::state::{Market, Order, OrderBook, OrderRequest, PortfolioState, Position};
use polymarket_rs::types::{ConditionId, OpenOrderParams, TokenId};
use polymarket_rs::{ClobClient, TradingClient};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Builder for creating an API client.
pub struct ApiClientBuilder {
    config: ApiConfig,
    private_key: Option<String>,
}

impl ApiClientBuilder {
    /// Create a new builder with default config.
    pub fn new() -> Self {
        Self {
            config: ApiConfig::default(),
            private_key: None,
        }
    }

    /// Set the API configuration.
    pub fn config(mut self, config: ApiConfig) -> Self {
        self.config = config;
        self
    }

    /// Set the private key for authenticated operations.
    pub fn private_key(mut self, key: impl Into<String>) -> Self {
        self.private_key = Some(key.into());
        self
    }

    /// Build the API client.
    pub async fn build(self) -> Result<ApiClient> {
        ApiClient::new(self.config, self.private_key).await
    }
}

impl Default for ApiClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// High-level API client for Polymarket.
pub struct ApiClient {
    /// Configuration.
    #[allow(dead_code)]
    config: ApiConfig,
    /// CLOB client for market data.
    clob_client: ClobClient,
    /// Trading client for authenticated endpoints (optional).
    trading_client: Option<TradingClient>,
    /// Rate limiter state.
    rate_limiter: Arc<RwLock<RateLimiter>>,
}

impl ApiClient {
    /// Create a new API client.
    pub async fn new(config: ApiConfig, _private_key: Option<String>) -> Result<Self> {
        let clob_client = ClobClient::new(&config.base_url);

        // Trading client requires proper wallet setup
        // TODO: Initialize trading client with credentials when private key is provided
        let trading_client = None;

        Ok(Self {
            config,
            clob_client,
            trading_client,
            rate_limiter: Arc::new(RwLock::new(RateLimiter::new(10))),
        })
    }

    /// Check if the client is authenticated (can trade).
    pub fn is_authenticated(&self) -> bool {
        self.trading_client.is_some()
    }

    /// Test connection to the API.
    pub async fn test_connection(&self) -> Result<bool> {
        self.rate_limit().await?;
        self.clob_client.get_ok().await.map_err(Error::Api)?;
        Ok(true)
    }

    /// Fetch markets from the API.
    pub async fn fetch_markets(&self) -> Result<Vec<Market>> {
        self.rate_limit().await?;

        let response = self
            .clob_client
            .get_markets(None)
            .await
            .map_err(Error::Api)?;

        Ok(response
            .data
            .into_iter()
            .map(super::DataConverter::convert_market)
            .collect())
    }

    /// Fetch a single market by condition ID.
    pub async fn fetch_market(&self, condition_id: &str) -> Result<Market> {
        self.rate_limit().await?;

        let market = self
            .clob_client
            .get_market(&ConditionId::new(condition_id))
            .await
            .map_err(Error::Api)?;

        Ok(super::DataConverter::convert_market(market))
    }

    /// Fetch the orderbook for a token.
    pub async fn fetch_orderbook(&self, token_id: &str) -> Result<OrderBook> {
        self.rate_limit().await?;

        let summary = self
            .clob_client
            .get_order_book(&TokenId::new(token_id))
            .await
            .map_err(Error::Api)?;

        Ok(super::DataConverter::convert_orderbook(summary))
    }

    /// Fetch orderbooks for multiple tokens.
    pub async fn fetch_orderbooks(&self, token_ids: &[String]) -> Result<Vec<OrderBook>> {
        self.rate_limit().await?;

        let params: Vec<polymarket_rs::types::BookParams> = token_ids
            .iter()
            .flat_map(|id| {
                vec![
                    polymarket_rs::types::BookParams::new(id, polymarket_rs::types::Side::Buy),
                    polymarket_rs::types::BookParams::new(id, polymarket_rs::types::Side::Sell),
                ]
            })
            .collect();

        let summaries = self
            .clob_client
            .get_order_books(&params)
            .await
            .map_err(Error::Api)?;

        Ok(summaries
            .into_iter()
            .map(super::DataConverter::convert_orderbook)
            .collect())
    }

    /// Fetch the spread for a token (returns just the spread value, not bid/ask).
    pub async fn fetch_spread(&self, token_id: &str) -> Result<rust_decimal::Decimal> {
        self.rate_limit().await?;

        let spread = self
            .clob_client
            .get_spread(&TokenId::new(token_id))
            .await
            .map_err(Error::Api)?;

        Ok(spread.spread)
    }

    /// Fetch open orders (requires authentication).
    pub async fn fetch_orders(&self) -> Result<Vec<Order>> {
        let trading = self
            .trading_client
            .as_ref()
            .ok_or_else(|| Error::auth("Not authenticated"))?;

        self.rate_limit().await?;

        let response = trading
            .get_orders(OpenOrderParams::default())
            .await
            .map_err(Error::Api)?;

        Ok(response
            .data
            .into_iter()
            .map(super::DataConverter::convert_order)
            .collect())
    }

    /// Fetch positions (requires authentication).
    pub async fn fetch_positions(&self) -> Result<Vec<Position>> {
        let _trading = self
            .trading_client
            .as_ref()
            .ok_or_else(|| Error::auth("Not authenticated"))?;

        self.rate_limit().await?;

        // TODO: Implement position fetching via DataClient
        Ok(Vec::new())
    }

    /// Fetch portfolio state (requires authentication).
    pub async fn fetch_portfolio(&self) -> Result<PortfolioState> {
        let _trading = self
            .trading_client
            .as_ref()
            .ok_or_else(|| Error::auth("Not authenticated"))?;

        self.rate_limit().await?;

        // TODO: Implement portfolio fetching
        Ok(PortfolioState::default())
    }

    /// Place an order (requires authentication).
    pub async fn place_order(&self, _request: OrderRequest) -> Result<Order> {
        let _trading = self
            .trading_client
            .as_ref()
            .ok_or_else(|| Error::auth("Not authenticated"))?;

        self.rate_limit().await?;

        // TODO: Implement order placement using TradingClient::create_and_post_order
        Err(Error::application("Order placement not yet implemented"))
    }

    /// Cancel an order (requires authentication).
    pub async fn cancel_order(&self, _order_id: &str) -> Result<()> {
        let _trading = self
            .trading_client
            .as_ref()
            .ok_or_else(|| Error::auth("Not authenticated"))?;

        self.rate_limit().await?;

        // TODO: Implement order cancellation using TradingClient::cancel
        Err(Error::application("Order cancellation not yet implemented"))
    }

    /// Apply rate limiting.
    async fn rate_limit(&self) -> Result<()> {
        let mut limiter = self.rate_limiter.write().await;
        limiter.wait().await
    }
}

/// Simple rate limiter.
struct RateLimiter {
    requests_per_second: u32,
    last_request: std::time::Instant,
    tokens: f64,
}

impl RateLimiter {
    fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
            last_request: std::time::Instant::now(),
            tokens: requests_per_second as f64,
        }
    }

    async fn wait(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_request).as_secs_f64();

        // Replenish tokens
        self.tokens = (self.tokens + elapsed * self.requests_per_second as f64)
            .min(self.requests_per_second as f64);

        if self.tokens < 1.0 {
            // Need to wait
            let wait_time = (1.0 - self.tokens) / self.requests_per_second as f64;
            tokio::time::sleep(std::time::Duration::from_secs_f64(wait_time)).await;
            self.tokens = 1.0;
        }

        self.tokens -= 1.0;
        self.last_request = std::time::Instant::now();

        Ok(())
    }
}
