//! Configuration management for Clobster.

mod settings;

pub use settings::{ApiConfig, Config, KeyBindings, ThemeConfig, UiConfig};

use crate::error::{Error, Result};
use directories::ProjectDirs;
use std::path::PathBuf;

/// Get the configuration directory path.
pub fn config_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "clobster", "clobster")
        .map(|dirs| dirs.config_dir().to_path_buf())
        .ok_or_else(|| Error::config("Could not determine config directory"))
}

/// Get the data directory path.
pub fn data_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "clobster", "clobster")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .ok_or_else(|| Error::config("Could not determine data directory"))
}

/// Get the log directory path.
pub fn log_dir() -> Result<PathBuf> {
    ProjectDirs::from("com", "clobster", "clobster")
        .map(|dirs| dirs.data_dir().join("logs"))
        .ok_or_else(|| Error::config("Could not determine log directory"))
}
