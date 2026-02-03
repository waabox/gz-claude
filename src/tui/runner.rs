//! Main TUI runner that orchestrates terminal initialization, event loop, and cleanup.
//!
//! This module ties together all TUI components: terminal management, application state,
//! view rendering, and input handling.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::config::Config;
use crate::error::Result;
use crate::tui::app::{AppState, View};
use crate::tui::terminal::{init, poll_event, restore, InputEvent, Tui};
use crate::tui::views::{FileBrowserView, ProjectsView, WorkspacesView};

/// Runs the TUI application with the given configuration.
///
/// Initializes the terminal, creates application state, runs the main event loop,
/// and restores the terminal on exit or error.
///
/// # Arguments
///
/// * `config` - Reference to the application configuration
///
/// # Returns
///
/// Ok(()) on successful execution, or an error if terminal operations fail.
///
/// # Errors
///
/// Returns an error if terminal initialization, event polling, or restoration fails.
pub fn run(config: &Config) -> Result<()> {
    let mut terminal = init()?;
    let mut state = AppState::new();

    let result = run_loop(&mut terminal, &mut state, config);

    restore()?;

    result
}

/// Main event loop that handles rendering and input.
///
/// Runs until `state.should_quit` is true. Each iteration:
/// 1. Draws the current view
/// 2. Polls for input events (100ms timeout)
/// 3. Handles any received input
///
/// # Arguments
///
/// * `terminal` - Mutable reference to the terminal
/// * `state` - Mutable reference to the application state
/// * `config` - Reference to the application configuration
///
/// # Returns
///
/// Ok(()) when the user quits, or an error if rendering or event polling fails.
fn run_loop(terminal: &mut Tui, state: &mut AppState, config: &Config) -> Result<()> {
    while !state.should_quit() {
        terminal.draw(|frame| {
            let area = frame.area();
            render_current_view(frame, area, state, config);
        })?;

        if let Some(event) = poll_event(100)? {
            handle_input(state, config, event);
        }
    }

    Ok(())
}

/// Renders the appropriate view based on the current application state.
///
/// Matches on the current view and creates the appropriate view component
/// to render to the frame.
///
/// # Arguments
///
/// * `frame` - The terminal frame to render to
/// * `area` - The rectangular area to render within
/// * `state` - Reference to the application state
/// * `config` - Reference to the application configuration
fn render_current_view(frame: &mut Frame, area: Rect, state: &AppState, config: &Config) {
    match state.current_view() {
        View::Workspaces => {
            let view = WorkspacesView::new(config, state.selected_index());
            view.render(frame, area);
        }
        View::Projects { workspace_id } => {
            let view = ProjectsView::new(config, workspace_id, state.selected_index());
            view.render(frame, area);
        }
        View::FileBrowser {
            workspace_id,
            project_index,
        } => {
            let view =
                FileBrowserView::new(config, workspace_id, *project_index, state.selected_index());
            view.render(frame, area);
        }
    }
}

/// Handles input events by updating the application state.
///
/// Processes navigation (up/down), selection (enter), back navigation,
/// quit requests, and other actions.
///
/// # Arguments
///
/// * `state` - Mutable reference to the application state
/// * `config` - Reference to the application configuration
/// * `event` - The input event to handle
fn handle_input(state: &mut AppState, config: &Config, event: InputEvent) {
    match event {
        InputEvent::Up => {
            let current = state.selected_index();
            if current > 0 {
                state.set_selected_index(current - 1);
            }
        }
        InputEvent::Down => {
            let current = state.selected_index();
            let max_index = get_max_index(state, config);
            if max_index > 0 && current < max_index - 1 {
                state.set_selected_index(current + 1);
            }
        }
        InputEvent::Enter => {
            handle_enter(state, config);
        }
        InputEvent::Back => {
            state.navigate_back();
        }
        InputEvent::Quit => {
            if matches!(state.current_view(), View::Workspaces) {
                state.quit();
            } else {
                state.navigate_back();
            }
        }
        InputEvent::Refresh => {
            // TODO: Implement refresh functionality (Etapa 4)
        }
        InputEvent::Action(_char) => {
            // TODO: Implement action handling (Etapa 4)
        }
    }
}

