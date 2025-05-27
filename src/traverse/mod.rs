//! Directory traversal and file listing functionality.
//!
//! This module provides tools to traverse directory structures and list files
//! with various filtering options including gitignore support and file type detection.
//!
//! # Pattern Matching
//!
//! The traverse functionality supports both glob and substring pattern matching for
//! file filtering, allowing for powerful and flexible directory exploration.

/// ## Glob Pattern Syntax
///
/// Glob patterns allow for rich file matching using special characters:
///
/// - **Single-character wildcards**: `?` matches any single character
///   - `file?.txt` matches `file1.txt` and `fileA.txt`, but not `file10.txt`
///   - `level?.txt` matches `level1.txt` and `levelA.txt` exactly
///   - `log_202?_??.txt` matches files like `log_2023_01.txt` or `log_2022_10.txt`
///   - `data_v?.json` matches `data_v1.json` or `data_v2.json` but not `data_v10.json`
///   - `user?.{json,xml}` matches `user1.json` or `userA.xml` but not `user10.xml`
///
/// - **Multi-character wildcards**: `*` matches any number of characters within a path segment
///   - `*.txt` matches all .txt files in the current directory
///   - `test_*.txt` matches `test_file.txt` and `test_data.txt`
///   - `*_controller.js` matches all JavaScript controller files
///   - `log_*.log` matches all log files with a prefix like `log_error.log` or `log_system.log`
///   - `*2023*.csv` matches all CSV files containing 2023 in their name
///
/// - **Recursive matching**: `**` matches any number of nested directories
///   - `**/*.rs` matches all Rust files in any subdirectory
///   - `src/**/test/*.rs` matches all Rust files in any `test` directory under `src`
///   - `**/*.{js,ts}` matches all JavaScript and TypeScript files anywhere
///   - `**/assets/**/*.{png,jpg,svg}` matches all images in any assets directory
///   - `**/{bin,lib}/**/*.so` matches all .so files in bin or lib directories at any depth
///
/// - **Prefix matching**: Using wildcards at the end of a pattern to match file prefixes
///   - `config_*` matches only files starting with "config_" in the current directory
///   - `**/prefix_*` matches files starting with "prefix_" in any directory
///   - `src/lib_*` matches files starting with "lib_" in the src directory
///   - `module_*.{rs,ts}` matches files starting with "module_" with .rs or .ts extensions
///   - `api_v1_*` matches all files starting with "api_v1_" in the current directory
///   - `**/model_*.{rs,go,py}` matches model files with specific extensions
///   - `src/**/util_*.*` matches utility files in any subdirectory of src
///   - `test_*` matches all files starting with "test_" in the current directory
///
/// - **Character classes**: `[abc]` matches any character in the set
///   - `file[123].txt` matches `file1.txt`, `file2.txt`, and `file3.txt`
///   - `[a-z]*.txt` matches any file starting with a lowercase letter
///   - `level[a-zA-Z].txt` matches `levelA.txt` or `levelb.txt`
///   - `user[A-D]_profile.json` matches files like `userA_profile.json` or `userC_profile.json`
///   - `log_202[0-3]_*.log` matches log files for years 2020-2023
///   - `[a-z][0-9]_*.data` matches files starting with a lowercase letter followed by a digit
///   - `report_q[1-4]_*.pdf` matches quarterly reports Q1-Q4
///   - `server[1-5]_config.yaml` matches configuration files for servers 1-5
///
/// - **Negated character classes**: `[!abc]` or `[^abc]` matches any character not in the set
///   - `[!0-9]*.txt` matches files not starting with a digit
///   - `file[^.].txt` matches `fileA.txt` but not `file..txt`
///   - `[!a-z]*.json` matches JSON files not starting with lowercase letters
///   - `user_[!0-5]*.log` matches user logs not in the 0-5 range
///   - `[!_.]*.config` matches config files not starting with _ or .
///   - `*[!~#]` matches files not ending with ~ or # (often temp files)
///   - `*.[!bak]` matches files without the .bak extension
///
/// - **Brace expansion**: `{a,b,c}` matches any of the comma-separated patterns
///   - `*.{txt,md,rs}` matches files with .txt, .md, or .rs extensions
///   - `{src,tests}/*.rs` matches Rust files in either src or tests directories
///   - `{config,settings}.*` matches config/settings files with any extension
///   - `{api,service,model}/*.{js,ts}` matches JavaScript/TypeScript files in specific directories
///   - `{docker,kubernetes,k8s}/*.{yml,yaml}` matches container configuration files
///   - `{2021,2022,2023}/*.{csv,xlsx}` matches data files for specific years
///   - `{debug,release}/bin/*.{exe,dll}` matches binary files in debug or release directories
///   - `{app,web}/{css,js,img}/*` matches frontend assets in different directories
///   - `{pkg,cmd,internal}/**/*.go` matches Go code in specific package directories
///
/// - **Complex combinations**:
///   - `**/{test,spec}/*[0-9]?.{js,ts}` combines multiple glob features
///   - `**/[a-z]*-[0-9].{txt,md,json}` matches specific naming patterns
///   - `src/**/{controllers,services}/*[A-Z]*{Controller,Service}.{ts,js}` matches controller and service classes
///   - `**/{v1,v2}/{api,internal}/**/*.[jt]s` combines version and directory patterns for JS/TS files
///   - `{tests,spec}/**/{unit,integration}/**/*_test_*.{js,go,rs}` matches test files in structured test directories
///   - `**/{bin,build}/{debug,release}/[a-z0-9]*[0-9]` matches binary files with specific naming patterns
///   - `**/202[0-3]/{q[1-4],annual}/**/*.{csv,json,xlsx}` matches financial data organized by year and quarter
///   - `**/{user,account,profile}/**/*[!test].{js,ts}` matches non-test files in specific feature directories
///   - `{apps,src}/{backend,frontend}/**/*.{css,scss}` matches style files in specific application directories
///   - `**/{lib,vendor}/[a-zA-Z]*[0-9]*.{so,dll,dylib}` matches versioned library files
///
/// ## Substring Pattern Matching
///
/// When a pattern doesn't contain glob special characters, it's treated as a simple
/// substring match against the entire file path:
///
/// - `config` matches any file with "config" in its path (e.g., "config.toml", "app_config.json")
/// - `test` matches any file with "test" in its path (e.g., "test_data.txt", "tests/example.rs")
/// - `controller` matches files like "UserController.js" or "api/controllers/auth.js"
/// - `2023` matches any file with "2023" in the path (e.g., "logs/2023/", "report-2023.pdf")
/// - `api` matches any API-related files regardless of location or naming convention
/// - `model` matches model files like "user_model.rb" or "src/models/post.rs"
/// - `util` matches utility files and directories like "utils.js" or "src/utils/"
/// - `backup` matches backup files like "config.backup" or "backups/data.json"
/// - `v1` matches versioned files like "api_v1.js" or "v1/endpoints.ts"
/// - Substring matching respects the `case_sensitive` option
///
/// For more examples and detailed usage patterns, see the `traverse_directory` function.
use anyhow::Result;
use globset::{GlobBuilder, GlobSetBuilder};
use infer::Infer;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// Common utilities for traverse and tree operations
pub mod common;
use crate::paths::remove_path_prefix;
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
/// use std::path::PathBuf;
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
///     depth: Some(10),
///     omit_path_prefix: None,
/// };
///
/// // Case-insensitive, include all files, with a substring pattern
/// let search_options = TraverseOptions {
///     case_sensitive: false,
///     respect_gitignore: false,
///     only_text_files: false,
///     pattern: Some("config".to_string()),
///     depth: None,
///     omit_path_prefix: None,
/// };
///
/// // With path prefix removal to show relative paths
/// let prefix_options = TraverseOptions {
///     case_sensitive: false,
///     respect_gitignore: true,
///     only_text_files: true,
///     pattern: None,
///     depth: Some(20),
///     omit_path_prefix: Some(PathBuf::from("/home/user/projects/myrepo")),
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
    /// - `doc*.pdf` - All PDF files starting with "doc" in the current directory
    /// - `*/**/backup_*` - All files starting with "backup_" in any subdirectory at least one level deep
    /// - `logs/*.log` - All log files in the logs directory
    /// - `**/*2023*` - All files containing "2023" in their name in any directory
    /// - `*_test.js` - All JavaScript files ending with "_test" in the current directory
    ///
    /// ### Prefix Matching
    /// - `prefix_*` - Matches all files starting with "prefix_" in the current directory only
    /// - `**/prefix_*` - Matches all files starting with "prefix_" in any directory
    /// - `src/module_*` - Matches files starting with "module_" in the src directory
    /// - `config_*.{json,yaml}` - Matches config files with specific prefix and extensions
    /// - `lib_*.rs` - All Rust files starting with "lib_" in the current directory
    /// - `**/api_v*.js` - All JavaScript files starting with "api_v" in any directory
    /// - `**/model_*.{py,rs,js}` - Files starting with "model_" with specified extensions
    /// - `src/**/util_*` - Files starting with "util_" in any subdirectory of src
    /// - `test_*.{rs,go}` - Test files in the current directory with specific extensions
    /// - `doc_draft_*.md` - Markdown files starting with "doc_draft_" in the current directory
    ///
    /// ### Character Classes
    /// - `file[123].txt` - Matches file1.txt, file2.txt, and file3.txt only
    /// - `[a-z]*.rs` - Rust files starting with a lowercase letter
    /// - `data/[0-9]?_*.dat` - Data files with specific naming pattern
    /// - `**/level[a-zA-Z0-9].txt` - Files named level followed by any letter or digit
    /// - `**/[!0-9]*.txt` - Files not starting with a digit
    /// - `report[A-D]_*.pdf` - PDF files starting with reportA_, reportB_, reportC_, or reportD_
    /// - `temp[_-]*.log` - Log files starting with temp_ or temp-
    /// - `**/[a-f][0-9]*.json` - JSON files starting with a-f followed by a digit
    /// - `**/*[!.][!.][!.]` - Files with exactly 3-character names, no dots
    /// - `data/202[0-3]*` - Files in data/ starting with years 2020-2023
    ///
    /// ### Brace Expansion
    /// - `*.{txt,md,rs}` - Files with .txt, .md, or .rs extensions
    /// - `**/{test,spec}/*.js` - All JS files in any "test" or "spec" directory
    /// - `{src,lib}/**/*.rs` - Rust files in src or lib directories or their subdirectories
    /// - `**/{configs,settings}/*.{json,yml}` - Configuration files with specific extensions
    /// - `{api,service,util}/*.{js,ts}` - JavaScript/TypeScript files in specific directories
    /// - `docs/{*.md,*.txt,README*}` - Documentation files with specific patterns
    /// - `**/build/{debug,release}/*.{exe,dll}` - Binary files in debug or release directories
    /// - `{2021,2022,2023}/{jan,feb,mar}/*.csv` - CSV files organized by year and month
    /// - `**/{styles,css,themes}/*.{css,scss}` - Style-related files in specific directories
    /// - `{bin,scripts}/{*.sh,*.bash,*.zsh}` - Shell scripts in specific directories
    ///
    /// ### Suffix Matching
    /// - `*.{rs,toml}` - Files with .rs or .toml extensions in the current directory
    /// - `**/*_test.rs` - All Rust files ending with "_test" in any directory
    /// - `**/auth*.{js,ts}` - Files containing "auth" in any directory
    /// - `*_backup.*` - Files ending with "_backup" with any extension
    /// - `**/*-v1.{json,yaml}` - Config files ending with "-v1" with specific extensions
    /// - `**/*_controller.{js,ts}` - Controller files with JS or TS extensions
    /// - `**/*_spec.{rb,py}` - Spec files in Ruby or Python
    /// - `**/*_example.*` - Any file ending with "_example" with any extension
    /// - `**/*demo.*` - Any file ending with "demo" with any extension
    /// - `**/*FINAL*` - Files with "FINAL" in their name (case-sensitive if enabled)
    ///
    /// ### Complex Patterns
    /// - `**/nested/**/*[0-9].{txt,md}` - Files ending with a digit in any nested directory
    /// - `**/{test,spec}_[a-z]*/*.{js,ts}` - Test files with specific naming patterns
    /// - `**/[a-z]*-[0-9].{txt,md,json}` - Files with specific name pattern (lowercase-digit.ext)
    /// - `**/{docs,images}/[!.]*` - Non-hidden files in docs or images directories
    /// - `**/*_{test,spec}/**/[a-z]*_test.{js,ts}` - Complex test file organization
    /// - `**/{bin,lib}/{debug,release}/**/*[0-9].{so,dll,dylib}` - Binary libraries with version numbers
    /// - `src/**/{model,schema}/*[A-Z]*.{rs,go}` - Model files starting with uppercase in specific directories
    /// - `**/{v1,v2,v3}/**/{public,private}/*.{js,ts}` - API version-specific files
    /// - `**/*[0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]*` - Files containing dates (YYYY-MM-DD)
    /// - `**/test*/**/{unit,integration}/**/*.{test,spec}.*` - Structured test files
    ///
    /// ## Substring Pattern Examples
    ///
    /// When a pattern doesn't contain glob special characters, it's treated as a simple substring match:
    ///
    /// - `config` - Any file with "config" in its path (e.g., "config.toml", "app_config.json")
    /// - `test` - Any file with "test" in its path (e.g., "test_data.txt", "tests/example.rs")
    /// - `README` - Any file with "README" in its path, case-sensitive if enabled
    /// - `util` - Any file with "util" in its path (e.g., "utils.rs", "utility.js")
    /// - `controller` - Any file with "controller" in its path
    /// - `model` - Any file with "model" in its path (e.g., "user_model.rs", "models/item.js")
    /// - `api` - Any file with "api" in its path (paths, filenames, extensions)
    /// - `2023` - Any file with "2023" in its path (useful for date-based searches)
    /// - `v1` - Any file with "v1" in its path (useful for versioned files)
    /// - `backup` - Any file with "backup" in its path
    pub pattern: Option<String>,

    /// Maximum depth of directory traversal (number of directory levels to explore).
    ///
    /// When `Some(depth)`, the traversal will only explore up to the specified number of directory levels.
    /// When `None`, the traversal will explore directories to their full depth.
    /// Default is `Some(20)` to prevent excessive traversal of deeply nested directories.
    ///
    /// # Examples
    ///
    /// - With `depth: Some(1)`, only files in the immediate directory will be included (no subdirectories)
    /// - With `depth: Some(2)`, files in the immediate directory and one level of subdirectories will be included
    /// - With `depth: Some(5)`, the traversal will go up to 5 levels deep
    /// - With `depth: None`, all subdirectories will be explored regardless of depth
    pub depth: Option<usize>,

    /// Optional path prefix to remove from file paths in traversal results.
    ///
    /// When set to `Some(path)`, this prefix will be removed from the beginning of each file path in the results.
    /// If a file path doesn't start with this prefix, it will be left unchanged.
    /// When set to `None` (default), file paths are returned as-is.
    ///
    /// This is useful when you want to display relative paths instead of full paths in results,
    /// or when you want to normalize paths for consistency.
    ///
    /// # Examples
    ///
    /// - `omit_path_prefix: Some(PathBuf::from("/home/user/projects/myrepo"))` will transform a file path like
    ///   `/home/user/projects/myrepo/src/main.rs` to `src/main.rs` in the results
    /// - `omit_path_prefix: None` will leave all file paths unchanged
    ///
    /// If a file path doesn't start with the specified prefix, it will remain unchanged. For example,
    /// with the prefix `/home/user/projects/myrepo`, a file path like `/var/log/syslog` would remain
    /// `/var/log/syslog` in the results.
    pub omit_path_prefix: Option<PathBuf>,
}

