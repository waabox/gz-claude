# gz-claude - Design Document

> Rust binary that orchestrates Zellij + Web Client + Claude Code with a Workspaces panel.

## Objective

When running `gz-claude`:

1. Opens Zellij with a predefined layout
2. Optionally starts the Zellij Web Client (local network only)
3. Left panel: TUI with Workspaces drill-down navigation
4. Central panel: dynamic terminal panes (Claude, Bash, etc.)

---

## General Architecture

### Single binary `gz-claude`

Execution modes:

```
gz-claude              -> Starts Zellij with the layout, optionally web client
gz-claude panel        -> Runs inside Zellij, renders the TUI
gz-claude --web        -> Override to force web client
gz-claude --no-web     -> Override to disable web client
```

### Startup Flow

1. `gz-claude` (without arguments) validates the configuration
2. If paths are invalid -> error with clear message and exit code 1
3. If all OK -> generates/updates the KDL layout in `~/.config/zellij/layouts/`
4. Launches `zellij --layout=gz-claude`
5. Optionally launches the web server according to config/flags
6. Zellij executes `gz-claude panel` in the left pane

### Rust Dependencies

- `ratatui` - TUI framework
- `crossterm` - terminal backend
- `tokio` - async runtime (for external processes)
- `toml` + `serde` - config parsing
- `git2` - native Git information (no shell out)
- `clap` - CLI argument parsing

### Crate Structure

```
gz-claude/
├── src/
│   ├── main.rs
│   ├── cli.rs          # clap args
│   ├── config/         # parsing and validation
│   ├── tui/            # ratatui components
│   ├── zellij/         # zellij CLI interaction
│   └── git/            # git2 wrappers
```

---

## Configuration

### Main File

`~/.config/gz-claude/config.toml`

```toml
# Global actions (available in all projects)
[global]
editor = "$EDITOR"  # command to open files, default $EDITOR
git_info_level = "minimal"  # minimal | standard | detailed

[global.actions]
c = { name = "Claude", command = "claude", icon = "C" }
b = { name = "Bash", command = "bash", icon = "B" }
g = { name = "Lazygit", command = "lazygit", icon = "G" }

[web_client]
auto_start = false
bind_address = "0.0.0.0"  # or specific IP
port = 8082
# token is auto-generated and saved to ~/.config/gz-claude/web_token

# Workspaces
[workspace.fanki]
name = "Fanki"

[workspace.fanki.actions]
t = { name = "Tests", command = "mvn test", icon = "T" }
d = { name = "Deploy", command = "make deploy", icon = "D" }

[[workspace.fanki.projects]]
name = "API Gateway"
path = "/Users/emiliano/dev/fanki/api-gateway"

[[workspace.fanki.projects]]
name = "Payments"
path = "/Users/emiliano/dev/fanki/payments"
[workspace.fanki.projects.actions]
t = { name = "Tests", command = "gradle test", icon = "T" }  # workspace override
s = { name = "Swagger", command = "make swagger", icon = "S" }  # extra action
```

### Action Resolution (Inheritance)

1. Global actions are loaded
2. Workspace actions are merged (override by key)
3. Project actions are merged (override by key)

### Startup Validation

- All paths must exist and be directories
- Action keys must be a single character
- Commands cannot be empty

---

## TUI - Navigation and Views

### Hierarchical Navigation (Drill-Down)

- View 1: Workspaces list -> Enter goes into the workspace
- View 2: Workspace projects list -> Enter goes into the project (file browser)
- View 3: Git info + project file tree + actions
- Backspace/Esc to go back

### View 1: Workspaces

```
┌─ gz-claude ─────────────────────┐
│                                 │
│  Workspaces                     │
│                                 │
│  > Fanki                        │
│    Helios                       │
│    Personal                     │
│                                 │
├─────────────────────────────────┤
│ Enter: select  q: quit          │
└─────────────────────────────────┘
```

### View 2: Workspace Projects

```
┌─ Fanki ─────────────────────────────┐
│                                     │
│  Projects                           │
│                                     │
│  > API Gateway      main *  C B     │
│    Payments         develop C B     │
│    Tickets          main    C B     │
│                                     │
├─────────────────────────────────────┤
│ Enter: browse  Cc:Claude  Bb:Bash   │
│ Esc: back  q: quit                  │
└─────────────────────────────────────┘
```

- `Enter` on a project: enters View 3 (project file browser)
- `c` with project selected: opens Claude in new pane with `cwd = project.path`
- `b` with project selected: opens Bash in new pane with `cwd = project.path`
- Icons are configurable with emojis by default

