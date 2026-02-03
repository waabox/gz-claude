//! Configuration parsing and validation.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{ConfigError, Result};

/// Root configuration structure.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    #[serde(default)]
    pub web_client: WebClientConfig,
    #[serde(default)]
    pub workspace: HashMap<String, Workspace>,
}

/// Global settings that apply to all workspaces.
#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_editor")]
    pub editor: String,
    #[serde(default)]
    pub git_info_level: GitInfoLevel,
    #[serde(default)]
    pub actions: HashMap<String, Action>,
}

fn default_editor() -> String {
    "$EDITOR".to_string()
}

/// Git information detail level.
#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GitInfoLevel {
    #[default]
    Minimal,
    Standard,
    Detailed,
}

/// Web client configuration.
#[derive(Debug, Deserialize)]
pub struct WebClientConfig {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8082
}

impl Default for WebClientConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            bind_address: default_bind_address(),
            port: default_port(),
        }
    }
}

/// A workspace containing multiple projects.
#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub name: String,
    #[serde(default)]
    pub actions: HashMap<String, Action>,
    #[serde(default)]
    pub projects: Vec<Project>,
}

/// A project within a workspace.
#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub actions: HashMap<String, Action>,
}

/// An action that can be triggered from the TUI.
#[derive(Debug, Deserialize, Clone)]
pub struct Action {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub icon: Option<String>,
}

impl Config {
    /// Load configuration from the default path (~/.config/gz-claude/config.toml).
    ///
    /// Reads the configuration file from the user's config directory and parses
    /// it as TOML into the Config structure.
    ///
    /// # Returns
    ///
    /// The parsed configuration or an error if the file doesn't exist or is invalid.
    ///
    /// # Errors
    ///
    /// - `ConfigError::NotFound` if the configuration file doesn't exist
    /// - `ConfigError::ReadError` if the file cannot be read
    /// - `ConfigError::ParseError` if the TOML content is invalid
    pub fn load() -> Result<Self> {
        let config_path = Self::default_path();
        Self::load_from(&config_path)
    }

    /// Load configuration from a specific path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the configuration file
    ///
    /// # Returns
    ///
    /// The parsed configuration or an error if the file doesn't exist or is invalid.
    ///
    /// # Errors
    ///
    /// - `ConfigError::NotFound` if the configuration file doesn't exist
    /// - `ConfigError::ReadError` if the file cannot be read
    /// - `ConfigError::ParseError` if the TOML content is invalid
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.clone()).into());
        }

        let content = fs::read_to_string(path)?;
        let config: Config =
            toml::from_str(&content).map_err(ConfigError::ParseError)?;
        Ok(config)
    }

    /// Returns the default configuration file path.
    ///
    /// The default path is `~/.config/gz-claude/config.toml` on Linux/macOS.
    /// Falls back to `./gz-claude/config.toml` if the config directory cannot be determined.
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gz-claude")
            .join("config.toml")
    }

    /// Returns the default configuration directory.
    ///
    /// The default directory is `~/.config/gz-claude` on Linux/macOS.
    /// Falls back to `./gz-claude` if the config directory cannot be determined.
    pub fn default_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gz-claude")
    }
}
