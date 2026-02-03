//! TUI components using ratatui.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]
#![allow(unused_imports)]

mod app;
mod file_tree;
mod runner;
mod terminal;
pub mod views;

pub use app::{AppState, View};
pub use file_tree::{FileNode, FileTree};
pub use runner::run;
pub use terminal::{init, poll_event, restore, InputEvent, Tui};
pub use views::WorkspacesView;
