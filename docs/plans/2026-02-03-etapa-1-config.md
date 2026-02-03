# Stage 1: Configuration - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Implement the configuration system with TOML parsing, strict path validation, and action inheritance at three levels (global -> workspace -> project).

**Architecture:** Structs with serde to deserialize config.toml. Two-phase validation: first TOML structure, then business rules (paths exist, valid keys). Config loader that generates example if not exists.

**Tech Stack:** serde + toml for parsing, dirs for system paths, thiserror for typed errors.

---

## Task 1: Define configuration structs

**Files:**
- Modify: `src/config/mod.rs`

**Step 1: Write configuration structs**

```rust
//! Configuration parsing and validation.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Root configuration structure.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub global: GlobalConfig,
    #[serde(default)]
    pub web_client: WebClientConfig,
    #[serde(default)]
    pub workspace: HashMap<String, Workspace>,
}

/// Global settings that apply to all workspaces.
#[derive(Debug, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_editor")]
    pub editor: String,
    #[serde(default)]
    pub git_info_level: GitInfoLevel,
    #[serde(default)]
    pub actions: HashMap<String, Action>,
}

fn default_editor() -> String {
    "$EDITOR".to_string()
}

/// Git information detail level.
#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GitInfoLevel {
    #[default]
    Minimal,
    Standard,
    Detailed,
}

/// Web client configuration.
#[derive(Debug, Deserialize)]
pub struct WebClientConfig {
    #[serde(default)]
    pub auto_start: bool,
    #[serde(default = "default_bind_address")]
    pub bind_address: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    8082
}

impl Default for WebClientConfig {
    fn default() -> Self {
        Self {
            auto_start: false,
            bind_address: default_bind_address(),
            port: default_port(),
        }
    }
}

/// A workspace containing multiple projects.
#[derive(Debug, Deserialize)]
pub struct Workspace {
    pub name: String,
    #[serde(default)]
    pub actions: HashMap<String, Action>,
    #[serde(default)]
    pub projects: Vec<Project>,
}

/// A project within a workspace.
#[derive(Debug, Deserialize)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub actions: HashMap<String, Action>,
}

/// An action that can be triggered from the TUI.
#[derive(Debug, Deserialize, Clone)]
pub struct Action {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub icon: Option<String>,
}
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat(config): add configuration structs with serde"
```

---

## Task 2: Create configuration error module

**Files:**
- Modify: `src/error.rs`

**Step 1: Extend errors for configuration**

```rust
//! Error types for gz-claude.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GzClaudeError {
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Zellij error: {0}")]
    Zellij(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found at {0}")]
    NotFound(PathBuf),

    #[error("Failed to read configuration file: {0}")]
    ReadError(#[from] std::io::Error),

    #[error("Failed to parse configuration: {0}")]
    ParseError(#[from] toml::de::Error),

    #[error("Invalid action key '{key}': must be a single character")]
    InvalidActionKey { key: String },

    #[error("Empty command for action '{action_name}'")]
    EmptyCommand { action_name: String },

    #[error("Project path does not exist: {path}")]
    PathNotFound { path: PathBuf },

    #[error("Project path is not a directory: {path}")]
    PathNotDirectory { path: PathBuf },

    #[error("No workspaces configured")]
    NoWorkspaces,
}

pub type Result<T> = std::result::Result<T, GzClaudeError>;
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/error.rs
git commit -m "feat(config): add configuration error types"
```

---

## Task 3: Implement configuration loading and parsing

**Files:**
- Modify: `src/config/mod.rs`

**Step 1: Add loading function**

Add at the end of `src/config/mod.rs`:

```rust
use crate::error::{ConfigError, Result};
use std::fs;

impl Config {
    /// Load configuration from the default path (~/.config/gz-claude/config.toml).
    pub fn load() -> Result<Self> {
        let config_path = Self::default_path();
        Self::load_from(&config_path)
    }

    /// Load configuration from a specific path.
    pub fn load_from(path: &PathBuf) -> Result<Self> {
        if !path.exists() {
            return Err(ConfigError::NotFound(path.clone()).into());
        }

        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Returns the default configuration file path.
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gz-claude")
            .join("config.toml")
    }

    /// Returns the default configuration directory.
    pub fn default_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("gz-claude")
    }
}
```

**Step 2: Update imports in mod.rs**

Add at the beginning after existing use statements:

```rust
use std::fs;
use crate::error::{ConfigError, Result};
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat(config): add config loading from TOML file"
```

---

## Task 4: Implement configuration validation

**Files:**
- Modify: `src/config/mod.rs`

**Step 1: Add validation method**

Add to impl Config:

