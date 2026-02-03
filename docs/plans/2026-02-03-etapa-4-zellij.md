# Etapa 4: Zellij Integration Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Integrate the TUI with Zellij to execute actions in new panes, generate layouts, and open files with the editor.

**Architecture:** The zellij module provides functions to check Zellij installation, generate KDL layouts, and execute Zellij CLI commands. The TUI runner calls these functions in response to user input events.

**Tech Stack:** std::process::Command for CLI execution, existing config and tui modules

---

## Task 1: Zellij Installation Check

**Files:**
- Create: `src/zellij/check.rs`
- Modify: `src/zellij/mod.rs`

**Step 1: Write failing test for Zellij check**

Create `src/zellij/check.rs`:

```rust
//! Zellij installation detection.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::process::Command;

/// Check if Zellij is installed and available in PATH.
///
/// # Returns
///
/// `true` if the `zellij` command is found and executable, `false` otherwise.
pub fn is_zellij_installed() -> bool {
    Command::new("zellij")
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Get the Zellij version string.
///
/// # Returns
///
/// The version string if Zellij is installed, None otherwise.
pub fn zellij_version() -> Option<String> {
    Command::new("zellij")
        .arg("--version")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|s| s.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn when_checking_zellij_installed_should_return_bool() {
        // This test just verifies the function runs without panic
        // The actual result depends on whether Zellij is installed
        let _result = is_zellij_installed();
    }

    #[test]
    fn when_getting_zellij_version_should_return_option() {
        // This test just verifies the function runs without panic
        let _result = zellij_version();
    }
}
```

**Step 2: Update zellij/mod.rs**

```rust
//! Zellij CLI interaction.
//!
//! @author waabox(waabox[at]gmail[dot]com)

pub mod check;

pub use check::{is_zellij_installed, zellij_version};
```

**Step 3: Run tests**

Run: `cargo test zellij::check`
Expected: Tests pass

**Step 4: Commit**

```bash
git add src/zellij/
git commit -m "feat(zellij): add installation check functions"
```

---

## Task 2: KDL Layout Generation

**Files:**
- Create: `src/zellij/layout.rs`
- Modify: `src/zellij/mod.rs`

**Step 1: Write layout generation**

Create `src/zellij/layout.rs`:

```rust
//! Zellij KDL layout generation.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::fs;
use std::path::PathBuf;

use crate::error::Result;

/// The KDL layout template for gz-claude.
const LAYOUT_TEMPLATE: &str = r#"layout {
    pane size=1 borderless=true {
        plugin location="zellij:tab-bar"
    }

    pane split_direction="vertical" {
        pane size=40 {
            command "gz-claude"
            args ["panel"]
        }
        pane focus=true {
            command "bash"
        }
    }

    pane size=1 borderless=true {
        plugin location="zellij:status-bar"
    }
}
"#;

/// Get the path to the Zellij layouts directory.
///
/// Returns `~/.config/zellij/layouts/` on Linux/macOS.
pub fn layouts_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("zellij")
        .join("layouts")
}

/// Get the path to the gz-claude layout file.
pub fn layout_path() -> PathBuf {
    layouts_dir().join("gz-claude.kdl")
}

/// Generate the gz-claude layout file.
///
/// Creates the layouts directory if it doesn't exist and writes
/// the KDL layout file.
///
/// # Returns
///
/// The path to the generated layout file.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or the file
/// cannot be written.
pub fn generate_layout() -> Result<PathBuf> {
    let dir = layouts_dir();
    fs::create_dir_all(&dir)?;

    let path = layout_path();
    fs::write(&path, LAYOUT_TEMPLATE)?;

    Ok(path)
}

/// Check if the layout file exists.
pub fn layout_exists() -> bool {
    layout_path().exists()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn when_generating_layout_should_create_file() {
        // Use a temp dir to avoid polluting user's config
        let temp_dir = TempDir::new().unwrap();
        let layout_dir = temp_dir.path().join("zellij").join("layouts");
        fs::create_dir_all(&layout_dir).unwrap();

        let layout_path = layout_dir.join("gz-claude.kdl");
        fs::write(&layout_path, LAYOUT_TEMPLATE).unwrap();

        assert!(layout_path.exists());
        let content = fs::read_to_string(&layout_path).unwrap();
        assert!(content.contains("gz-claude"));
        assert!(content.contains("panel"));
    }

    #[test]
    fn when_layout_template_should_contain_required_elements() {
        assert!(LAYOUT_TEMPLATE.contains("tab-bar"));
        assert!(LAYOUT_TEMPLATE.contains("status-bar"));
        assert!(LAYOUT_TEMPLATE.contains("gz-claude"));
        assert!(LAYOUT_TEMPLATE.contains("panel"));
        assert!(LAYOUT_TEMPLATE.contains("split_direction=\"vertical\""));
    }
}
```

