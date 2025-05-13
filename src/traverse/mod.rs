//! Directory traversal and file listing functionality.
//!
//! This module provides tools to traverse directory structures and list files
//! with various filtering options including gitignore support and file type detection.
//!
//! # Pattern Matching
//!
//! The traverse functionality supports both glob and substring pattern matching for
//! file filtering, allowing for powerful and flexible directory exploration.
//!
//! ## Glob Pattern Syntax
//!
//! Glob patterns allow for rich file matching using special characters:
//!
//! - **Single-character wildcards**: `?` matches any single character
//!   - `file?.txt` matches `file1.txt` and `fileA.txt`, but not `file10.txt`
//!   - `level?.txt` matches `level1.txt` and `levelA.txt` exactly
//!
//! - **Multi-character wildcards**: `*` matches any number of characters within a path segment
//!   - `*.txt` matches all .txt files in the current directory
//!   - `test_*.txt` matches `test_file.txt` and `test_data.txt`
//!
//! - **Recursive matching**: `**` matches any number of nested directories
//!   - `**/*.rs` matches all Rust files in any subdirectory
//!   - `src/**/test/*.rs` matches all Rust files in any `test` directory under `src`
//!
//! - **Prefix matching**: Using wildcards at the end of a pattern to match file prefixes
//!   - `config_*` matches only files starting with "config_" in the current directory
//!   - `**/prefix_*` matches files starting with "prefix_" in any directory
//!   - `src/lib_*` matches files starting with "lib_" in the src directory
//!   - `module_*.{rs,ts}` matches files starting with "module_" with .rs or .ts extensions
//!
//! - **Character classes**: `[abc]` matches any character in the set
//!   - `file[123].txt` matches `file1.txt`, `file2.txt`, and `file3.txt`
//!   - `[a-z]*.txt` matches any file starting with a lowercase letter
//!   - `level[a-zA-Z].txt` matches `levelA.txt` or `levelb.txt`
//!
//! - **Negated character classes**: `[!abc]` or `[^abc]` matches any character not in the set
//!   - `[!0-9]*.txt` matches files not starting with a digit
//!   - `file[^.].txt` matches `fileA.txt` but not `file..txt`
//!
//! - **Brace expansion**: `{a,b,c}` matches any of the comma-separated patterns
//!   - `*.{txt,md,rs}` matches files with .txt, .md, or .rs extensions
//!   - `{src,tests}/*.rs` matches Rust files in either src or tests directories
//!   - `{config,settings}.*` matches config/settings files with any extension
//!
//! - **Complex combinations**:
//!   - `**/{test,spec}/*[0-9]?.{js,ts}` combines multiple glob features
//!   - `**/[a-z]*-[0-9].{txt,md,json}` matches specific naming patterns
//!
//! ## Substring Pattern Matching
//!
//! When a pattern doesn't contain glob special characters, it's treated as a simple
//! substring match against the entire file path:
//!
//! - `config` matches any file with "config" in its path (e.g., "config.toml", "app_config.json")
//! - `test` matches any file with "test" in its path (e.g., "test_data.txt", "tests/example.rs")
//! - Substring matching respects the `case_sensitive` option
//!
//! For more examples and detailed usage patterns, see the `traverse_directory` function.

