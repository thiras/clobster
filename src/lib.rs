//! # Clobster - Polymarket TUI Framework
//!
//! A production-grade terminal user interface for interacting with Polymarket
//! prediction markets. Built with ratatui and polymarket-rs.
//!
//! ## Architecture
//!
//! The application follows a clean architecture pattern:
//!
//! - **App**: Core application state and lifecycle management
//! - **UI**: Layout and rendering logic
//! - **Components**: Reusable TUI widgets
//! - **API**: Polymarket API integration layer
//! - **State**: Centralized state management
//! - **Events**: Input handling and event processing
//! - **Config**: Configuration management

pub mod api;
pub mod app;
pub mod components;
pub mod config;
pub mod error;
pub mod events;
pub mod state;
pub mod strategy;
pub mod ui;

pub use app::App;
pub use config::Config;
pub use error::{Error, Result};
pub use strategy::{Signal, Strategy, StrategyConfig, StrategyContext, StrategyEngine};
