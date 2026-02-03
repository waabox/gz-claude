//! File tree model for the file browser.
//!
//! Provides data structures for representing a file system tree with
//! expandable directories and a flattened view for rendering in the TUI.
//!
//! @author waabox(waabox[at]gmail[dot]com)

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// Represents a node in the file tree.
///
/// Each node corresponds to either a file or a directory in the file system.
/// Directories can be expanded to show their children.
#[derive(Debug, Clone)]
pub struct FileNode {
    /// The name of the file or directory (not the full path).
    pub name: String,
    /// The full path to this file or directory.
    pub path: PathBuf,
    /// Whether this node is a directory.
    pub is_dir: bool,
    /// Whether this directory is expanded (only meaningful for directories).
    pub expanded: bool,
    /// The depth of this node in the tree (root is 0).
    pub depth: usize,
    /// Child nodes (only populated for directories).
    pub children: Vec<FileNode>,
}

impl FileNode {
    /// Creates a new FileNode from a path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file or directory
    /// * `depth` - The depth of this node in the tree
    ///
    /// # Returns
    ///
    /// Some(FileNode) if the path exists and can be read, None otherwise.
    pub fn new(path: &Path, depth: usize) -> Option<Self> {
        let metadata = fs::metadata(path).ok()?;
        let name = path.file_name()?.to_str()?.to_string();

        Some(Self {
            name,
            path: path.to_path_buf(),
            is_dir: metadata.is_dir(),
            expanded: false,
            depth,
            children: Vec::new(),
        })
    }

    /// Loads children for this directory node.
    ///
    /// Reads the directory contents, filters out hidden files (starting with '.'),
    /// and sorts children with directories first, then alphabetically (case-insensitive).
    ///
    /// Does nothing if this node is not a directory.
    pub fn load_children(&mut self) {
        if !self.is_dir {
            return;
        }

        let entries = match fs::read_dir(&self.path) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        let mut children: Vec<FileNode> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry
                    .file_name()
                    .to_str()
                    .map(|name| !name.starts_with('.'))
                    .unwrap_or(false)
            })
            .filter_map(|entry| FileNode::new(&entry.path(), self.depth + 1))
            .collect();

        children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        self.children = children;
    }

    /// Toggles the expanded state of this directory node.
    ///
    /// If expanding, loads children if they haven't been loaded yet.
    /// Does nothing if this node is not a directory.
    pub fn toggle_expanded(&mut self) {
        if !self.is_dir {
            return;
        }

        self.expanded = !self.expanded;

        if self.expanded && self.children.is_empty() {
            self.load_children();
        }
    }
}

/// A file tree structure with a flattened view for TUI rendering.
///
/// The tree maintains both the hierarchical structure and a flat list
/// of visible nodes for efficient navigation and rendering.
#[derive(Debug)]
pub struct FileTree {
    /// The root node of the tree.
    pub root: FileNode,
    /// Flattened indices for visible nodes (used for navigation).
    pub flat_list: Vec<FlatNodeRef>,
}

/// Reference to a node in the flattened view.
///
/// Stores the path to reach a node from the root (as indices into children arrays).
#[derive(Debug, Clone)]
pub struct FlatNodeRef {
    /// Path of indices from root to this node.
    pub path: Vec<usize>,
}

impl FileTree {
    /// Creates a new FileTree rooted at the specified path.
    ///
    /// The root is automatically expanded to show its immediate children.
    ///
    /// # Arguments
    ///
    /// * `root_path` - The path to use as the root of the tree
    ///
    /// # Returns
    ///
    /// Some(FileTree) if the root path exists and is a directory, None otherwise.
    pub fn new(root_path: &Path) -> Option<Self> {
        let mut root = FileNode::new(root_path, 0)?;

        if !root.is_dir {
            return None;
        }

        root.expanded = true;
        root.load_children();

        let mut tree = Self {
            root,
            flat_list: Vec::new(),
        };

        tree.rebuild_flat_list();

        Some(tree)
    }

    /// Rebuilds the flat list of visible nodes.
    ///
    /// Traverses the tree and collects references to all currently visible nodes.
    fn rebuild_flat_list(&mut self) {
        let mut flat_list = Vec::new();

        // Add root
        flat_list.push(FlatNodeRef { path: vec![] });

        // Recursively add visible children
        if self.root.expanded {
            Self::collect_visible_children(&mut flat_list, &[], &self.root.children);
        }

        self.flat_list = flat_list;
    }

    /// Recursively collects visible children into the flat list.
    fn collect_visible_children(
        flat_list: &mut Vec<FlatNodeRef>,
        parent_path: &[usize],
        children: &[FileNode],
    ) {
        for (index, child) in children.iter().enumerate() {
            let mut child_path = parent_path.to_vec();
            child_path.push(index);

            flat_list.push(FlatNodeRef {
                path: child_path.clone(),
            });

            if child.is_dir && child.expanded {
                Self::collect_visible_children(flat_list, &child_path, &child.children);
            }
        }
    }

