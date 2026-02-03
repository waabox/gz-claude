//! Integration tests for CLI.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn when_running_without_args_should_show_starting_message() {
    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Starting gz-claude"));
}

#[test]
fn when_running_panel_subcommand_should_show_panel_mode_message() {
    let mut cmd = Command::cargo_bin("gz-claude").unwrap();
    cmd.arg("panel")
        .assert()
        .success()
        .stdout(predicate::str::contains("Running in panel mode"));
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
    cmd.args(["--web", "--no-web"])
        .assert()
        .failure();
}
