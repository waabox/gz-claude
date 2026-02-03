//! Zellij command execution utilities.
//!
//! Provides functions to start Zellij sessions and manage panes programmatically
//! through the Zellij CLI.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::path::Path;
use std::process::Command;

use crate::error::{GzClaudeError, Result};

/// Starts a new Zellij session with the gz-claude layout.
///
/// Executes `zellij --layout gz-claude` to launch Zellij with the pre-configured
/// workspace layout that includes the gz-claude panel.
///
/// # Returns
///
/// Returns `Ok(())` if Zellij starts successfully, or an error if the command fails.
///
/// # Errors
///
/// Returns `GzClaudeError::Zellij` if:
/// - Zellij is not installed or not in PATH
/// - The gz-claude layout file does not exist
/// - The Zellij process fails to start or exits with an error
///
/// # Example
///
/// ```no_run
/// use gz_claude::zellij::start_zellij;
///
/// match start_zellij() {
///     Ok(()) => println!("Zellij session started"),
///     Err(e) => eprintln!("Failed to start Zellij: {}", e),
/// }
/// ```
pub fn start_zellij() -> Result<()> {
    let output = Command::new("zellij")
        .arg("--layout")
        .arg("gz-claude")
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to execute zellij: {}", e)))?;

    if !output.success() {
        return Err(GzClaudeError::Zellij(format!(
            "Zellij exited with status: {}",
            output
        )));
    }

    Ok(())
}

/// Opens a new Zellij pane and executes a command.
///
/// Creates a new pane in the current Zellij session with the specified working
/// directory and runs the provided command.
///
/// # Arguments
///
/// * `cwd` - The working directory for the new pane
/// * `command` - The command string to execute (will be split by whitespace)
///
/// # Returns
///
/// Returns `Ok(())` if the pane is created successfully, or an error if the command fails.
///
/// # Errors
///
/// Returns `GzClaudeError::Zellij` if:
/// - Not running inside a Zellij session
/// - The Zellij action command fails
/// - The specified working directory is invalid
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use gz_claude::zellij::open_pane;
///
/// let cwd = Path::new("/home/user/project");
/// match open_pane(cwd, "cargo build --release") {
///     Ok(()) => println!("Pane opened with cargo build"),
///     Err(e) => eprintln!("Failed to open pane: {}", e),
/// }
/// ```
pub fn open_pane(cwd: &Path, command: &str) -> Result<()> {
    let command_parts: Vec<&str> = command.split_whitespace().collect();

    if command_parts.is_empty() {
        return Err(GzClaudeError::Zellij(
            "Cannot open pane with empty command".to_string(),
        ));
    }

    let mut cmd = Command::new("zellij");
    cmd.arg("action")
        .arg("new-pane")
        .arg("--cwd")
        .arg(cwd)
        .arg("--");

    for part in &command_parts {
        cmd.arg(part);
    }

    let output = cmd
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to execute zellij action: {}", e)))?;

    if !output.success() {
        return Err(GzClaudeError::Zellij(format!(
            "Zellij action failed with status: {}",
            output
        )));
    }

    Ok(())
}

/// Runs a command in a floating pane.
///
/// Creates a floating pane with the specified command. Floating panes
/// appear over the layout and don't have splitting restrictions.
/// User can toggle between floating/embedded with Ctrl+p w.
///
/// # Arguments
///
/// * `pane_name` - Unique name for the pane (for tracking)
/// * `command` - The full command string to execute (e.g., "claude /path/to/project")
///
/// # Returns
///
/// Returns `Ok(())` if the pane is created successfully.
///
/// # Errors
///
/// Returns `GzClaudeError::Zellij` if any Zellij action fails.
/// Runs a command in the main (central) pane by writing to the terminal.
///
/// If `fullscreen` is true, the pane will be toggled to fullscreen mode
/// for optimal viewing from the web client.
pub fn run_in_main_pane(command: &str, fullscreen: bool) -> Result<()> {
    if command.trim().is_empty() {
        return Err(GzClaudeError::Zellij(
            "Cannot run empty command".to_string(),
        ));
    }

    // Move focus to the right pane
    Command::new("zellij")
        .args(["action", "move-focus", "right"])
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to move focus: {}", e)))?;

    // Small delay
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Write the command to the terminal (with newline to execute)
    let cmd_with_newline = format!("{}\n", command);
    Command::new("zellij")
        .args(["action", "write-chars", &cmd_with_newline])
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to write command: {}", e)))?;

    if fullscreen {
        // Toggle fullscreen for web client viewing
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = Command::new("zellij")
            .args(["action", "toggle-fullscreen"])
            .status();
        // Don't move focus back - stay on the fullscreen pane
    } else {
        // Move focus back to gz-claude
        std::thread::sleep(std::time::Duration::from_millis(50));
        let _ = Command::new("zellij")
            .args(["action", "move-focus", "left"])
            .status();
    }

    Ok(())
}

