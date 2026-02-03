//! Session state management for gz-claude.
//!
//! Tracks Zellij panes associated with projects, enabling:
//! - Focus existing panes instead of creating duplicates
//! - Session restoration on restart
//!
//! @author waabox(waabox[at]gmail[dot]com)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::Result;

/// Session state file name.
const SESSION_FILE: &str = "session.json";

/// Information about an open pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneInfo {
    /// Unique name for the pane (used for Zellij identification).
    pub pane_name: String,
    /// The command running in the pane.
    pub command: String,
}

/// Session state tracking open panes and Zellij session.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Session {
    /// Name of the Zellij session.
    pub zellij_session: String,
    /// Map of project paths to their pane info.
    pub panes: HashMap<PathBuf, PaneInfo>,
}

impl Session {
    /// Create a new session with the given Zellij session name.
    pub fn new(zellij_session: String) -> Self {
        Self {
            zellij_session,
            panes: HashMap::new(),
        }
    }

    /// Returns the path to the session file.
    pub fn session_path() -> PathBuf {
        Config::default_dir().join(SESSION_FILE)
    }

    /// Check if a session file exists.
    pub fn exists() -> bool {
        Self::session_path().exists()
    }

    /// Load session from file.
    ///
    /// # Returns
    ///
    /// The loaded session, or None if file doesn't exist or is invalid.
    pub fn load() -> Option<Self> {
        let path = Self::session_path();
        if !path.exists() {
            return None;
        }

        let content = fs::read_to_string(&path).ok()?;
        serde_json::from_str(&content).ok()
    }

    /// Save session to file.
    ///
    /// # Returns
    ///
    /// Ok(()) if saved successfully.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = Self::session_path();

        // Ensure directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&path, content)?;

        Ok(())
    }

    /// Delete the session file.
    pub fn delete() -> Result<()> {
        let path = Self::session_path();
        if path.exists() {
            fs::remove_file(&path)?;
        }
        Ok(())
    }

    /// Register a pane for a project.
    ///
    /// # Arguments
    ///
    /// * `project_path` - The project directory path
    /// * `pane_name` - Unique name for the Zellij pane
    /// * `command` - The command running in the pane
    pub fn register_pane(&mut self, project_path: PathBuf, pane_name: String, command: String) {
        self.panes.insert(
            project_path,
            PaneInfo {
                pane_name,
                command,
            },
        );
    }

    /// Get pane info for a project.
    ///
    /// # Arguments
    ///
    /// * `project_path` - The project directory path
    ///
    /// # Returns
    ///
    /// The pane info if the project has an open pane.
    pub fn get_pane(&self, project_path: &PathBuf) -> Option<&PaneInfo> {
        self.panes.get(project_path)
    }

    /// Remove a pane registration.
    ///
    /// # Arguments
    ///
    /// * `project_path` - The project directory path
    pub fn remove_pane(&mut self, project_path: &PathBuf) {
        self.panes.remove(project_path);
    }

    /// Generate a unique pane name for a project.
    ///
    /// # Arguments
    ///
    /// * `project_path` - The project directory path
    ///
    /// # Returns
    ///
    /// A unique pane name based on the project path.
    pub fn generate_pane_name(project_path: &PathBuf) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        project_path.hash(&mut hasher);
        let hash = hasher.finish();

        format!("gz-{:x}", hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn when_creating_session_should_have_empty_panes() {
        let session = Session::new("test-session".to_string());
        assert_eq!(session.zellij_session, "test-session");
        assert!(session.panes.is_empty());
    }

    #[test]
    fn when_registering_pane_should_store_info() {
        let mut session = Session::new("test-session".to_string());
        let path = PathBuf::from("/test/project");

        session.register_pane(path.clone(), "gz-abc123".to_string(), "claude".to_string());

        let pane = session.get_pane(&path).unwrap();
        assert_eq!(pane.pane_name, "gz-abc123");
        assert_eq!(pane.command, "claude");
    }

    #[test]
    fn when_generating_pane_name_should_be_deterministic() {
        let path = PathBuf::from("/test/project");
        let name1 = Session::generate_pane_name(&path);
        let name2 = Session::generate_pane_name(&path);
        assert_eq!(name1, name2);
        assert!(name1.starts_with("gz-"));
    }

    #[test]
    fn when_removing_pane_should_no_longer_exist() {
        let mut session = Session::new("test-session".to_string());
        let path = PathBuf::from("/test/project");

        session.register_pane(path.clone(), "gz-abc123".to_string(), "claude".to_string());
        session.remove_pane(&path);

        assert!(session.get_pane(&path).is_none());
    }
}
