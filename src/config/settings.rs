//! Configuration settings for Clobster.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration struct.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// API configuration.
    pub api: ApiConfig,
    /// UI configuration.
    pub ui: UiConfig,
    /// Key bindings.
    pub keybindings: KeyBindings,
    /// Theme configuration.
    pub theme: ThemeConfig,
}

impl Config {
    /// Load configuration from file, returning default if file doesn't exist or fails.
    pub fn load_or_default() -> crate::Result<Self> {
        Self::load(None)
    }

    /// Load configuration from file.
    pub fn load(path: Option<PathBuf>) -> crate::Result<Self> {
        let config_path = path.unwrap_or_else(|| {
            super::config_dir()
                .map(|p| p.join("config.toml"))
                .unwrap_or_else(|_| PathBuf::from("config.toml"))
        });

        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content).map_err(|e| crate::Error::config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    /// Save configuration to file.
    pub fn save(&self, path: Option<PathBuf>) -> crate::Result<()> {
        let config_path = path.unwrap_or_else(|| {
            super::config_dir()
                .map(|p| p.join("config.toml"))
                .unwrap_or_else(|_| PathBuf::from("config.toml"))
        });

        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content =
            toml::to_string_pretty(self).map_err(|e| crate::Error::config(e.to_string()))?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }
}

/// API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ApiConfig {
    /// Polymarket API base URL.
    pub base_url: String,
    /// WebSocket URL.
    pub ws_url: String,
    /// Request timeout in seconds.
    pub timeout_secs: u64,
    /// Maximum retries for failed requests.
    pub max_retries: u32,
    /// Rate limit (requests per second).
    pub rate_limit: u32,
    /// Path to credentials file (optional).
    pub credentials_path: Option<PathBuf>,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            base_url: "https://clob.polymarket.com".to_string(),
            ws_url: "wss://ws-subscriptions-clob.polymarket.com/ws".to_string(),
            timeout_secs: 30,
            max_retries: 3,
            rate_limit: 10,
            credentials_path: None,
        }
    }
}

/// UI configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// Tick rate in milliseconds for UI updates.
    pub tick_rate_ms: u64,
    /// Enable mouse support.
    pub mouse_support: bool,
    /// Enable Unicode symbols.
    pub unicode_symbols: bool,
    /// Number of markets to display per page.
    pub markets_per_page: usize,
    /// Number of orders to display per page.
    pub orders_per_page: usize,
    /// Show status bar.
    pub show_status_bar: bool,
    /// Show help bar.
    pub show_help_bar: bool,
    /// Auto-refresh interval in seconds (0 to disable).
    pub auto_refresh_secs: u64,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            tick_rate_ms: 250,
            mouse_support: true,
            unicode_symbols: true,
            markets_per_page: 20,
            orders_per_page: 15,
            show_status_bar: true,
            show_help_bar: true,
            auto_refresh_secs: 30,
        }
    }
}

/// Key bindings configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeyBindings {
    /// Quit the application.
    pub quit: String,
    /// Show help.
    pub help: String,
    /// Navigate up.
    pub up: String,
    /// Navigate down.
    pub down: String,
    /// Navigate left.
    pub left: String,
    /// Navigate right.
    pub right: String,
    /// Select/confirm.
    pub select: String,
    /// Cancel/back.
    pub back: String,
    /// Refresh data.
    pub refresh: String,
    /// Switch to markets view.
    pub markets: String,
    /// Switch to orderbook view.
    pub orderbook: String,
    /// Switch to orders view.
    pub orders: String,
    /// Switch to positions view.
    pub positions: String,
    /// Switch to portfolio view.
    pub portfolio: String,
    /// Open search.
    pub search: String,
    /// Place order.
    pub place_order: String,
    /// Cancel order.
    pub cancel_order: String,
}

impl Default for KeyBindings {
    fn default() -> Self {
        Self {
            quit: "q".to_string(),
            help: "?".to_string(),
            up: "k".to_string(),
            down: "j".to_string(),
            left: "h".to_string(),
            right: "l".to_string(),
            select: "Enter".to_string(),
            back: "Esc".to_string(),
            refresh: "r".to_string(),
            markets: "1".to_string(),
            orderbook: "2".to_string(),
            orders: "3".to_string(),
            positions: "4".to_string(),
            portfolio: "5".to_string(),
            search: "/".to_string(),
            place_order: "p".to_string(),
            cancel_order: "x".to_string(),
        }
    }
}

/// Theme configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Primary color (hex).
    pub primary: String,
    /// Secondary color (hex).
    pub secondary: String,
    /// Accent color (hex).
    pub accent: String,
    /// Success color (hex).
    pub success: String,
    /// Warning color (hex).
    pub warning: String,
    /// Error color (hex).
    pub error: String,
    /// Background color (hex).
    pub background: String,
    /// Foreground/text color (hex).
    pub foreground: String,
    /// Border color (hex).
    pub border: String,
    /// Selection/highlight color (hex).
    pub selection: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            primary: "#5c6bc0".to_string(),
            secondary: "#7986cb".to_string(),
            accent: "#ff7043".to_string(),
            success: "#66bb6a".to_string(),
            warning: "#ffa726".to_string(),
            error: "#ef5350".to_string(),
            background: "#1e1e2e".to_string(),
            foreground: "#cdd6f4".to_string(),
            border: "#45475a".to_string(),
            selection: "#585b70".to_string(),
        }
    }
}
