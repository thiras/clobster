//! Clobster - A Terminal UI for Polymarket
//!
//! A production-grade terminal user interface for the Polymarket
//! prediction market platform, built with ratatui and polymarket-rs.

use clobster::{App, Config, Result};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "clobster=info".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .init();

    // Load configuration
    let config = Config::load_or_default()?;

    // Run the application
    let mut app = App::new(config).await?;
    app.run().await?;

    Ok(())
}
