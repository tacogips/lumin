//! Directory traversal and file listing functionality.
//!
//! This module provides tools to traverse directory structures and list files
//! with various filtering options including gitignore support and file type detection.
//!
//! The traverse functionality supports both glob and substring pattern matching for
//! file filtering, allowing for powerful and flexible directory exploration:
//!
//! - Single-character wildcards: `?` matches any single character
//! - Multi-character wildcards: `*` matches any number of characters
//! - Recursive matching: `**` matches any number of nested directories
//! - Character classes: `[abc]` matches any character in the set
//! - Negated character classes: `[!0-9]` matches any character not in the set
//! - Brace expansion: `{txt,md}` matches either "txt" or "md"
//! - Complex directory patterns: `**/dir/*/*.txt`
//!
//! For examples and more details on pattern syntax, see the `traverse_directory` function.

use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use infer::Infer;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Common utilities for traverse and tree operations
pub mod common;
use crate::telemetry::{LogMessage, log_with_context};
use common::{build_walk, is_hidden_path};

/// Configuration options for directory traversal operations.
///
/// Controls the behavior of the traversal functionality, including case sensitivity,
/// gitignore handling, file type filtering, and pattern matching.
///
/// # Examples
///
/// ```
/// use lumin::traverse::TraverseOptions;
///
/// // Default options: case-insensitive, respect gitignore, only text files, no pattern
/// let default_options = TraverseOptions::default();
///
/// // Case-sensitive, include binary files, with a glob pattern
/// let custom_options = TraverseOptions {
///     case_sensitive: true,
///     respect_gitignore: true,
///     only_text_files: false,
///     pattern: Some("**/*.{rs,toml}".to_string()),
/// };
///
/// // Case-insensitive, include all files, with a substring pattern
/// let search_options = TraverseOptions {
///     case_sensitive: false,
///     respect_gitignore: false,
///     only_text_files: false,
///     pattern: Some("config".to_string()),
/// };
/// ```
#[derive(Debug, Clone)]
pub struct TraverseOptions {
    /// Whether file path matching should be case sensitive.
    ///
    /// When `true`, file paths must exactly match the case in the pattern.
    /// When `false` (default), file paths will match regardless of case.
    ///
    /// # Examples
    ///
    /// - With `case_sensitive: true`, pattern "Config" will match "Config.txt" but not "config.txt"
    /// - With `case_sensitive: false`, pattern "config" will match both "config.txt" and "Config.txt"
    pub case_sensitive: bool,

    /// Whether to respect .gitignore files when determining which files to include.
    ///
    /// When `true` (default), files and directories listed in .gitignore will be excluded.
    /// When `false`, all files will be included, even those that would normally be ignored.
    ///
    /// # Examples
    ///
    /// - With `respect_gitignore: true`, files like .git/, node_modules/, tmp files,
    ///   or patterns specified in .gitignore will be excluded
    /// - With `respect_gitignore: false`, all files will be included regardless of
    ///   their presence in .gitignore files
    pub respect_gitignore: bool,

    /// Whether to only return text files (filtering out binary files).
    ///
    /// When `true` (default), binary files like images, executables, etc. will be excluded.
    /// When `false`, all files will be included regardless of their content type.
    ///
    /// # Examples
    ///
    /// - With `only_text_files: true`, files like .txt, .md, .rs will be included,
    ///   but .jpg, .png, executables will be excluded
    /// - With `only_text_files: false`, all files will be included regardless of their type
    pub only_text_files: bool,

