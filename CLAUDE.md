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
