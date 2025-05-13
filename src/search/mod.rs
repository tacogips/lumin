//! File content searching functionality using regex patterns.
//!
//! This module provides tools to search for text patterns in files
//! within a specified directory, with options for case sensitivity
//! and gitignore handling.
//!
//! The search functionality uses regular expressions for advanced pattern matching,
//! supporting features such as:
//!
//! - Basic literal matching (e.g., `apple`)
//! - Wildcards (e.g., `a..le` to match "apple")
//! - Character classes (e.g., `[0-9]+` to match one or more digits)
//! - Word boundaries (e.g., `\bword\b` to match "word" as a standalone word)
//! - Anchors for line start/end (e.g., `^Line` to match at line start, `file$` at line end)
//! - Alternation (e.g., `apple|orange` to match either term)
//! - Repetition (e.g., `a{3,}` to match 3 or more 'a's)
//! - Quantifiers (e.g., `a+` for one or more, `a*` for zero or more)
//!
//! For more complex searching, see the examples in the `search_files` function.

use anyhow::{Context, Result};
use globset;
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::telemetry::{LogMessage, log_with_context};

/// Configuration options for file search operations.
///
/// Controls the behavior of the search functionality, including case sensitivity
/// and how gitignore files are handled.
///
/// # Examples
///
/// ```
/// use lumin::search::SearchOptions;
///
/// // Default options: case-insensitive search respecting gitignore files
/// let default_options = SearchOptions::default();
///
/// // Case-sensitive search, ignoring gitignore files
/// let custom_options = SearchOptions {
///     case_sensitive: true,
///     respect_gitignore: false,
///     exclude_glob: None,
/// };
///
/// // Case-insensitive search, respecting gitignore files
/// let mixed_options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
/// };
/// ```
pub struct SearchOptions {
    /// Whether the search should be case sensitive.
    ///
    /// When `true`, matches will only be found when the exact case matches.
    /// When `false` (default), matches will be found regardless of letter case.
    ///
    /// # Examples
    ///
    /// - With `case_sensitive: true`, searching for "PATTERN" will only match "PATTERN", not "pattern"
    /// - With `case_sensitive: false`, searching for "pattern" will match both "pattern" and "PATTERN"
    pub case_sensitive: bool,

    /// Whether to respect .gitignore files when determining which files to search.
    ///
    /// When `true` (default), files listed in .gitignore will be excluded from the search.
    /// When `false`, all files will be searched, including those that would normally be ignored.
    ///
    /// # Examples
    ///
    /// - With `respect_gitignore: true`, searching will skip files like .git/, node_modules/,
    ///   .tmp, or any patterns specified in .gitignore files
    /// - With `respect_gitignore: false`, searching will include all files, even those listed
    ///   in .gitignore files
    pub respect_gitignore: bool,
    
    /// Optional list of glob patterns for files to exclude from the search.
    ///
    /// When provided, files matching any of these patterns will be excluded from the search,
    /// even if they would otherwise be included based on other criteria.
    ///
    /// # Examples
    ///
    /// - `exclude_glob: Some(vec!["*.json".to_string()])` will exclude all JSON files
    /// - `exclude_glob: Some(vec!["test/**/*.rs".to_string()])` will exclude all Rust files in any test directory
    /// - `exclude_glob: Some(vec!["**/node_modules/**".to_string(), "**/.git/**".to_string()])` will exclude
    ///   both node_modules and .git directories and their contents
    /// - `exclude_glob: None` means no files will be excluded based on glob patterns
    pub exclude_glob: Option<Vec<String>>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
            exclude_glob: None,
        }
    }
}

/// Represents a single search match result.
///
/// Contains information about where a match was found, including the file path,
/// line number, and the actual content of the matching line.
///
/// # Examples
///
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let pattern = "example";
/// let directory = Path::new("src");
/// let options = SearchOptions::default();
///
/// match search_files(pattern, directory, &options) {
///     Ok(results) => {
///         for result in results {
///             println!("Found '{}' in {}:{}: {}",
///                      pattern,
///                      result.file_path.display(),
///                      result.line_number,
///                      result.line_content.trim());
///         }
///     },
///     Err(e) => eprintln!("Search error: {}", e),
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    /// Path to the file containing the match.
    ///
    /// This is the absolute or relative path to the file where the match was found,
    /// depending on the input provided to the search function.
    pub file_path: PathBuf,

    /// Line number where the match was found (1-based).
    ///
    /// Note: Line numbers start at 1, not 0, to match standard editor and command-line
    /// tool conventions.
    pub line_number: u64,

    /// Content of the line containing the match.
    ///
    /// This contains the entire line where the match was found, not just the
    /// matched substring. The matched pattern may appear anywhere within this string.
    pub line_content: String,
}