/// Returns the maximum index for the current view.
///
/// The maximum index is the count of items in the current list view,
/// used for bounds checking during navigation.
///
/// # Arguments
///
/// * `state` - Reference to the application state
/// * `config` - Reference to the application configuration
///
/// # Returns
///
/// The count of items in the current view.
fn get_max_index(state: &AppState, config: &Config) -> usize {
    match state.current_view() {
        View::Workspaces => config.workspace.len(),
        View::Projects { workspace_id } => config
            .workspace
            .get(workspace_id)
            .map(|w| w.projects.len())
            .unwrap_or(0),
        View::FileBrowser {
            workspace_id,
            project_index,
        } => {
            let view =
                FileBrowserView::new(config, workspace_id, *project_index, state.selected_index());
            view.visible_count()
        }
    }
}

/// Handles the Enter key press based on the current view.
///
/// - Workspaces view: navigates to the selected workspace's projects
/// - Projects view: navigates to the selected project's file browser
/// - FileBrowser view: TODO for file opening/directory expansion
///
/// # Arguments
///
/// * `state` - Mutable reference to the application state
/// * `config` - Reference to the application configuration
fn handle_enter(state: &mut AppState, config: &Config) {
    match state.current_view() {
        View::Workspaces => {
            let view = WorkspacesView::new(config, state.selected_index());
            let workspace_ids = view.workspace_ids();
            if let Some(workspace_id) = workspace_ids.get(state.selected_index()) {
                state.navigate_to_workspace(workspace_id.to_string());
            }
        }
        View::Projects { .. } => {
            let project_index = state.selected_index();
            state.navigate_to_project(project_index);
        }
        View::FileBrowser { .. } => {
            // TODO: Implement file opening and directory expansion (Etapa 4)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GlobalConfig, WebClientConfig, Workspace};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut workspaces = HashMap::new();
        workspaces.insert(
            "workspace-a".to_string(),
            Workspace {
                name: "Workspace A".to_string(),
                actions: HashMap::new(),
                projects: vec![],
            },
        );
        workspaces.insert(
            "workspace-b".to_string(),
            Workspace {
                name: "Workspace B".to_string(),
                actions: HashMap::new(),
                projects: vec![],
            },
        );

        Config {
            global: GlobalConfig {
                editor: "$EDITOR".to_string(),
                git_info_level: Default::default(),
                actions: HashMap::new(),
            },
            web_client: WebClientConfig::default(),
            workspace: workspaces,
        }
    }

    #[test]
    fn when_handling_up_input_should_decrease_index() {
        let config = create_test_config();
        let mut state = AppState::new();
        state.set_selected_index(2);

        handle_input(&mut state, &config, InputEvent::Up);

        assert_eq!(state.selected_index(), 1);
    }

    #[test]
    fn when_handling_up_at_zero_should_stay_at_zero() {
        let config = create_test_config();
        let mut state = AppState::new();
        state.set_selected_index(0);

        handle_input(&mut state, &config, InputEvent::Up);

        assert_eq!(state.selected_index(), 0);
    }

    #[test]
    fn when_handling_quit_at_workspaces_should_set_should_quit() {
        let config = create_test_config();
        let mut state = AppState::new();

        handle_input(&mut state, &config, InputEvent::Quit);

        assert!(state.should_quit());
    }

    #[test]
    fn when_handling_quit_at_projects_should_navigate_back() {
        let config = create_test_config();
        let mut state = AppState::new();
        state.navigate_to_workspace("workspace-a".to_string());

        handle_input(&mut state, &config, InputEvent::Quit);

        assert!(!state.should_quit());
        assert_eq!(*state.current_view(), View::Workspaces);
    }
}
