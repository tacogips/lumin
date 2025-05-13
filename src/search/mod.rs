//! File content searching functionality using regex patterns.
//!
//! This module provides tools to search for text patterns in files
//! within a specified directory, with options for case sensitivity
//! and gitignore handling.
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
use globset;
use grep::matcher::Matcher;
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
///     match_content_omit_num: None,
/// };
///
/// // Case-insensitive search, respecting gitignore files, with content truncation
/// let mixed_options = SearchOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     exclude_glob: None,
///     match_content_omit_num: Some(30), // Only show 30 characters before and after matches
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
    /// - `match_content_omit_num: Some(20)` will keep only 20 characters before and after each match
    /// - `match_content_omit_num: None` will retain the complete line content without truncation
    pub match_content_omit_num: Option<usize>,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
            exclude_glob: None,
            match_content_omit_num: None,
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
    ///
    /// If `match_content_omit_num` was set in the search options, this might contain
    /// only partial line content, with characters beyond the specified limit around each
    /// match omitted. Check the `content_omitted` field to determine if content was truncated.
    pub line_content: String,

    /// Indicates whether content was omitted from the line_content.
    ///
    /// When `true`, it means that the line_content has been truncated and only includes
    /// the specified number of characters around each match as configured by
    /// `match_content_omit_num` in the search options.
    ///
    /// When `false`, the entire original line content is preserved.
    pub content_omitted: bool,
}

/// Searches for the specified regex pattern in files within the given directory.
///
/// This function performs a regex-based search across all files in the specified directory
/// (and subdirectories), applying filters based on the provided options. It uses the
/// regex syntax provided by the underlying `grep` crate.
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
///     match_content_omit_num: None,
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
///     match_content_omit_num: Some(50), // Limit content to 50 chars before and after matches
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
/// // and limit the displayed content to 50 characters around each match
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
///     match_content_omit_num: Some(20), // Only show 20 characters before and after matches
/// };
///
/// let results = search_files(
///     "important_pattern",
///     Path::new("src"),
///     &options
/// ).unwrap();
///
/// for result in results {
///     println!("{}: {}{}",
///         result.file_path.display(),
///         result.line_content,
///         if result.content_omitted { " (truncated)" } else { "" });
/// }
///
/// // Will find "important_pattern" in any case, respecting gitignore files,
/// // but only showing 20 characters of context before and after each match
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
/// let options = SearchOptions { case_sensitive: true, ..SearchOptions::default() };
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
///     match_content_omit_num: None,
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
    
        // Process all matches
        for (line_number, content) in matches {
            // Calculate which parts of the content to keep and whether any was omitted
            let (keep_ranges, content_omitted) = if let Some(omit_num) = options.match_content_omit_num {
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
                        if merged_ranges.len() > 1 || merged_ranges[0].0 > 0 || merged_ranges.last().unwrap().1 < content.len() {
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
                        if last_end > 0 { // Don't add marker if we're at the beginning
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
    
            results.push(SearchResult {
                file_path: file_path.clone(),
                line_number,
                line_content,
                content_omitted,
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
