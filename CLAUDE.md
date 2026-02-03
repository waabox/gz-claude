# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

gz-claude is a Rust TUI that orchestrates Zellij workspaces with Claude Code integration. It provides a panel-based interface for managing multiple projects across workspaces with drill-down navigation.

## Build Commands

```bash
# Build
cargo build

# Build release
cargo build --release

# Run (starts Zellij with layout)
cargo run

# Run panel mode (inside Zellij)
cargo run -- panel

# Run top bar mode (inside Zellij)
cargo run -- topbar

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
├── main.rs       # Entry point, CLI dispatch
├── cli.rs        # clap argument definitions
├── error.rs      # Error types with thiserror
├── config/       # Configuration parsing (config.json)
│   ├── mod.rs        # Config structs, parsing, validation
│   └── tests.rs      # Configuration tests
├── tui/          # ratatui TUI components
│   ├── mod.rs        # Module exports
│   ├── app.rs        # Application state machine
│   ├── runner.rs     # Main event loop
│   ├── terminal.rs   # Terminal setup/teardown
│   ├── file_tree.rs  # File tree component
│   └── views/        # View components
│       ├── mod.rs
│       ├── workspaces.rs  # View 1: Workspaces list
│       ├── projects.rs    # View 2: Projects list
│       └── file_browser.rs # View 3: File browser
├── zellij/       # Zellij CLI interaction
│   ├── mod.rs        # Module exports
│   ├── commands.rs   # zellij action commands
│   ├── layout.rs     # KDL layout generation
│   ├── check.rs      # Zellij environment detection
│   └── web.rs        # Web client management
├── session/      # Session state management
│   └── mod.rs
└── git/          # git2 wrappers
    ├── mod.rs        # Git info extraction
    └── tests.rs      # Git tests
```

## Key Concepts

### Execution Modes

- `gz-claude` - Main entry: validates config, generates layout, starts Zellij
- `gz-claude panel` - TUI panel mode: runs inside Zellij left pane
- `gz-claude topbar` - Top bar mode: runs inside Zellij
- `--web` / `--no-web` - Force enable/disable web client

### Configuration

Located at `~/.gz-claude/config.json`. Actions inherit hierarchically:
1. Global actions (base)
2. Workspace actions (override/extend)
3. Project actions (override/extend)

### Views (Drill-down Navigation)

1. **Workspaces** - List of configured workspaces
2. **Projects** - Projects within workspace + git status + actions
3. **File Browser** - Git info + file tree + actions

## Design Document

See `docs/plans/2026-02-03-gz-claude-design.md` for the full design specification.

## Implementation Status

- Stage 0: Project Bootstrap
- Stage 1: Configuration
- Stage 2: Git Integration
- Stage 3: TUI Components
- Stage 4: Zellij Integration
- Stage 5: Web Client (in progress)

## Code Style

- Follow Rust idioms
- Use `thiserror` for error types
- Prefer explicit error handling over `unwrap()`
- Document public functions with `///` comments
- Author tag: `@author waabox(waabox[at]gmail[dot]com)`
