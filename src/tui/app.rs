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
    /// Whether the command bar is currently visible.
    command_bar_visible: bool,
    /// The index of the currently selected command in the command bar.
    command_bar_selected: usize,
}

impl AppState {
    /// Creates a new AppState starting at the Workspaces view.
    ///
    /// # Returns
    ///
    /// A new AppState initialized with the Workspaces view, selection at index 0,
    /// should_quit set to false, an empty set of expanded directories, and
    /// command bar hidden.
    pub fn new() -> Self {
        Self {
            current_view: View::Workspaces,
            selected_index: 0,
            should_quit: false,
            expanded_dirs: HashSet::new(),
            command_bar_visible: false,
            command_bar_selected: 0,
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

    /// Toggles the visibility of the command bar.
    ///
    /// When showing the command bar, resets the selection to 0.
    pub fn toggle_command_bar(&mut self) {
        self.command_bar_visible = !self.command_bar_visible;
        if self.command_bar_visible {
            self.command_bar_selected = 0;
        }
    }

    /// Returns whether the command bar is currently visible.
    pub fn is_command_bar_visible(&self) -> bool {
        self.command_bar_visible
    }

    /// Hides the command bar.
    pub fn hide_command_bar(&mut self) {
        self.command_bar_visible = false;
        self.command_bar_selected = 0;
    }

    /// Returns the currently selected command bar index.
    pub fn command_bar_selected(&self) -> usize {
        self.command_bar_selected
    }

    /// Selects the next command in the command bar.
    ///
    /// Wraps around to the first command if at the end.
    ///
    /// # Arguments
    ///
    /// * `max` - The total number of commands in the command bar
    pub fn command_bar_select_next(&mut self, max: usize) {
        if max > 0 {
            self.command_bar_selected = (self.command_bar_selected + 1) % max;
        }
    }

    /// Selects the previous command in the command bar.
    ///
    /// Wraps around to the last command if at the beginning.
    ///
    /// # Arguments
    ///
    /// * `max` - The total number of commands in the command bar
    pub fn command_bar_select_prev(&mut self, max: usize) {
        if max > 0 {
            if self.command_bar_selected == 0 {
                self.command_bar_selected = max - 1;
            } else {
                self.command_bar_selected -= 1;
            }
        }
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

    #[test]
    fn when_toggling_command_bar_should_change_visibility() {
        let mut app_state = AppState::new();

        assert!(!app_state.is_command_bar_visible());

        app_state.toggle_command_bar();
        assert!(app_state.is_command_bar_visible());

        app_state.toggle_command_bar();
        assert!(!app_state.is_command_bar_visible());
    }

    #[test]
    fn when_selecting_next_command_should_wrap_around() {
        let mut app_state = AppState::new();
        app_state.toggle_command_bar();

        app_state.command_bar_select_next(3);
        assert_eq!(app_state.command_bar_selected(), 1);

        app_state.command_bar_select_next(3);
        assert_eq!(app_state.command_bar_selected(), 2);

        app_state.command_bar_select_next(3);
        assert_eq!(app_state.command_bar_selected(), 0);
    }

    #[test]
    fn when_selecting_prev_command_should_wrap_around() {
        let mut app_state = AppState::new();
        app_state.toggle_command_bar();

        app_state.command_bar_select_prev(3);
        assert_eq!(app_state.command_bar_selected(), 2);

        app_state.command_bar_select_prev(3);
        assert_eq!(app_state.command_bar_selected(), 1);

        app_state.command_bar_select_prev(3);
        assert_eq!(app_state.command_bar_selected(), 0);
    }

    #[test]
    fn when_hiding_command_bar_should_reset_selection() {
        let mut app_state = AppState::new();
        app_state.toggle_command_bar();
        app_state.command_bar_select_next(3);
        app_state.command_bar_select_next(3);
        assert_eq!(app_state.command_bar_selected(), 2);

        app_state.hide_command_bar();

        assert!(!app_state.is_command_bar_visible());
        assert_eq!(app_state.command_bar_selected(), 0);
    }
}
