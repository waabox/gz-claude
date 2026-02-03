//! Terminal setup and event handling for the TUI.
//!
//! Provides initialization, cleanup, and input event polling for the terminal interface.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use std::io::{self, Stdout};
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use crate::error::Result;

/// Type alias for the terminal with crossterm backend.
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Represents user input events from the terminal.
///
/// Abstracts keyboard input into semantic application events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    /// Navigate up in a list (Up arrow or 'k').
    Up,
    /// Navigate down in a list (Down arrow or 'j').
    Down,
    /// Confirm selection (Enter).
    Enter,
    /// Navigate back (Esc or Backspace).
    Back,
    /// Quit the application ('q').
    Quit,
    /// Refresh the current view ('r').
    Refresh,
    /// Custom action triggered by a character key.
    Action(char),
}

/// Initializes the terminal for TUI rendering.
///
/// Enters alternate screen mode and enables raw mode for direct keyboard input.
///
/// # Returns
///
/// A configured Terminal instance ready for rendering.
///
/// # Errors
///
/// Returns an error if terminal initialization fails.
pub fn init() -> Result<Tui> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

/// Restores the terminal to its original state.
///
/// Leaves alternate screen mode and disables raw mode.
///
/// # Returns
///
/// Ok(()) on successful restoration.
///
/// # Errors
///
/// Returns an error if terminal restoration fails.
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

/// Polls for input events with a timeout.
///
/// Non-blocking poll that returns immediately if no event is available
/// within the specified timeout.
///
/// # Arguments
///
/// * `timeout_ms` - Maximum time to wait for an event in milliseconds
///
/// # Returns
///
/// Some(InputEvent) if an event was received, None if timeout occurred.
///
/// # Errors
///
/// Returns an error if event polling fails.
pub fn poll_event(timeout_ms: u64) -> Result<Option<InputEvent>> {
    if event::poll(Duration::from_millis(timeout_ms))? {
        if let Event::Key(key_event) = event::read()? {
            return Ok(key_to_event(key_event));
        }
    }
    Ok(None)
}

/// Converts a KeyEvent to an InputEvent.
///
/// Maps keyboard input to semantic application events.
///
/// # Arguments
///
/// * `key` - The keyboard event to convert
///
/// # Returns
///
/// Some(InputEvent) for recognized keys, None for unhandled keys.
fn key_to_event(key: KeyEvent) -> Option<InputEvent> {
    match key.code {
        KeyCode::Up => Some(InputEvent::Up),
        KeyCode::Down => Some(InputEvent::Down),
        KeyCode::Enter => Some(InputEvent::Enter),
        KeyCode::Esc | KeyCode::Backspace => Some(InputEvent::Back),
        KeyCode::Char(c) => {
            if key.modifiers.is_empty() || key.modifiers == KeyModifiers::SHIFT {
                match c {
                    'k' => Some(InputEvent::Up),
                    'j' => Some(InputEvent::Down),
                    'q' => Some(InputEvent::Quit),
                    'r' => Some(InputEvent::Refresh),
                    _ => Some(InputEvent::Action(c)),
                }
            } else {
                None
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn when_pressing_up_or_k_should_return_up_event() {
        let up_arrow = create_key_event(KeyCode::Up, KeyModifiers::NONE);
        let k_key = create_key_event(KeyCode::Char('k'), KeyModifiers::NONE);

        assert_eq!(key_to_event(up_arrow), Some(InputEvent::Up));
        assert_eq!(key_to_event(k_key), Some(InputEvent::Up));
    }

    #[test]
    fn when_pressing_down_or_j_should_return_down_event() {
        let down_arrow = create_key_event(KeyCode::Down, KeyModifiers::NONE);
        let j_key = create_key_event(KeyCode::Char('j'), KeyModifiers::NONE);

        assert_eq!(key_to_event(down_arrow), Some(InputEvent::Down));
        assert_eq!(key_to_event(j_key), Some(InputEvent::Down));
    }

    #[test]
    fn when_pressing_enter_should_return_enter_event() {
        let enter_key = create_key_event(KeyCode::Enter, KeyModifiers::NONE);

        assert_eq!(key_to_event(enter_key), Some(InputEvent::Enter));
    }

    #[test]
    fn when_pressing_esc_or_backspace_should_return_back_event() {
        let esc_key = create_key_event(KeyCode::Esc, KeyModifiers::NONE);
        let backspace_key = create_key_event(KeyCode::Backspace, KeyModifiers::NONE);

        assert_eq!(key_to_event(esc_key), Some(InputEvent::Back));
        assert_eq!(key_to_event(backspace_key), Some(InputEvent::Back));
    }

    #[test]
    fn when_pressing_q_should_return_quit_event() {
        let q_key = create_key_event(KeyCode::Char('q'), KeyModifiers::NONE);

        assert_eq!(key_to_event(q_key), Some(InputEvent::Quit));
    }

    #[test]
    fn when_pressing_r_should_return_refresh_event() {
        let r_key = create_key_event(KeyCode::Char('r'), KeyModifiers::NONE);

        assert_eq!(key_to_event(r_key), Some(InputEvent::Refresh));
    }

    #[test]
    fn when_pressing_other_char_should_return_action_event() {
        let a_key = create_key_event(KeyCode::Char('a'), KeyModifiers::NONE);
        let x_key = create_key_event(KeyCode::Char('x'), KeyModifiers::NONE);
        let one_key = create_key_event(KeyCode::Char('1'), KeyModifiers::NONE);

        assert_eq!(key_to_event(a_key), Some(InputEvent::Action('a')));
        assert_eq!(key_to_event(x_key), Some(InputEvent::Action('x')));
        assert_eq!(key_to_event(one_key), Some(InputEvent::Action('1')));
    }
}
