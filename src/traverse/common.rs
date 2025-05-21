//! Common utilities shared between traverse and tree modules.
//!
//! This module provides shared functionality for directory traversal operations.

use anyhow::{Context, Result};
use globset;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

use crate::telemetry::{LogMessage, log_with_context};

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

/// Collects a list of files within the given directory, with support for exclude glob patterns.
///
/// This function applies gitignore filtering and exclude_glob filtering based on the provided options.
///
/// # Arguments
///
/// * `directory` - The directory path to collect files from
/// * `respect_gitignore` - Whether to respect gitignore rules
/// * `case_sensitive` - Whether file path matching should be case sensitive
/// * `exclude_glob` - Optional list of glob patterns to exclude files from the results
///
/// # Returns
///
/// A vector of file paths to be searched
///
/// # Errors
///
/// Returns an error if there's an issue accessing the directory or files, or if there's an error
/// compiling the exclude glob patterns
pub fn collect_files_with_excludes(
    directory: &Path,
    respect_gitignore: bool,
    case_sensitive: bool,
    exclude_glob: Option<&Vec<String>>,
) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    // Use the common walker builder
    let walker = build_walk(directory, respect_gitignore, case_sensitive)?;

    // Compile exclude glob patterns if provided
    let glob_set = if let Some(exclude_patterns) = exclude_glob {
        if !exclude_patterns.is_empty() {
            let mut builder = globset::GlobSetBuilder::new();
            for pattern in exclude_patterns {
                // Build glob with appropriate case sensitivity
                let glob = if case_sensitive {
                    globset::GlobBuilder::new(pattern).build()
                } else {
                    globset::GlobBuilder::new(pattern)
                        .case_insensitive(true)
                        .build()
                }
                .with_context(|| format!("Failed to compile glob pattern: {}", pattern))?;

                builder.add(glob);
            }
            Some(builder.build().context("Failed to build glob set")?)
        } else {
            None
        }
    } else {
        None
    };

    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    // Skip files that match any of the exclude globs
                    if let Some(ref glob_set) = glob_set {
                        // Get the path relative to the search directory for better glob matching
                        let rel_path = path.strip_prefix(directory).unwrap_or(path);
                        if glob_set.is_match(rel_path) {
                            // Skip this file as it matches an exclude pattern
                            continue;
                        }
                    }
                    files.push(path.to_path_buf());
                }
            }
            Err(err) => {
                log_with_context(
                    log::Level::Warn,
                    LogMessage {
                        message: format!("Error walking directory: {}", err),
                        module: "traverse",
                        context: Some(vec![
                            ("directory", directory.display().to_string()),
                        ]),
                    },
                );
            }
        }
    }

    Ok(files)
}
