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

ローカルファイルの検索と内容の表示を行います。

### fileの検索

- `search` package内にロジックが定義してあります。
  検索対象directoryを指定することでそのdirectory以下のfileをgrepします
  検索対象directoryに .gitignore がある場合はそこに記述されているfileは除外しますが、関数に渡すparameterで除外しないようにもします。
- その他ignore caseするかどうかをパラメータで切り替えられます

### fileのtraverse

- `traverse` package内にロジックが定義してあります

- 検索対象directoryを指定することでそのdirectory以下のfile名を検索します。
  検索対象directoryに .gitignore がある場合はそこに記述されているfileは除外しますが、関数に渡すparameterで除外しないようにもします。

- その他ignore caseするかどうかをパラメータで切り替えられます

- defaultの挙動として、infer crateを使用し、このcrateでparseできないfileはtext fileであると判定し、そのtext fileのみを返します。この除外を許可するかどうかをパラメータで切り替えられます。

### fileのview

file pathを指定し内容を表示する関数が定義してあります。

関数の戻り値は下記です

```json
{
  "file_path": "...",
  "file_type": "...",
  "contents": "..."
}
```