    /// Returns the number of currently visible nodes.
    pub fn visible_count(&self) -> usize {
        self.flat_list.len()
    }

    /// Gets a reference to the visible node at the specified index.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the flattened visible list
    ///
    /// # Returns
    ///
    /// Some(&FileNode) if the index is valid, None otherwise.
    pub fn get_visible_node(&self, index: usize) -> Option<&FileNode> {
        let node_ref = self.flat_list.get(index)?;
        self.get_node_by_path(&node_ref.path)
    }

    /// Gets a node by its path from the root.
    fn get_node_by_path(&self, path: &[usize]) -> Option<&FileNode> {
        if path.is_empty() {
            return Some(&self.root);
        }

        let mut current = &self.root;
        for &idx in path {
            current = current.children.get(idx)?;
        }
        Some(current)
    }

    /// Gets a mutable node by its path from the root.
    fn get_node_by_path_mut(&mut self, path: &[usize]) -> Option<&mut FileNode> {
        if path.is_empty() {
            return Some(&mut self.root);
        }

        let mut current = &mut self.root;
        for &idx in path {
            current = current.children.get_mut(idx)?;
        }
        Some(current)
    }

    /// Toggles the expand/collapse state of the node at the specified index.
    ///
    /// If the node is a directory, it will be expanded or collapsed.
    /// The flat list is rebuilt after toggling.
    ///
    /// # Arguments
    ///
    /// * `index` - The index in the flattened visible list
    pub fn toggle_at(&mut self, index: usize) {
        let path = match self.flat_list.get(index) {
            Some(node_ref) => node_ref.path.clone(),
            None => return,
        };

        if let Some(node) = self.get_node_by_path_mut(&path) {
            node.toggle_expanded();
        }

        self.rebuild_flat_list();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let dir = TempDir::new().unwrap();
        let root = dir.path();

        // Create structure:
        // root/
        //   src/
        //     main.rs
        //   README.md
        fs::create_dir(root.join("src")).unwrap();
        fs::File::create(root.join("src/main.rs")).unwrap();
        fs::File::create(root.join("README.md")).unwrap();

        dir
    }

    #[test]
    fn when_creating_file_tree_should_load_root_children() {
        let temp_dir = setup_test_dir();

        let tree = FileTree::new(temp_dir.path()).unwrap();

        assert!(tree.root.expanded);
        assert_eq!(tree.root.children.len(), 2);

        let child_names: Vec<&str> = tree.root.children.iter().map(|c| c.name.as_str()).collect();
        assert!(child_names.contains(&"src"));
        assert!(child_names.contains(&"README.md"));
    }

    #[test]
    fn when_getting_visible_count_should_include_expanded_nodes() {
        let temp_dir = setup_test_dir();

        let mut tree = FileTree::new(temp_dir.path()).unwrap();

        // Initially: root + src + README.md = 3
        assert_eq!(tree.visible_count(), 3);

        // Expand src directory (index 1, since src comes before README.md due to dir-first sorting)
        tree.toggle_at(1);

        // Now: root + src + main.rs + README.md = 4
        assert_eq!(tree.visible_count(), 4);
    }

    #[test]
    fn when_getting_visible_node_should_return_correct_node() {
        let temp_dir = setup_test_dir();

        let tree = FileTree::new(temp_dir.path()).unwrap();

        // Index 0: root
        let root_node = tree.get_visible_node(0).unwrap();
        assert_eq!(root_node.path, temp_dir.path());

        // Index 1: src (directories come first)
        let src_node = tree.get_visible_node(1).unwrap();
        assert_eq!(src_node.name, "src");
        assert!(src_node.is_dir);

        // Index 2: README.md
        let readme_node = tree.get_visible_node(2).unwrap();
        assert_eq!(readme_node.name, "README.md");
        assert!(!readme_node.is_dir);

        // Index 3: out of bounds
        assert!(tree.get_visible_node(3).is_none());
    }

    #[test]
    fn when_sorting_children_should_put_directories_first() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create structure with mixed files and dirs:
        // root/
        //   zebra/        (dir)
        //   apple/        (dir)
        //   banana.txt    (file)
        //   aardvark.txt  (file)
        fs::create_dir(root.join("zebra")).unwrap();
        fs::create_dir(root.join("apple")).unwrap();
        fs::File::create(root.join("banana.txt")).unwrap();
        fs::File::create(root.join("aardvark.txt")).unwrap();

        let tree = FileTree::new(root).unwrap();

        let children = &tree.root.children;
        assert_eq!(children.len(), 4);

        // Directories first (alphabetically): apple, zebra
        assert_eq!(children[0].name, "apple");
        assert!(children[0].is_dir);
        assert_eq!(children[1].name, "zebra");
        assert!(children[1].is_dir);

        // Files next (alphabetically): aardvark.txt, banana.txt
        assert_eq!(children[2].name, "aardvark.txt");
        assert!(!children[2].is_dir);
        assert_eq!(children[3].name, "banana.txt");
        assert!(!children[3].is_dir);
    }
}
