//! Tests for path prefix removal functionality in the tree module.

use anyhow::Result;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

use crate::tree::{generate_tree, TreeOptions};

/// Creates a temporary directory with test files for path prefix testing
fn create_test_directory_structure(dir: &Path) -> Result<()> {
    // Create a set of test files with nested directories
    let test_files = [
        "file1.txt",
        "file2.rs",
        "src/main.rs",
        "src/lib.rs",
        "src/utils/helper.rs",
        "docs/readme.md",
        "docs/examples/example1.md",
        "docs/examples/example2.md",
        "tests/test_main.rs",
    ];

    for file_path in &test_files {
        let full_path = dir.join(file_path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(full_path)?;
        file.write_all(format!("Test content for {}", file_path).as_bytes())?;
    }

    Ok(())
}

/// Normalizes path separators to forward slashes for consistent testing
fn normalize_path(path: &str) -> String {
    path.replace("\\", "/")
}

#[test]
fn test_omit_path_prefix_basic() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create test directory structure
    create_test_directory_structure(temp_path)?;
    
    // Test with path prefix removal
    let options = TreeOptions {
        case_sensitive: false,
        respect_gitignore: false, // No gitignore in temp dir
        depth: None,
        omit_path_prefix: Some(temp_path.to_path_buf()),
    };
    
    let tree_result = generate_tree(temp_path, &options)?;
    
    // Verify results
    assert!(!tree_result.is_empty(), "Tree result should not be empty");
    
    // Check that directory paths don't have the temp path prefix
    for dir_tree in &tree_result {
        let normalized_dir = normalize_path(&dir_tree.dir);
        assert!(!normalized_dir.contains(temp_path.to_string_lossy().as_ref()),
                "Directory path '{}' should not contain the temp path prefix", normalized_dir);
        
        // Check that relative paths like "src", "docs", etc. are preserved
        // We can't assert exact paths due to platform differences, but we can check that paths are short
        assert!(normalized_dir.len() < temp_path.to_string_lossy().len(),
                "Directory path '{}' should be shorter than the original temp path", normalized_dir);
    }
    
    // Check that all expected directory structures are present
    let dir_names: Vec<String> = tree_result
        .iter()
        .map(|d| normalize_path(&d.dir))
        .collect();
    
    // Check for the root directory (could be "" or "." depending on implementation)
    assert!(dir_names.iter().any(|d| d.is_empty() || d == "."), 
            "Root directory should be present in the results");
    
    // Check for expected subdirectories
    let expected_dirs = ["src", "docs", "docs/examples", "tests", "src/utils"];
    for expected in &expected_dirs {
        let found = dir_names.iter().any(|d| d.ends_with(expected));
        assert!(found, "Directory '{}' should be present in the results", expected);
    }
    
    Ok(())
}

#[test]
fn test_omit_path_prefix_without_removal() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create test directory structure
    create_test_directory_structure(temp_path)?;
    
    // Test without path prefix removal
    let options = TreeOptions {
        case_sensitive: false,
        respect_gitignore: false,
        depth: None,
        omit_path_prefix: None, // No prefix removal
    };
    
    let tree_result = generate_tree(temp_path, &options)?;
    
    // Verify that directory paths contain the temp path prefix
    for dir_tree in &tree_result {
        let normalized_dir = normalize_path(&dir_tree.dir);
        assert!(normalized_dir.starts_with(&normalize_path(&temp_path.to_string_lossy())),
                "Directory path '{}' should start with the temp path prefix", normalized_dir);
    }
    
    Ok(())
}

#[test]
fn test_omit_path_prefix_partial_match() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create test directory structure
    create_test_directory_structure(temp_path)?;
    
    // Create a path that is not a prefix of our temp directory
    let non_matching_prefix = if cfg!(windows) {
        PathBuf::from("C:\\some\\other\\path")
    } else {
        PathBuf::from("/some/other/path")
    };
    
    let options = TreeOptions {
        case_sensitive: false,
        respect_gitignore: false,
        depth: None,
        omit_path_prefix: Some(non_matching_prefix.clone()),
    };
    
    let tree_result = generate_tree(temp_path, &options)?;
    
    // Verify that directory paths are unchanged (since prefix doesn't match)
    for dir_tree in &tree_result {
        let normalized_dir = normalize_path(&dir_tree.dir);
        assert!(normalized_dir.starts_with(&normalize_path(&temp_path.to_string_lossy())),
                "Directory path '{}' should start with the temp path prefix when non-matching prefix is used", 
                normalized_dir);
    }
    
    Ok(())
}

#[test]
fn test_omit_path_prefix_with_depth_limit() -> Result<()> {
    // Create a temporary directory
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create test directory structure
    create_test_directory_structure(temp_path)?;
    
    // Test with depth limit and path prefix removal
    let options = TreeOptions {
        case_sensitive: false,
        respect_gitignore: false,
        depth: Some(1), // Only top-level directories
        omit_path_prefix: Some(temp_path.to_path_buf()),
    };
    
    let tree_result = generate_tree(temp_path, &options)?;
    
    // Verify results have prefixes removed and respect depth limit
    assert!(!tree_result.is_empty(), "Tree result should not be empty");
    
    // Check that directory paths don't have the temp path prefix
    for dir_tree in &tree_result {
        let normalized_dir = normalize_path(&dir_tree.dir);
        assert!(!normalized_dir.contains(temp_path.to_string_lossy().as_ref()),
                "Directory path '{}' should not contain the temp path prefix", normalized_dir);
        
        // For depth 1, we should only see entries like "", "src", "docs", "tests"
        // and not deeper paths like "src/utils" or "docs/examples"
        assert!(!normalized_dir.contains("/"), 
                "With depth=1, directory path '{}' should not contain subdirectories", normalized_dir);
    }
    
    Ok(())
}