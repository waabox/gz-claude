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
        c = { name = "Claude", command = "claude", icon = "🤖" }

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
