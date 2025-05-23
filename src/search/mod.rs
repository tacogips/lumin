//! File content searching functionality using regex patterns.
//!
//! This module provides tools to search for text patterns in files
//! within a specified directory, with options for case sensitivity,
//! gitignore handling, and context control. The search functionality
//! supports displaying context lines after matches, similar to grep's
//! -A (after-context) option.
//!
//! ## Regex Pattern Syntax
//!
//! The search functionality uses regular expressions for advanced pattern matching.
//! Here are the key regex features supported with examples:
//!
//! ### Basic Patterns
//! - **Literal text**: `apple` matches the text "apple" anywhere in a line
//! - **Escaped special characters**: `\.` matches a literal dot, `\*` matches a literal asterisk
//! - **Character wildcards**: `.` matches any character, so `a..le` matches "apple", "able", etc.
//!
//! ### Character Classes
//! - **Digit matching**: `\d+` or `[0-9]+` matches one or more digits like "123"
//! - **Letter matching**: `[a-zA-Z]+` matches one or more letters
//! - **Custom character sets**: `[aeiou]` matches any vowel
//! - **Negated sets**: `[^0-9]` matches any character that is not a digit
//! - **Predefined classes**: `\w` (word chars), `\s` (whitespace), `\d` (digits)
//!
//! ### Anchors and Boundaries
//! - **Line anchors**: `^start` matches "start" only at line beginning
//! - **End anchors**: `end$` matches "end" only at line end
//! - **Word boundaries**: `\bword\b` matches "word" but not "sword" or "wordsmith"
//!
//! ### Repetition and Quantifiers
//! - **Zero or more**: `a*` matches "", "a", "aa", "aaa", etc.
//! - **One or more**: `a+` matches "a", "aa", "aaa", etc. (but not "")
//! - **Optional**: `colou?r` matches both "color" and "colour"
//! - **Exact count**: `[0-9]{3}` matches exactly 3 digits
//! - **Range**: `[0-9]{2,4}` matches between 2 and 4 digits
//! - **Lazy matching**: `a.*?b` matches "a" followed by "b" with minimal characters between
//!
//! ### Alternation and Grouping
//! - **Alternatives**: `cat|dog` matches either "cat" or "dog"
//! - **Grouping**: `(ab)+` matches "ab", "abab", etc.
//! - **Non-capturing groups**: `(?:abc)+` same as above but doesn't capture
//!
//! ### Other Regex Features
//! - **Capturing groups**: `(pattern)` captures and remembers matched text
//! - **Non-capturing groups**: `(?:pattern)` groups without capturing
//! - **Case-insensitive flag**: The search supports case-insensitive mode via options
//!
//! > **Note**: The search functionality uses the `grep` crate which doesn't support lookaround assertions
//! > (lookahead, lookbehind). If these features are needed, consider post-processing results.
//!
//! ### Common Patterns
//! - **Email**: `[\w.%+-]+@[\w.-]+\.[a-zA-Z]{2,}`
//! - **URLs**: `https?://[\w\.-]+\.[a-zA-Z]{2,}(?:/[\w\.-]*)*`
//! - **IP addresses**: `\b(?:\d{1,3}\.){3}\d{1,3}\b`
//! - **Function definitions**: `fn\s+\w+\s*\([^)]*\)`
//! - **ISO dates**: `\d{4}-\d{2}-\d{2}`
//!
//! For more comprehensive examples and details, see the documentation of the `search_files` function.

use anyhow::{Context, Result};
use grep::matcher::Matcher;
use grep::regex::RegexMatcher;
// Import removed: grep::searcher::sinks::UTF8; (no longer needed)
use grep::searcher::{BinaryDetection, SearcherBuilder};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

use crate::paths::remove_path_prefix;
use crate::telemetry::{LogMessage, log_with_context};
use crate::traverse::common;

