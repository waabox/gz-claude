//! Zellij CLI interaction.
//!
//! This module provides utilities for interacting with the Zellij terminal
//! multiplexer, including installation checks, session management, and
//! workspace orchestration.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code, unused_imports)]

mod check;
mod commands;
mod layout;

pub use check::{is_zellij_installed, zellij_version};
pub use commands::{open_file_in_editor, open_pane, start_zellij};
pub use layout::{generate_layout, layout_exists, layout_path, layouts_dir, LAYOUT_TEMPLATE};