```rust
    /// Validate the configuration.
    ///
    /// Checks:
    /// - At least one workspace exists
    /// - All action keys are single characters
    /// - All action commands are non-empty
    /// - All project paths exist and are directories
    pub fn validate(&self) -> Result<()> {
        if self.workspace.is_empty() {
            return Err(ConfigError::NoWorkspaces.into());
        }

        // Validate global actions
        self.validate_actions(&self.global.actions)?;

        // Validate each workspace
        for workspace in self.workspace.values() {
            self.validate_actions(&workspace.actions)?;

            for project in &workspace.projects {
                self.validate_actions(&project.actions)?;
                self.validate_project_path(project)?;
            }
        }

        Ok(())
    }

    fn validate_actions(&self, actions: &HashMap<String, Action>) -> Result<()> {
        for (key, action) in actions {
            if key.chars().count() != 1 {
                return Err(ConfigError::InvalidActionKey { key: key.clone() }.into());
            }
            if action.command.trim().is_empty() {
                return Err(ConfigError::EmptyCommand {
                    action_name: action.name.clone(),
                }
                .into());
            }
        }
        Ok(())
    }

    fn validate_project_path(&self, project: &Project) -> Result<()> {
        if !project.path.exists() {
            return Err(ConfigError::PathNotFound {
                path: project.path.clone(),
            }
            .into());
        }
        if !project.path.is_dir() {
            return Err(ConfigError::PathNotDirectory {
                path: project.path.clone(),
            }
            .into());
        }
        Ok(())
    }
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat(config): add configuration validation"
```

---

## Task 5: Implement example config generation

**Files:**
- Modify: `src/config/mod.rs`

**Step 1: Add constant with example config**

Add at the beginning of the file after imports:

```rust
const EXAMPLE_CONFIG: &str = r#"# gz-claude configuration file
# See: docs/plans/2026-02-03-gz-claude-design.md

[global]
editor = "$EDITOR"
git_info_level = "minimal"  # minimal | standard | detailed

[global.actions]
c = { name = "Claude", command = "claude", icon = "C" }
b = { name = "Bash", command = "bash", icon = "B" }
g = { name = "Lazygit", command = "lazygit", icon = "G" }

[web_client]
auto_start = false
bind_address = "0.0.0.0"
port = 8082

# Example workspace - customize for your projects
[workspace.example]
name = "Example Workspace"

[workspace.example.actions]
t = { name = "Tests", command = "cargo test", icon = "T" }

[[workspace.example.projects]]
name = "My Project"
path = "/path/to/your/project"
"#;
```

**Step 2: Add method to create example config**

Add to impl Config:

```rust
    /// Create an example configuration file at the default path.
    /// Returns the path where the file was created.
    pub fn create_example() -> Result<PathBuf> {
        let config_dir = Self::default_dir();
        fs::create_dir_all(&config_dir)?;

        let config_path = Self::default_path();
        fs::write(&config_path, EXAMPLE_CONFIG)?;

        Ok(config_path)
    }

    /// Load configuration, creating an example if it doesn't exist.
    /// Returns (Config, was_created) tuple.
    pub fn load_or_create_example() -> Result<(Self, bool)> {
        let config_path = Self::default_path();

        if !config_path.exists() {
            Self::create_example()?;
            return Err(ConfigError::NotFound(config_path).into());
        }

        let config = Self::load_from(&config_path)?;
        Ok((config, false))
    }
```

**Step 3: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 4: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat(config): add example config generation"
```

---

## Task 6: Implement action resolution with inheritance

**Files:**
- Modify: `src/config/mod.rs`

**Step 1: Add method to resolve actions**

Add to impl Config:

```rust
    /// Resolve actions for a specific project, applying inheritance:
    /// global -> workspace -> project
    pub fn resolve_actions(
        &self,
        workspace_id: &str,
        project_index: usize,
    ) -> HashMap<String, Action> {
        let mut actions = self.global.actions.clone();

        if let Some(workspace) = self.workspace.get(workspace_id) {
            // Merge workspace actions (override global)
            for (key, action) in &workspace.actions {
                actions.insert(key.clone(), action.clone());
            }

            // Merge project actions (override workspace)
            if let Some(project) = workspace.projects.get(project_index) {
                for (key, action) in &project.actions {
                    actions.insert(key.clone(), action.clone());
                }
            }
        }

        actions
    }
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/config/mod.rs
git commit -m "feat(config): add action inheritance resolution"
```

---

## Task 7: Create unit tests for configuration

**Files:**
- Create: `src/config/tests.rs`
- Modify: `src/config/mod.rs`

**Step 1: Create tests file**

Create `src/config/tests.rs`:

```rust
//! Tests for configuration module.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use super::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn create_temp_config(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn when_parsing_valid_config_should_succeed() {
    let content = r#"
        [global]
        editor = "vim"
        git_info_level = "standard"

        [global.actions]
        c = { name = "Claude", command = "claude", icon = "C" }

        [web_client]
        auto_start = true
        port = 9000

        [workspace.test]
        name = "Test Workspace"

        [[workspace.test.projects]]
        name = "Project 1"
        path = "/tmp"
    "#;

    let file = create_temp_config(content);
    let config = Config::load_from(&file.path().to_path_buf()).unwrap();

    assert_eq!(config.global.editor, "vim");
    assert_eq!(config.global.git_info_level, GitInfoLevel::Standard);
    assert!(config.web_client.auto_start);
    assert_eq!(config.web_client.port, 9000);
    assert_eq!(config.workspace.len(), 1);
}

