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

## Technical Implementation

For more detailed information about implementation details, challenges faced, and solutions applied, please refer to the devlog.md file.