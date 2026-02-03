# gz-claude

A Rust TUI for orchestrating Zellij workspaces with Claude Code integration.

## Features

- **Workspace Management**: Organize projects into logical workspaces with hierarchical navigation
- **Panel-based TUI**: Navigate workspaces, projects, and files in a drill-down interface
- **Zellij Integration**: Automatic layout generation and pane management
- **Git Information**: Branch status, dirty indicators, ahead/behind tracking
- **Configurable Actions**: Define custom shortcuts for Claude, Bash, Lazygit, and more
- **Web Client Support**: Optional Zellij web client for remote access

## Installation

```bash
# Clone the repository
git clone https://github.com/waabox/gz-claude.git
cd gz-claude

# Build release binary
cargo build --release

# Install (optional)
cargo install --path .
```

### Requirements

- Rust 1.75+
- [Zellij](https://zellij.dev/) terminal multiplexer

## Usage

```bash
# Start gz-claude (opens Zellij with layout)
gz-claude

# Run with web client enabled
gz-claude --web

# Run with web client disabled
gz-claude --no-web

# Run panel mode (inside Zellij - called automatically by layout)
gz-claude panel

# Run top bar mode (inside Zellij)
gz-claude topbar
```

## Configuration

Configuration file: `~/.config/gz-claude/config.toml`

```toml
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

[workspace.mywork]
name = "My Work"

[[workspace.mywork.projects]]
name = "Project A"
path = "/path/to/project-a"

[[workspace.mywork.projects]]
name = "Project B"
path = "/path/to/project-b"
```

### Action Inheritance

Actions are resolved hierarchically:
1. Global actions (base)
2. Workspace actions (override/extend)
3. Project actions (override/extend)

## Navigation

| Key | Action |
|-----|--------|
| `j/k` or arrows | Navigate up/down |
| `Enter` | Select / Open / Expand |
| `Esc` or `Backspace` | Go back |
| `r` | Refresh git info |
| `q` | Quit (workspaces view only) |
| Action keys | Execute configured action |

## Views

1. **Workspaces**: List of configured workspaces
2. **Projects**: Projects within a workspace with git status and action shortcuts
3. **File Browser**: Git info, file tree, and available actions

## Architecture

```
src/
├── main.rs       # Entry point, CLI dispatch
├── cli.rs        # clap argument definitions
├── error.rs      # Error types with thiserror
├── config/       # Configuration parsing (config.toml)
├── tui/          # ratatui TUI components
│   ├── app.rs        # Application state
│   ├── runner.rs     # Event loop
│   ├── terminal.rs   # Terminal setup
│   ├── file_tree.rs  # File tree component
│   └── views/        # View components
├── zellij/       # Zellij CLI interaction
│   ├── commands.rs   # Zellij action commands
│   ├── layout.rs     # KDL layout generation
│   ├── check.rs      # Environment detection
│   └── web.rs        # Web client management
├── session/      # Session management
└── git/          # git2 wrappers for repo info
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Check without building
cargo check

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT

## Author

@author waabox(waabox[at]gmail[dot]com)