### View 3: Project File Browser

```
┌─ API Gateway ───────────────────────┐
│ main * │ +2 -1 │ 3 staged           │
├─────────────────────────────────────┤
│  > src/                             │
│      main/                          │
│      test/                          │
│    pom.xml                          │
│    README.md                        │
├─────────────────────────────────────┤
│ Cc Bb Gg Tt │ Enter: open/expand    │
│ Esc: back                           │
└─────────────────────────────────────┘
```

- `Enter` on folder: expand/collapse
- `Enter` on file: opens in new pane with `$EDITOR`
- Configured actions remain available

### Controls

- `↑/↓` or `j/k`: navigate
- `Enter`: select / open file / expand folder
- `Esc` or `Backspace`: go back
- `r`: refresh git info
- `q`: quit (only in View 1)
- Action keys: execute command in new pane

### Git Info Levels

Configurable with `git_info_level`:

- **minimal**: Current branch + dirty indicator (`main *`)
- **standard**: Branch + dirty + ahead/behind + staged/unstaged count
- **detailed**: All of the above + list of modified files

---

## Zellij Integration

### Layout Generation

When running `gz-claude`, it generates `~/.config/zellij/layouts/gz-claude.kdl`:

```kdl
layout {
    pane size=1 borderless=true {
        plugin location="zellij:tab-bar"
    }

    pane split_direction="vertical" {
        pane size=40 command="gz-claude" {
            args "panel"
        }
        pane focus=true command="bash"
    }

    pane size=1 borderless=true {
        plugin location="zellij:status-bar"
    }
}
```

### Actions from TUI

Open pane with command:

```bash
zellij action new-pane --cwd "/path/to/project" -- claude
```

Open file with editor:

```bash
zellij action new-pane --cwd "/path/to/project" -- $EDITOR file.rs
```

### Web Client

If `web_client.auto_start = true` or `--web` is used:

```bash
zellij web --listen 192.168.1.100:8082
```

The token is generated once and saved to `~/.config/gz-claude/web_token`. The TUI shows the complete URL when the web server is active.

### Zellij Detection

- `gz-claude panel` verifies it runs inside Zellij (`ZELLIJ` variable)
- If not in Zellij, shows error and suggests running `gz-claude` without arguments

---

## Validation and Error Handling

### At `gz-claude` Startup

1. **Find config:** `~/.config/gz-claude/config.toml`
   - If not exists -> create example config and show message

2. **Validate TOML structure:**
   - Syntax errors -> show line and column of error

3. **Validate project paths:**
   ```
   Error: Invalid configuration

   The following project paths do not exist:

     - Fanki / API Gateway
       /Users/emiliano/dev/fanki/api-gateway

     - Helios / Backend
       /Users/emiliano/dev/helios/backend

   Please fix these paths in ~/.config/gz-claude/config.toml
   ```

4. **Validate actions:**
   - Duplicate keys at same level -> error
   - Key is not a single character -> error
   - Empty command -> error

5. **Validate Zellij installed:**
   ```
   Error: Zellij not found

   gz-claude requires Zellij to be installed.
   Install it from: https://zellij.dev/documentation/installation
   ```

### Inside the TUI

- Git errors -> show `[git error]` instead of branch
- Action execution error -> show temporary notification in TUI
- Refresh (`r`) fails -> show message, keep last known state

---

## Implementation Plan

### Stage 0: Project Bootstrap
- Create `Cargo.toml` with dependencies
- Directory structure
- Basic CLI with clap (`gz-claude`, `gz-claude panel`)

### Stage 1: Configuration
- Config structs with serde
- `config.toml` parsing
- Complete validation (paths, actions, keys)
- Example config generation
- Unit tests for parsing and validation

### Stage 2: Git
- Wrapper over `git2`
- Get: branch, dirty status, ahead/behind, staged/unstaged count
- Configurable detail levels
- Tests with test repos

### Stage 3: TUI
- Setup ratatui + crossterm
- View 1: Workspaces
- View 2: Projects with icons
- View 3: File browser
- Drill-down navigation
- Dynamic action bar

### Stage 4: Zellij Integration
- Generate KDL layout
- Execute `zellij action new-pane`
- Open files with editor
- Detect Zellij environment

### Stage 5: Web Client
- Token management
- Conditional web server startup
- Show URL in TUI when active

Each stage ends with testable functionality and commit.
