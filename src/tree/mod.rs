use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[cfg(test)]
mod path_prefix_test;

// Reuse the common traversal logic
use crate::paths::remove_path_prefix;
use crate::telemetry::{LogMessage, log_with_context};
use crate::traverse::common::{build_walk, is_hidden_path};

/// Configuration options for directory tree operations.
#[derive(Debug, Clone)]
pub struct TreeOptions {
    /// Whether file path matching should be case sensitive
    pub case_sensitive: bool,

    /// Whether to respect .gitignore files when determining which files to include
    pub respect_gitignore: bool,

    /// Maximum depth of directory traversal (number of directory levels to explore)
    pub depth: Option<usize>,

    /// Optional path prefix to remove from directory paths in tree results.
    ///
    /// When set to `Some(path)`, this prefix will be removed from the beginning of each directory path in the results.
    /// If a directory path doesn't start with this prefix, it will be left unchanged.
    /// When set to `None` (default), directory paths are returned as-is.
    ///
    /// This is useful when you want to display relative paths instead of full paths in results,
    /// or when you want to normalize paths for consistency.
    ///
    /// # Examples
    ///
    /// - `omit_path_prefix: Some(PathBuf::from("/home/user/projects/myrepo"))` will transform a directory path like
    ///   `/home/user/projects/myrepo/src/util` to `src/util` in the results
    /// - `omit_path_prefix: None` will leave all directory paths unchanged
    pub omit_path_prefix: Option<PathBuf>,
}

impl Default for TreeOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
            depth: Some(20),
            omit_path_prefix: None,
        }
    }
}

/// Represents a directory entry in the tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum Entry {
    #[serde(rename = "file")]
    File { name: String },

    #[serde(rename = "directory")]
    Directory { name: String },
}

/// Represents a directory and its contents in the tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DirectoryTree {
    /// Path to the directory
    pub dir: String,

    /// List of entries in this directory
    pub entries: Vec<Entry>,
}

/// Generates a directory tree structure for the specified directory.
///
/// # Arguments
///
/// * `directory` - The directory path to generate the tree for
/// * `options` - Configuration options for the operation
///
/// # Returns
///
/// A vector of DirectoryTree objects representing the hierarchical structure
///
/// # Errors
///
/// Returns an error if there's an issue accessing the directory or files
pub fn generate_tree(directory: &Path, options: &TreeOptions) -> Result<Vec<DirectoryTree>> {
    // Use the common builder setup from traverse module
    let walker = build_walk(
        directory,
        options.respect_gitignore,
        options.case_sensitive,
        options.depth,
    )?;

    // Map to organize entries by directory
    let mut dirs_map: HashMap<String, Vec<Entry>> = HashMap::new();

    // Process root directory with path prefix removal if configured
    let root_dir_path = if let Some(prefix) = &options.omit_path_prefix {
        let processed_path = remove_path_prefix(directory, prefix);
        processed_path
    } else {
        directory.to_path_buf()
    };

    // Add the root directory as the first entry
    let root_dir_key = root_dir_path.to_string_lossy().to_string();
    dirs_map.insert(root_dir_key.clone(), Vec::new());

    // Process each entry from the walker
    for result in walker {
        let entry = match result {
            Ok(entry) => entry,
            Err(err) => {
                log_with_context(
                    log::Level::Warn,
                    LogMessage {
                        message: format!("Error walking directory: {}", err),
                        module: "tree",
                        context: Some(vec![("directory", directory.display().to_string())]),
                    },
                );
                continue;
            }
        };

        let path = entry.path();

        // Skip the directory itself
        if path == directory {
            continue;
        }

        // Skip if respecting gitignore and this is a hidden path
        if options.respect_gitignore && is_hidden_path(path) {
            continue;
        }

        // Process the path with prefix removal if configured
        let processed_path = if let Some(prefix) = &options.omit_path_prefix {
            remove_path_prefix(path, prefix)
        } else {
            path.to_path_buf()
        };

        // For files directly in the root directory
        if let Some(parent) = path.parent() {
            if parent == directory {
                if path.is_file() {
                    let entry = Entry::File {
                        name: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    };

                    dirs_map
                        .entry(root_dir_key.clone())
                        .or_default()
                        .push(entry);
                } else if path.is_dir() {
                    // Add directory to root's entries
                    let dir_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let entry = Entry::Directory {
                        name: dir_name.clone(),
                    };
                    dirs_map
                        .entry(root_dir_key.clone())
                        .or_default()
                        .push(entry);

                    // Also create an entry for this directory with processed path
                    let sub_dir_key = processed_path.to_string_lossy().to_string();
                    dirs_map.insert(sub_dir_key, Vec::new());
                }
            } else {
                // For entries not directly in root
                // Get the processed parent path
                let processed_parent = if let Some(processed_parent) = processed_path.parent() {
                    processed_parent.to_path_buf()
                } else {
                    // Fallback if we can't get the parent of processed path
                    if let Some(prefix) = &options.omit_path_prefix {
                        remove_path_prefix(parent, prefix)
                    } else {
                        parent.to_path_buf()
                    }
                };

                let parent_key = processed_parent.to_string_lossy().to_string();

                // Make sure the parent directory exists in our map
                if !dirs_map.contains_key(&parent_key) {
                    dirs_map.insert(parent_key.clone(), Vec::new());
                }

                if path.is_file() {
                    let entry = Entry::File {
                        name: path
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                    };

                    dirs_map.entry(parent_key).or_default().push(entry);
                } else if path.is_dir() {
                    // Add directory to parent's entries
                    let dir_name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    let entry = Entry::Directory { name: dir_name };
                    dirs_map.entry(parent_key).or_default().push(entry);

                    // Also create an entry for this directory with processed path
                    let sub_dir_key = processed_path.to_string_lossy().to_string();
                    dirs_map.insert(sub_dir_key, Vec::new());
                }
            }
        }
    }
    // Convert the map to a vector of DirectoryTree objects
    let mut result: Vec<DirectoryTree> = dirs_map
        .into_iter()
        .filter(|(_, entries)| !entries.is_empty()) // Filter out empty directories
        .map(|(dir, entries)| DirectoryTree { dir, entries })
        .collect();

    // If no directories have entries, add at least the root directory with a placeholder
    if result.is_empty() {
        // Apply path prefix removal to root directory if configured
        let root_dir_path = if let Some(prefix) = &options.omit_path_prefix {
            remove_path_prefix(directory, prefix)
        } else {
            directory.to_path_buf()
        };

        result.push(DirectoryTree {
            dir: root_dir_path.to_string_lossy().to_string(),
            entries: vec![Entry::Directory {
                name: ".".to_string(),
            }],
        });
    }

    // Sort by directory path
    result.sort_by(|a, b| a.dir.cmp(&b.dir));

    Ok(result)
}