use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use infer::Infer;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Common utilities for traverse and tree operations
pub mod common;
use crate::telemetry::{log_with_context, LogMessage};
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
    /// ### Basic Wildcards
    /// - `*.txt` - All files with .txt extension in the current directory
    /// - `**/*.txt` - All .txt files in any subdirectory (recursive)
    /// - `file?.txt` - Matches file1.txt or fileA.txt, but not file10.txt (? matches one character)
    /// - `src/*.rs` - All Rust files in the src directory
    /// - `**/test_*.rs` - All Rust files starting with "test_" in any directory
    ///
    /// ### Prefix Matching
    /// - `prefix_*` - Matches all files starting with "prefix_" in the current directory only
    /// - `**/prefix_*` - Matches all files starting with "prefix_" in any directory
    /// - `src/module_*` - Matches files starting with "module_" in the src directory
    /// - `config_*.{json,yaml}` - Matches config files with specific prefix and extensions
    ///
    /// ### Character Classes
    /// - `file[123].txt` - Matches file1.txt, file2.txt, and file3.txt only
    /// - `[a-z]*.rs` - Rust files starting with a lowercase letter
    /// - `data/[0-9]?_*.dat` - Data files with specific naming pattern
    /// - `**/level[a-zA-Z0-9].txt` - Files named level followed by any letter or digit
    /// - `**/[!0-9]*.txt` - Files not starting with a digit
    ///
    /// ### Brace Expansion
    /// - `*.{txt,md,rs}` - Files with .txt, .md, or .rs extensions
    /// - `**/{test,spec}/*.js` - All JS files in any "test" or "spec" directory
    /// - `{src,lib}/**/*.rs` - Rust files in src or lib directories or their subdirectories
    /// - `**/{configs,settings}/*.{json,yml}` - Configuration files with specific extensions
    ///
    /// ### Complex Patterns
    /// - `**/nested/**/*[0-9].{txt,md}` - Files ending with a digit in any nested directory
    /// - `**/{test,spec}_[a-z]*/*.{js,ts}` - Test files with specific naming patterns
    /// - `**/[a-z]*-[0-9].{txt,md,json}` - Files with specific name pattern (lowercase-digit.ext)
    /// - `**/{docs,images}/[!.]*` - Non-hidden files in docs or images directories
    ///
    /// ## Substring Pattern Examples
    ///
    /// When a pattern doesn't contain glob special characters, it's treated as a simple substring match:
    /// 
    /// - `config` - Any file with "config" in its path (e.g., "config.toml", "app_config.json")
    /// - `test` - Any file with "test" in its path (e.g., "test_data.txt", "tests/example.rs")
    /// - `README` - Any file with "README" in its path, case-sensitive if enabled
    /// - `util` - Any file with "util" in its path (e.g., "utils.rs", "utility.js")
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
/// ## Basic Usage
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
/// ## Using Glob Patterns
///
/// ### Basic Wildcards
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Find all Rust source files in any subdirectory
/// let rust_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("**/*.rs".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Find files with specific single-character wildcard
/// let numbered_files = traverse_directory(
///     Path::new("data"),
///     &TraverseOptions {
///         pattern: Some("file?.txt".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
/// ```
///
/// ### Character Classes
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Find files with specific character patterns
/// let level_files = traverse_directory(
///     Path::new("docs"),
///     &TraverseOptions {
///         pattern: Some("level[1-3].txt".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Files not starting with a digit
/// let non_numeric_files = traverse_directory(
///     Path::new("reports"),
///     &TraverseOptions {
///         pattern: Some("[!0-9]*.pdf".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
/// ```
///
/// ### Brace Expansion
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Find all text files with common extensions
/// let text_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("**/*.{txt,md,rs}".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Find config files in specific directories
/// let config_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("**/{configs,settings}/*.{json,yml,toml}".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
/// ```
///
/// ### Complex Patterns
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Complex pattern combining multiple features
/// let test_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("**/{test,spec}/*[0-9]/*.{rs,ts}".to_string()),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
/// ```
///
/// ## Using Substring Patterns
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
///
/// // Find files with case-sensitive matching
/// let case_sensitive_search = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         pattern: Some("README".to_string()),
///         case_sensitive: true,
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
    let pattern_matcher = if let Some(pattern) = &options.pattern {
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
            Some(builder.build()?)
        } else {
            // For simple substring matching, we'll use String.contains() later
            None
        }
    } else {
        None
    };

    // Walk the directory
    for result in walker {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    // Check if the path matches the pattern if one is provided
                    let matches_pattern = if let Some(ref pattern) = options.pattern {
                        if let Some(ref glob_matcher) = pattern_matcher {
                            // Use glob matching
                            let rel_path = path.strip_prefix(directory).unwrap_or(path);
                            glob_matcher.is_match(rel_path)
                        } else {
                            // Use simple substring matching on filename and path
                            let path_str = path.to_string_lossy();
                            if options.case_sensitive {
                                // Case sensitive substring match
                                path_str.contains(pattern)
                            } else {
                                // Case insensitive substring match
                                path_str.to_lowercase().contains(&pattern.to_lowercase())
                            }
                        }
                    } else {
                        true // Include all files if no pattern is specified
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
                    },
                );
            }
        }
    }

    // Sort results by path
    results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    Ok(results)
}