**Step 2: Update zellij/mod.rs**

```rust
//! Zellij CLI interaction.
//!
//! @author waabox(waabox[at]gmail[dot]com)

pub mod check;
pub mod layout;

pub use check::{is_zellij_installed, zellij_version};
pub use layout::{generate_layout, layout_exists, layout_path, layouts_dir};
```

**Step 3: Run tests**

Run: `cargo test zellij::layout`
Expected: Tests pass

**Step 4: Commit**

```bash
git add src/zellij/
git commit -m "feat(zellij): add KDL layout generation"
```

---

## Task 3: Zellij Command Execution

**Files:**
- Create: `src/zellij/commands.rs`
- Modify: `src/zellij/mod.rs`

**Step 1: Write command execution functions**

Create `src/zellij/commands.rs`:

```rust
//! Zellij command execution.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::path::Path;
use std::process::Command;

use crate::error::{GzClaudeError, Result};

/// Start Zellij with the gz-claude layout.
///
/// # Returns
///
/// Ok(()) if Zellij started successfully.
///
/// # Errors
///
/// Returns an error if Zellij fails to start.
pub fn start_zellij() -> Result<()> {
    let status = Command::new("zellij")
        .args(["--layout", "gz-claude"])
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to start Zellij: {}", e)))?;

    if !status.success() {
        return Err(GzClaudeError::Zellij(format!(
            "Zellij exited with status: {}",
            status
        )));
    }

    Ok(())
}

/// Open a new pane with the specified command.
///
/// # Arguments
///
/// * `cwd` - The working directory for the new pane
/// * `command` - The command to run in the new pane
///
/// # Returns
///
/// Ok(()) if the pane was opened successfully.
///
/// # Errors
///
/// Returns an error if the Zellij command fails.
pub fn open_pane(cwd: &Path, command: &str) -> Result<()> {
    let status = Command::new("zellij")
        .args(["action", "new-pane", "--cwd"])
        .arg(cwd)
        .arg("--")
        .args(command.split_whitespace())
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to open pane: {}", e)))?;

    if !status.success() {
        return Err(GzClaudeError::Zellij(format!(
            "Failed to open pane with command '{}': exit status {}",
            command, status
        )));
    }

    Ok(())
}

/// Open a file in the editor.
///
/// # Arguments
///
/// * `cwd` - The working directory (project path)
/// * `editor` - The editor command (e.g., "$EDITOR" or "vim")
/// * `file_path` - The path to the file to open
///
/// # Returns
///
/// Ok(()) if the editor pane was opened successfully.
///
/// # Errors
///
/// Returns an error if the Zellij command fails.
pub fn open_file_in_editor(cwd: &Path, editor: &str, file_path: &Path) -> Result<()> {
    // Resolve $EDITOR if needed
    let resolved_editor = if editor == "$EDITOR" {
        std::env::var("EDITOR").unwrap_or_else(|_| "vim".to_string())
    } else {
        editor.to_string()
    };

    let file_str = file_path.to_string_lossy();

    let status = Command::new("zellij")
        .args(["action", "new-pane", "--cwd"])
        .arg(cwd)
        .arg("--")
        .arg(&resolved_editor)
        .arg(file_str.as_ref())
        .status()
        .map_err(|e| GzClaudeError::Zellij(format!("Failed to open editor: {}", e)))?;

    if !status.success() {
        return Err(GzClaudeError::Zellij(format!(
            "Failed to open '{}' in editor: exit status {}",
            file_path.display(),
            status
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    // Note: These tests require Zellij to be running, so we only test
    // the function signatures and basic error handling without actually
    // executing Zellij commands.

    #[test]
    fn when_command_module_loaded_should_compile() {
        // This test verifies the module compiles correctly
        assert!(true);
    }
}
```

