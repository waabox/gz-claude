//! Git repository information using git2.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use std::path::Path;

use git2::{Repository, Status, StatusOptions};

/// Information about a Git repository.
#[derive(Debug, Clone, Default)]
pub struct GitInfo {
    /// Current branch name (None if detached HEAD).
    pub branch: Option<String>,
    /// Whether there are uncommitted changes.
    pub is_dirty: bool,
    /// Number of commits ahead of upstream.
    pub ahead: u32,
    /// Number of commits behind upstream.
    pub behind: u32,
    /// Number of staged files.
    pub staged_count: u32,
    /// Number of unstaged modified files.
    pub unstaged_count: u32,
    /// List of modified files (only populated for detailed level).
    pub modified_files: Vec<String>,
}

impl GitInfo {
    /// Format as minimal string: "main *" or "main".
    pub fn format_minimal(&self) -> String {
        let branch = self.branch.as_deref().unwrap_or("HEAD");
        if self.is_dirty {
            format!("{} *", branch)
        } else {
            branch.to_string()
        }
    }

    /// Format as standard string: "main * | +2 -1 | 3S 2U".
    pub fn format_standard(&self) -> String {
        let branch = self.branch.as_deref().unwrap_or("HEAD");
        let dirty = if self.is_dirty { " *" } else { "" };
        let ahead_behind = if self.ahead > 0 || self.behind > 0 {
            format!(" | +{} -{}", self.ahead, self.behind)
        } else {
            String::new()
        };
        let staged_unstaged = if self.staged_count > 0 || self.unstaged_count > 0 {
            format!(" | {}S {}U", self.staged_count, self.unstaged_count)
        } else {
            String::new()
        };
        format!("{}{}{}{}", branch, dirty, ahead_behind, staged_unstaged)
    }
}

/// Get the current branch name from a repository.
fn get_current_branch(repo: &Repository) -> Option<String> {
    let head = repo.head().ok()?;
    if head.is_branch() {
        head.shorthand().map(String::from)
    } else {
        // Detached HEAD
        None
    }
}

/// Check if the repository has uncommitted changes.
fn is_repo_dirty(repo: &Repository) -> bool {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false);

    match repo.statuses(Some(&mut opts)) {
        Ok(statuses) => !statuses.is_empty(),
        Err(_) => false,
    }
}

/// Get the number of commits ahead and behind the upstream branch.
fn get_ahead_behind(repo: &Repository) -> (u32, u32) {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return (0, 0),
    };

    let local_oid = match head.target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    // Get the upstream branch
    let branch_name = match head.shorthand() {
        Some(name) => name,
        None => return (0, 0),
    };

    let branch = match repo.find_branch(branch_name, git2::BranchType::Local) {
        Ok(b) => b,
        Err(_) => return (0, 0),
    };

    let upstream = match branch.upstream() {
        Ok(u) => u,
        Err(_) => return (0, 0), // No upstream configured
    };

    let upstream_oid = match upstream.get().target() {
        Some(oid) => oid,
        None => return (0, 0),
    };

    match repo.graph_ahead_behind(local_oid, upstream_oid) {
        Ok((ahead, behind)) => (ahead as u32, behind as u32),
        Err(_) => (0, 0),
    }
}

/// Count staged and unstaged files.
fn count_staged_unstaged(repo: &Repository) -> (u32, u32) {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return (0, 0),
    };

    let mut staged = 0u32;
    let mut unstaged = 0u32;

    for entry in statuses.iter() {
        let status = entry.status();

        // Staged changes (index)
        if status.intersects(
            Status::INDEX_NEW
                | Status::INDEX_MODIFIED
                | Status::INDEX_DELETED
                | Status::INDEX_RENAMED
                | Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }

        // Unstaged changes (workdir)
        if status.intersects(
            Status::WT_NEW
                | Status::WT_MODIFIED
                | Status::WT_DELETED
                | Status::WT_RENAMED
                | Status::WT_TYPECHANGE,
        ) {
            unstaged += 1;
        }
    }

    (staged, unstaged)
}

/// Get list of modified files (for detailed level).
fn get_modified_files(repo: &Repository) -> Vec<String> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true)
        .recurse_untracked_dirs(false);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    statuses
        .iter()
        .filter_map(|entry| entry.path().map(String::from))
        .collect()
}

/// Open a Git repository at the given path.
/// Returns None if the path is not a Git repository.
pub fn open_repo(path: &Path) -> Option<Repository> {
    Repository::open(path).ok()
}