/// Configuration options for file search operations.
///
/// Controls the behavior of the search functionality, including case sensitivity
/// and how gitignore files are handled.
///
/// # Examples
///
/// ```
/// use lumin::search::SearchOptions;
/// use std::path::PathBuf;
///
/// // Default options: case-insensitive search respecting gitignore files
/// let default_options = SearchOptions::default();
///
/// // Case-sensitive search, ignoring gitignore files
/// let custom_options = SearchOptions {
///     case_sensitive: true,
///     respect_gitignore: false,
///     exclude_glob: None,
///     include_glob: None,
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 0, // No lines before matches
///     after_context: 0, // Only show matching lines, no context
///     skip: None,
///     take: None,
/// };
///
/// // Case-insensitive search, respecting gitignore files, with content truncation
/// let mixed_options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: None,
///     omit_path_prefix: None,
///     match_content_omit_num: Some(30), // Only show 30 characters before and after matches (full matches always preserved)
///     depth: Some(20),
///     before_context: 2, // Show 2 lines before each match
///     after_context: 2, // Show 2 lines after each match
///     skip: None,
///     take: None,
/// };
///
/// // File type-focused search (only search specific file types)
/// let filetype_options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()]), // Only search Rust and TOML files
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 0,
///     after_context: 0,
///     skip: None,
///     take: None,
/// };
///
/// // Context-focused search (like grep -B3 -A2 pattern)
/// let context_options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: None,
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 3, // Show 3 lines before each match
///     after_context: 2, // Show 2 lines after each match
///     skip: None,
///     take: None,
/// };
///
/// // Search with path prefix removal (to show relative paths in results)
/// let path_prefix_options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: None,
///     omit_path_prefix: Some(PathBuf::from("/home/user/projects/myrepo")), // Remove this prefix from result paths
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 0,
///     after_context: 0,
///     skip: None,
///     take: None,
/// };
/// ```
#[derive(Clone)]
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
    /// **Important**: Glob patterns are matched against paths that are relative to the search directory.
    /// To ensure patterns work correctly with directory hierarchies, follow these guidelines:
    /// 
    /// 1. To exclude files in a specific subdirectory at any level, use `**/dirname/**` 
    ///    (not just `dirname/**`)
    /// 2. To exclude file extensions, use `**/*.ext` to exclude files with that extension anywhere
    /// 3. For nested directories, always prefix with `**/` to match at any level
    /// 4. For files directly in the root directory (no subdirectories), you generally need 
    ///    to filter the search results since glob patterns don't have a direct way to 
    ///    match only root-level files
    ///
    /// # Examples
    ///
    /// - `exclude_glob: Some(vec!["**/*.json".to_string()])` will exclude all JSON files
    /// - `exclude_glob: Some(vec!["**/test/**/*.rs".to_string()])` will exclude all Rust files in any test directory
    /// - `exclude_glob: Some(vec!["**/node_modules/**".to_string(), "**/.git/**".to_string()])` will exclude
    ///   both node_modules and .git directories and their contents
    /// - `exclude_glob: Some(vec!["**/target/**".to_string()])` will exclude Rust build artifacts
    /// - `exclude_glob: None` means no files will be excluded based on glob patterns
    pub exclude_glob: Option<Vec<String>>,

    /// Optional list of glob patterns for files to include in the search.
    ///
    /// When provided, only files matching at least one of these patterns will be included in the search.
    /// This can be used to limit searches to specific file types or directories.
    /// When `None` (default), all files will be searched (subject to other filtering options).
    ///
    /// Note: If both `include_glob` and `exclude_glob` are specified, a file will be included only if
    /// it matches at least one include pattern AND doesn't match any exclude pattern.
    ///
    /// **Important**: Glob patterns are matched against paths that are relative to the search directory.
    /// To ensure patterns work correctly with directory hierarchies, follow these guidelines:
    /// 
    /// 1. To match files in a specific subdirectory at any level, use `**/dirname/**` 
    ///    (not just `dirname/**`)
    /// 2. To match file extensions, use `**/*.ext` to match files with that extension anywhere
    /// 3. For nested directories, always prefix with `**/` to match at any level
    /// 4. For files directly in the root directory (no subdirectories), you generally need 
    ///    to filter the search results since glob patterns don't have a direct way to 
    ///    match only root-level files
    ///
    /// # Examples
    ///
    /// - `include_glob: Some(vec!["**/*.rs".to_string()])` will only search Rust files
    /// - `include_glob: Some(vec!["**/src/**".to_string()])` will only search files in any src directory and its subdirectories
    /// - `include_glob: Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()])` will only search Rust and TOML files
    /// - `include_glob: Some(vec!["**/nested/**".to_string()])` will only search files in directories named "nested" at any level
    /// - `include_glob: None` means all files will be included (subject to other filtering criteria)
    pub include_glob: Option<Vec<String>>,

    /// Optional path prefix to remove from file paths in search results.
    ///
    /// When set to `Some(path)`, this prefix will be removed from the beginning of each file path in the search results.
    /// If a file path doesn't start with this prefix, it will be left unchanged.
    /// When set to `None` (default), file paths are returned as-is.
    ///
    /// This is useful when you want to display relative paths instead of full paths in search results,
    /// or when you want to normalize paths for consistency.
    ///
    /// # Examples
    ///
    /// - `omit_path_prefix: Some(PathBuf::from("/home/user/projects/myrepo"))` will transform a file path like
    ///   `/home/user/projects/myrepo/src/main.rs` to `src/main.rs` in the search results
    /// - `omit_path_prefix: None` will leave all file paths unchanged
    ///
    /// If a file path doesn't start with the specified prefix, it will remain unchanged. For example,
    /// with the prefix `/home/user/projects/myrepo`, a file path like `/var/log/syslog` would remain
    /// `/var/log/syslog` in the search results.
    pub omit_path_prefix: Option<PathBuf>,

    /// Optional setting to limit the number of characters displayed around matches in search results.
    ///
    /// When set to `Some(n)`, the line content in search results will only include `n` UTF-8 characters
    /// before and after each matched pattern. Characters outside this range will be omitted.
    /// When set to `None` (default), the entire line content is preserved.
    ///
    /// Note: If multiple matches occur on the same line, each match will preserve its surrounding
    /// context as specified, which means the total line content may exceed `n*2` characters.
    ///
    /// # Behavior
    ///
    /// The entire matched pattern will always be preserved, even if it's longer than `n` characters.
    /// This ensures that you can always see the complete match, which is important for context.
    ///
    /// The `match_content_omit_num` parameter controls how much context outside the match is shown,
    /// not whether the match itself is truncated. For example, if `match_content_omit_num` is set to 5
    /// and the match is "verylongmatch", you will still see the entire match, plus 5 characters before
    /// and after it.
    ///
    /// # Examples
    ///
    /// - `match_content_omit_num: Some(20)` will keep only 20 characters before and after each match,
    ///   while always preserving the entire matched pattern regardless of its length
    /// - `match_content_omit_num: None` will retain the complete line content without truncation
    pub match_content_omit_num: Option<usize>,

    /// Maximum depth of directory traversal (number of directory levels to explore).
    ///
    /// When `Some(depth)`, the search will only explore up to the specified number of directory levels.
    /// When `None`, the search will explore directories to their full depth.
    /// Default is `Some(20)` to prevent excessive traversal of deeply nested directories.
    ///
    /// # Examples
    ///
    /// - With `depth: Some(1)`, only files in the immediate directory will be searched (no subdirectories)
    /// - With `depth: Some(2)`, files in the immediate directory and one level of subdirectories will be searched
    /// - With `depth: Some(5)`, the search will go up to 5 levels deep
    /// - With `depth: None`, all subdirectories will be explored regardless of depth
    pub depth: Option<usize>,

    /// Number of lines to display before each match (similar to grep's -B option).
    ///
    /// When set to a value greater than 0, this many lines before each match will be included
    /// in the search results, allowing you to see the context preceding each match.
    /// When set to 0 (default), no lines before the matching lines are included.
    ///
    /// # Examples
    ///
    /// - `before_context: 0` (default) - No lines before matches are included in results
    /// - `before_context: 3` - Each match will include the 3 lines that precede it, plus the matching line
    ///
    /// This is particularly useful for understanding the context of a match, such as seeing
    /// function or class declarations before matching a specific method or property,
    /// or understanding the conditions that led to an error before matching an error message.
    pub before_context: usize,

    /// Number of lines to display after each match (similar to grep's -A option).
    ///
    /// When set to a value greater than 0, this many lines after each match will be included
    /// in the search results, allowing you to see the context following each match.
    /// When set to 0 (default), only the matching lines are returned.
    ///
    /// # Examples
    ///
    /// - `after_context: 0` (default) - Only matching lines are included in results
    /// - `after_context: 3` - Each match will include the matching line plus the 3 lines that follow it
    ///
    /// This is particularly useful for understanding the context of a match, such as seeing
    /// the body of a function after matching its definition, or viewing the full error message
    /// after matching an error indicator.
    pub after_context: usize,

    /// Optional number of search result items to skip (for pagination).
    ///
    /// When set to `Some(n)`, the function will skip the first `n` search result items.
    /// When combined with `take`, this enables pagination of search results.
    /// When set to `None` (default), no results are skipped.
    ///
    /// # Examples
    ///
    /// - `skip: Some(0)` - Start from the first result (equivalent to `None`)
    /// - `skip: Some(10)` - Skip the first 10 results, useful for showing the second page
    /// - `skip: Some(20)` - Skip the first 20 results, useful for showing the third page if page size is 10
    /// - `skip: None` - No results are skipped, start from the beginning
    ///
    /// Note that `skip` uses 0-based indexing, where `skip: Some(0)` means to start from the first result.
    pub skip: Option<usize>,

    /// Optional number of search result items to return (for pagination).
    ///
    /// When set to `Some(n)`, the function will return at most `n` search result items.
    /// When combined with `skip`, this enables pagination of search results.
    /// When set to `None` (default), all matching results are returned.
    ///
    /// # Examples
    ///
    /// - `take: Some(10)` - Return up to 10 results, useful for showing 10 items per page
    /// - `take: Some(20)` - Return up to 20 results
    /// - `take: Some(100)` - Return up to 100 results
    /// - `take: None` - Return all results (no limit)
    ///
    /// For pagination with a page size of 10, you would use:
    /// - Page 1: `skip: None, take: Some(10)` or `skip: Some(0), take: Some(10)`
    /// - Page 2: `skip: Some(10), take: Some(10)`
    /// - Page 3: `skip: Some(20), take: Some(10)`
    pub take: Option<usize>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
            exclude_glob: None,
            include_glob: None,
            omit_path_prefix: None,
            match_content_omit_num: None,
            depth: Some(20),
            before_context: 0,
            after_context: 0,
            skip: None,
            take: None,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub total_number: usize,
    pub lines: Vec<SearchResultLine>,
}
impl SearchResult {
    /// Extracts a subset of search result lines from a specified range.
    ///
    /// # Arguments
    ///
    /// * `from` - The starting index (1-based) to extract from, inclusive
    /// * `to` - The ending index (1-based) to extract to, inclusive
    ///
    /// # Returns
    ///
    /// A new `SearchResult` with only the lines in the specified range.
    /// The `total_number` field retains the original total count.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lumin::search::SearchResult;
    /// // Create some search results
    /// let my_search_results = SearchResult {
    ///     total_number: 25,
    ///     lines: vec![/* SearchResultLine items */],
    /// };
    ///
    /// // Extract the first 10 results
    /// let first_page = my_search_results.clone().split(1, 10);
    ///
    /// // Extract the second page of 10 results
    /// let second_page = my_search_results.split(11, 20);
    /// ```
    pub fn split(self, from: usize, to: usize) -> Self {
        // Convert from 1-based to 0-based indexing
        let from_idx = from.saturating_sub(1);
        let to_idx = to.min(self.lines.len());

        // Create a new result with the subset of lines
        SearchResult {
            total_number: self.total_number,
            lines: self
                .lines
                .into_iter()
                .skip(from_idx)
                .take(to_idx.saturating_sub(from_idx))
                .collect(),
        }
    }

