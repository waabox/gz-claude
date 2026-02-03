//! Integration tests for CLI.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

fn setup_test_config(dir: &TempDir) -> std::path::PathBuf {
    // On macOS, dirs::config_dir() returns ~/Library/Application Support
    // On Linux, it returns ~/.config
    #[cfg(target_os = "macos")]
    let config_dir = dir
        .path()
        .join("Library")
        .join("Application Support")
        .join("gz-claude");
    #[cfg(not(target_os = "macos"))]
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
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Configuration loaded successfully",
        ));
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

#[test]
fn when_running_panel_with_valid_config_should_start_tui() {
    // This test verifies panel mode starts with valid config.
    // We can't test the actual TUI, but we can test it initializes.
    let temp_dir = TempDir::new().unwrap();
    setup_test_config(&temp_dir);

    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    // Set ZELLIJ env to pretend we're inside Zellij.
    // But the TUI will fail to initialize without a real terminal,
    // so we just verify it gets past the Zellij check.
    let assertion = cmd
        .arg("panel")
        .env("HOME", temp_dir.path())
        .env("ZELLIJ", "true")
        .timeout(std::time::Duration::from_millis(500))
        .assert();
    // The process will fail because there's no terminal,
    // but it won't fail with the "must be run inside Zellij" error.
    // Verify it does NOT contain the Zellij environment check error.
    assertion.stderr(predicate::str::contains("must be run inside Zellij").not());
}
