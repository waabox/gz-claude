//! Error types for gz-claude.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use thiserror::Error;

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum GzClaudeError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Zellij error: {0}")]
    Zellij(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, GzClaudeError>;
