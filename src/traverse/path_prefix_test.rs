//! Tests for path prefix removal functionality in the traverse module.

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::traverse::{TraverseOptions, traverse_directory};

/// Creates a temporary directory with test files for path prefix testing
fn create_test_files(dir: &Path) -> Result<Vec<String>> {
    // Create a structured set of test files with various extensions and in subdirectories
    let test_files = [
        "file1.txt",
        "file2.rs",
        "README.md",
        "src/main.rs",
        "src/lib.rs",
        "docs/api.md",
        "tests/test_util.rs",
        "assets/images/logo.png",
    ];

    for file_path in &test_files {
        let full_path = dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(full_path)?;
        file.write_all(format!("Test content for {}", file_path).as_bytes())?;
    }

    Ok(test_files.iter().map(|s| s.to_string()).collect())
}

/// Normalizes path separators to forward slashes for consistent testing
fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace("\\", "/").to_string()
}

#[test]
fn test_omit_path_prefix_basic() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    let test_files = create_test_files(temp_path)?;

    // Test with path prefix removal
    let options = TraverseOptions {
        case_sensitive: false,
        respect_gitignore: false, // No gitignore in temp dir
        only_text_files: false,   // Include all files for testing
        pattern: None,
        depth: None,
        omit_path_prefix: Some(temp_path.to_path_buf()),
    };

    let results = traverse_directory(temp_path, &options)?;

    // Verify results
    assert_eq!(
        results.len(),
        test_files.len(),
        "Should find all test files"
    );

    // Check that prefixes were removed
    for result in &results {
        // Paths should not start with the temp directory
        assert!(
            !result.file_path.starts_with(temp_path),
            "Path {} should not start with prefix {}",
            result.file_path.display(),
            temp_path.display()
        );

        // Normalize paths for comparison
        let normalized_path = normalize_path(&result.file_path);

        // Check that each result corresponds to one of our test files
        let found = test_files.iter().any(|f| normalized_path == *f);
        assert!(
            found,
            "File path '{}' not found in test files",
            normalized_path
        );
    }

    Ok(())
}

#[test]
fn test_omit_path_prefix_without_removal() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    let _test_files = create_test_files(temp_path)?;

    // Test without path prefix removal
    let options = TraverseOptions {
        case_sensitive: false,
        respect_gitignore: false,
        only_text_files: false,
        pattern: None,
        depth: None,
        omit_path_prefix: None, // No prefix removal
    };

    let results = traverse_directory(temp_path, &options)?;

    // Check that paths retain their prefix
    for result in &results {
        assert!(
            result.file_path.starts_with(temp_path),
            "Path {} should start with prefix {}",
            result.file_path.display(),
            temp_path.display()
        );
    }

    Ok(())
}

#[test]
fn test_omit_path_prefix_with_pattern() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    let _test_files = create_test_files(temp_path)?;

    // Test with both path prefix removal and a pattern
    let options = TraverseOptions {
        case_sensitive: false,
        respect_gitignore: false,
        only_text_files: false,
        pattern: Some("**/*.rs".to_string()), // Only Rust files
        depth: None,
        omit_path_prefix: Some(temp_path.to_path_buf()),
    };

    let results = traverse_directory(temp_path, &options)?;

    // Should only find Rust files
    assert!(results.len() > 0, "Should find some Rust files");
    assert!(results.len() < 8, "Should not find all test files");

    // Check that only Rust files are included and prefixes are removed
    for result in &results {
        // Path should not start with prefix
        assert!(!result.file_path.starts_with(temp_path));

        // Path should end with .rs
        let normalized_path = normalize_path(&result.file_path);
        assert!(
            normalized_path.ends_with(".rs"),
            "File '{}' should be a Rust file",
            normalized_path
        );

        // File type should be "rs"
        assert_eq!(result.file_type, "rs");
    }

    Ok(())
}

#[test]
fn test_omit_path_prefix_partial_match() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    let _test_files = create_test_files(temp_path)?;

    // Create a path that is not a prefix of our temp directory
    let non_matching_prefix = if cfg!(windows) {
        PathBuf::from("C:\\some\\other\\path")
    } else {
        PathBuf::from("/some/other/path")
    };

    let options = TraverseOptions {
        case_sensitive: false,
        respect_gitignore: false,
        only_text_files: false,
        pattern: None,
        depth: None,
        omit_path_prefix: Some(non_matching_prefix.clone()),
    };

    let results = traverse_directory(temp_path, &options)?;

    // Paths should remain unchanged since the prefix doesn't match
    for result in &results {
        assert!(
            result.file_path.starts_with(temp_path),
            "Path {} should start with temp dir {} when prefix {} doesn't match",
            result.file_path.display(),
            temp_path.display(),
            non_matching_prefix.display()
        );
    }

    Ok(())
}

#[test]
fn test_omit_path_prefix_with_depth_limit() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    // Create test files
    let _test_files = create_test_files(temp_path)?;

    // Test with depth limit and path prefix removal
    let options = TraverseOptions {
        case_sensitive: false,
        respect_gitignore: false,
        only_text_files: false,
        pattern: None,
        depth: Some(1), // Only files in the root directory
        omit_path_prefix: Some(temp_path.to_path_buf()),
    };

    let results = traverse_directory(temp_path, &options)?;

    // Should only find files in the root directory
    assert!(results.len() > 0, "Should find some files");
    assert!(results.len() < 8, "Should not find files in subdirectories");

    // Check that paths are correctly processed
    for result in &results {
        // Path should not start with prefix
        assert!(!result.file_path.starts_with(temp_path));

        // Path should not contain any directory separators
        let normalized_path = normalize_path(&result.file_path);
        assert!(
            !normalized_path.contains('/'),
            "File '{}' should be in the root directory",
            normalized_path
        );
    }

    Ok(())
}
