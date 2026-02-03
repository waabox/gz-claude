//! Zellij KDL layout generation utilities.
//!
//! Provides functions to generate and manage the gz-claude Zellij layout file,
//! which defines the pane arrangement for the integrated development environment.
//!
//! @author waabox(waabox[at]gmail[dot]com)

use std::fs;
use std::path::PathBuf;

use crate::error::Result;

/// KDL layout template for the gz-claude Zellij workspace.
///
/// This layout creates a four-row structure:
/// - Row 1: Tab bar plugin (borderless, 1 row)
/// - Row 2: gz-claude top bar with web URL (borderless, 1 row)
/// - Row 3: Vertical split with gz-claude panel (40 cols) and focused shell
/// - Row 4: Status bar plugin (borderless, 1 row)
pub const LAYOUT_TEMPLATE: &str = r#"layout {
    pane size=1 borderless=true {
        plugin location="zellij:tab-bar"
    }

    pane size=1 borderless=true command="gz-claude" {
        args "top-bar"
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
"#;

/// Returns the path to the Zellij layouts directory.
///
/// The layouts directory is located at `~/.config/zellij/layouts/`.
///
/// # Returns
///
/// Returns a `PathBuf` pointing to the Zellij layouts directory.
///
/// # Panics
///
/// Panics if the user's home directory cannot be determined.
pub fn layouts_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Unable to determine home directory")
        .join(".config")
        .join("zellij")
        .join("layouts")
}

/// Returns the path to the gz-claude layout file.
///
/// The layout file is located at `~/.config/zellij/layouts/gz-claude.kdl`.
///
/// # Returns
///
/// Returns a `PathBuf` pointing to the gz-claude layout file.
pub fn layout_path() -> PathBuf {
    layouts_dir().join("gz-claude.kdl")
}

/// Checks whether the gz-claude layout file exists.
///
/// # Returns
///
/// Returns `true` if the layout file exists at the expected path, `false` otherwise.
///
/// # Example
///
/// ```no_run
/// use gz_claude::zellij::layout_exists;
///
/// if !layout_exists() {
///     println!("Layout file needs to be generated");
/// }
/// ```
pub fn layout_exists() -> bool {
    layout_path().exists()
}

/// Generates the gz-claude Zellij layout file.
///
/// Creates the layouts directory if it does not exist and writes the KDL layout
/// template to `~/.config/zellij/layouts/gz-claude.kdl`.
///
/// # Returns
///
/// Returns `Ok(PathBuf)` containing the path to the generated layout file on success,
/// or an error if directory creation or file writing fails.
///
/// # Errors
///
/// Returns an error if:
/// - The layouts directory cannot be created
/// - The layout file cannot be written
///
/// # Example
///
/// ```no_run
/// use gz_claude::zellij::generate_layout;
///
/// match generate_layout() {
///     Ok(path) => println!("Layout generated at: {}", path.display()),
///     Err(e) => eprintln!("Failed to generate layout: {}", e),
/// }
/// ```
pub fn generate_layout() -> Result<PathBuf> {
    let dir = layouts_dir();
    fs::create_dir_all(&dir)?;

    let path = layout_path();
    fs::write(&path, LAYOUT_TEMPLATE)?;

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn when_generating_layout_should_create_file() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let layouts_path = temp_dir.path().join("layouts");
        let layout_file = layouts_path.join("gz-claude.kdl");

        fs::create_dir_all(&layouts_path).expect("Failed to create layouts directory");
        fs::write(&layout_file, LAYOUT_TEMPLATE).expect("Failed to write layout file");

        assert!(layout_file.exists());

        let content = fs::read_to_string(&layout_file).expect("Failed to read layout file");
        assert_eq!(content, LAYOUT_TEMPLATE);
    }

    #[test]
    fn when_layout_template_should_contain_required_elements() {
        assert!(LAYOUT_TEMPLATE.contains("layout {"));
        assert!(LAYOUT_TEMPLATE.contains("plugin location=\"zellij:tab-bar\""));
        assert!(LAYOUT_TEMPLATE.contains("plugin location=\"zellij:status-bar\""));
        assert!(LAYOUT_TEMPLATE.contains("split_direction=\"vertical\""));
        assert!(LAYOUT_TEMPLATE.contains("command=\"gz-claude\""));
        assert!(LAYOUT_TEMPLATE.contains("args \"panel\""));
        assert!(LAYOUT_TEMPLATE.contains("args \"top-bar\""));
        assert!(LAYOUT_TEMPLATE.contains("command=\"bash\""));
        assert!(LAYOUT_TEMPLATE.contains("focus=true"));
        assert!(LAYOUT_TEMPLATE.contains("size=40"));
        assert!(LAYOUT_TEMPLATE.contains("borderless=true"));
    }
}