/// Runs a command in a floating pane.
///
/// If `fullscreen` is true, the pane will be toggled to fullscreen mode
/// for optimal viewing from the web client.
pub fn run_in_floating_pane(pane_name: &str, command: &str, fullscreen: bool) -> Result<()> {
    if command.trim().is_empty() {
        return Err(GzClaudeError::Zellij(
            "Cannot run empty command".to_string(),
        ));
    }

    let command_parts: Vec<&str> = command.split_whitespace().collect();

    let mut cmd = Command::new("zellij");
    cmd.arg("run")
        .arg("--floating")
        .arg("--width")
        .arg("80%")
        .arg("--height")
        .arg("80%")
        .arg("--name")
        .arg(pane_name)
        .arg("--");

    for part in &command_parts {
        cmd.arg(part);
    }

    let output = cmd
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to open floating pane: {}", e)))?;

    if !output.success() {
        return Err(GzClaudeError::Zellij(format!(
            "Zellij run failed with status: {}",
            output
        )));
    }

    if fullscreen {
        // Toggle fullscreen for web client viewing
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = Command::new("zellij")
            .args(["action", "toggle-fullscreen"])
            .status();
    }

    Ok(())
}

/// Focus an existing pane by moving to the right.
///
/// Since Zellij doesn't support focus-by-name directly, this just
/// moves focus to the right pane.
///
/// # Returns
///
/// Returns `Ok(())` if focus moved successfully.
pub fn focus_main_pane() -> Result<()> {
    let output = Command::new("zellij")
        .args(["action", "move-focus", "right"])
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to move focus: {}", e)))?;

    if !output.success() {
        return Err(GzClaudeError::Zellij(
            "Failed to focus main pane".to_string(),
        ));
    }

    Ok(())
}

/// Toggle fullscreen mode for the currently focused pane.
///
/// This is useful for the web client to show only the Claude terminal.
pub fn toggle_fullscreen() -> Result<()> {
    let output = Command::new("zellij")
        .args(["action", "toggle-fullscreen"])
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to toggle fullscreen: {}", e)))?;

    if !output.success() {
        return Err(GzClaudeError::Zellij(
            "Failed to toggle fullscreen".to_string(),
        ));
    }

    Ok(())
}

/// Opens a file in an editor within a new Zellij pane.
///
/// Creates a new pane in the current Zellij session and opens the specified file
/// in the given editor. If the editor is "$EDITOR", it resolves the actual editor
/// from the environment variable, defaulting to "vim" if not set.
///
/// # Arguments
///
/// * `cwd` - The working directory for the new pane
/// * `editor` - The editor command to use (use "$EDITOR" to resolve from environment)
/// * `file_path` - The path to the file to open
///
/// # Returns
///
/// Returns `Ok(())` if the editor pane is created successfully, or an error if the command fails.
///
/// # Errors
///
/// Returns `GzClaudeError::Zellij` if:
/// - Not running inside a Zellij session
/// - The Zellij action command fails
/// - The specified working directory or file path is invalid
///
/// # Example
///
/// ```no_run
/// use std::path::Path;
/// use gz_claude::zellij::open_file_in_editor;
///
/// let cwd = Path::new("/home/user/project");
/// let file = Path::new("/home/user/project/src/main.rs");
///
/// // Using the $EDITOR environment variable
/// match open_file_in_editor(cwd, "$EDITOR", file) {
///     Ok(()) => println!("File opened in editor"),
///     Err(e) => eprintln!("Failed to open file: {}", e),
/// }
///
/// // Using a specific editor
/// match open_file_in_editor(cwd, "nvim", file) {
///     Ok(()) => println!("File opened in neovim"),
///     Err(e) => eprintln!("Failed to open file: {}", e),
/// }
/// ```
pub fn open_file_in_editor(cwd: &Path, editor: &str, file_path: &Path) -> Result<()> {
    let resolved_editor = if editor == "$EDITOR" {
        std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string())
    } else {
        editor.to_string()
    };

    let output = Command::new("zellij")
        .arg("action")
        .arg("new-pane")
        .arg("--cwd")
        .arg(cwd)
        .arg("--")
        .arg(&resolved_editor)
        .arg(file_path)
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to execute zellij action: {}", e)))?;

    if !output.success() {
        return Err(GzClaudeError::Zellij(format!(
            "Zellij action failed with status: {}",
            output
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_command_module_loaded_should_compile() {
        // This test verifies that the module compiles correctly and all functions
        // are properly defined. Actual Zellij command execution cannot be tested
        // without a running Zellij session.
        assert!(true);
    }
}
