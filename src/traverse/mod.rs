//! Directory traversal and file listing functionality.
//!
//! This module provides tools to traverse directory structures and list files
//! with various filtering options including gitignore support and file type detection.

use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use infer::Infer;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Common utilities for traverse and tree operations
pub mod common;
use common::{build_walk, is_hidden_path};
use crate::telemetry::{log_with_context, LogMessage};

/// Configuration options for directory traversal operations.
#[derive(Debug, Clone)]
pub struct TraverseOptions {
    /// Whether file path matching should be case sensitive
    pub case_sensitive: bool,

    /// Whether to respect .gitignore files when determining which files to include
    pub respect_gitignore: bool,

    /// Whether to only return text files (filtering out binary files)
    pub only_text_files: bool,

    /// Optional pattern to filter files by path.
    /// 
    /// Supports two types of patterns:
    /// - Glob patterns (e.g., "*.rs", "**/*.txt") with special characters like *, ?, [], etc.
    /// - Simple substring patterns (e.g., "README", "config") for searching within file paths
    /// 
    /// The pattern type is automatically detected based on glob special characters.
    /// Pattern matching respects the `case_sensitive` setting.
    pub pattern: Option<String>,
}

impl Default for TraverseOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
            only_text_files: true,
            pattern: None,
        }
    }
}

/// Represents a single file found during directory traversal.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraverseResult {
    /// Path to the file
    pub file_path: PathBuf,

    /// The detected or inferred file type (typically the file extension)
    pub file_type: String,
}

impl TraverseResult {
    /// Determines if a file is hidden (starts with a dot or is in a hidden directory).
    ///
    /// # Returns
    ///
    /// `true` if the file is hidden, `false` otherwise
    pub fn is_hidden(&self) -> bool {
        is_hidden_path(&self.file_path)
    }
}

/// Traverses the specified directory and returns a list of files matching the given criteria.
///
/// # Arguments
///
/// * `directory` - The directory path to traverse
/// * `options` - Configuration options for the traversal operation, including:
///   - Case sensitivity settings
///   - Gitignore respect
///   - Text-only filtering
///   - Pattern matching (both glob and substring patterns)
///
/// # Returns
///
/// A vector of traverse results, each containing the file path and type information.
/// Results are filtered according to the options provided, including any pattern matching.
///
/// # Errors
///
/// Returns an error if there's an issue accessing the directory or files, or if 
/// pattern compilation fails
pub fn traverse_directory(
    directory: &Path,
    options: &TraverseOptions,
) -> Result<Vec<TraverseResult>> {
    let mut results = Vec::new();
    let infer = Infer::new();

    // Use the common walker builder
    let walker = build_walk(directory, options.respect_gitignore, options.case_sensitive)?;

    // Set up pattern matching if pattern provided
    let (pattern_matcher, regex_matcher) = if let Some(pattern) = &options.pattern {
        // Check if pattern contains glob special characters
        let is_glob_pattern = pattern.contains('*')
            || pattern.contains('?')
            || pattern.contains('[')
            || pattern.contains(']');

        if is_glob_pattern {
            // Use glob pattern matching for patterns with glob syntax
            let mut builder = GlobSetBuilder::new();
            let glob = if options.case_sensitive {
                // Case sensitive matching
                GlobBuilder::new(pattern).build()?
            } else {
                // Case insensitive matching
                GlobBuilder::new(pattern).case_insensitive(true).build()?
            };
            builder.add(glob);
            (Some(builder.build()?), None)
        } else {
            // Use regex for simple substring matching
            let regex_pattern = if options.case_sensitive {
                format!(r".*{}.*", regex::escape(pattern)) // Match anywhere in the string
            } else {
                format!(r"(?i).*{}.*", regex::escape(pattern)) // Case insensitive match
            };
            (None, Some(Regex::new(&regex_pattern)?))
        }
    } else {
        (None, None)
    };

    // Walk the directory
    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    // Check if the path matches the pattern if one is provided
                    let matches_pattern = match (&pattern_matcher, &regex_matcher) {
                        (Some(glob_matcher), None) => {
                            // Use glob matching
                            let rel_path = path.strip_prefix(directory).unwrap_or(path);
                            glob_matcher.is_match(rel_path)
                        }
                        (None, Some(regex)) => {
                            // Use regex for substring matching on filename and path
                            let path_str = path.to_string_lossy();
                            regex.is_match(&path_str)
                        }
                        _ => true, // Include all files if no pattern is specified
                    };

                    // Only proceed if the file matches the pattern
                    if !matches_pattern {
                        continue;
                    }

                    // Check if we should include this file based on text/binary filter
                    let include = if options.only_text_files {
                        // Read a small amount of the file to determine its type
                        match std::fs::read(path) {
                            Ok(_) => {
                                // If infer can determine a type, it's probably not a text file
                                match infer.get_from_path(path) {
                                    Ok(Some(kind)) => kind.mime_type().starts_with("text/"),
                                    Ok(None) => true, // Consider as text if infer couldn't determine a type
                                    Err(_) => false,  // Skip files with errors
                                }
                            }
                            Err(_) => false, // Skip files we can't read
                        }
                    } else {
                        true
                    };

                    if include {
                        // Get file type (simplified)
                        let file_type = if let Some(ext) = path.extension().and_then(|e| e.to_str())
                        {
                            ext.to_lowercase()
                        } else {
                            "unknown".to_string()
                        };

                        results.push(TraverseResult {
                            file_path: path.to_path_buf(),
                            file_type,
                        });
                    }
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
                    }
                );
            }
        }
    }

    // Sort results by path
    results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    Ok(results)
}
