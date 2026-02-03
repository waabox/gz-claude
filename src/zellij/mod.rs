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
mod web;

pub use check::{is_zellij_installed, zellij_version};
pub use commands::{focus_main_pane, open_file_in_editor, open_pane, run_in_floating_pane, run_in_main_pane, start_zellij};
pub use layout::{generate_layout, layout_exists, layout_path, layouts_dir, LAYOUT_TEMPLATE};
pub use web::{clear_web_url, copy_to_clipboard, create_web_token, ensure_ssl_certs, get_local_ip, load_web_url, save_web_url, start_web_server, web_url};