#[test]
fn when_parsing_minimal_config_should_use_defaults() {
    let content = r#"
        [global]

        [workspace.test]
        name = "Test"

        [[workspace.test.projects]]
        name = "P1"
        path = "/tmp"
    "#;

    let file = create_temp_config(content);
    let config = Config::load_from(&file.path().to_path_buf()).unwrap();

    assert_eq!(config.global.editor, "$EDITOR");
    assert_eq!(config.global.git_info_level, GitInfoLevel::Minimal);
    assert!(!config.web_client.auto_start);
    assert_eq!(config.web_client.port, 8082);
}

#[test]
fn when_validating_config_with_invalid_action_key_should_fail() {
    let content = r#"
        [global]

        [global.actions]
        invalid_key = { name = "Test", command = "test" }

        [workspace.test]
        name = "Test"

        [[workspace.test.projects]]
        name = "P1"
        path = "/tmp"
    "#;

    let file = create_temp_config(content);
    let config = Config::load_from(&file.path().to_path_buf()).unwrap();
    let result = config.validate();

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("single character"));
}

#[test]
fn when_validating_config_with_empty_command_should_fail() {
    let content = r#"
        [global]

        [global.actions]
        c = { name = "Claude", command = "   " }

        [workspace.test]
        name = "Test"

        [[workspace.test.projects]]
        name = "P1"
        path = "/tmp"
    "#;

    let file = create_temp_config(content);
    let config = Config::load_from(&file.path().to_path_buf()).unwrap();
    let result = config.validate();

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Empty command"));
}

#[test]
fn when_validating_config_with_nonexistent_path_should_fail() {
    let content = r#"
        [global]

        [workspace.test]
        name = "Test"

        [[workspace.test.projects]]
        name = "P1"
        path = "/nonexistent/path/that/does/not/exist"
    "#;

    let file = create_temp_config(content);
    let config = Config::load_from(&file.path().to_path_buf()).unwrap();
    let result = config.validate();

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("does not exist"));
}

#[test]
fn when_validating_config_with_no_workspaces_should_fail() {
    let content = r#"
        [global]
    "#;

    let file = create_temp_config(content);
    let config = Config::load_from(&file.path().to_path_buf()).unwrap();
    let result = config.validate();

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("No workspaces"));
}

#[test]
fn when_resolving_actions_should_apply_inheritance() {
    let content = r#"
        [global]

        [global.actions]
        c = { name = "Global Claude", command = "claude-global" }
        g = { name = "Git", command = "git" }

        [workspace.test]
        name = "Test"

        [workspace.test.actions]
        c = { name = "Workspace Claude", command = "claude-workspace" }
        t = { name = "Tests", command = "cargo test" }

        [[workspace.test.projects]]
        name = "P1"
        path = "/tmp"

        [workspace.test.projects.actions]
        c = { name = "Project Claude", command = "claude-project" }
        p = { name = "Project Only", command = "project-cmd" }
    "#;

    let file = create_temp_config(content);
    let config = Config::load_from(&file.path().to_path_buf()).unwrap();
    let actions = config.resolve_actions("test", 0);

    // Project level overrides workspace which overrides global
    assert_eq!(actions.get("c").unwrap().command, "claude-project");
    // Workspace level (not overridden by project)
    assert_eq!(actions.get("t").unwrap().command, "cargo test");
    // Global level (not overridden)
    assert_eq!(actions.get("g").unwrap().command, "git");
    // Project only
    assert_eq!(actions.get("p").unwrap().command, "project-cmd");
}
```

**Step 2: Add tests module to mod.rs**

Add at the end of `src/config/mod.rs`:

```rust
#[cfg(test)]
mod tests;
```

**Step 3: Run tests**

Run: `cargo test config`
Expected: 7 tests pass

**Step 4: Commit**

```bash
git add src/config/
git commit -m "test(config): add unit tests for configuration"
```

---

## Task 8: Integrate configuration in main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: Update main.rs to load configuration**

```rust
//! gz-claude: TUI for orchestrating Zellij workspaces with Claude Code.
//!
//! @author waabox(waabox[at]gmail[dot]com)

