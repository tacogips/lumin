# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

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

- Logic is defined in the `search` package.
  Specify a target directory to grep through files under that directory.
  Files listed in .gitignore (if present in the target directory) are excluded by default, but this can be overridden with a parameter.
- Case sensitivity can be toggled via parameters.

### File Traversal

- Logic is defined in the `traverse` package.

- Specify a target directory to search for file names under that directory.
  Files listed in .gitignore (if present in the target directory) are excluded by default, but this can be overridden with a parameter.

- Case sensitivity can be toggled via parameters.

- By default, the library uses the `infer` crate and only returns files that cannot be parsed by this crate (identifying them as text files). This filtering behavior can be toggled via parameters.

### File Viewing

A function is defined to display file contents when given a file path.

The function returns the following:

```json
{
  "file_path": "...",
  "file_type": "...",
  "contents": "..."
}
```