impl Default for TraverseOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
            only_text_files: true,
            pattern: None,
            depth: Some(20),
            omit_path_prefix: None,
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
/// use std::path::{Path, PathBuf};
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
///   - `depth`: Optional maximum directory traversal depth (default: 20)
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
/// ### Controlling Directory Traversal Depth
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::Path;
///
/// // Find all files in the current directory only (no subdirectories)
/// let top_level_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         depth: Some(1),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Find all files up to 5 levels deep
/// let limited_depth_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         depth: Some(5),
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
///
/// // Find all files with unlimited depth
/// let all_files = traverse_directory(
///     Path::new("."),
///     &TraverseOptions {
///         depth: None,
///         ..TraverseOptions::default()
///     }
/// ).unwrap();
/// ```
///
/// ## Using Substring Patterns
/// ```no_run
/// use lumin::traverse::{TraverseOptions, traverse_directory};
/// use std::path::{Path, PathBuf};
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
///
/// // Find files with path prefix removal (to show relative paths in results)
/// let path_prefix_options = traverse_directory(
///     Path::new("/home/user/project"),
///     &TraverseOptions {
///         pattern: Some("**/*.rs".to_string()),
///         omit_path_prefix: Some(PathBuf::from("/home/user/project")), // Remove this prefix from result paths
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
    let walker = build_walk(
        directory,
        options.respect_gitignore,
        options.case_sensitive,
        options.depth,
    )?;

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

                        // Apply path prefix removal if configured
                        let processed_path = if let Some(prefix) = &options.omit_path_prefix {
                            remove_path_prefix(&path.to_path_buf(), prefix)
                        } else {
                            path.to_path_buf()
                        };

                        results.push(TraverseResult {
                            file_path: processed_path,
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

#[cfg(test)]
mod path_prefix_test;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_omit_path_prefix() -> Result<()> {
        // Create a temporary directory
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create some test files
        let test_files = ["file1.txt", "file2.rs", "subdir/file3.md"];
        for file_path in &test_files {
            let full_path = temp_path.join(file_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = File::create(full_path)?;
            file.write_all(b"test content")?;
        }

        // Test with path prefix removal
        let options = TraverseOptions {
            case_sensitive: false,
            respect_gitignore: false, // No gitignore in temp dir
            only_text_files: true,
            pattern: None,
            depth: None,
            omit_path_prefix: Some(temp_path.to_path_buf()),
        };

        let results = traverse_directory(temp_path, &options)?;

        // Check that prefixes were removed
        for result in &results {
            // Paths should not start with the temp directory
            assert!(!result.file_path.starts_with(temp_path));

            // Check that each file exists in our test files array (after normalization)
            let normalized_path = result.file_path.to_string_lossy().to_string();
            let found = test_files
                .iter()
                .any(|f| normalized_path == *f || normalized_path.replace("\\", "/") == *f);
            assert!(
                found,
                "File path {} not found in test files",
                normalized_path
            );
        }

        // Test without path prefix removal
        let options_no_prefix = TraverseOptions {
            omit_path_prefix: None,
            ..options
        };

        let results_no_prefix = traverse_directory(temp_path, &options_no_prefix)?;

        // Check that prefixes were not removed
        for result in &results_no_prefix {
            // Paths should start with the temp directory
            assert!(result.file_path.starts_with(temp_path));
        }

        Ok(())
    }
}