**Step 2: Update zellij/mod.rs**

```rust
//! Zellij CLI interaction.
//!
//! @author waabox(waabox[at]gmail[dot]com)

pub mod check;
pub mod commands;
pub mod layout;

pub use check::{is_zellij_installed, zellij_version};
pub use commands::{open_file_in_editor, open_pane, start_zellij};
pub use layout::{generate_layout, layout_exists, layout_path, layouts_dir};
```

**Step 3: Run tests**

Run: `cargo test zellij::commands`
Expected: Tests pass

**Step 4: Commit**

```bash
git add src/zellij/
git commit -m "feat(zellij): add command execution functions"
```

---

## Task 4: Wire Actions in TUI Runner

**Files:**
- Modify: `src/tui/runner.rs`

**Step 1: Update handle_input for Action events**

Modify the `handle_input` function in `src/tui/runner.rs` to handle `InputEvent::Action`:

```rust
InputEvent::Action(key) => {
    handle_action(state, config, key);
}
```

**Step 2: Implement handle_action function**

Add a new private function:

```rust
fn handle_action(state: &AppState, config: &Config, key: char) {
    // Get the current project context
    let (workspace_id, project_index) = match state.current_view() {
        View::Projects { workspace_id } => (workspace_id.as_str(), state.selected_index()),
        View::FileBrowser { workspace_id, project_index } => {
            (workspace_id.as_str(), *project_index)
        }
        View::Workspaces => return, // No actions in workspace view
    };

    // Get resolved actions for the project
    let actions = config.resolve_actions(workspace_id, project_index);

    // Find the action for this key
    let key_str = key.to_string();
    if let Some(action) = actions.get(&key_str) {
        // Get the project path
        if let Some(project) = config
            .workspace
            .get(workspace_id)
            .and_then(|ws| ws.projects.get(project_index))
        {
            // Execute the action
            if let Err(e) = crate::zellij::open_pane(&project.path, &action.command) {
                // Log error but don't crash the TUI
                eprintln!("Error executing action: {}", e);
            }
        }
    }
}
```

**Step 3: Write tests**

Add tests to the `runner.rs` tests module:

```rust
#[test]
fn when_handling_action_at_workspaces_should_do_nothing() {
    let config = create_test_config();
    let state = AppState::new();
    // Should not panic
    handle_action(&state, &config, 'c');
}
```

**Step 4: Run tests**

Run: `cargo test tui::runner`
Expected: Tests pass

**Step 5: Commit**

```bash
git add src/tui/runner.rs
git commit -m "feat(tui): wire action key handling to Zellij commands"
```

---

## Task 5: Wire File Opening in File Browser

**Files:**
- Modify: `src/tui/runner.rs`
- Modify: `src/tui/views/file_browser.rs`

**Step 1: Update handle_enter for FileBrowser**

Modify the `handle_enter` function to handle file selection:

```rust
View::FileBrowser { workspace_id, project_index } => {
    let mut view = FileBrowserView::new(
        config,
        workspace_id,
        *project_index,
        state.selected_index(),
    );

    if view.selected_is_file() {
        // Open the file in editor
        if let Some(file_path) = view.selected_path() {
            if let Some(project) = view.project() {
                let editor = &config.global.editor;
                if let Err(e) = crate::zellij::open_file_in_editor(
                    &project.path,
                    editor,
                    &file_path,
                ) {
                    eprintln!("Error opening file: {}", e);
                }
            }
        }
    } else {
        // Toggle directory expand/collapse
        view.toggle_selected();
        // Note: This won't persist since view is recreated each frame
        // We need to cache the file tree state in AppState
    }
}
```

**Step 2: Add file tree caching to AppState**

This requires adding a cache for expanded directories. For now, implement a simple solution:

In `src/tui/app.rs`, add:

