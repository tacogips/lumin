# File Searcher

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