    /// Sorts the search result lines by file path and line number.
    ///
    /// This method sorts the lines in-place, first by file path (lexicographically) and then
    /// by line number (numerically) within each file.
    ///
    /// # Returns
    ///
    /// A reference to self for method chaining.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use lumin::search::SearchResult;
    /// // Create some search results
    /// let mut my_search_results = SearchResult {
    ///     total_number: 25,
    ///     lines: vec![/* SearchResultLine items */],
    /// };
    ///
    /// // Sort the results by file path and line number
    /// my_search_results.sort_by_path_and_line();
    /// ```
    pub fn sort_by_path_and_line(&mut self) -> &mut Self {
        self.lines.sort_by(|a, b| {
            // First compare file paths
            let path_cmp = a.file_path.cmp(&b.file_path);
            // If paths are equal, compare line numbers
            if path_cmp == std::cmp::Ordering::Equal {
                a.line_number.cmp(&b.line_number)
            } else {
                path_cmp
            }
        });
        self
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
///     Ok(search_result) => {
///         println!("Total matches: {}", search_result.total_number);
///
///         // Get the first 10 results for pagination
///         let page_1 = search_result.split(1, 10);
///         println!("Showing results 1-10 of {}", page_1.total_number);
///
///         for result in page_1.lines {
///             println!("Found '{}' in {}:{}: {}{}",
///                      pattern,
///                      result.file_path.display(),
///                      result.line_number,
///                      result.line_content.trim(),
///                      if result.content_omitted { " (truncated)" } else { "" });
///         }
///     },
///     Err(e) => eprintln!("Search error: {}", e),
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResultLine {
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
    /// Trailing newlines are removed from the line content.
    ///
    /// If `match_content_omit_num` was set in the search options, this might contain
    /// only partial line content, with characters beyond the specified limit around each
    /// match omitted. Check the `content_omitted` field to determine if content was truncated.
    ///
    /// Note that the entire matched pattern will always be preserved, even if
    /// `match_content_omit_num` is smaller than the match length. Only context around
    /// the match is subject to omission.
    pub line_content: String,

    /// Indicates whether content was omitted from the line_content.
    ///
    /// When `true`, it means that the line_content has been truncated and only includes
    /// the specified number of characters around each match as configured by
    /// `match_content_omit_num` in the search options.
    ///
    /// When `false`, the entire original line content is preserved.
    ///
    /// Note that even when content is omitted (`true`), the entire matched pattern
    /// is always fully preserved, regardless of its length compared to `match_content_omit_num`.
    /// Only the surrounding context before and after the match is affected by truncation.
    pub content_omitted: bool,

    /// Indicates whether this result is a context line rather than a direct match.
    ///
    /// When `true`, this line was included as context (either before or after a match)
    /// rather than containing a direct match to the search pattern.
    ///
    /// When `false`, this line directly matches the search pattern.
    ///
    /// This is useful for displaying context lines differently or for filtering results
    /// to show only direct matches when desired.
    pub is_context: bool,
}

/// Returns only the total number of lines that match a search pattern within files in a directory.
///
/// This is a convenience function that wraps `search_files` when you only need to know the
/// total count of matches without the detailed content of each match. It's more efficient for
/// scenarios where you only need the match count, such as determining result density or
/// checking if any matches exist at all.
///
/// # Arguments
///
/// * `pattern` - The regular expression pattern to search for. Supports the same regex syntax
///   as `search_files`. See the documentation of `search_files` for detailed regex examples.
///
/// * `directory` - The directory path to search in. All files within this directory and its
///   subdirectories will be searched, subject to filtering by the options.
///
/// * `options` - Configuration options for the search operation, identical to those used by
///   `search_files`.
///
/// # Returns
///
/// On success, returns a `Result` containing the total number of matching lines found.
/// This count includes context lines if before_context or after_context options are set.
///
/// # Errors
///
/// Returns the same errors as `search_files`:
/// - If the regex pattern is invalid
/// - If there's an issue accessing the directory or files
/// - If the search operation fails due to I/O or other system issues
///
/// # Examples
///
/// ```no_run
/// use lumin::search::{SearchOptions, search_files_total_match_line_number};
/// use std::path::Path;
///
/// let pattern = "TODO";
/// let directory = Path::new("src");
/// let options = SearchOptions::default();
///
/// match search_files_total_match_line_number(pattern, directory, &options) {
///     Ok(count) => println!("Found {} matches for '{}'", count, pattern),
///     Err(e) => eprintln!("Search error: {}", e),
/// }
/// ```
///
/// Using custom search options:
///
/// ```no_run
/// use lumin::search::{SearchOptions, search_files_total_match_line_number};
/// use std::path::Path;
///
/// let pattern = "error";
/// let directory = Path::new("logs");
///
/// // Only search .log files, case-sensitive
/// let options = SearchOptions {
///     case_sensitive: true,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: Some(vec!["**/*.log".to_string()]),
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 0,
///     after_context: 0,
///     skip: None,
///     take: None,
/// };
///
/// let count = search_files_total_match_line_number(pattern, directory, &options)
///     .unwrap_or(0);
///
/// println!("Found {} occurrences of '{}' in log files", count, pattern);
/// ```
pub fn search_files_total_match_line_number(
    pattern: &str,
    directory: &Path,
    options: &SearchOptions,
) -> Result<usize> {
    let result = search_files(pattern, directory, options)?;
    Ok(result.total_number)
}

/// Searches for the specified regex pattern in files within the given directory.
///
/// This function performs a regex-based search across all files in the specified directory
/// (and subdirectories), applying filters based on the provided options. It uses the
/// regex syntax provided by the underlying `grep` crate.
///
/// Note: The current implementation is naive and not optimized for performance.
/// This will be improved in future versions.
///
/// # Arguments
///
/// * `pattern` - The regular expression pattern to search for. Supports standard regex syntax.
///   See the "Regex Pattern Examples" section below for detailed examples.
///
/// # Regex Syntax Reference
///
/// ### Basic Literals and Escaping
/// - Simple text literals match themselves: `apple` matches "apple" anywhere in text
/// - To match regex special characters literally, escape with backslash:
///   - `\(` matches a literal "(" character
///   - `\*` matches a literal "*" character
///   - `\.` matches a literal "." character
///   - `\\` matches a literal "\" character
///   - `\+` matches a literal "+" character
///   - `\?` matches a literal "?" character
///   - `\[` matches a literal "[" character
///   - `\{` matches a literal "{" character
///   - `\^` matches a literal "^" character
///   - `\$` matches a literal "$" character
///   - `\|` matches a literal "|" character
///
/// ### Wildcards and Character Classes
/// - `.` matches any single character except newline: `a.c` matches "abc", "a@c", etc.
/// - `[abc]` matches any character in the set: `[aeiou]` matches any vowel
/// - `[^abc]` matches any character not in the set: `[^0-9]` matches any non-digit
/// - `[a-z]` matches character ranges: `[a-z]` matches lowercase letters
/// - `\d` matches any digit (equivalent to `[0-9]`): `\d{3}` matches 3 digits
/// - `\D` matches any non-digit (equivalent to `[^0-9]`)
/// - `\w` matches any word character (alphanumeric + underscore): `\w+` matches words
/// - `\W` matches any non-word character (anything except alphanumeric + underscore)
/// - `\s` matches any whitespace character (space, tab, newline, etc.)
/// - `\S` matches any non-whitespace character
///
/// ### Anchors and Boundaries
/// - `^` matches start of line: `^Hello` matches "Hello" only at line start
/// - `$` matches end of line: `world$` matches "world" only at line end
/// - `\b` matches word boundary: `\bword\b` matches "word" but not "sword" or "wordsmith"
/// - `\B` matches non-word boundary: `\Bcat\B` matches "cat" in "concatenate" but not "cat"
///
/// ### Quantifiers and Repetition
/// - `*` matches 0 or more: `a*` matches "", "a", "aa", "aaa", etc.
/// - `+` matches 1 or more: `a+` matches "a", "aa", "aaa", etc. (but not "")
/// - `?` matches 0 or 1: `colou?r` matches both "color" and "colour"
/// - `{n}` matches exactly n times: `a{3}` matches exactly "aaa"
/// - `{n,}` matches n or more times: `a{2,}` matches "aa", "aaa", etc.
/// - `{n,m}` matches between n and m times: `a{2,4}` matches "aa", "aaa", or "aaaa"
/// - Quantifiers are greedy by default, add `?` to make them lazy:
///   - `.*` is greedy, matches as much as possible
///   - `.*?` is lazy, matches as little as possible
///
/// ### Alternation and Grouping
/// - `|` for alternatives: `cat|dog` matches either "cat" or "dog"
/// - `(...)` for grouping: `(abc)+` matches "abc", "abcabc", etc.
/// - `(?:...)` for non-capturing groups: `(?:abc)+` same as above but doesn't capture
///
/// ### Limitations
/// - The search functionality is based on the `grep` crate, which does not support lookaround assertions
///   (lookahead/lookbehind). If these features are needed, consider post-processing the results.
/// - Capturing groups are supported but not directly accessible in results
/// - Some advanced regex features may not be available; see the grep crate documentation for details
///
/// ### Special Pattern Flags
/// - For case-insensitive matching, use the option parameter rather than embedding flags
/// - `(?i)` inline case-insensitive flag: `(?i)hello` matches "hello", "Hello", "HELLO", etc.
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
/// A `SearchResult` object containing:
/// - `total_number`: The total number of search result lines found
/// - `lines`: A vector of `SearchResultLine` objects, each containing:
///   - The path to the file with a match
///   - The line number where the match was found (1-based)
///   - The full content of the line containing the match
///   - Whether any content was omitted
///   - Whether the line is a context line or a direct match
///
/// The results are not sorted in any particular order.
///
/// The `SearchResult` structure also provides a `split` method for pagination.
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
/// let search_result = search_files(
///     "function",
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// println!("Found {} matches", search_result.total_number);
///
/// // Iterate through the result lines
/// for line in &search_result.lines {
///     println!("{}: {}:{}",
///         line.file_path.display(),
///         line.line_number,
///         line.line_content);
/// }
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
///     include_glob: None,
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 0,
///     after_context: 0,
///     skip: None,
///     take: None,
/// };
///
/// let search_result = search_files(
///     "ERROR",
///     Path::new("logs"),
///     &options
/// ).unwrap();
///
/// println!("Found {} matches", search_result.total_number);
///
/// // Will only find "ERROR" in uppercase, and will include files listed in .gitignore
/// ```
///
/// Using exclude_glob to skip specific file types with context:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: Some(vec!["*.json".to_string(), "test/**/*.rs".to_string()]),
///     include_glob: None, // Search all files not excluded
///     omit_path_prefix: None,
///     match_content_omit_num: Some(50), // Limit context to 50 chars before and after each match (preserving full matches)
///     depth: Some(20),
///     before_context: 2, // Show 2 lines before each match
///     after_context: 5, // Show 5 lines after each match
///     skip: None,
///     take: None,
/// };
///
/// let results = search_files(
///     "password",
///     Path::new("src"),
///     &options
/// ).unwrap();
///
/// // Will find "password" in any case, respecting gitignore files,
/// // but excluding all JSON files and Rust files in test directories,
/// // limit the displayed content to 50 characters around each match,
/// // show 2 lines before and 5 lines after each match
/// ```
///
/// Using include_glob to search only specific file types:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: Some(vec!["**/*.rs".to_string(), "**/*.toml".to_string()]), // Only search Rust and TOML files
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 0,
///     after_context: 0,
///     skip: None,
///     take: None,
/// };
///
/// let results = search_files(
///     "dependencies",
///     Path::new("."),
///     &options
/// ).unwrap();
///
/// // Will find "dependencies" only in Rust (.rs) and TOML (.toml) files
/// // across the entire project, respecting gitignore files
/// ```
///
/// Combining include_glob and exclude_glob for precise file targeting:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: Some(vec!["**/target/**".to_string(), "**/node_modules/**".to_string()]),
///     include_glob: Some(vec!["**/*.rs".to_string(), "**/*.md".to_string()]), // Only search Rust and Markdown files
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 1,
///     after_context: 1,
///     skip: None,
///     take: None,
/// };
///
/// let results = search_files(
///     "TODO",
///     Path::new("."),
///     &options
/// ).unwrap();
///
/// // Will find "TODO" comments only in Rust and Markdown files,
/// // while excluding any files in target/ and node_modules/ directories,
/// // showing 1 line of context before and after each match
/// ```
///
/// Using content omission to focus on matches in long lines:
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// let options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: None,
///     omit_path_prefix: None,
///     match_content_omit_num: Some(20), // Only show 20 characters around matches while preserving entire matches
///     depth: Some(20),
///     before_context: 0,
///     after_context: 3, // Show 3 lines of context after each match
///     skip: None,
///     take: None,
/// };
///
/// let search_result = search_files(
///     "important_pattern",
///     Path::new("src"),
///     &options
/// ).unwrap();
///
/// println!("Found {} lines (including context lines)", search_result.total_number);
///
/// for result in search_result.lines {
///     // Display context lines differently
///     if result.is_context {
///         println!("{}: [Context] {}",
///             result.file_path.display(),
///             result.line_content);
///     } else {
///         println!("{}: {}{}",
///             result.file_path.display(),
///             result.line_content,
///             if result.content_omitted { " (truncated)" } else { "" });
///     }
/// }
///
/// // Will find "important_pattern" in any case, respecting gitignore files,
/// // but only showing 20 characters of context before and after each match, plus the 3 lines
/// // that follow each matching line
/// ```
///
/// ## Regex Pattern Examples
///
/// ### Basic Text Searching
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Simple literal text search
/// let results = search_files("hello world", Path::new("docs"), &SearchOptions::default()).unwrap();
///
/// // Case-sensitive search for exact match
/// let options = SearchOptions { case_sensitive: true, skip: None, take: None, ..SearchOptions::default() };
/// let results = search_files("ERROR", Path::new("logs"), &options).unwrap();
///
/// // Matching words with word boundaries
/// let results = search_files(r"\berror\b", Path::new("logs"), &SearchOptions::default()).unwrap();
/// ```
///
/// ### Special Character Escaping
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Searching for text with special regex characters (escaping required)
/// let results = search_files(
///     r"filename\.txt",  // Escape dot with backslash
///     Path::new("docs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Searching for parentheses (need escaping)
/// let results = search_files(
///     r"function\(\)",  // Escape parentheses
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Searching for paths with backslashes
/// let results = search_files(
///     r"C:\\Windows\\System32",  // Double backslashes to escape
///     Path::new("docs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Searching for asterisks, plus signs, or question marks
/// let pattern = r"wildcard\*\.txt or plus\+\.txt or optional\?\.txt";
/// let results = search_files(pattern, Path::new("docs"), &SearchOptions::default()).unwrap();
/// ```
///
/// ### Pattern Matching with Wildcards
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Match any character (except newline)
/// let results = search_files(
///     r"log_2023.0[1-6].txt",  // Matches log_2023.01.txt through log_2023.06.txt
///     Path::new("logs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Using character classes
/// let results = search_files(
///     r"user[0-9]+\.json",  // Matches user followed by one or more digits
///     Path::new("data"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Using predefined character classes
/// let results = search_files(
///     r"\d{4}-\d{2}-\d{2}",  // ISO date format (YYYY-MM-DD)
///     Path::new("logs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Using negated character classes
/// let results = search_files(
///     r"[^a-zA-Z]status",  // "status" not preceded by a letter
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
/// ```
///
/// ### Line Anchors and Boundaries
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Match at start of line
/// let results = search_files(
///     r"^function",  // Lines starting with "function"
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Match at end of line
/// let results = search_files(
///     r";$",  // Lines ending with semicolon
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Match whole word only
/// let results = search_files(
///     r"\bimport\b",  // "import" as a complete word
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Lines starting with whitespace then a pattern
/// let results = search_files(
///     r"^\s+[a-z]+:",  // Indented labels like "    label:"
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
/// ```
///
/// ### Repetition and Quantifiers
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // One or more occurrences
/// let results = search_files(
///     r"ERROR+",  // Matches "ERROR", "ERRORR", etc.
///     Path::new("logs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Zero or more occurrences
/// let results = search_files(
///     r"DEBUG:.*exception",  // DEBUG: followed by anything, then "exception"
///     Path::new("logs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Optional character
/// let results = search_files(
///     r"servers?\.[a-z]+\.com",  // Matches "server.domain.com" or "servers.domain.com"
///     Path::new("config"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Specific repetition counts
/// let results = search_files(
///     r"[A-F0-9]{6}",  // Six hex digits (like color codes #RRGGBB)
///     Path::new("styles"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Between n and m repetitions
/// let results = search_files(
///     r"\d{3,4}",  // 3 or 4 digit numbers
///     Path::new("data"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Greedy vs. lazy matching
/// let results = search_files(
///     r"<div>.*?</div>",  // Match <div> tags non-greedily
///     Path::new("templates"),
///     &SearchOptions::default()
/// ).unwrap();
/// ```
///
/// ### Alternation and Grouping
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Alternative patterns
/// let results = search_files(
///     r"error|warning|fatal",  // Match any of the three terms
///     Path::new("logs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Grouping for alternation
/// let results = search_files(
///     r"(Error|Warning): (disk|memory|cpu) (usage|failure)",  // Structured log lines
///     Path::new("logs"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Repeating groups
/// let results = search_files(
///     r"(ab)+cd",  // Matches "abcd", "ababcd", etc.
///     Path::new("data"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Non-capturing groups for efficiency
/// let results = search_files(
///     r"(?:https?|ftp)://[\w.-]+\.[a-zA-Z]{2,6}",  // URL pattern
///     Path::new("docs"),
///     &SearchOptions::default()
/// ).unwrap();
/// ```
///
/// ### Lookarounds (Advanced Features)
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Positive lookahead
/// let results = search_files(
///     r"TODO(?=:)",  // "TODO" only when followed by colon
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Negative lookahead
/// let results = search_files(
///     r"function\s+\w+(?!\s*\()",  // Function names not followed by parentheses
///     Path::new("src"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Positive lookbehind
/// let results = search_files(
///     r"(?<=@)\w+",  // Words following @ symbol (like Twitter handles)
///     Path::new("social"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Negative lookbehind
/// let results = search_files(
///     r"(?<!\w)\d+(?!\w)",  // Numbers not part of alphanumeric strings
///     Path::new("data"),
///     &SearchOptions::default()
/// ).unwrap();
/// ```
///
/// ### Practical Pattern Examples
/// ```no_run
/// use lumin::search::{SearchOptions, search_files};
/// use std::path::Path;
///
/// // Find all email addresses in files
/// let email_pattern = r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}";
/// let results = search_files(
///     email_pattern,
///     Path::new("data"),
///     &SearchOptions::default()
/// ).unwrap();
///
/// // Find all function definitions with parameters, excluding test files
/// let function_pattern = r"fn\s+\w+\s*\([^)]*\)";
/// let options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: Some(vec!["**/tests/**".to_string(), "**/*_test.rs".to_string()]),
///     include_glob: None,
///     omit_path_prefix: None,
///     match_content_omit_num: None,
///     depth: Some(20),
///     before_context: 0,
///     after_context: 0,
///     skip: None,
///     take: None,
/// };
/// let results = search_files(
///     function_pattern,
///     Path::new("src"),
///     &options
/// ).unwrap();
///
/// // Find IP addresses
/// let ip_pattern = r"\b(?:\d{1,3}\.){3}\d{1,3}\b";
/// let results = search_files(ip_pattern, Path::new("logs"), &SearchOptions::default()).unwrap();
///
/// // Find HTTP/HTTPS URLs
/// let url_pattern = r"https?://[\w.-]+\.[a-zA-Z]{2,}(?:/[\w.-]*)*";
/// let results = search_files(url_pattern, Path::new("docs"), &SearchOptions::default()).unwrap();
///
/// // Find TODO comments with assignee
/// let todo_pattern = r"TODO\([a-zA-Z]+\):";
/// let results = search_files(todo_pattern, Path::new("src"), &SearchOptions::default()).unwrap();
///
/// // Find JSON keys and their values
/// let json_pattern = r#""([\w.-]+)"\s*:\s*"([^"]*)"|("[\w.-]+")\s*:\s*(\d+|true|false|null)"#;
/// let results = search_files(json_pattern, Path::new("config"), &SearchOptions::default()).unwrap();
///
/// // Find all markdown headers
/// let markdown_header_pattern = r"^#{1,6}\s+.*";
/// let results = search_files(markdown_header_pattern, Path::new("docs"), &SearchOptions::default()).unwrap();
///
/// // Find all CSS color codes
/// let css_color_pattern = r"#[a-fA-F0-9]{3,6}|rgb\(\d+,\s*\d+,\s*\d+\)";
/// let results = search_files(css_color_pattern, Path::new("styles"), &SearchOptions::default()).unwrap();
///
/// // Use content omission and context lines in large files with long lines
/// let long_line_options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     include_glob: Some(vec!["**/*.log".to_string()]), // Only search log files
///     omit_path_prefix: None,
///     match_content_omit_num: Some(30), // Show only 30 characters before and after matches
///     depth: Some(20),
///     before_context: 2, // Show 2 lines before each match
///     after_context: 2, // Show 2 lines after each match
///     skip: None,
///     take: None,
/// };
///
/// let long_results = search_files(
///     r"important_pattern",
///     Path::new("logs"),
///     &long_line_options
/// ).unwrap();
///
/// // Process results, showing both matches and context lines differently
/// for result in long_results.lines {
///     if result.is_context {
///         // Display context lines differently
///         println!("{}: [Context] {}",
///             result.file_path.display(),
///             result.line_content);
///     } else {
///         // Display actual matches with truncation indicator if needed
///         println!("{}: [Match] {}{}",
///             result.file_path.display(),
///             result.line_content,
///             if result.content_omitted { " (truncated)" } else { "" });
///     }
///
///     // The entire match pattern is always preserved completely in the line_content,
///     // even when content_omitted is true and other parts of the line are truncated
/// }
/// ```
pub fn search_files(
    pattern: &str,
    directory: &Path,
    options: &SearchOptions,
) -> Result<SearchResult> {
    // Create the matcher with the appropriate case sensitivity
    let matcher = if options.case_sensitive {
        RegexMatcher::new(pattern)
    } else {
        // For case insensitive search, we add the case-insensitive flag to the regex
        RegexMatcher::new(&format!("(?i){}", pattern))
    }
    .context("Failed to create regular expression matcher")?;

    // Build the list of files to search
    // TODO: Implement parallel search by using callbacks in the file traverser
    let files =
        collect_files(directory, options).context("Failed to collect files for searching")?;

    let mut result_lines = Vec::new();

    // Set up the searcher
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .before_context(options.before_context)
        .after_context(options.after_context)
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

        // Define a custom sink to handle both matches and context lines
        struct MatchCollector<'a> {
            // We don't need to store the matcher reference in this implementation
            matches: &'a mut Vec<(u64, String, bool)>, // (line_number, content, is_context)
        }

        impl<'a> grep::searcher::Sink for MatchCollector<'a> {
            type Error = std::io::Error;

            // Handle match lines
            fn matched(
                &mut self,
                _searcher: &grep::searcher::Searcher,
                mat: &grep::searcher::SinkMatch<'_>,
            ) -> Result<bool, Self::Error> {
                let line = String::from_utf8_lossy(mat.bytes())
                    .to_string()
                    .trim_end_matches('\n')
                    .to_string();
                self.matches
                    .push((mat.line_number().unwrap_or(0), line, false)); // Not a context line
                Ok(true)
            }

            // Handle context lines
            fn context(
                &mut self,
                _searcher: &grep::searcher::Searcher,
                ctx: &grep::searcher::SinkContext<'_>,
            ) -> Result<bool, Self::Error> {
                let line = String::from_utf8_lossy(ctx.bytes())
                    .to_string()
                    .trim_end_matches('\n')
                    .to_string();
                self.matches
                    .push((ctx.line_number().unwrap_or(0), line, true)); // Is a context line
                Ok(true)
            }
        }

        let collector = MatchCollector {
            matches: &mut matches,
        };

        searcher
            .search_file(&matcher, &file, collector)
            .with_context(|| format!("Error searching file {}", file_path.display()))?;

        // Process all matches
        for (line_number, content, is_context) in matches {
            // Apply path prefix removal if configured
            let processed_path = if let Some(prefix) = &options.omit_path_prefix {
                remove_path_prefix(&file_path, prefix)
            } else {
                file_path.clone()
            };
    
            // For context lines, we don't need to apply omission logic
            if is_context {
                result_lines.push(SearchResultLine {
                    file_path: processed_path,
                    line_number,
                    line_content: content,
                    content_omitted: false,
                    is_context: true,
                });
                continue;
            }

            // For actual matches, apply omission if needed
            // Calculate which parts of the content to keep and whether any was omitted
            let (keep_ranges, content_omitted) = if let Some(omit_num) =
                options.match_content_omit_num
            {
                // Apply content omission
                let mut keep_ranges = Vec::new();
                let mut any_omitted = false;

                // Find all matches in the line
                let mut match_positions = Vec::new();

                // Collect all match positions using matcher's find_iter method
                let _ = matcher.find_iter(content.as_bytes(), |m| {
                    let start = m.start();
                    let end = m.end();

                    // Ensure valid UTF-8 boundaries
                    let utf8_start = content[..start]
                        .char_indices()
                        .map(|(i, _)| i)
                        .filter(|&i| i <= start)
                        .last()
                        .unwrap_or(0);

                    let utf8_end = if end < content.len() {
                        content[end..]
                            .char_indices()
                            .map(|(i, _)| i + end)
                            .next()
                            .unwrap_or(content.len())
                    } else {
                        content.len()
                    };

                    match_positions.push((utf8_start, utf8_end));
                    true // Continue searching
                });

                // No matches found (shouldn't happen, but handle it anyway)
                if match_positions.is_empty() {
                    (vec![(0, content.len())], false)
                } else {
                    // Calculate context ranges for each match
                    for (match_start, match_end) in match_positions {
                        // Calculate context start (omit_num characters before match)
                        let context_start = if match_start > 0 {
                            let char_count = content[..match_start].chars().count();
                            let chars_to_keep = if char_count > omit_num {
                                char_count - omit_num
                            } else {
                                0
                            };

                            content[..match_start]
                                .char_indices()
                                .map(|(i, _)| i)
                                .nth(chars_to_keep)
                                .unwrap_or(0)
                        } else {
                            0
                        };

                        // Calculate context end (omit_num characters after match)
                        let context_end = if match_end < content.len() {
                            let chars_after = content[match_end..].chars().take(omit_num).count();
                            content[match_end..]
                                .char_indices()
                                .map(|(i, _)| i + match_end)
                                .nth(chars_after)
                                .unwrap_or(content.len())
                        } else {
                            content.len()
                        };

                        // Add this range to our keep_ranges
                        keep_ranges.push((context_start, context_end));
                    }

                    // Sort and merge overlapping ranges
                    if !keep_ranges.is_empty() {
                        keep_ranges.sort_by_key(|&(start, _)| start);

                        let mut merged_ranges = Vec::new();
                        let mut current_range = keep_ranges[0];

                        for &(start, end) in keep_ranges.iter().skip(1) {
                            if start <= current_range.1 {
                                // Ranges overlap, merge them
                                current_range.1 = current_range.1.max(end);
                            } else {
                                // No overlap, push current range and start a new one
                                merged_ranges.push(current_range);
                                current_range = (start, end);
                            }
                        }
                        merged_ranges.push(current_range);

                        // Check if any content would be omitted
                        if merged_ranges.len() > 1
                            || merged_ranges[0].0 > 0
                            || merged_ranges.last().unwrap().1 < content.len()
                        {
                            any_omitted = true;
                        }

                        (merged_ranges, any_omitted)
                    } else {
                        // Fallback (shouldn't reach here)
                        (vec![(0, content.len())], false)
                    }
                }
            } else {
                // No omission requested
                (vec![(0, content.len())], false)
            };

            // Build the final content string using the keep ranges
            let line_content = if content_omitted {
                let mut result = String::new();
                let mut last_end = 0;

                for &(start, end) in &keep_ranges {
                    // Add omission marker if there's a gap
                    if start > last_end {
                        if last_end > 0 {
                            // Don't add marker if we're at the beginning
                            result.push_str("<omit>");
                        }
                    }

                    // Add the content from this range
                    result.push_str(&content[start..end]);
                    last_end = end;
                }

                // Add final omission marker if needed
                if last_end < content.len() {
                    result.push_str("<omit>");
                }

                result
            } else {
                // No omission, use the original content
                content
            };

            result_lines.push(SearchResultLine {
                file_path: processed_path,
                line_number,
                line_content,
                content_omitted,
                is_context: false,
            });
        }
    }

