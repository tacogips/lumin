//! Common utilities shared between traverse and tree modules.
//!
//! This module provides shared functionality for directory traversal operations.

use anyhow::Result;
use ignore::WalkBuilder;
use std::path::Path;

/// Builds a configured file system walker based on the provided options.
///
/// # Arguments
///
/// * `directory` - The directory path to traverse
/// * `respect_gitignore` - Whether to respect gitignore rules
/// * `case_sensitive` - Whether file path matching should be case sensitive
///
/// # Returns
///
/// A configured WalkBuilder for traversing the file system
///
/// # Errors
///
/// Returns an error if there's an issue setting up the walker
pub fn build_walk(
    directory: &Path,
    respect_gitignore: bool,
    case_sensitive: bool,
) -> Result<ignore::Walk> {
    // Configure the file traversal
    let mut builder = WalkBuilder::new(directory);
    builder.git_ignore(respect_gitignore);
    // When respecting gitignore, hidden files are skipped; otherwise they're included
    builder.hidden(respect_gitignore);
    if !case_sensitive {
        builder.ignore_case_insensitive(true);
    }
    // Additional settings to ensure we fully respect/ignore gitignore as needed
    if !respect_gitignore {
        builder.ignore(false); // Turn off all ignore logic
        builder.git_exclude(false); // Don't use git exclude files
        builder.git_global(false); // Don't use global git ignore
    }

    Ok(builder.build())
}

/// Determines if a path is hidden (starts with a dot or is in a hidden directory).
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// `true` if the path is hidden, `false` otherwise
pub fn is_hidden_path(path: &Path) -> bool {
    // Check if the file name starts with a dot
    let file_is_hidden = path
        .file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|name| name.starts_with("."));

    // Also check if the file is in a hidden directory
    let path_contains_hidden_dir = path
        .to_string_lossy()
        .split('/')
        .any(|part| part.starts_with(".") && !part.is_empty());

    file_is_hidden || path_contains_hidden_dir
}
