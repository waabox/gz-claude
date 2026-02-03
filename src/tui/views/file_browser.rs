//! File browser view component for the TUI.
//!
//! Displays a file tree with navigation, expand/collapse functionality,
//! git information, and action icons.
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
use std::path::PathBuf;

use crate::config::{Action, Config, Project};
use crate::git::{get_git_info, GitInfo};
use crate::tui::file_tree::FileTree;

/// View component for displaying and navigating a file tree within a project.
///
/// Renders a hierarchical file tree with expand/collapse functionality for directories,
/// git information in the title, and action icons in the help area.
pub struct FileBrowserView<'a> {
    config: &'a Config,
    workspace_id: &'a str,
    project_index: usize,
    selected: usize,
    file_tree: Option<FileTree>,
    git_info: Option<GitInfo>,
}

impl<'a> FileBrowserView<'a> {
    /// Creates a new FileBrowserView with the given configuration, workspace, project, and selection.
    ///
    /// Loads the file tree from the project path and git information during construction.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the application configuration containing workspaces
    /// * `workspace_id` - The identifier of the workspace containing the project
    /// * `project_index` - The index of the project within the workspace
    /// * `selected` - Index of the currently selected item in the file tree
    ///
    /// # Returns
    ///
    /// A new FileBrowserView instance with pre-loaded file tree and git information.
    pub fn new(
        config: &'a Config,
        workspace_id: &'a str,
        project_index: usize,
        selected: usize,
    ) -> Self {
        let project = config
            .workspace
            .get(workspace_id)
            .and_then(|w| w.projects.get(project_index));

        let file_tree = project.and_then(|p| FileTree::new(&p.path));
        let git_info = project.and_then(|p| get_git_info(&p.path, config.global.git_info_level));

        Self {
            config,
            workspace_id,
            project_index,
            selected,
            file_tree,
            git_info,
        }
    }

    /// Returns a reference to the project being displayed.
    ///
    /// # Returns
    ///
    /// Some reference to the project if it exists, None otherwise.
    pub fn project(&self) -> Option<&Project> {
        self.config
            .workspace
            .get(self.workspace_id)
            .and_then(|w| w.projects.get(self.project_index))
    }

