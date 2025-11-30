//! Market-related state.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Market status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MarketStatus {
    #[default]
    Active,
    Closed,
    Resolved,
    Paused,
}

impl std::fmt::Display for MarketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "Active"),
            Self::Closed => write!(f, "Closed"),
            Self::Resolved => write!(f, "Resolved"),
            Self::Paused => write!(f, "Paused"),
        }
    }
}

/// A market on Polymarket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Market ID (condition_id).
    pub id: String,
    /// Market question/title.
    pub question: String,
    /// Market description.
    pub description: String,
    /// Market status.
    pub status: MarketStatus,
    /// End date/time.
    pub end_date: Option<DateTime<Utc>>,
    /// Category/tags.
    pub tags: Vec<String>,
    /// Outcomes.
    pub outcomes: Vec<Outcome>,
    /// Total volume traded.
    pub volume: Decimal,
    /// Total liquidity.
    pub liquidity: Decimal,
    /// Market image URL.
    pub image_url: Option<String>,
    /// Created timestamp.
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp.
    pub updated_at: DateTime<Utc>,
}

impl Market {
    /// Get the best bid price for an outcome.
    pub fn best_bid(&self, outcome_index: usize) -> Option<Decimal> {
        self.outcomes.get(outcome_index).map(|o| o.bid)
    }

    /// Get the best ask price for an outcome.
    pub fn best_ask(&self, outcome_index: usize) -> Option<Decimal> {
        self.outcomes.get(outcome_index).map(|o| o.ask)
    }

    /// Get the mid price for an outcome.
    pub fn mid_price(&self, outcome_index: usize) -> Option<Decimal> {
        self.outcomes
            .get(outcome_index)
            .map(|o| (o.bid + o.ask) / Decimal::TWO)
    }

    /// Get the spread for an outcome.
    pub fn spread(&self, outcome_index: usize) -> Option<Decimal> {
        self.outcomes.get(outcome_index).map(|o| o.ask - o.bid)
    }

    /// Check if the market is tradeable.
    pub fn is_tradeable(&self) -> bool {
        self.status == MarketStatus::Active
    }
}

/// An outcome within a market.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    /// Token ID for this outcome.
    pub token_id: String,
    /// Outcome name (e.g., "Yes", "No").
    pub name: String,
    /// Current best bid price.
    pub bid: Decimal,
    /// Current best ask price.
    pub ask: Decimal,
    /// Last traded price.
    pub last_price: Decimal,
    /// 24h volume.
    pub volume_24h: Decimal,
    /// Price change in last 24h.
    pub price_change_24h: Decimal,
}

impl Outcome {
    /// Get the mid price.
    pub fn mid_price(&self) -> Decimal {
        (self.bid + self.ask) / Decimal::TWO
    }

    /// Get the spread.
    pub fn spread(&self) -> Decimal {
        self.ask - self.bid
    }

    /// Get the spread as a percentage of the mid price.
    pub fn spread_percent(&self) -> Decimal {
        let mid = self.mid_price();
        if mid.is_zero() {
            Decimal::ZERO
        } else {
            (self.spread() / mid) * Decimal::ONE_HUNDRED
        }
    }
}

/// State for market-related data.
#[derive(Debug, Default)]
pub struct MarketState {
    /// All loaded markets.
    pub markets: Vec<Market>,
    /// Currently selected market index.
    pub selected_index: Option<usize>,
    /// Search query filter.
    pub search_query: Option<String>,
    /// Status filter.
    pub status_filter: Option<MarketStatus>,
    /// Whether markets are currently loading.
    pub loading: bool,
    /// Last update timestamp.
    pub last_updated: Option<DateTime<Utc>>,
    /// Scroll offset for display.
    pub scroll_offset: usize,
}

impl MarketState {
    /// Get the currently selected market.
    pub fn selected_market(&self) -> Option<&Market> {
        self.selected_index
            .and_then(|i| self.filtered_markets().get(i).copied())
    }

    /// Get filtered markets based on search and status filter.
    pub fn filtered_markets(&self) -> Vec<&Market> {
        self.markets
            .iter()
            .filter(|m| {
                // Apply status filter
                if let Some(status) = &self.status_filter
                    && m.status != *status
                {
                    return false;
                }

                // Apply search filter
                if let Some(query) = &self.search_query {
                    let query_lower = query.to_lowercase();
                    if !m.question.to_lowercase().contains(&query_lower)
                        && !m.description.to_lowercase().contains(&query_lower)
                        && !m
                            .tags
                            .iter()
                            .any(|t| t.to_lowercase().contains(&query_lower))
                    {
                        return false;
                    }
                }

                true
            })
            .collect()
    }

    /// Get the count of filtered markets.
    pub fn filtered_count(&self) -> usize {
        self.filtered_markets().len()
    }
}