mod cli;
mod config;
mod error;
mod git;
mod tui;
mod zellij;

use clap::Parser;
use cli::{Cli, Command};
use config::Config;

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Panel) => {
            run_panel();
        }
        None => {
            run_main(cli.web, cli.no_web);
        }
    }
}

fn run_main(force_web: bool, force_no_web: bool) {
    // Load configuration
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            if matches!(
                e,
                error::GzClaudeError::Config(error::ConfigError::NotFound(_))
            ) {
                // Create example config
                match Config::create_example() {
                    Ok(path) => {
                        println!(
                            "Created example configuration at {}\n\
                             Please edit it to add your workspaces and run again.",
                            path.display()
                        );
                    }
                    Err(e) => {
                        eprintln!("Error creating example config: {}", e);
                    }
                }
            } else {
                eprintln!("Error loading configuration: {}", e);
            }
            std::process::exit(1);
        }
    };

    // Validate configuration
    if let Err(e) = config.validate() {
        eprintln!("Error: Invalid configuration\n\n{}", e);
        eprintln!(
            "\nPlease fix the configuration at {}",
            Config::default_path().display()
        );
        std::process::exit(1);
    }

    // Determine web client behavior
    let start_web = if force_web {
        true
    } else if force_no_web {
        false
    } else {
        config.web_client.auto_start
    };

    println!("Configuration loaded successfully!");
    println!("Workspaces: {}", config.workspace.len());
    println!("Web client: {}", if start_web { "enabled" } else { "disabled" });
    println!("\nStarting gz-claude...");
}

fn run_panel() {
    // Check if running inside Zellij
    if std::env::var("ZELLIJ").is_err() {
        eprintln!(
            "Error: gz-claude panel must be run inside Zellij.\n\
             Run 'gz-claude' without arguments to start Zellij with the proper layout."
        );
        std::process::exit(1);
    }

    println!("Running in panel mode (inside Zellij)");
}
```

**Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate configuration loading in main"
```

---

## Task 9: Update integration tests

**Files:**
- Modify: `tests/cli_test.rs`

**Step 1: Update test to reflect new behavior**

```rust
//! Integration tests for CLI.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_config(dir: &TempDir) -> std::path::PathBuf {
    let config_dir = dir.path().join(".config").join("gz-claude");
    fs::create_dir_all(&config_dir).unwrap();

    let config_content = r#"
        [global]
        editor = "vim"

        [global.actions]
        c = { name = "Claude", command = "claude" }

        [workspace.test]
        name = "Test Workspace"

        [[workspace.test.projects]]
        name = "Test Project"
        path = "/tmp"
    "#;

    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, config_content).unwrap();
    config_path
}

#[test]
fn when_running_without_config_should_create_example() {
    let temp_dir = TempDir::new().unwrap();
    let home = temp_dir.path();

    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.env("HOME", home)
        .env("XDG_CONFIG_HOME", home.join(".config"))
        .assert()
        .failure()
        .stdout(predicate::str::contains("Created example configuration"));
}

#[test]
fn when_running_with_valid_config_should_succeed() {
    let temp_dir = TempDir::new().unwrap();
    setup_test_config(&temp_dir);

    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join(".config"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Configuration loaded successfully"));
}

#[test]
fn when_running_panel_outside_zellij_should_fail() {
    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.arg("panel")
        .env_remove("ZELLIJ")
        .assert()
        .failure()
        .stderr(predicate::str::contains("must be run inside Zellij"));
}

#[test]
fn when_running_with_help_flag_should_show_help() {
    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("TUI for orchestrating Zellij"));
}

#[test]
fn when_running_with_version_flag_should_show_version() {
    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("gz-claude"));
}

#[test]
fn when_running_with_web_and_no_web_flags_should_fail() {
    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.args(["--web", "--no-web"]).assert().failure();
}
```

**Step 2: Run all tests**

Run: `cargo test`
Expected: All tests pass

**Step 3: Commit**

```bash
git add tests/cli_test.rs
git commit -m "test: update CLI integration tests for config loading"
```

---

## Final Verification

**Run all checks:**

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

Expected: All pass without errors or warnings.

---

## Commit Summary

1. `feat(config): add configuration structs with serde`
2. `feat(config): add configuration error types`
3. `feat(config): add config loading from TOML file`
4. `feat(config): add configuration validation`
5. `feat(config): add example config generation`
6. `feat(config): add action inheritance resolution`
7. `test(config): add unit tests for configuration`
8. `feat: integrate configuration loading in main`
9. `test: update CLI integration tests for config loading`
