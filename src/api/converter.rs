//! Data conversion utilities for API responses.

use crate::state::{
    Market, MarketStatus, Order, OrderBookDepth, OrderSide, OrderStatus, OrderType, Outcome,
    PriceLevel,
};
use chrono::{DateTime, Utc};
use polymarket_rs::types::Side;
use rust_decimal::Decimal;

/// Converts API responses to internal state types.
pub struct DataConverter;

impl DataConverter {
    /// Convert a polymarket-rs market to our internal Market type.
    pub fn convert_market(market: polymarket_rs::types::Market) -> Market {
        let outcomes = market
            .tokens
            .iter()
            .map(|token| Outcome {
                token_id: token.token_id.clone(),
                name: token.outcome.clone(),
                bid: Decimal::ZERO, // Will be updated from orderbook
                ask: Decimal::ZERO,
                last_price: Decimal::ZERO,
                volume_24h: Decimal::ZERO,
                price_change_24h: Decimal::ZERO,
            })
            .collect();

        Market {
            id: market.condition_id,
            question: market.question,
            description: market.description,
            status: Self::convert_market_status(&market.active, &market.closed),
            end_date: market.end_date_iso,
            tags: market.category.map(|c| vec![c]).unwrap_or_default(),
            outcomes,
            volume: Decimal::ZERO,    // Not directly available
            liquidity: Decimal::ZERO, // Not directly available
            image_url: Some(market.icon),
            created_at: Utc::now(), // API doesn't provide this
            updated_at: Utc::now(),
        }
    }

    /// Convert a polymarket-rs order to our internal Order type.
    pub fn convert_order(order: polymarket_rs::types::OpenOrder) -> Order {
        let remaining_size = order.original_size - order.size_matched;

        Order {
            id: order.id.to_string(),
            market_id: order.market.clone(),
            market_question: String::new(), // Would need to be looked up
            token_id: order.asset_id,
            outcome_name: order.outcome,
            side: Self::convert_side(&order.side),
            order_type: Self::convert_order_type(&order.order_type),
            price: order.price,
            original_size: order.original_size,
            remaining_size,
            filled_size: order.size_matched,
            status: Self::convert_order_status(&order.status),
            created_at: DateTime::from_timestamp(order.created_at as i64, 0)
                .unwrap_or_else(Utc::now),
            updated_at: Utc::now(),
            expires_at: if order.expiration > 0 {
                DateTime::from_timestamp(order.expiration as i64, 0)
            } else {
                None
            },
        }
    }

    fn convert_market_status(active: &bool, closed: &bool) -> MarketStatus {
        if *closed {
            MarketStatus::Closed
        } else if *active {
            MarketStatus::Active
        } else {
            MarketStatus::Paused
        }
    }

    fn convert_side(side: &Side) -> OrderSide {
        match side {
            Side::Buy => OrderSide::Buy,
            Side::Sell => OrderSide::Sell,
        }
    }

    fn convert_order_type(order_type: &polymarket_rs::types::OrderType) -> OrderType {
        match order_type {
            polymarket_rs::types::OrderType::Gtc => OrderType::Limit,
            polymarket_rs::types::OrderType::Fok => OrderType::Market,
            polymarket_rs::types::OrderType::Gtd => OrderType::Limit,
        }
    }

    fn convert_order_status(status: &str) -> OrderStatus {
        match status.to_uppercase().as_str() {
            "LIVE" | "OPEN" => OrderStatus::Open,
            "MATCHED" | "FILLED" => OrderStatus::Filled,
            "CANCELLED" | "CANCELED" => OrderStatus::Cancelled,
            "EXPIRED" => OrderStatus::Expired,
            _ => OrderStatus::Pending,
        }
    }

    /// Convert a polymarket-rs order book summary to our internal OrderBookDepth type.
    pub fn convert_orderbook(book: polymarket_rs::types::OrderBookSummary) -> OrderBookDepth {
        let bids = book
            .bids
            .into_iter()
            .map(|level| PriceLevel::new(level.price, level.size))
            .collect();

        let asks = book
            .asks
            .into_iter()
            .map(|level| PriceLevel::new(level.price, level.size))
            .collect();

        // API timestamp is in milliseconds, convert to seconds for DateTime
        let timestamp =
            DateTime::from_timestamp_millis(book.timestamp as i64).unwrap_or_else(Utc::now);

        OrderBookDepth {
            market_id: book.market,
            token_id: book.asset_id,
            hash: book.hash,
            timestamp,
            bids,
            asks,
        }
    }
}
