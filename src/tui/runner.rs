//! Main TUI runner that orchestrates terminal initialization, event loop, and cleanup.
//!
//! This module ties together all TUI components: terminal management, application state,
//! view rendering, and input handling.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use ratatui::layout::Rect;
use ratatui::Frame;
use std::cell::RefCell;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;
use crate::session::Session;
use crate::tui::app::{AppState, View};
use crate::tui::terminal::{init, poll_event, restore, InputEvent, Tui};
use crate::tui::views::{FileBrowserView, ProjectsView, WorkspacesView};

/// Thread-local session state for the TUI.
thread_local! {
    static SESSION: RefCell<Option<Session>> = RefCell::new(None);
    static MAIN_PANE_USED: RefCell<bool> = RefCell::new(false);
}

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
    // Initialize or load session
    let session = Session::load().unwrap_or_else(|| {
        let zellij_session = std::env::var("ZELLIJ_SESSION_NAME")
            .unwrap_or_else(|_| "gz-claude".to_string());
        Session::new(zellij_session)
    });

    SESSION.with(|s| {
        *s.borrow_mut() = Some(session);
    });

    let mut terminal = init()?;
    let mut state = AppState::new();

    let result = run_loop(&mut terminal, &mut state, config);

    // Save session on exit
    SESSION.with(|s| {
        if let Some(session) = s.borrow().as_ref() {
            let _ = session.save();
        }
    });

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
            let view = FileBrowserView::with_expanded(
                config,
                workspace_id,
                *project_index,
                state.selected_index(),
                state.expanded_dirs(),
            );
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
            // Views are recreated on each render, so git info refreshes automatically.
            // The 'r' key serves as a signal to the user that data has been refreshed.
        }
        InputEvent::Action(key) => {
            handle_action(state, config, key);
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
            let view = FileBrowserView::with_expanded(
                config,
                workspace_id,
                *project_index,
                state.selected_index(),
                state.expanded_dirs(),
            );
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
        View::FileBrowser {
            workspace_id,
            project_index,
        } => {
            let view = FileBrowserView::with_expanded(
                config,
                workspace_id,
                *project_index,
                state.selected_index(),
                state.expanded_dirs(),
            );

            if view.selected_is_file() {
                // Open the file in editor
                if let Some(file_path) = view.selected_path() {
                    if let Some(project) = view.project() {
                        let editor = &config.global.editor;
                        if let Err(e) =
                            crate::zellij::open_file_in_editor(&project.path, editor, &file_path)
                        {
                            eprintln!("Error opening file: {}", e);
                        }
                    }
                }
            } else if let Some(dir_path) = view.selected_path() {
                // Toggle directory expand/collapse
                state.toggle_dir_expanded(dir_path);
            }
        }
    }
}

/// Handles action key presses by executing Zellij commands.
///
/// Resolves actions based on inheritance (global -> workspace -> project),
/// finds the action matching the pressed key, and opens a new Zellij pane
/// with the configured command. Tracks panes in session state.
///
/// Actions are only available in Projects and FileBrowser views. In the
/// Workspaces view, this function returns early without action.
///
/// # Arguments
///
/// * `state` - Reference to the application state
/// * `config` - Reference to the application configuration
/// * `key` - The action key that was pressed
fn handle_action(state: &AppState, config: &Config, key: char) {
    let (workspace_id, project_index) = match state.current_view() {
        View::Projects { workspace_id } => (workspace_id.as_str(), state.selected_index()),
        View::FileBrowser {
            workspace_id,
            project_index,
        } => (workspace_id.as_str(), *project_index),
        View::Workspaces => return,
    };

    let actions = config.resolve_actions(workspace_id, project_index);
    let key_str = key.to_string();

    if let Some(action) = actions.get(&key_str) {
        if let Some(project) = config
            .workspace
            .get(workspace_id)
            .and_then(|ws| ws.projects.get(project_index))
        {
            let project_path = project.path.clone();
            let pane_name = Session::generate_pane_name(&project_path);
            let full_command = format!("{} {}", action.command, project.path.display());

            // Check if main pane is already used
            let main_used = MAIN_PANE_USED.with(|m| *m.borrow());

            if !main_used {
                // First project goes to main pane, fullscreen for web client
                if crate::zellij::run_in_main_pane(&full_command, true).is_ok() {
                    MAIN_PANE_USED.with(|m| *m.borrow_mut() = true);
                }
            } else {
                // Subsequent projects go to floating panes, fullscreen for web client
                let _ = crate::zellij::run_in_floating_pane(&pane_name, &full_command, true);
            }
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

    #[test]
    fn when_handling_action_at_workspaces_should_do_nothing() {
        let config = create_test_config();
        let state = AppState::new();

        // Verify we're at Workspaces view
        assert_eq!(*state.current_view(), View::Workspaces);

        // Call handle_action directly - should return early without panicking
        handle_action(&state, &config, 'c');

        // State should remain unchanged
        assert_eq!(*state.current_view(), View::Workspaces);
        assert_eq!(state.selected_index(), 0);
        assert!(!state.should_quit());
    }
}
