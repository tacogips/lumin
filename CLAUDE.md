# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Documentation

When working with this codebase, refer to the following key documents:

- **CLAUDE.md** (this file): Primary instructions for working with the codebase
- **devlog.md**: Development log with design decisions, implementation details, and roadmap

To understand the history, architecture decisions, and implementation details of this project, always refer to the devlog.md file. When making significant changes, update devlog.md to document your work by following the instructions at the top of that file.

## Build & Run Commands

- Build: `cargo build`
- Run: `cargo run`
- Release build: `cargo build --release`
- Test: `cargo test`
- Run single test: `cargo test test_name`
- Lint: `cargo clippy`
- Format: `cargo fmt`

## Code Style Guidelines

- Use Rust 2024 edition conventions
- Format with `rustfmt` (default settings)
- Use descriptive variable and function names in snake_case
- Prefer Result<T, E> over unwrap()/expect() for error handling
- Organize imports alphabetically with std first, then external crates
- Use structured logging via the log crate when implementing logging
- Add type annotations for public functions/methods
- Match arms should be aligned
- Use Rust's ownership system effectively (avoid unnecessary clones)
- Actively use cargo-docs (mcp) to investigate crate usage patterns

## Library Features

This library provides functionality for searching and displaying local files.

### File Searching

- Primarily using the `grep` crate as a library
- Logic is defined in the `search` package.
  Specify a target directory to grep through files under that directory.
  Files listed in .gitignore (if present in the target directory) are excluded by default, but this can be overridden with a parameter.
- Case sensitivity can be toggled via parameters.

### File Traversal

- Primarily using the `eza` crate as a library
- Logic is defined in the `traverse` package.

- Specify a target directory to search for file names under that directory.
  Files listed in .gitignore (if present in the target directory) are excluded by default, but this can be overridden with a parameter.

- Case sensitivity can be toggled via parameters.

- By default, the library uses the `infer` crate and only returns files that cannot be parsed by this crate (identifying them as text files). This filtering behavior can be toggled via parameters.

### File Viewing

A function is defined to display file contents when given a file path.

The view_file function returns a structured FileView with type-safe content representation:

```rust
pub struct FileView {
    pub file_path: PathBuf,
    pub file_type: String,
    pub contents: FileContents,
}

// Content is represented as an enum with different variants
pub enum FileContents {
    Text { content: String, metadata: TextMetadata },
    Binary { message: String, metadata: BinaryMetadata },
    Image { message: String, metadata: ImageMetadata },
}
```

When serialized to JSON, the output looks like:

```json
{
  "file_path": "path/to/file",
  "file_type": "text/plain",
  "contents": {
    "type": "text",
    "content": "file contents...",
    "metadata": {
      "line_count": 42,
      "char_count": 1234
    }
  }
}
```

## Development Guidelines

### Making Changes

When making significant changes to the codebase:

1. Follow the code style guidelines
2. Ensure all tests pass with `cargo test`
3. Run linting with `cargo clippy`
4. Format code with `cargo fmt`
5. **Document your changes in devlog.md**

### Documenting in devlog.md

After implementing significant changes:

1. Review your changes to understand what should be documented
2. Edit devlog.md to update relevant sections
3. Add a new entry in the "Recent Changes" section with today's date and a summary of changes
4. Follow the instructions at the top of devlog.md for proper documentation format

When asked to "update devlog.md", proceed directly to editing the file following the guidelines contained within it. This ensures that design decisions and implementation details are properly documented for future reference.
