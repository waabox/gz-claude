//! Application state and view management for the TUI.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Represents the current view in the TUI application.
///
/// The application supports three navigation levels:
/// - Workspaces: displays the list of available workspaces
/// - Projects: displays projects within a selected workspace
/// - FileBrowser: displays files within a selected project
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// List of workspaces.
    Workspaces,
    /// Projects within a specific workspace.
    Projects {
        /// The identifier of the selected workspace.
        workspace_id: String,
    },
    /// File browser for a specific project within a workspace.
    FileBrowser {
        /// The identifier of the workspace containing the project.
        workspace_id: String,
        /// The index of the selected project within the workspace.
        project_index: usize,
    },
}

/// Application state for the TUI.
///
/// Manages the current view, selection state, and application lifecycle.
#[derive(Debug, Clone)]
pub struct AppState {
    /// The current view being displayed.
    current_view: View,
    /// The index of the currently selected item in the list.
    selected_index: usize,
    /// Whether the application should quit.
    should_quit: bool,
    /// Set of expanded directory paths in the file browser.
    expanded_dirs: HashSet<PathBuf>,
}

impl AppState {
    /// Creates a new AppState starting at the Workspaces view.
    ///
    /// # Returns
    ///
    /// A new AppState initialized with the Workspaces view, selection at index 0,
    /// should_quit set to false, and an empty set of expanded directories.
    pub fn new() -> Self {
        Self {
            current_view: View::Workspaces,
            selected_index: 0,
            should_quit: false,
            expanded_dirs: HashSet::new(),
        }
    }

    /// Returns a reference to the current view.
    pub fn current_view(&self) -> &View {
        &self.current_view
    }

    /// Returns the currently selected index.
    pub fn selected_index(&self) -> usize {
        self.selected_index
    }

    /// Returns whether the application should quit.
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }

    /// Sets the should_quit flag to true.
    pub fn quit(&mut self) {
        self.should_quit = true;
    }

    /// Sets the selected index.
    ///
    /// # Arguments
    ///
    /// * `index` - The new selected index
    pub fn set_selected_index(&mut self, index: usize) {
        self.selected_index = index;
    }

    /// Toggles the expanded state of a directory.
    ///
    /// If the directory is currently expanded, it will be collapsed.
    /// If collapsed, it will be expanded.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the directory to toggle
    pub fn toggle_dir_expanded(&mut self, path: PathBuf) {
        if self.expanded_dirs.contains(&path) {
            self.expanded_dirs.remove(&path);
        } else {
            self.expanded_dirs.insert(path);
        }
    }

    /// Checks if a directory is currently expanded.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the directory to check
    ///
    /// # Returns
    ///
    /// True if the directory is expanded, false otherwise.
    pub fn is_dir_expanded(&self, path: &Path) -> bool {
        self.expanded_dirs.contains(path)
    }

    /// Returns a reference to the set of expanded directories.
    ///
    /// # Returns
    ///
    /// A reference to the HashSet of expanded directory paths.
    pub fn expanded_dirs(&self) -> &HashSet<PathBuf> {
        &self.expanded_dirs
    }

    /// Navigates to the Projects view for the specified workspace.
    ///
    /// Resets the selected index to 0.
    ///
    /// # Arguments
    ///
    /// * `workspace_id` - The identifier of the workspace to navigate to
    pub fn navigate_to_workspace(&mut self, workspace_id: String) {
        self.current_view = View::Projects { workspace_id };
        self.selected_index = 0;
    }

    /// Navigates to the FileBrowser view for the specified project.
    ///
    /// Requires being in the Projects view. Resets the selected index to 0.
    ///
    /// # Arguments
    ///
    /// * `project_index` - The index of the project to navigate to
    ///
    /// # Panics
    ///
    /// Panics if called when not in the Projects view.
    pub fn navigate_to_project(&mut self, project_index: usize) {
        let workspace_id = match &self.current_view {
            View::Projects { workspace_id } => workspace_id.clone(),
            _ => panic!("Cannot navigate to project from non-Projects view"),
        };
        self.current_view = View::FileBrowser {
            workspace_id,
            project_index,
        };
        self.selected_index = 0;
    }

    /// Navigates back one level in the view hierarchy.
    ///
    /// - FileBrowser -> Projects (same workspace)
    /// - Projects -> Workspaces
    /// - Workspaces -> no change
    ///
    /// Resets the selected index to 0 on navigation.
    pub fn navigate_back(&mut self) {
        self.current_view = match &self.current_view {
            View::Workspaces => View::Workspaces,
            View::Projects { .. } => View::Workspaces,
            View::FileBrowser { workspace_id, .. } => View::Projects {
                workspace_id: workspace_id.clone(),
            },
        };
        self.selected_index = 0;
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_creating_app_state_should_start_at_workspaces_view() {
        let app_state = AppState::new();

        assert_eq!(*app_state.current_view(), View::Workspaces);
        assert_eq!(app_state.selected_index(), 0);
        assert!(!app_state.should_quit());
    }

    #[test]
    fn when_navigating_to_workspace_should_change_view_to_projects() {
        let mut app_state = AppState::new();
        app_state.set_selected_index(2);

        app_state.navigate_to_workspace("my-workspace".to_string());

        assert_eq!(
            *app_state.current_view(),
            View::Projects {
                workspace_id: "my-workspace".to_string()
            }
        );
        assert_eq!(app_state.selected_index(), 0);
    }

    #[test]
    fn when_navigating_back_from_projects_should_return_to_workspaces() {
        let mut app_state = AppState::new();
        app_state.navigate_to_workspace("my-workspace".to_string());
        app_state.set_selected_index(3);

        app_state.navigate_back();

        assert_eq!(*app_state.current_view(), View::Workspaces);
        assert_eq!(app_state.selected_index(), 0);
    }

    #[test]
    fn when_navigating_to_project_should_change_view_to_file_browser() {
        let mut app_state = AppState::new();
        app_state.navigate_to_workspace("my-workspace".to_string());
        app_state.set_selected_index(1);

        app_state.navigate_to_project(2);

        assert_eq!(
            *app_state.current_view(),
            View::FileBrowser {
                workspace_id: "my-workspace".to_string(),
                project_index: 2
            }
        );
        assert_eq!(app_state.selected_index(), 0);
    }

    #[test]
    fn when_navigating_back_from_file_browser_should_return_to_projects() {
        let mut app_state = AppState::new();
        app_state.navigate_to_workspace("my-workspace".to_string());
        app_state.navigate_to_project(1);
        app_state.set_selected_index(5);

        app_state.navigate_back();

        assert_eq!(
            *app_state.current_view(),
            View::Projects {
                workspace_id: "my-workspace".to_string()
            }
        );
        assert_eq!(app_state.selected_index(), 0);
    }
}