    /// Optional pattern to filter files by path.
    ///
    /// Supports two types of patterns:
    /// - Glob patterns (e.g., "*.rs", "**/*.txt") with special characters like *, ?, [], etc.
    /// - Simple substring patterns (e.g., "README", "config") for searching within file paths
    ///
    /// The pattern type is automatically detected based on glob special characters.
    /// Pattern matching respects the `case_sensitive` setting.
    ///
    /// ## Glob Pattern Examples
    ///
    /// - `*.txt` - All files with .txt extension in the current directory
    /// - `**/*.txt` - All .txt files in any subdirectory (recursive)
    /// - `src/*.rs` - All Rust files in the src directory
    /// - `**/test_*.rs` - All Rust files starting with "test_" in any directory
    /// - `**/{test,spec}/*.js` - All JS files in any "test" or "spec" directory
    /// - `data/[0-9]?_*.dat` - Data files with specific naming pattern
    ///
    /// ## Substring Pattern Examples
    ///
    /// - `config` - Any file with "config" in its path (e.g., "config.toml", "app_config.json")
    /// - `test` - Any file with "test" in its path (e.g., "test_data.txt", "tests/example.rs")
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
///
/// Contains information about the file, including its path and detected type.
///
/// # Examples
///
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// let options = TraverseOptions::default();
/// match traverse_directory(Path::new("src"), &options) {
///     Ok(results) => {
///         for result in results {
///             println!("{} [{}] {}",
///                      if result.is_hidden() { "*" } else { " " },
///                      result.file_type,
///                      result.file_path.display());
///         }
///     },
///     Err(e) => eprintln!("Traversal error: {}", e),
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraverseResult {
    /// Path to the file.
    ///
    /// This is the absolute or relative path to the file, depending on the
    /// input provided to the traverse function.
    pub file_path: PathBuf,

    /// The detected or inferred file type (typically the file extension).
    ///
    /// This is usually the lowercase file extension (e.g., "txt", "rs", "toml"),
    /// or "unknown" if the type couldn't be determined.
    pub file_type: String,
}

impl TraverseResult {
    /// Determines if a file is hidden (starts with a dot or is in a hidden directory).
    ///
    /// A file is considered hidden if:
    /// - Its name starts with a dot (e.g., ".gitignore")
    /// - It's in a directory whose name starts with a dot (e.g., ".git/config")
    ///
    /// # Returns
    ///
    /// `true` if the file is hidden, `false` otherwise
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lumin::traverse::{TraverseOptions, traverse_directory};
    /// use std::path::Path;
    ///
    /// let results = traverse_directory(
    ///     Path::new("."),
    ///     &TraverseOptions {
    ///         respect_gitignore: false,
    ///         ..TraverseOptions::default()
    ///     }
    /// ).unwrap();
    ///
    /// // Find all hidden files
    /// let hidden_files: Vec<_> = results.into_iter()
    ///     .filter(|r| r.is_hidden())
    ///     .collect();
    ///
    /// for file in hidden_files {
    ///     println!("Hidden file: {}", file.file_path.display());
    /// }
    /// ```
    pub fn is_hidden(&self) -> bool {
        is_hidden_path(&self.file_path)
    }
}

/// Traverses the specified directory and returns a list of files matching the given criteria.
///
/// This function scans the directory and its subdirectories, applying filters based on
/// the provided options. It can filter files by type (text/binary), respect gitignore rules,
/// and match files against specified glob or substring patterns.
///
/// # Arguments
///
/// * `directory` - The directory path to traverse, as a Path reference.
///
/// * `options` - Configuration options for the traversal operation, including:
///   - `case_sensitive`: Controls whether pattern matching is case-sensitive
///   - `respect_gitignore`: Controls whether .gitignore rules are applied
///   - `only_text_files`: Controls whether binary files are excluded
///   - `pattern`: Optional glob or substring pattern for filtering files
///
/// # Returns
///
/// A vector of `TraverseResult` objects, each containing:
/// - The path to the file
/// - The detected file type (typically the extension)
///
/// The results are sorted alphabetically by file path.
///
/// # Errors
///
/// Returns an error if:
/// - There's an issue accessing the directory or files
/// - Pattern compilation fails (for invalid glob patterns)
///
/// # Examples
///
/// Basic traversal with default options:
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Find all text files, respecting .gitignore
/// let results = traverse_directory(
///     Path::new("src"),
///     &TraverseOptions::default()
/// ).unwrap();
///
/// println!("Found {} files", results.len());
/// ```
///
/// Using glob patterns:
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Find all Rust source files
/// let results = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("**/*.rs".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Find all text files in the test directory structure
/// let test_files = traverse_directory(
///     Path::new("tests"),
///     &TraverseOptions {
///         pattern: Some("**/{test,spec}_*.{rs,txt}".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Find any Cargo.toml or package.json files
/// let project_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("**/Cargo.toml".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
/// ```
///
/// Using substring patterns:
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Find all files with "config" in their name
/// let config_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("config".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Find all files containing "test" in their path, including binary files
/// let test_related = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("test".to_string()),
///         only_text_files: false,
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
/// ```
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
                        context: Some(vec![("directory", directory.display().to_string())]),
                    },
                );
            }
        }
    }

    // Sort results by path
    results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    Ok(results)
}