    /// Returns the resolved actions for the current project.
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
            .resolve_actions(self.workspace_id, self.project_index);
        let mut sorted: Vec<(String, Action)> = actions.into_iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));
        sorted
    }

    /// Returns the number of currently visible nodes in the file tree.
    ///
    /// # Returns
    ///
    /// The count of visible nodes, or 0 if no file tree is loaded.
    pub fn visible_count(&self) -> usize {
        self.file_tree
            .as_ref()
            .map(|ft| ft.visible_count())
            .unwrap_or(0)
    }

    /// Toggles the expand/collapse state of the selected item.
    ///
    /// If the selected item is a directory, it will be expanded or collapsed.
    /// Does nothing if the selected item is a file or no file tree is loaded.
    pub fn toggle_selected(&mut self) {
        if let Some(ref mut file_tree) = self.file_tree {
            file_tree.toggle_at(self.selected);
        }
    }

    /// Checks if the currently selected item is a file (not a directory).
    ///
    /// # Returns
    ///
    /// True if the selected item is a file, false if it's a directory or no file tree is loaded.
    pub fn selected_is_file(&self) -> bool {
        self.file_tree
            .as_ref()
            .and_then(|ft| ft.get_visible_node(self.selected))
            .map(|node| !node.is_dir)
            .unwrap_or(false)
    }

    /// Returns the path of the currently selected item.
    ///
    /// # Returns
    ///
    /// Some path if a file tree is loaded and selection is valid, None otherwise.
    pub fn selected_path(&self) -> Option<PathBuf> {
        self.file_tree
            .as_ref()
            .and_then(|ft| ft.get_visible_node(self.selected))
            .map(|node| node.path.clone())
    }

    /// Refreshes the git information for the current project.
    ///
    /// Call this method when the git status of the project may have changed.
    pub fn refresh_git_info(&mut self) {
        self.git_info = self
            .project()
            .and_then(|p| get_git_info(&p.path, self.config.global.git_info_level));
    }

    /// Renders the file browser view to the terminal frame.
    ///
    /// The layout consists of three areas:
    /// - Title area (3 lines): displays "{project.name}  {git_info}" header
    /// - File tree area (flexible): displays file tree with indentation and icons
    /// - Help area (3 lines): displays action shortcuts and navigation hints
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
        self.render_file_tree(frame, chunks[1]);
        self.render_help(frame, chunks[2]);
    }

    /// Renders the title area with project name and git info.
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let project_name = self
            .project()
            .map(|p| p.name.as_str())
            .unwrap_or("Unknown Project");

        let git_info_text = self
            .git_info
            .as_ref()
            .map(|info| info.format_standard())
            .unwrap_or_default();

        let mut spans = vec![Span::styled(
            project_name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )];

        if !git_info_text.is_empty() {
            spans.push(Span::styled(
                format!("  {}", git_info_text),
                Style::default().fg(Color::DarkGray),
            ));
        }

        let title =
            Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::BOTTOM));

        frame.render_widget(title, area);
    }

    /// Renders the file tree list with indentation and expand/collapse icons.
    fn render_file_tree(&self, frame: &mut Frame, area: Rect) {
        let Some(ref file_tree) = self.file_tree else {
            let list = List::new(Vec::<ListItem>::new());
            frame.render_widget(list, area);
            return;
        };

        let items: Vec<ListItem> = (0..file_tree.visible_count())
            .filter_map(|index| {
                let node = file_tree.get_visible_node(index)?;
                let is_selected = index == self.selected;

                // Build indentation based on depth
                let indent = "  ".repeat(node.depth);

                // Build directory/file icon
                let icon = if node.is_dir {
                    if node.expanded {
                        "v "
                    } else {
                        "> "
                    }
                } else {
                    "  "
                };

                // Build the display line
                let prefix = if is_selected { "> " } else { "  " };

                if is_selected {
                    let line = Line::from(vec![
                        Span::styled(
                            prefix,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            format!("{}{}{}", indent, icon, &node.name),
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]);
                    Some(ListItem::new(line))
                } else {
                    let line = Line::from(vec![
                        Span::raw(prefix),
                        Span::raw(format!("{}{}{}", indent, icon, &node.name)),
                    ]);
                    Some(ListItem::new(line))
                }
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, area);
    }

    /// Renders the help area with action shortcuts and navigation hints.
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let actions = self.resolved_actions();
        let action_hints: Vec<String> = actions
            .iter()
            .map(|(key, action)| {
                let icon = action.icon.as_deref().unwrap_or("");
                format!("{}{}:{}", icon, key, action.name)
            })
            .collect();

        let help_text = format!("{}  Enter: open/expand  Esc: back", action_hints.join("  "));

        let help = Paragraph::new(help_text)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(help, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GlobalConfig, WebClientConfig, Workspace};
    use std::collections::HashMap;
    use tempfile::TempDir;

    fn create_test_config_with_project(project_path: PathBuf) -> Config {
        let mut global_actions = HashMap::new();
        global_actions.insert(
            "c".to_string(),
            Action {
                name: "Claude".to_string(),
                command: "claude".to_string(),
                icon: Some("C".to_string()),
            },
        );

        let projects = vec![Project {
            name: "Test Project".to_string(),
            path: project_path,
            actions: HashMap::new(),
        }];

        let mut workspaces = HashMap::new();
        workspaces.insert(
            "test-workspace".to_string(),
            Workspace {
                name: "Test Workspace".to_string(),
                actions: HashMap::new(),
                projects,
            },
        );

        Config {
            global: GlobalConfig {
                editor: "$EDITOR".to_string(),
                git_info_level: Default::default(),
                actions: global_actions,
            },
            web_client: WebClientConfig::default(),
            workspace: workspaces,
        }
    }

    fn setup_test_project_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create a minimal file structure
        std::fs::create_dir(root.join("src")).unwrap();
        std::fs::File::create(root.join("src/main.rs")).unwrap();
        std::fs::File::create(root.join("README.md")).unwrap();

        dir
    }

    #[test]
    fn when_creating_view_should_load_file_tree() {
        let temp_dir = setup_test_project_dir();
        let config = create_test_config_with_project(temp_dir.path().to_path_buf());

        let view = FileBrowserView::new(&config, "test-workspace", 0, 0);

        assert!(view.file_tree.is_some());
        assert!(view.visible_count() > 0);
    }

    #[test]
    fn when_getting_project_should_return_correct_project() {
        let temp_dir = setup_test_project_dir();
        let config = create_test_config_with_project(temp_dir.path().to_path_buf());

        let view = FileBrowserView::new(&config, "test-workspace", 0, 0);

        let project = view.project();

        assert!(project.is_some());
        assert_eq!(project.unwrap().name, "Test Project");
    }
}
