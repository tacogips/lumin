# lumin: A File Searcher

A Rust utility for searching, traversing, and viewing files with rich filtering options and type-aware content handling.

## Features

- **Search**: Find text patterns in files using regex
- **Traverse**: List files in directories with advanced filtering
- **View**: Display file contents with type detection (text, binary, image)

## Installation

```
cargo install --path .
```

## Usage

### Search for text patterns

```
lumin search <PATTERN> <DIRECTORY> [OPTIONS]
```

Options:

- `--case-sensitive`: Enable case-sensitive matching
- `--ignore-gitignore`: Ignore .gitignore rules
- `--omit-context <NUM>`: Limit context around matches to show only NUM characters before and after each match (the matched pattern itself is always displayed in full)
- `-B, --before-context <NUM>`: Show NUM lines before each match (similar to grep's -B option)
- `-A, --after-context <NUM>`: Show NUM lines after each match (similar to grep's -A option)
- Both -B and -A can be combined to show context on both sides of matches

### Traverse directories

```
lumin traverse <DIRECTORY> [OPTIONS]
```

Options:

- `--case-sensitive`: Enable case-sensitive filtering
- `--ignore-gitignore`: Ignore .gitignore rules
- `--all-files`: Include binary files (default: text files only)

### View file contents

```
lumin view <FILE_PATH> [OPTIONS]
```

The view command outputs a structured JSON with:

- File path
- File type
- Contents (text, binary, or image with appropriate metadata)

## Key Features

- Gitignore-aware operations
- Type detection using extension and content analysis
- Strongly typed output structures
- Comprehensive error handling
- Comprehensive context control for search results (before/after matches) to focus on relevant code

## Development

```
# Build
cargo build

# Test
cargo test

# Format code
cargo fmt

# Run linter
cargo clippy
```

## License

MIT

## Contributing

See [CONTRIBUTING.md](./tests/test_dir_1/docs/CONTRIBUTING.md) for guidelines.
