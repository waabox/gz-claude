//! Tests for Git module.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use super::*;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn create_test_repo() -> TempDir {
    let dir = TempDir::new().unwrap();
    let path = dir.path();

    // Initialize repo
    Command::new("git")
        .args(["init"])
        .current_dir(path)
        .output()
        .unwrap();

    // Configure git user for commits
    Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(path)
        .output()
        .unwrap();

    Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(path)
        .output()
        .unwrap();

    dir
}

fn create_file(dir: &TempDir, name: &str, content: &str) {
    fs::write(dir.path().join(name), content).unwrap();
}

fn git_add(dir: &TempDir, file: &str) {
    Command::new("git")
        .args(["add", file])
        .current_dir(dir.path())
        .output()
        .unwrap();
}

fn git_commit(dir: &TempDir, msg: &str) {
    Command::new("git")
        .args(["commit", "-m", msg])
        .current_dir(dir.path())
        .output()
        .unwrap();
}

#[test]
fn when_path_is_not_repo_should_return_none() {
    let dir = TempDir::new().unwrap();
    let info = get_git_info(dir.path(), GitInfoLevel::Minimal);
    assert!(info.is_none());
}

#[test]
fn when_repo_is_clean_should_not_be_dirty() {
    let dir = create_test_repo();
    create_file(&dir, "file.txt", "content");
    git_add(&dir, "file.txt");
    git_commit(&dir, "Initial commit");

    let info = get_git_info(dir.path(), GitInfoLevel::Minimal).unwrap();
    assert!(!info.is_dirty);
}

#[test]
fn when_repo_has_changes_should_be_dirty() {
    let dir = create_test_repo();
    create_file(&dir, "file.txt", "content");
    git_add(&dir, "file.txt");
    git_commit(&dir, "Initial commit");

    // Make a change
    create_file(&dir, "file.txt", "modified content");

    let info = get_git_info(dir.path(), GitInfoLevel::Minimal).unwrap();
    assert!(info.is_dirty);
}

#[test]
fn when_repo_has_untracked_should_be_dirty() {
    let dir = create_test_repo();
    create_file(&dir, "file.txt", "content");
    git_add(&dir, "file.txt");
    git_commit(&dir, "Initial commit");

    // Add untracked file
    create_file(&dir, "untracked.txt", "new file");

    let info = get_git_info(dir.path(), GitInfoLevel::Minimal).unwrap();
    assert!(info.is_dirty);
}

#[test]
fn when_getting_branch_should_return_current_branch() {
    let dir = create_test_repo();
    create_file(&dir, "file.txt", "content");
    git_add(&dir, "file.txt");
    git_commit(&dir, "Initial commit");

    let info = get_git_info(dir.path(), GitInfoLevel::Minimal).unwrap();
    // Default branch could be "main" or "master" depending on git config
    assert!(info.branch.is_some());
}

#[test]
fn when_standard_level_should_include_staged_count() {
    let dir = create_test_repo();
    create_file(&dir, "file.txt", "content");
    git_add(&dir, "file.txt");
    git_commit(&dir, "Initial commit");

    // Stage a new file
    create_file(&dir, "staged.txt", "staged content");
    git_add(&dir, "staged.txt");

    let info = get_git_info(dir.path(), GitInfoLevel::Standard).unwrap();
    assert_eq!(info.staged_count, 1);
}

#[test]
fn when_standard_level_should_include_unstaged_count() {
    let dir = create_test_repo();
    create_file(&dir, "file.txt", "content");
    git_add(&dir, "file.txt");
    git_commit(&dir, "Initial commit");

    // Modify without staging
    create_file(&dir, "file.txt", "modified");

    let info = get_git_info(dir.path(), GitInfoLevel::Standard).unwrap();
    assert_eq!(info.unstaged_count, 1);
}

#[test]
fn when_detailed_level_should_include_modified_files() {
    let dir = create_test_repo();
    create_file(&dir, "file.txt", "content");
    git_add(&dir, "file.txt");
    git_commit(&dir, "Initial commit");

    // Modify file
    create_file(&dir, "file.txt", "modified");

    let info = get_git_info(dir.path(), GitInfoLevel::Detailed).unwrap();
    assert!(info.modified_files.contains(&"file.txt".to_string()));
}

#[test]
fn when_formatting_minimal_dirty_should_show_asterisk() {
    let info = GitInfo {
        branch: Some("main".to_string()),
        is_dirty: true,
        ..Default::default()
    };
    assert_eq!(info.format_minimal(), "main *");
}

#[test]
fn when_formatting_minimal_clean_should_not_show_asterisk() {
    let info = GitInfo {
        branch: Some("main".to_string()),
        is_dirty: false,
        ..Default::default()
    };
    assert_eq!(info.format_minimal(), "main");
}

#[test]
fn when_formatting_standard_should_include_all_info() {
    let info = GitInfo {
        branch: Some("main".to_string()),
        is_dirty: true,
        ahead: 2,
        behind: 1,
        staged_count: 3,
        unstaged_count: 2,
        ..Default::default()
    };
    let formatted = info.format_standard();
    assert!(formatted.contains("main"));
    assert!(formatted.contains("*"));
    assert!(formatted.contains("+2"));
    assert!(formatted.contains("-1"));
    assert!(formatted.contains("3S"));
    assert!(formatted.contains("2U"));
}
