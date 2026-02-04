//! TUI view components.
//!
//! @author waabox(waabox[at]gmail[dot]com)

pub mod command_bar;
pub mod file_browser;
pub mod projects;
pub mod workspaces;

pub use command_bar::CommandBar;
pub use file_browser::FileBrowserView;
pub use projects::ProjectsView;
pub use workspaces::WorkspacesView;
