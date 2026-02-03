//! Workspaces view component for the TUI.
//!
//! Displays a list of available workspaces with selection highlighting
//! and keyboard navigation hints.
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

use crate::config::Config;

/// View component for displaying and selecting workspaces.
///
/// Renders a list of workspace names sorted alphabetically with visual
/// indication of the currently selected item. The view includes a title
/// area, scrollable list, and help text.
pub struct WorkspacesView<'a> {
    config: &'a Config,
    selected: usize,
}

impl<'a> WorkspacesView<'a> {
    /// Creates a new WorkspacesView with the given configuration and selection.
    ///
    /// # Arguments
    ///
    /// * `config` - Reference to the application configuration containing workspaces
    /// * `selected` - Index of the currently selected workspace
    ///
    /// # Returns
    ///
    /// A new WorkspacesView instance.
    pub fn new(config: &'a Config, selected: usize) -> Self {
        Self { config, selected }
    }

    /// Returns the sorted list of workspace identifiers.
    ///
    /// Workspace IDs are sorted alphabetically to ensure consistent ordering
    /// across renders and sessions.
    ///
    /// # Returns
    ///
    /// A vector of workspace ID references sorted alphabetically.
    pub fn workspace_ids(&self) -> Vec<&str> {
        let mut ids: Vec<&str> = self.config.workspace.keys().map(|s| s.as_str()).collect();
        ids.sort();
        ids
    }

    /// Returns the number of workspaces in the configuration.
    ///
    /// # Returns
    ///
    /// The count of available workspaces.
    pub fn len(&self) -> usize {
        self.config.workspace.len()
    }

    /// Checks if there are no workspaces in the configuration.
    ///
    /// # Returns
    ///
    /// True if no workspaces are configured, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.config.workspace.is_empty()
    }

    /// Renders the workspaces view to the terminal frame.
    ///
    /// The layout consists of three areas:
    /// - Title area (3 lines): displays "Workspaces" header with cyan styling
    /// - List area (flexible): displays workspace names with selection highlighting
    /// - Help area (3 lines): displays keyboard navigation hints
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

    /// Renders the title area with "Workspaces" header.
    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let title = Paragraph::new("Workspaces")
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::BOTTOM));

        frame.render_widget(title, area);
    }

    /// Renders the list of workspaces with selection highlighting.
    fn render_list(&self, frame: &mut Frame, area: Rect) {
        let workspace_ids = self.workspace_ids();

        let items: Vec<ListItem> = workspace_ids
            .iter()
            .enumerate()
            .map(|(index, id)| {
                let workspace = self.config.workspace.get(*id);
                let display_name = workspace.map(|w| w.name.as_str()).unwrap_or(*id);

                if index == self.selected {
                    let line = Line::from(vec![
                        Span::styled(
                            "> ",
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled(
                            display_name,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::BOLD),
                        ),
                    ]);
                    ListItem::new(line)
                } else {
                    ListItem::new(Line::from(format!("  {}", display_name)))
                }
            })
            .collect();

        let list = List::new(items);
        frame.render_widget(list, area);
    }

    /// Renders the help area with keyboard navigation hints.
    fn render_help(&self, frame: &mut Frame, area: Rect) {
        let help_text = Paragraph::new("Enter: select  q: quit")
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::TOP));

        frame.render_widget(help_text, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GlobalConfig, Workspace};
    use std::collections::HashMap;

    fn create_test_config() -> Config {
        let mut workspaces = HashMap::new();
        workspaces.insert(
            "fanki".to_string(),
            Workspace {
                name: "Fanki".to_string(),
                actions: HashMap::new(),
                projects: vec![],
            },
        );
        workspaces.insert(
            "helios".to_string(),
            Workspace {
                name: "Helios".to_string(),
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
            web_client: Default::default(),
            workspace: workspaces,
        }
    }

    fn create_empty_config() -> Config {
        Config {
            global: GlobalConfig {
                editor: "$EDITOR".to_string(),
                git_info_level: Default::default(),
                actions: HashMap::new(),
            },
            web_client: Default::default(),
            workspace: HashMap::new(),
        }
    }

    #[test]
    fn when_creating_view_should_return_sorted_workspace_ids() {
        let config = create_test_config();
        let view = WorkspacesView::new(&config, 0);

        let ids = view.workspace_ids();

        assert_eq!(ids, vec!["fanki", "helios"]);
    }

    #[test]
    fn when_getting_len_should_return_workspace_count() {
        let config = create_test_config();
        let view = WorkspacesView::new(&config, 0);

        let count = view.len();

        assert_eq!(count, 2);
    }

    #[test]
    fn when_config_has_no_workspaces_should_return_empty() {
        let config = create_empty_config();
        let view = WorkspacesView::new(&config, 0);

        assert!(view.is_empty());
        assert_eq!(view.len(), 0);
        assert!(view.workspace_ids().is_empty());
    }
}
