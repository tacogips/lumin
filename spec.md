# Library Specification

This document describes the specifications and implementation details of the lumin library.

## Library Features

This library provides functionality for searching and displaying local files.

### File Searching

- Primarily using the `grep` crate as a library
- Logic is defined in the `search` package.
  Specify a target directory to grep through files under that directory.
  Files listed in .gitignore (if present in the target directory) are excluded by default, but this can be overridden with a parameter.
- Case sensitivity can be toggled via parameters.
- Search results are automatically sorted by file path (lexicographically) and line number (numerically) for deterministic ordering.
- Supports rich configuration via the `SearchOptions` struct:
  - Case sensitivity control with `case_sensitive` field
  - Gitignore respect control with `respect_gitignore` field
  - File inclusion/exclusion with `include_glob` and `exclude_glob` fields (both use relative paths consistently)
  - Path prefix omission with `omit_path_prefix` for display purposes
  - Match content context control with `match_content_omit_num`
  - Depth limiting with `depth` field
  - Pagination support with `skip` and `take` fields
- Supports context control:
  - Before-context option to show N lines preceding each match (similar to grep's -B option)
  - After-context option to show N lines following each match (similar to grep's -A option)
  - Both options can be combined to show context on both sides of matches
  - Option to limit displayed context around matches to a specific number of characters
  - Context lines are visually distinguished from match lines in output

#### Glob Pattern Consistency

Both `include_glob` and `exclude_glob` parameters work consistently with relative paths:

- **Path Matching**: All glob patterns are matched against paths relative to the search directory
- **Consistency**: Both inclusion and exclusion filters use the same path format, making the API intuitive
- **Example**: When searching in `/home/user/project`:
  - File `/home/user/project/src/main.rs` is matched against the relative path `src/main.rs`
  - Pattern `**/*.rs` matches all Rust files in any subdirectory
  - Pattern `src/**` matches all files in the src directory
- **Historical Note**: This consistency was implemented to fix an earlier inconsistency where `include_glob` used absolute paths while `exclude_glob` used relative paths

### File Traversal

- Primarily using the `eza` crate as a library
- Logic is defined in the `traverse` package.

- Specify a target directory to search for file names under that directory.
  Files listed in .gitignore (if present in the target directory) are excluded by default, but this can be overridden with a parameter.

- Supports pattern matching to filter files:
  - Glob patterns (e.g., `*.rs`, `**/*.txt`) using the `globset` crate
  - Simple substring matching (e.g., `README`, `config`) using the `regex` crate
  - Automatically detects pattern type and applies appropriate matching strategy
  - Pattern matching respects case sensitivity settings
  - **Glob patterns use relative paths consistently with the search module**

- Case sensitivity can be toggled via parameters.

- By default, the library uses the `infer` crate and only returns files that cannot be parsed by this crate (identifying them as text files). This filtering behavior can be toggled via parameters.

- Provides a generic traversal function that can be used to collect arbitrary data:
  ```rust
  pub fn traverse_with_callback<T, F>(
      directory: &Path,
      respect_gitignore: bool,
      case_sensitive: bool,
      exclude_glob: Option<&Vec<String>>, // Uses relative paths consistently
      initial: T,
      callback: F,
  ) -> Result<T>
  ```

#### Traverse Module Glob Pattern Consistency

The traverse module follows the same relative path pattern matching as the search module:

- **`exclude_glob` parameter**: Uses relative paths for consistent behavior
- **`pattern` field in TraverseOptions**: Uses relative paths for glob pattern matching
- **Unified behavior**: All glob patterns across search and traverse modules work the same way
- **Example**: When traversing `/home/user/project`:
  - File `/home/user/project/src/main.rs` is matched against relative path `src/main.rs`
  - Pattern `**/*.rs` matches all Rust files in any subdirectory
  - Pattern `src/**` matches all files in the src directory
  - This enables custom processing during traversal without creating intermediate collections
  - Can be used to implement specialized traversal functions for specific needs
  - All existing traversal functions are implemented on top of this generic function

### Directory Tree Structure

- Built on top of the traversal functionality
- Logic is defined in the `tree` package.

- Provides a hierarchical view of directory structures with files and subdirectories
- Respects filtering options:
  - gitignore respect can be toggled
  - case sensitivity can be toggled

- The output is a structured JSON representation of the directory tree:

```json
[
  {
    "dir": "path/to/directory",
    "entries": [
      { "type": "file", "name": "file1.txt" },
      { "type": "directory", "name": "subdir" }
    ]
  },
  {
    "dir": "path/to/directory/subdir",
    "entries": [
      { "type": "file", "name": "file2.md" }
    ]
  }
]
```

### File Viewing

A function is defined to display file contents when given a file path, with support for line-based filtering.

The view_file function returns a structured FileView with type-safe content representation:

```rust
pub struct ViewOptions {
    pub max_size: Option<usize>,
    pub line_from: Option<usize>,
    pub line_to: Option<usize>,
}

pub struct FileView {
    pub file_path: PathBuf,
    pub file_type: String,
    pub contents: FileContents,
    pub total_line_num: Option<usize>, // Total number of lines, only present for text files
}

// Content is represented as an enum with different variants
pub enum FileContents {
    Text { content: TextContent, metadata: TextMetadata },
    Binary { message: String, metadata: BinaryMetadata },
    Image { message: String, metadata: ImageMetadata },
}

pub struct TextContent {
    pub line_contents: Vec<LineContent>,
}

pub struct LineContent {
    pub line_number: usize,
    pub line: String,
}
```

Features:
- File type detection for text, binary, and image files
- Size limiting to avoid loading very large files
- Line-based filtering to view specific portions of text files
- Optimized size checking when line filtering is used (allows viewing portions of large files)
- Total line number information for text files via the `total_line_num` field
- Graceful handling of out-of-range line specifications

The command line output format is:
```
filepath:line_number:content
```

For binary files, the output is simplified to show file type information:
```
filepath: Binary file detected, size: X bytes, type: Y
```

## Common Features Across Modules

All modules share these common features:

- Option to respect or ignore gitignore files
- Case sensitivity options for file matching
- Structured output formats with rich metadata

## Technical Implementation

For more detailed information about implementation details, challenges faced, and solutions applied, please refer to the devlog.md file.