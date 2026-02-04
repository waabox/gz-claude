//! Projects view component for the TUI.
//!
//! Displays a list of projects within a workspace with git information,
//! selection highlighting, and action icons.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::config::{Action, Config, Workspace};
use crate::git::{get_git_info, GitInfo};

/// View component for displaying and selecting projects within a workspace.
///
/// Renders a list of project names with git information and action icons,
/// sorted by display order with visual indication of the currently selected item.
/// The view includes a title area, scrollable list, and help text with available actions.
pub struct ProjectsView<'a> {
    config: &'a Config,
    workspace_id: &'a str,
    selected: usize,
    git_info_cache: Vec<Option<GitInfo>>,
}

impl<'a> ProjectsView<'a> {
    /// Creates a new ProjectsView with the given configuration, workspace, and selection.
    ///
    /// Loads git information for all projects in the workspace during construction.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the application configuration containing workspaces
    /// * `workspace_id` - The identifier of the workspace to display
    /// * `selected` - Index of the currently selected project
    ///
    /// # Returns
    ///
    /// A new ProjectsView instance with pre-loaded git information.
    pub fn new(config: &'a Config, workspace_id: &'a str, selected: usize) -> Self {
        let git_info_cache = Self::load_git_info(config, workspace_id);
        Self {
            config,
            workspace_id,
            selected,
            git_info_cache,
        }
    }

    /// Loads git information for all projects in the workspace.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the application configuration
    /// * `workspace_id` - The identifier of the workspace
    ///
    /// # Returns
    ///
    /// A vector of optional GitInfo for each project in the workspace.
    fn load_git_info(config: &Config, workspace_id: &str) -> Vec<Option<GitInfo>> {
        let Some(workspace) = config.workspace.get(workspace_id) else {
            return Vec::new();
        };

        workspace
            .projects
            .iter()
            .map(|project| get_git_info(&project.path, config.global.git_info_level))
            .collect()
    }

    /// Refreshes the git information cache for all projects.
    ///
    /// Call this method when the git status of projects may have changed.
    pub fn refresh_git_info(&mut self) {
        self.git_info_cache = Self::load_git_info(self.config, self.workspace_id);
    }

    /// Returns a reference to the workspace being displayed.
    ///
    /// # Returns
    ///
    /// Some reference to the workspace if it exists, None otherwise.
    pub fn workspace(&self) -> Option<&Workspace> {
        self.config.workspace.get(self.workspace_id)
    }

    /// Returns the resolved actions for the currently selected project.
    ///
    /// Actions are resolved following inheritance: global -> workspace -> project.
    /// The result is sorted alphabetically by key for consistent display.
    ///
    /// # Returns
    ///
    /// A vector of (key, Action) tuples sorted by key.
    pub fn resolved_actions(&self) -> Vec<(String, Action)> {
        let actions = self
            .config
            .resolve_actions(self.workspace_id, self.selected);
        let mut sorted: Vec<(String, Action)> = actions.into_iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        sorted
    }

    /// Returns the number of projects in the workspace.
    ///
    /// # Returns
    ///
    /// The count of projects, or 0 if the workspace doesn't exist.
    pub fn len(&self) -> usize {
        self.workspace().map(|w| w.projects.len()).unwrap_or(0)
    }

