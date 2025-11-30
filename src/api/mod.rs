//! Polymarket API integration.
//!
//! This module provides a high-level interface to the Polymarket API,
//! handling authentication, rate limiting, and data conversion.

mod client;
mod converter;

pub use client::{ApiClient, ApiClientBuilder};
pub use converter::DataConverter;
