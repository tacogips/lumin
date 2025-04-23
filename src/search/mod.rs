//! File content searching functionality using regex patterns.
//!
//! This module provides tools to search for text patterns in files
//! within a specified directory, with options for case sensitivity
//! and gitignore handling.

use anyhow::{Context, Result};
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};
use ignore::WalkBuilder;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::telemetry::{log_with_context, LogMessage};

/// Configuration options for file search operations.
pub struct SearchOptions {
    /// Whether the search should be case sensitive.
    /// When false, matches will be found regardless of letter case.
    pub case_sensitive: bool,

    /// Whether to respect .gitignore files when determining which files to search.
    /// When true, files listed in .gitignore will be excluded from the search.
    pub respect_gitignore: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
        }
    }
}

/// Represents a single search match result.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    /// Path to the file containing the match
    pub file_path: PathBuf,

    /// Line number where the match was found (1-based)
    pub line_number: u64,

    /// Content of the line containing the match
    pub line_content: String,
}

/// Searches for the specified regex pattern in files within the given directory.
///
/// # Arguments
///
/// * `pattern` - The regular expression pattern to search for
/// * `directory` - The directory path to search in
/// * `options` - Configuration options for the search operation
///
/// # Returns
///
/// A vector of search results, each containing the file path, line number, and line content
/// where a match was found.
///
/// # Errors
///
/// Returns an error if:
/// - The regex pattern is invalid
/// - There's an issue accessing the directory or files
/// - The search operation fails
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
                        context: Some(vec![
                            ("file_path", file_path.display().to_string()),
                        ]),
                    }
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
/// This function applies gitignore filtering based on the provided options.
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
/// Returns an error if there's an issue accessing the directory or files
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

    for result in builder.build() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    files.push(path.to_path_buf());
                }
            }
            Err(err) => {
                log_with_context(
                    log::Level::Warn,
                    LogMessage {
                        message: format!("Error walking directory: {}", err),
                        module: "search",
                        context: Some(vec![
                            ("directory", directory.display().to_string()),
                        ]),
                    }
                );
            }
        }
    }

    Ok(files)
}
