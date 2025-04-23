# Development Log

## Implementation Summary

This document describes the implementation of the file-searcher utility, focusing on design decisions, challenges encountered, and solutions applied.

## Core Features

### Search Functionality (`search/mod.rs`)
- **Description**: Searches for patterns in files using the `grep` crate
- **Key components**:
  - `SearchOptions`: Controls case sensitivity and gitignore respect
  - `SearchResult`: Contains matched file path, line number, and content
  - `search_files()`: Main search function
  - `collect_files()`: Helper to gather files respecting gitignore settings

### Traverse Functionality (`traverse/mod.rs`)
- **Description**: Traverses directories and lists files using the `ignore` crate
- **Key components**:
  - `TraverseOptions`: Controls case sensitivity, gitignore respect, and text-only filtering
  - `TraverseResult`: Contains file path and file type
  - `is_hidden()`: Detects hidden files and files in hidden directories
  - `traverse_directory()`: Main directory traversal function

### View Functionality (`view/mod.rs`)
- **Description**: Displays file contents with metadata using the `infer` crate
- **Key components**:
  - `ViewOptions`: Controls size limits
  - `FileView`: Structured output with file path, type, and contents
  - `view_file()`: Main function for viewing files

### CLI Interface (`main.rs`)
- **Description**: Command-line interface using the `clap` crate
- **Key components**:
  - `Cli`: Main CLI structure with subcommands
  - `Commands`: Enum of available commands (search, traverse, view)
  - Command-specific option handling

## Technical Considerations

### Error Handling
- Using `anyhow` crate for comprehensive error handling
- Context-rich error messages with `with_context` and `context` methods
- Proper propagation of errors using the `?` operator
- Early returns for validation errors

### File Type Detection
- Multi-layered approach to file type detection:
  1. Extension-based detection for common file types
  2. Content-based detection using the `infer` crate
  3. Heuristic text/binary detection as fallback
- Special handling for different file categories (text, image, binary)

### Gitignore Handling
- Comprehensive gitignore handling using the `ignore` crate
- Special attention to hidden files and directories
- Options to respect or ignore gitignore rules
- Proper handling of the `.hidden` builder flag (critical for test passing)

### JSON Output Structure
- Structured JSON output for the view command:
  - For text files: Content and metadata (line count, character count)
  - For binary files: Message about binary detection and metadata
  - For images: Special identification and metadata

### Testing
- Comprehensive test suite with serial execution using `serial_test`
- Test setup/teardown using `TestEnvironment` with RAII pattern
- Tests for all edge cases:
  - Case sensitivity
  - Gitignore respect/ignore
  - Hidden files
  - Binary files
  - Size limits

## Challenges and Solutions

### Gitignore Handling
- **Challenge**: Proper handling of gitignore files, especially when toggling respect_gitignore flag
- **Solution**: 
  ```rust
  // When not respecting gitignore, explicitly include hidden files and dirs
  builder.hidden(options.respect_gitignore);
  // Additional settings to ensure we fully respect/ignore gitignore as needed
  if !options.respect_gitignore {
      builder.ignore(false); // Turn off all ignore logic
      builder.git_exclude(false); // Don't use git exclude files
      builder.git_global(false); // Don't use global git ignore
  }
  ```

### Hidden File Detection
- **Challenge**: Properly identifying files in hidden directories
- **Solution**: Enhanced the `is_hidden()` method to check both file name and path components:
  ```rust
  pub fn is_hidden(&self) -> bool {
      // Check if the file name starts with a dot
      let file_is_hidden = self.file_path
          .file_name()
          .and_then(|n| n.to_str())
          .is_some_and(|name| name.starts_with("."));
          
      // Also check if the file is in a hidden directory
      let path_contains_hidden_dir = self.file_path
          .to_string_lossy()
          .split('/')
          .any(|part| part.starts_with(".") && !part.is_empty());
          
      file_is_hidden || path_contains_hidden_dir
  }
  ```

### File Type Detection
- **Challenge**: Reliable detection of file types, especially distinguishing text and binary
- **Solution**: Multi-layered approach combining extension hints and content analysis:
  ```rust
  // First try extension-based detection
  let extension_type = path
      .extension()
      .and_then(|ext| ext.to_str())
      .map(|ext| match ext.to_lowercase().as_str() {
          "txt" | "md" | "rs" | "toml" => Some("text/plain"),
          // ... more extensions ...
          _ => None
      })
      .unwrap_or(None);
      
  // Then try content-based detection with fallbacks
  let file_type = match infer.get_from_path(path) {
      Ok(Some(kind)) => kind.mime_type().to_string(),
      Ok(None) => {
          if let Some(ext_type) = extension_type {
              ext_type.to_string()
          } else {
              // Heuristic analysis as last resort
              // ...
          }
      },
      Err(e) => return Err(anyhow!("Failed to determine file type: {}", e)),
  };
  ```

### JSON Structure
- **Challenge**: Creating a flexible, informative JSON structure for file contents
- **Solution**: Structured JSON with different formats for text vs. binary:
  ```rust
  let contents = if file_type.starts_with("text/") {
      // Text files get content + metadata
      json!({
          "content": text,
          "metadata": {
              "line_count": line_count,
              "char_count": char_count
          }
      })
  } else if file_type.starts_with("image/") {
      // Special handling for images
      json!({
          "message": format!("Image file detected: {}", file_type),
          "metadata": {
              "binary": true,
              "size_bytes": metadata.len(),
              "media_type": "image"
          }
      })
  } else {
      // Other binary files
      json!({
          "message": format!("Binary file detected, size: {} bytes", metadata.len()),
          "metadata": {
              "binary": true,
              "size_bytes": metadata.len(),
              "mime_type": file_type
          }
      })
  };
  ```

### Test Compatibility
- **Challenge**: Making tests work with changing output formats
- **Solution**: Added fallback paths in tests to handle both formats:
  ```rust
  if let Some(content) = file_view.contents.get("content") {
      // Check new format
      assert!(content.as_str().unwrap_or("").contains("#"));
  } else {
      // Fallback for old format
      assert!(file_view.contents.to_string().contains("#"));
  }
  ```

## Future Work

### Performance Improvements
- Consider adding parallelism for faster searches in large codebases
- Implement caching for frequently accessed directories
- Add progress indicators for long-running operations

### Feature Enhancements
- Add support for other VCS ignore files (.hgignore, .svnignore)
- Implement find/replace functionality
- Add interactive mode for viewing large files
- Support for archive traversal (zip, tar)

### Output Formats
- Add additional output formats (CSV, plain text)
- Support for colorized output
- Code-aware output formatting based on file type

### Code Quality
- Continue improving error messages
- Add more detailed documentation
- Consider breaking large modules into smaller, more focused ones