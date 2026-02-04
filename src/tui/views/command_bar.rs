//! Command bar view component for the TUI.
//!
//! Displays a horizontal list of commands that can be selected and executed.
//! Activated with ':' (vim-style), navigated with h/l or arrows, executed with Enter.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::config::CommandBarItem;

/// View component for displaying a command bar at the bottom of the TUI.
///
/// Renders a horizontal list of commands with visual indication of the
/// currently selected item. Commands are displayed as bracketed items
/// with optional icons.
pub struct CommandBar<'a> {
    commands: &'a [CommandBarItem],
    selected: usize,
}

impl<'a> CommandBar<'a> {
    /// Creates a new CommandBar with the given commands and selection.
    ///
    /// # Arguments
    ///
    /// * `commands` - Slice of command bar items to display
    /// * `selected` - Index of the currently selected command
    ///
    /// # Returns
    ///
    /// A new CommandBar instance.
    pub fn new(commands: &'a [CommandBarItem], selected: usize) -> Self {
        Self { commands, selected }
    }

    /// Returns the number of commands in the bar.
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Returns whether the command bar has no commands.
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    /// Returns the currently selected command, if any.
    pub fn selected_command(&self) -> Option<&CommandBarItem> {
        self.commands.get(self.selected)
    }

    /// Renders the command bar to the terminal frame.
    ///
    /// The bar displays:
    /// - A ':' prefix to indicate vim-style command mode
    /// - Bracketed command names with optional icons
    /// - Selected command highlighted in yellow
    ///
    /// # Arguments
    ///
    /// * `frame` - The terminal frame to render to
    /// * `area` - The rectangular area to render within
    pub fn render(&self, frame: &mut Frame, area: Rect) {
        if self.commands.is_empty() {
            let empty_text = Paragraph::new(": (no commands configured)")
                .style(Style::default().fg(Color::DarkGray));
            frame.render_widget(empty_text, area);
            return;
        }

        let mut spans = vec![
            Span::styled(": ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ];

        for (index, item) in self.commands.iter().enumerate() {
            let is_selected = index == self.selected;

            // Add separator between commands
            if index > 0 {
                spans.push(Span::raw(" "));
            }

            let style = if is_selected {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Build the command display: [icon name] or [name]
            let display = if let Some(icon) = &item.icon {
                format!("[{} {}]", icon, item.name)
            } else {
                format!("[{}]", item.name)
            };

            spans.push(Span::styled(display, style));
        }

        // Add help hint at the end
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "h/l:nav  Enter:run  Esc:close",
            Style::default().fg(Color::DarkGray),
        ));

        let line = Line::from(spans);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_commands() -> Vec<CommandBarItem> {
        vec![
            CommandBarItem {
                key: "p".to_string(),
                name: "Pipeline".to_string(),
                command: "gitlab-pipeline".to_string(),
                icon: Some("🚀".to_string()),
            },
            CommandBarItem {
                key: "d".to_string(),
                name: "Deploy".to_string(),
                command: "deploy-status".to_string(),
                icon: None,
            },
        ]
    }

    #[test]
    fn when_creating_command_bar_should_have_correct_count() {
        let commands = create_test_commands();
        let bar = CommandBar::new(&commands, 0);

        assert_eq!(bar.len(), 2);
        assert!(!bar.is_empty());
    }

    #[test]
    fn when_selecting_command_should_return_correct_item() {
        let commands = create_test_commands();
        let bar = CommandBar::new(&commands, 1);

        let selected = bar.selected_command();

        assert!(selected.is_some());
        assert_eq!(selected.unwrap().name, "Deploy");
    }

    #[test]
    fn when_empty_commands_should_return_none() {
        let commands: Vec<CommandBarItem> = vec![];
        let bar = CommandBar::new(&commands, 0);

        assert!(bar.is_empty());
        assert!(bar.selected_command().is_none());
    }
}
