//! Main application module.
//!
//! This module contains the main `App` struct that coordinates
//! the event loop, state management, and rendering.

use crate::api::ApiClient;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::events::EventHandler;
use crate::state::{Action, Store};
use crate::ui::Ui;

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io::{self, Stdout};
use tokio::sync::mpsc;

/// The main application.
pub struct App {
    /// Terminal.
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Application store.
    store: Store,
    /// Event handler.
    event_handler: EventHandler,
    /// Action receiver.
    action_rx: mpsc::UnboundedReceiver<Action>,
    /// API client.
    api_client: Option<ApiClient>,
    /// Configuration.
    #[allow(dead_code)]
    config: Config,
}

impl App {
    /// Create a new application.
    pub async fn new(config: Config) -> Result<Self> {
        // Set up terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Create action channel
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        // Create store
        let store = Store::new(action_tx.clone());

        // Create event handler
        let event_handler = EventHandler::new(action_tx);

        // Try to create API client
        let api_client = match ApiClient::new(config.api.clone(), None).await {
            Ok(client) => Some(client),
            Err(e) => {
                tracing::warn!("Failed to create API client: {}", e);
                None
            }
        };

        Ok(Self {
            terminal,
            store,
            event_handler,
            action_rx,
            api_client,
            config,
        })
    }

    /// Run the application event loop.
    pub async fn run(&mut self) -> Result<()> {
        // Initial connection test
        if let Some(client) = &self.api_client {
            match client.test_connection().await {
                Ok(true) => {
                    self.store.reduce(Action::SetConnected(true));
                    // Load initial data
                    self.store.dispatch(Action::RefreshMarkets)?;
                }
                Ok(false) | Err(_) => {
                    self.store.reduce(Action::SetConnected(false));
                }
            }
        }

        // Main event loop
        loop {
            // Update event handler with current state
            self.event_handler.update_store_snapshot(&self.store);

            // Render UI
            self.terminal.draw(|frame| {
                Ui::render(frame, &self.store);
            })?;

            // Handle events and actions
            tokio::select! {
                // Handle terminal events
                result = self.event_handler.next() => {
                    if let Some(action) = result? {
                        self.handle_action(action).await?;
                    }
                }

                // Handle actions from the channel
                Some(action) = self.action_rx.recv() => {
                    self.handle_action(action).await?;
                }
            }

            // Check if we should quit
            if self.store.app.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle an action.
    async fn handle_action(&mut self, action: Action) -> Result<()> {
        match &action {
            Action::RefreshAll => {
                self.refresh_all().await?;
            }
            Action::RefreshMarkets | Action::LoadMarkets => {
                self.refresh_markets().await?;
            }
            Action::RefreshOrders | Action::LoadOrders => {
                self.refresh_orders().await?;
            }
            Action::RefreshPortfolio | Action::LoadPortfolio => {
                self.refresh_portfolio().await?;
            }
            Action::RefreshOrderBook(token_id) | Action::LoadOrderBook(token_id) => {
                self.refresh_orderbook(token_id).await?;
            }
            _ => {
                // Let the store handle the action
                self.store.reduce(action);
            }
        }

        Ok(())
    }

    /// Refresh all data.
    async fn refresh_all(&mut self) -> Result<()> {
        self.store.reduce(Action::SetLoading(true));

        // Refresh in parallel
        let markets = self.fetch_markets().await;
        let orders = self.fetch_orders().await;
        let portfolio = self.fetch_portfolio().await;

        if let Ok(markets) = markets {
            self.store.reduce(Action::MarketsLoaded(markets));
        }
        if let Ok(orders) = orders {
            self.store.reduce(Action::OrdersLoaded(orders));
        }
        if let Ok(portfolio) = portfolio {
            self.store.reduce(Action::PortfolioLoaded(portfolio));
        }

        self.store.reduce(Action::SetLoading(false));
        Ok(())
    }

    /// Refresh markets.
    async fn refresh_markets(&mut self) -> Result<()> {
        self.store.reduce(Action::LoadMarkets);

        match self.fetch_markets().await {
            Ok(markets) => {
                self.store.reduce(Action::MarketsLoaded(markets));
            }
            Err(e) => {
                self.store.reduce(Action::SetError(e.to_string()));
            }
        }

        Ok(())
    }

    /// Refresh orders.
    async fn refresh_orders(&mut self) -> Result<()> {
        self.store.reduce(Action::LoadOrders);

        match self.fetch_orders().await {
            Ok(orders) => {
                self.store.reduce(Action::OrdersLoaded(orders));
            }
            Err(e) => {
                self.store.reduce(Action::SetError(e.to_string()));
            }
        }

        Ok(())
    }

    /// Refresh portfolio.
    async fn refresh_portfolio(&mut self) -> Result<()> {
        self.store.reduce(Action::LoadPortfolio);

        match self.fetch_portfolio().await {
            Ok(portfolio) => {
                self.store.reduce(Action::PortfolioLoaded(portfolio));
            }
            Err(e) => {
                self.store.reduce(Action::SetError(e.to_string()));
            }
        }

        Ok(())
    }

    /// Refresh order book for a specific token.
    async fn refresh_orderbook(&mut self, token_id: &str) -> Result<()> {
        self.store
            .reduce(Action::LoadOrderBook(token_id.to_string()));

        match self.fetch_orderbook(token_id).await {
            Ok(book) => {
                self.store.reduce(Action::OrderBookLoaded(book));
            }
            Err(e) => {
                self.store.reduce(Action::SetError(e.to_string()));
            }
        }

        Ok(())
    }

    /// Fetch markets from the API.
    async fn fetch_markets(&self) -> Result<Vec<crate::state::Market>> {
        if let Some(client) = &self.api_client {
            client.fetch_markets().await
        } else {
            Err(Error::application("No API client available"))
        }
    }

    /// Fetch orders from the API.
    async fn fetch_orders(&self) -> Result<Vec<crate::state::Order>> {
        if let Some(client) = &self.api_client {
            client.fetch_orders().await
        } else {
            Ok(Vec::new()) // Return empty if not authenticated
        }
    }

    /// Fetch portfolio from the API.
    async fn fetch_portfolio(&self) -> Result<crate::state::PortfolioState> {
        if let Some(client) = &self.api_client {
            client.fetch_portfolio().await
        } else {
            Ok(crate::state::PortfolioState::default())
        }
    }

    /// Fetch order book for a token from the API.
    async fn fetch_orderbook(&self, token_id: &str) -> Result<crate::state::OrderBookDepth> {
        if let Some(client) = &self.api_client {
            client.fetch_orderbook(token_id).await
        } else {
            Err(Error::application("No API client available"))
        }
    }
}

impl Drop for App {
    fn drop(&mut self) {
        // Restore terminal state
        let _ = disable_raw_mode();
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
