# Stage 0: Project Bootstrap - Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Create the base Rust project structure with functional CLI that distinguishes between `gz-claude` and `gz-claude panel` modes.

**Architecture:** Single binary with subcommands using clap. Modular structure prepared for the following stages (config, tui, zellij, git).

**Tech Stack:** Rust 1.75+, clap 4.x (derive), thiserror

---

## Task 1: Create Cargo.toml with dependencies

**Files:**
- Create: `Cargo.toml`

**Step 1: Create Cargo.toml file**

```toml
[package]
name = "gz-claude"
version = "0.1.0"
edition = "2021"
description = "TUI for orchestrating Zellij workspaces with Claude Code"
authors = ["waabox"]
license = "MIT"

[dependencies]
# CLI
clap = { version = "4.5", features = ["derive"] }

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Config (Stage 1)
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"

# TUI (Stage 3)
ratatui = "0.29"
crossterm = "0.28"

# Git (Stage 2)
git2 = "0.19"

# Async (Stage 4)
tokio = { version = "1.43", features = ["full"] }

# Directories
dirs = "6.0"

[dev-dependencies]
tempfile = "3.15"
assert_cmd = "2.0"
predicates = "3.1"

[profile.release]
lto = true
strip = true
```

**Step 2: Verify compilation**

Run: `cargo check`
Expected: Compiles without errors (downloads dependencies)

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: initialize Cargo.toml with dependencies"
```

---

## Task 2: Create directory structure

**Files:**
- Create: `src/main.rs`
- Create: `src/cli.rs`
- Create: `src/config/mod.rs`
- Create: `src/tui/mod.rs`
- Create: `src/zellij/mod.rs`
- Create: `src/git/mod.rs`
- Create: `src/error.rs`

**Step 1: Create initial main.rs**

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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Panel) => {
            println!("Running in panel mode (inside Zellij)");
        }
        None => {
            println!("Starting gz-claude...");
        }
    }
}
```

**Step 2: Create cli.rs with clap**

```rust
//! CLI argument parsing for gz-claude.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use clap::{Parser, Subcommand};

/// TUI for orchestrating Zellij workspaces with Claude Code.
#[derive(Parser, Debug)]
#[command(name = "gz-claude")]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Force enable web client
    #[arg(long)]
    pub web: bool,

    /// Force disable web client
    #[arg(long, conflicts_with = "web")]
    pub no_web: bool,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the TUI panel (inside Zellij)
    Panel,
}
```

**Step 3: Create error.rs**

```rust
//! Error types for gz-claude.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use thiserror::Error;

#[derive(Error, Debug)]
pub enum GzClaudeError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Zellij error: {0}")]
    Zellij(String),

    #[error("Git error: {0}")]
    Git(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GzClaudeError>;
```

**Step 4: Create placeholder modules**

`src/config/mod.rs`:
```rust
//! Configuration parsing and validation.
//!
//! @author waabox(waabox[at]gmail[dot]com)
```

`src/tui/mod.rs`:
```rust
//! TUI components using ratatui.
//!
//! @author waabox(waabox[at]gmail[dot]com)
```

`src/zellij/mod.rs`:
```rust
//! Zellij CLI interaction.
//!
//! @author waabox(waabox[at]gmail[dot]com)
```

`src/git/mod.rs`:
```rust
//! Git repository information using git2.
//!
//! @author waabox(waabox[at]gmail[dot]com)
```

**Step 5: Verify compilation**

Run: `cargo build`
Expected: Compiles without errors

**Step 6: Commit**

```bash
git add src/
git commit -m "feat: add project structure with CLI skeleton"
```

---

## Task 3: Add CLI integration tests

**Files:**
- Create: `tests/cli_test.rs`

**Step 1: Create CLI test**

```rust
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
```

**Step 2: Run tests**

Run: `cargo test`
Expected: 5 tests pass

**Step 3: Commit**

```bash
git add tests/
git commit -m "test: add CLI integration tests"
```

---

## Task 4: Create .gitignore and project files

**Files:**
- Create: `.gitignore`
- Create: `rust-toolchain.toml`

**Step 1: Create .gitignore**

```gitignore
# Rust
/target/
Cargo.lock

# IDE
.idea/
.vscode/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Test artifacts
*.log
```

**Step 2: Create rust-toolchain.toml**

```toml
[toolchain]
channel = "stable"
```

**Step 3: Commit**

```bash
git add .gitignore rust-toolchain.toml
git commit -m "chore: add .gitignore and rust-toolchain.toml"
```

---

## Task 5: Create CLAUDE.md for the project

**Files:**
- Create: `CLAUDE.md`

**Step 1: Create CLAUDE.md**

```markdown
# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

gz-claude is a Rust TUI that orchestrates Zellij workspaces with Claude Code integration. It provides a panel-based interface for managing multiple projects across workspaces.

## Build Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run
cargo run

# Run panel mode (inside Zellij)
cargo run -- panel

# Run with web client
cargo run -- --web

# Run tests
cargo test

# Run specific test
cargo test test_name

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## Architecture

```
src/
├── main.rs      # Entry point, CLI dispatch
├── cli.rs       # clap argument definitions
├── error.rs     # Error types with thiserror
├── config/      # Configuration parsing (config.toml)
├── tui/         # ratatui TUI components
├── zellij/      # Zellij CLI interaction
└── git/         # git2 wrappers for repo info
```

## Design Document

See `docs/plans/2026-02-03-gz-claude-design.md` for the full design specification.

## Code Style

- Follow Rust idioms
- Use `thiserror` for error types
- Prefer explicit error handling over `unwrap()`
- Document public functions with `///` comments
- Author tag: `@author waabox(waabox[at]gmail[dot]com)`
```

**Step 2: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: add CLAUDE.md for Claude Code guidance"
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

1. `chore: initialize Cargo.toml with dependencies`
2. `feat: add project structure with CLI skeleton`
3. `test: add CLI integration tests`
4. `chore: add .gitignore and rust-toolchain.toml`
5. `docs: add CLAUDE.md for Claude Code guidance`
