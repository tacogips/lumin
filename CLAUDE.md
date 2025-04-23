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

## このライブラリの機能

ローカルファイルの検索と内容の表示を行います

### fileの検索

https://github.com/BurntSushi/ripgrep(https://crates.io/crates/ripgrep)および
