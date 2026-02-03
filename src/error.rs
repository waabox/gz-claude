//! Error types for gz-claude.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GzClaudeError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Zellij error: {0}")]
    Zellij(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found at {0}")]
    NotFound(PathBuf),

    #[error("Failed to read configuration file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid action key '{key}': must be a single character")]
    InvalidActionKey { key: String },

    #[error("Empty command for action '{action_name}'")]
    EmptyCommand { action_name: String },

    #[error("Project path does not exist: {path}")]
    PathNotFound { path: PathBuf },

    #[error("Project path is not a directory: {path}")]
    PathNotDirectory { path: PathBuf },

    #[error("No workspaces configured")]
    NoWorkspaces,
}

pub type Result<T> = std::result::Result<T, GzClaudeError>;