/// Searches for the specified regex pattern in files within the given directory.
///
/// This function performs a regex-based search across all files in the specified directory
/// (and subdirectories), applying filters based on the provided options. It uses the
/// regex syntax provided by the underlying `grep` crate.
///
/// # Arguments
///
/// * `pattern` - The regular expression pattern to search for. Supports standard regex syntax:
///   - Basic literals: `apple` to match the word "apple"
///   - Wildcards: `.` matches any character, so `a..le` matches "apple"
///   - Character classes: `[0-9]+` matches one or more digits
///   - Word boundaries: `\bword\b` matches "word" as a whole word
///   - Line anchors: `^Line` matches "Line" at the start of a line, `file$` at the end
///   - Alternation: `apple|orange` matches either "apple" or "orange"
///   - Repetition: `a{3,}` matches 3 or more consecutive 'a's
///   - Quantifiers: `a+` matches one or more 'a's, `a*` matches zero or more
///   - Capture groups: `(group)` allows grouping parts of the pattern
///
/// * `directory` - The directory path to search in. All files within this directory and
///   its subdirectories will be searched, subject to filtering by the options.
///
/// * `options` - Configuration options for the search operation:
///   - `case_sensitive`: Controls whether the pattern is matched exactly (true) or ignoring case (false)
///   - `respect_gitignore`: Controls whether files listed in .gitignore are skipped (true) or included (false)
///   - `exclude_glob`: Optional list of glob patterns to exclude files from the search
///
/// # Returns
///
/// A vector of `SearchResult` objects, each containing:
/// - The path to the file with a match
/// - The line number where the match was found (1-based)
/// - The full content of the line containing the match
///
/// The results are not sorted in any particular order.
///
/// # Errors
///
/// Returns an error if:
/// - The regex pattern is invalid (e.g., unbalanced parentheses, invalid syntax)
/// - There's an issue accessing the directory or files (e.g., permissions, not found)
/// - The search operation fails due to I/O or other system issues
///
/// # Examples
///
/// Basic search with default options:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let results = search_files(
///     "function",
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// println!("Found {} matches", results.len());
/// ```
///
/// Case-sensitive search ignoring gitignore files:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let options = SearchOptions {
///     case_sensitive: true,
///     respect_gitignore: false,
///     exclude_glob: None,
/// };
///
/// let results = search_files(
///     "ERROR",
///     Path::new("logs"),
///     &options
/// ).unwrap();
///
/// // Will only find "ERROR" in uppercase, and will include files listed in .gitignore
/// ```
///
/// Using exclude_glob to skip specific file types:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: Some(vec!["*.json".to_string(), "test/**/*.rs".to_string()]),
/// };
///
/// let results = search_files(
///     "password",
///     Path::new("src"),
///     &options
/// ).unwrap();
///
/// // Will find "password" in any case, respecting gitignore files,
/// // but excluding all JSON files and Rust files in test directories
/// ```
///
/// Using regex features:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Find all email addresses in files
/// let email_pattern = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";
/// let results = search_files(
///     email_pattern,
///     Path::new("data"),
///     &SearchOptions::default() // Uses default options (exclude_glob is None)
/// ).unwrap();
///
/// // Find all function definitions with parameters, excluding test files
/// let function_pattern = r"fn\s+\w+\s*\([^)]*\)";
/// let options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: Some(vec!["**/tests/**".to_string(), "**/*_test.rs".to_string()]),
/// };
/// let results = search_files(
///     function_pattern,
///     Path::new("src"),
///     &options
/// ).unwrap();
/// ```
pub fn search_files(
    pattern: &str,
    directory: &Path,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>> {
    // Create the matcher with the appropriate case sensitivity
    let matcher = if options.case_sensitive {
        RegexMatcher::new(pattern)
    } else {
        // For case insensitive search, we add the case-insensitive flag to the regex
        RegexMatcher::new(&format!("(?i){}", pattern))
    }
    .context("Failed to create regular expression matcher")?;

    // Build the list of files to search
    let files =
        collect_files(directory, options).context("Failed to collect files for searching")?;

    let mut results = Vec::new();

    // Set up the searcher
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();

    // Search each file
    for file_path in files {
        let file = match File::open(&file_path) {
            Ok(f) => f,
            Err(e) => {
                log_with_context(
                    log::Level::Warn,
                    LogMessage {
                        message: format!("Failed to open file: {}", e),
                        module: "search",
                        context: Some(vec![("file_path", file_path.display().to_string())]),
                    },
                );
                continue;
            }
        };

        // Create a sink that collects the results
        let mut matches = Vec::new();
        searcher
            .search_file(
                &matcher,
                &file,
                UTF8(|line_number, line| {
                    matches.push((line_number, line.to_string()));
                    Ok(true)
                }),
            )
            .with_context(|| format!("Error searching file {}", file_path.display()))?;

        // Process the matches
        for (line_number, content) in matches {
            results.push(SearchResult {
                file_path: file_path.clone(),
                line_number,
                line_content: content,
            });
        }
    }

    Ok(results)
}

/// Collects a list of files within the given directory that should be included in the search.
///
/// This function applies both gitignore filtering and exclude_glob filtering based on the provided options.
///
/// # Arguments
///
/// * `directory` - The directory path to collect files from
/// * `options` - Configuration options that affect which files are included
///
/// # Returns
///
/// A vector of file paths to be searched
///
/// # Errors
///
/// Returns an error if there's an issue accessing the directory or files, or if there's an error
/// compiling the exclude glob patterns
fn collect_files(directory: &Path, options: &SearchOptions) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let mut builder = WalkBuilder::new(directory);
    builder.git_ignore(options.respect_gitignore);
    // When not respecting gitignore, explicitly include hidden files and dirs
    builder.hidden(options.respect_gitignore);
    // Additional settings to ensure we fully respect/ignore gitignore as needed
    if !options.respect_gitignore {
        builder.ignore(false); // Turn off all ignore logic
        builder.git_exclude(false); // Don't use git exclude files
        builder.git_global(false); // Don't use global git ignore
    }
    
    // Compile exclude glob patterns if provided
    let glob_set = if let Some(exclude_patterns) = &options.exclude_glob {
        if !exclude_patterns.is_empty() {
            let mut builder = globset::GlobSetBuilder::new();
            for pattern in exclude_patterns {
                // Build glob with appropriate case sensitivity
                let glob = if options.case_sensitive {
                    globset::GlobBuilder::new(pattern).build()
                } else {
                    globset::GlobBuilder::new(pattern).case_insensitive(true).build()
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

    for result in builder.build() {
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
                        module: "search",
                        context: Some(vec![("directory", directory.display().to_string())]),
                    },
                );
            }
        }
    }

    Ok(files)
}
