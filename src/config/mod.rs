//! Configuration parsing and validation.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::error::{ConfigError, Result};

const EXAMPLE_CONFIG: &str = r#"# gz-claude configuration file
# See: docs/plans/2026-02-03-gz-claude-design.md

[global]
editor = "$EDITOR"
git_info_level = "minimal"  # minimal | standard | detailed

[global.actions]
c = { name = "Claude", command = "claude", icon = "🤖" }
b = { name = "Bash", command = "bash", icon = "💻" }
g = { name = "Lazygit", command = "lazygit", icon = "󰊢" }

[web_client]
auto_start = false
bind_address = "0.0.0.0"
port = 8082

# Example workspace - customize for your projects
[workspace.example]
name = "Example Workspace"

[workspace.example.actions]
t = { name = "Tests", command = "cargo test", icon = "🧪" }

[[workspace.example.projects]]
name = "My Project"
path = "/path/to/your/project"
"#;

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

    /// Create an example configuration file at the default path.
    /// Returns the path where the file was created.
    pub fn create_example() -> Result<PathBuf> {
        let config_dir = Self::default_dir();
        fs::create_dir_all(&config_dir)?;

        let config_path = Self::default_path();
        fs::write(&config_path, EXAMPLE_CONFIG)?;

        Ok(config_path)
    }

    /// Load configuration, creating an example if it doesn't exist.
    /// Returns (Config, was_created) tuple.
    pub fn load_or_create_example() -> Result<(Self, bool)> {
        let config_path = Self::default_path();

        if !config_path.exists() {
            Self::create_example()?;
            return Err(ConfigError::NotFound(config_path).into());
        }

        let config = Self::load_from(&config_path)?;
        Ok((config, false))
    }

    /// Validate the configuration.
    ///
    /// Checks:
    /// - At least one workspace exists
    /// - All action keys are single characters
    /// - All action commands are non-empty
    /// - All project paths exist and are directories
    ///
    /// # Returns
    ///
    /// Ok(()) if the configuration is valid.
    ///
    /// # Errors
    ///
    /// - `ConfigError::NoWorkspaces` if no workspaces are defined
    /// - `ConfigError::InvalidActionKey` if an action key is not a single character
    /// - `ConfigError::EmptyCommand` if an action command is empty or whitespace
    /// - `ConfigError::PathNotFound` if a project path does not exist
    /// - `ConfigError::PathNotDirectory` if a project path is not a directory
    pub fn validate(&self) -> Result<()> {
        if self.workspace.is_empty() {
            return Err(ConfigError::NoWorkspaces.into());
        }

        // Validate global actions
        self.validate_actions(&self.global.actions)?;

        // Validate each workspace
        for workspace in self.workspace.values() {
            self.validate_actions(&workspace.actions)?;

            for project in &workspace.projects {
                self.validate_actions(&project.actions)?;
                self.validate_project_path(project)?;
            }
        }

        Ok(())
    }

    fn validate_actions(&self, actions: &HashMap<String, Action>) -> Result<()> {
        for (key, action) in actions {
            if key.chars().count() != 1 {
                return Err(ConfigError::InvalidActionKey { key: key.clone() }.into());
            }
            if action.command.trim().is_empty() {
                return Err(ConfigError::EmptyCommand {
                    action_name: action.name.clone(),
                }
                .into());
            }
        }
        Ok(())
    }

    fn validate_project_path(&self, project: &Project) -> Result<()> {
        if !project.path.exists() {
            return Err(ConfigError::PathNotFound {
                path: project.path.clone(),
            }
            .into());
        }
        if !project.path.is_dir() {
            return Err(ConfigError::PathNotDirectory {
                path: project.path.clone(),
            }
            .into());
        }
        Ok(())
    }
}