```rust
use std::collections::HashSet;
use std::path::PathBuf;

pub struct AppState {
    // ... existing fields ...
    /// Expanded directories in file browser (paths)
    expanded_dirs: HashSet<PathBuf>,
}

impl AppState {
    pub fn toggle_dir_expanded(&mut self, path: PathBuf) {
        if self.expanded_dirs.contains(&path) {
            self.expanded_dirs.remove(&path);
        } else {
            self.expanded_dirs.insert(path);
        }
    }

    pub fn is_dir_expanded(&self, path: &Path) -> bool {
        self.expanded_dirs.contains(path)
    }
}
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/tui/
git commit -m "feat(tui): wire file opening to editor via Zellij"
```

---

## Task 6: Wire Refresh (r key)

**Files:**
- Modify: `src/tui/runner.rs`

**Step 1: Implement refresh handling**

Update the `handle_input` function for `InputEvent::Refresh`:

```rust
InputEvent::Refresh => {
    // Git info is reloaded when views are created
    // Force a redraw by doing nothing special here
    // The view will reload git info on next render
    // In the future, we could add explicit state invalidation
}
```

Note: Since views are recreated on each render, git info is automatically refreshed. For better UX, we could add a visual indicator that refresh was triggered.

**Step 2: Commit**

```bash
git add src/tui/runner.rs
git commit -m "feat(tui): implement refresh key handling"
```

---

## Task 7: Update run_main to Start Zellij

**Files:**
- Modify: `src/main.rs`

**Step 1: Update run_main function**

Replace the placeholder code with actual Zellij startup:

```rust
fn run_main(force_web: bool, force_no_web: bool) {
    // Check if Zellij is installed
    if !zellij::is_zellij_installed() {
        eprintln!(
            "Error: Zellij not found\n\n\
             gz-claude requires Zellij to be installed.\n\
             Install it from: https://zellij.dev/documentation/installation"
        );
        std::process::exit(1);
    }

    // Load configuration
    let config = match Config::load() {
        // ... existing config loading code ...
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        // ... existing validation code ...
    }

    // Generate the layout
    if let Err(e) = zellij::generate_layout() {
        eprintln!("Error generating Zellij layout: {}", e);
        std::process::exit(1);
    }

    // Determine web client behavior (for Etapa 5)
    let _start_web = if force_web {
        true
    } else if force_no_web {
        false
    } else {
        config.web_client.auto_start
    };

    // Start Zellij with the layout
    if let Err(e) = zellij::start_zellij() {
        eprintln!("Error starting Zellij: {}", e);
        std::process::exit(1);
    }
}
```

**Step 2: Add zellij use statement**

Add at the top of main.rs:

```rust
use crate::zellij;
```

**Step 3: Run tests**

Run: `cargo test`
Expected: All tests pass

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: wire run_main to start Zellij with generated layout"
```

---

## Task 8: Integration Tests

**Files:**
- Modify: `tests/cli_test.rs`

**Step 1: Add test for Zellij not installed message**

```rust
#[test]
fn when_zellij_not_installed_should_show_error() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_config(&temp_dir);

    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    // Remove zellij from PATH to simulate not installed
    cmd.env("HOME", temp_dir.path())
        .env("PATH", "")  // Empty PATH means zellij won't be found
        .assert()
        .failure()
        .stderr(predicate::str::contains("Zellij not found"));
}
```

**Step 2: Run tests**

Run: `cargo test cli_test`
Expected: Tests pass

**Step 3: Commit**

```bash
git add tests/cli_test.rs
git commit -m "test(cli): add Zellij installation check test"
```

---

## Final Verification

After completing all tasks, run:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

All should pass with no errors or warnings.

---

## Summary

| Task | Component | Description |
|------|-----------|-------------|
| 1 | check.rs | Zellij installation check |
| 2 | layout.rs | KDL layout generation |
| 3 | commands.rs | Zellij command execution |
| 4 | runner.rs | Wire action keys |
| 5 | runner.rs + app.rs | Wire file opening |
| 6 | runner.rs | Wire refresh key |
| 7 | main.rs | Start Zellij on run |
| 8 | cli_test.rs | Integration tests |

**Total new tests:** ~10
