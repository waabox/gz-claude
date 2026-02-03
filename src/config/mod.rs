//! Configuration parsing and validation.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

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