    // Create the SearchResult with the total count and lines
    let total_number = result_lines.len();

    // Create the result and sort it by file path and line number
    let mut result = SearchResult {
        total_number,
        lines: result_lines,
    };

    // Sort the results for consistent ordering
    result.sort_by_path_and_line();

    // Apply pagination if skip and take are specified
    if options.skip.is_some() || options.take.is_some() {
        // Calculate the 1-based indices for split
        let from = match options.skip {
            Some(skip) => skip + 1, // Convert 0-based skip to 1-based from
            None => 1,              // Start from the first result if skip is None
        };

        let to = match options.take {
            Some(take) => from + take - 1, // Calculate the last index (inclusive)
            None => result.lines.len(),    // Use all results if take is None
        };

        // Use the built-in split method to paginate the results
        result = result.split(from, to);
    }

    Ok(result)
}

/// Collects a list of files within the given directory that should be included in the search.
///
/// This function applies gitignore filtering, exclude_glob filtering, and include_glob filtering
/// based on the provided options. It uses the generic `traverse_with_callback` function to
/// efficiently collect matching files.
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
/// compiling the glob patterns
fn collect_files(directory: &Path, options: &SearchOptions) -> Result<Vec<PathBuf>> {
    let include_glob = options.include_glob.as_ref();

    // Use the generic traverse function directly
    common::traverse_with_callback(
        directory,
        options.respect_gitignore,
        options.case_sensitive,
        options.depth,
        options.exclude_glob.as_ref(),
        Vec::new(), // Start with an empty vector
        |mut files, path| {
            // If include_glob is specified, only include files that match at least one pattern
            if let Some(include_patterns) = include_glob {
                // Check if file matches any of the include patterns
                let is_included =
                    common::path_matches_any_glob(path, include_patterns, options.case_sensitive)?;

                // Only add the file if it matches an include pattern
                if is_included {
                    files.push(path.to_path_buf());
                }
            } else {
                // No include_glob, so include all files
                files.push(path.to_path_buf());
            }

            Ok(files)
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Creates a temporary directory with test files for pagination testing
    fn create_test_files(dir: &Path) -> Result<()> {
        // Create a set of test files with known content
        let test_data = [
            (
                "file1.txt",
                "Line 1\nLine with match pattern here\nLine 3\n",
            ),
            (
                "file2.txt",
                "No match\nAnother line\nHas pattern in this line\n",
            ),
            (
                "file3.txt",
                "pattern at start of line\nNo match line\nEnding line\n",
            ),
            (
                "file4.txt",
                "Regular line\nHas pattern twice on this pattern line\nLast line\n",
            ),
            (
                "file5.txt",
                "First line\nSecond line with pattern content\nLast content\n",
            ),
        ];

        for (filename, content) in &test_data {
            let file_path = dir.join(filename);
            let mut file = File::create(file_path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    /// Creates a base SearchOptions object with common test settings
    fn create_base_options() -> SearchOptions {
        SearchOptions {
            case_sensitive: false,
            respect_gitignore: false, // No gitignore in our temp dir
            exclude_glob: None,
            include_glob: None,
            omit_path_prefix: None,
            match_content_omit_num: None,
            depth: None,
            before_context: 0,
            after_context: 0,
            skip: None,
            take: None,
        }
    }

    #[test]
    fn test_pagination() -> Result<()> {
        // Create a temporary directory for our test files
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create test files in the temporary directory
        create_test_files(temp_path)?;

        // The pattern to search for (appears in all test files)
        let pattern = "pattern";

        // Test case 1: No pagination (should return all results)
        let options = create_base_options();
        let results = search_files(pattern, temp_path, &options)?;
        assert_eq!(
            results.total_number, 5,
            "Should find matches in all 5 files"
        );
        assert_eq!(results.lines.len(), 5, "Should return all 5 matching lines");

        // Test case 2: Skip first 2 results
        let mut options_skip = create_base_options();
        options_skip.skip = Some(2);
        let results_skip = search_files(pattern, temp_path, &options_skip)?;
        assert_eq!(
            results_skip.total_number, 5,
            "Total count should still be 5"
        );
        assert_eq!(
            results_skip.lines.len(),
            3,
            "Should return 3 matching lines after skipping 2"
        );

        // Test case 3: Take only 2 results
        let mut options_take = create_base_options();
        options_take.take = Some(2);
        let results_take = search_files(pattern, temp_path, &options_take)?;
        assert_eq!(
            results_take.total_number, 5,
            "Total count should still be 5"
        );
        assert_eq!(
            results_take.lines.len(),
            2,
            "Should return only 2 matching lines"
        );

        // Test case 4: Skip 1, take 3
        let mut options_skip_take = create_base_options();
        options_skip_take.skip = Some(1);
        options_skip_take.take = Some(3);
        let results_skip_take = search_files(pattern, temp_path, &options_skip_take)?;
        assert_eq!(
            results_skip_take.total_number, 5,
            "Total count should still be 5"
        );
        assert_eq!(
            results_skip_take.lines.len(),
            3,
            "Should return 3 matching lines"
        );

        // Test case 5: Skip beyond end of results
        let mut options_skip_all = create_base_options();
        options_skip_all.skip = Some(10); // More than we have results
        let results_skip_all = search_files(pattern, temp_path, &options_skip_all)?;
        assert_eq!(
            results_skip_all.total_number, 5,
            "Total count should still be 5"
        );
        assert_eq!(
            results_skip_all.lines.len(),
            0,
            "Should return 0 matching lines when skipped all"
        );

        // Test case 6: Small take with large skip
        let mut options_edge = create_base_options();
        options_edge.skip = Some(4);
        options_edge.take = Some(10); // More than remaining after skip
        let results_edge = search_files(pattern, temp_path, &options_edge)?;
        assert_eq!(
            results_edge.total_number, 5,
            "Total count should still be 5"
        );
        assert_eq!(
            results_edge.lines.len(),
            1,
            "Should return only 1 matching line"
        );

        Ok(())
    }
}

// Additional tests focused on collect_files function, particularly include_glob functionality
#[cfg(test)]
mod mod_tests;

// Specific tests for collect_files
#[cfg(test)]
mod collect_files_test;

// Tests for pagination and sorting behavior
#[cfg(test)]
mod pagination_test;

// Tests for path prefix removal functionality
#[cfg(test)]
mod path_prefix_test;