    /// Checks if there are no projects in the workspace.
    ///
    /// # Returns
    ///
    /// True if no projects exist or workspace doesn't exist, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Renders the projects view to the terminal frame.
    ///
    /// The layout consists of three areas:
    /// - Title area (3 lines): displays "{workspace.name} - Projects" header with cyan styling
    /// - List area (flexible): displays project names with git info and action icons
    /// - Help area (3 lines): displays keyboard navigation hints and action shortcuts
    ///
    /// # Arguments
    ///
    /// * `frame` - The terminal frame to render to
    /// * `area` - The rectangular area to render within
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Length(3),
            ])
            .split(area);

        self.render_title(frame, chunks[0]);
        self.render_list(frame, chunks[1]);
        self.render_help(frame, chunks[2]);
    }

    /// Renders the title area with workspace name and "Projects" header.
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title_text = self
            .workspace()
            .map(|w| format!("{} - Projects", w.name))
            .unwrap_or_else(|| "Projects".to_string());

        let title = Paragraph::new(title_text)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::BOTTOM));

        frame.render_widget(title, area);
    }

    /// Renders the list of projects with git info and action icons.
    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let Some(workspace) = self.workspace() else {
            let list = List::new(Vec::<ListItem>::new());
            frame.render_widget(list, area);
            return;
        };

        let items: Vec<ListItem> = workspace
            .projects
            .iter()
            .enumerate()
            .map(|(index, project)| {
                let git_info_text = self
                    .git_info_cache
                    .get(index)
                    .and_then(|opt| opt.as_ref())
                    .map(|info| info.format_minimal())
                    .unwrap_or_default();

                let icons = self.collect_action_icons(index);

                if index == self.selected {
                    let mut spans = vec![
                        Span::styled(
                            "> ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            &project.name,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ];

                    if !git_info_text.is_empty() {
                        spans.push(Span::styled(
                            format!("  {}", git_info_text),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }

                    if !icons.is_empty() {
                        spans.push(Span::styled(
                            format!("  {}", icons),
                            Style::default().fg(Color::Yellow),
                        ));
                    }

                    ListItem::new(Line::from(spans))
                } else {
                    let mut spans = vec![Span::raw("  "), Span::raw(&project.name)];

                    if !git_info_text.is_empty() {
                        spans.push(Span::styled(
                            format!("  {}", git_info_text),
                            Style::default().fg(Color::DarkGray),
                        ));
                    }

                    if !icons.is_empty() {
                        spans.push(Span::raw(format!("  {}", icons)));
                    }

                    ListItem::new(Line::from(spans))
                }
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, area);
    }

    /// Collects action icons for a specific project.
    fn collect_action_icons(&self, project_index: usize) -> String {
        let actions = self
            .config
            .resolve_actions(self.workspace_id, project_index);
        let mut sorted: Vec<(&String, &Action)> = actions.iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(b.0));

        sorted
            .iter()
            .filter_map(|(_, action)| action.icon.as_ref())
            .cloned()
            .collect::<Vec<String>>()
            .join(" ")
    }

    /// Renders the help area with keyboard navigation hints and action shortcuts.
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let actions = self.resolved_actions();
        let action_hints: Vec<String> = actions
            .iter()
            .map(|(key, action)| {
                let icon = action.icon.as_deref().unwrap_or("");
                format!("{}{}: {}", icon, key, action.name)
            })
            .collect();

        let help_text = format!("Enter: browse  {}  Esc: back", action_hints.join("  "));

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(help, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GlobalConfig, Project, WebClientConfig};
    use std::collections::HashMap;
    use std::path::PathBuf;

    fn create_test_config_with_projects() -> Config {
        let mut global_actions = HashMap::new();
        global_actions.insert(
            "c".to_string(),
            Action {
                name: "Claude".to_string(),
                command: "claude".to_string(),
                icon: Some("C".to_string()),
            },
        );

        let mut workspace_actions = HashMap::new();
        workspace_actions.insert(
            "t".to_string(),
            Action {
                name: "Tests".to_string(),
                command: "cargo test".to_string(),
                icon: Some("T".to_string()),
            },
        );

        let projects = vec![
            Project {
                name: "Project Alpha".to_string(),
                path: PathBuf::from("/tmp/alpha"),
                actions: HashMap::new(),
                command_bar: vec![],
            },
            Project {
                name: "Project Beta".to_string(),
                path: PathBuf::from("/tmp/beta"),
                actions: HashMap::new(),
                command_bar: vec![],
            },
            Project {
                name: "Project Gamma".to_string(),
                path: PathBuf::from("/tmp/gamma"),
                actions: HashMap::new(),
                command_bar: vec![],
            },
        ];

        let mut workspaces = HashMap::new();
        workspaces.insert(
            "fanki".to_string(),
            Workspace {
                name: "Fanki".to_string(),
                actions: workspace_actions,
                command_bar: vec![],
                projects,
            },
        );

        Config {
            global: GlobalConfig {
                editor: "$EDITOR".to_string(),
                git_info_level: Default::default(),
                actions: global_actions,
                command_bar: vec![],
            },
            web_client: WebClientConfig::default(),
            workspace: workspaces,
        }
    }

    fn create_empty_workspace_config() -> Config {
        let mut workspaces = HashMap::new();
        workspaces.insert(
            "empty".to_string(),
            Workspace {
                name: "Empty Workspace".to_string(),
                actions: HashMap::new(),
                command_bar: vec![],
                projects: vec![],
            },
        );

        Config {
            global: GlobalConfig {
                editor: "$EDITOR".to_string(),
                git_info_level: Default::default(),
                actions: HashMap::new(),
                command_bar: vec![],
            },
            web_client: WebClientConfig::default(),
            workspace: workspaces,
        }
    }

    #[test]
    fn when_creating_view_should_have_correct_project_count() {
        let config = create_test_config_with_projects();
        let view = ProjectsView::new(&config, "fanki", 0);

        let count = view.len();

        assert_eq!(count, 3);
        assert!(!view.is_empty());
    }

    #[test]
    fn when_getting_resolved_actions_should_include_global_actions() {
        let config = create_test_config_with_projects();
        let view = ProjectsView::new(&config, "fanki", 0);

        let actions = view.resolved_actions();

        let action_keys: Vec<&str> = actions.iter().map(|(k, _)| k.as_str()).collect();
        assert!(
            action_keys.contains(&"c"),
            "Should include global action 'c'"
        );
        assert!(
            action_keys.contains(&"t"),
            "Should include workspace action 't'"
        );
        assert_eq!(actions.len(), 2);
    }

    #[test]
    fn when_workspace_not_found_should_return_empty() {
        let config = create_empty_workspace_config();
        let view = ProjectsView::new(&config, "nonexistent", 0);

        assert!(view.is_empty());
        assert_eq!(view.len(), 0);
        assert!(view.workspace().is_none());
    }
}
