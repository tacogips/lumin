use anyhow::Result;
use lumin::traverse::{TraverseOptions, traverse_directory};
use std::path::Path;

#[test]
fn test_traverse_basic() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions::default();

    let results = traverse_directory(directory, &options)?;

    // Should find all text files (default ignores binary files)
    assert!(!results.is_empty());

    // Should find text files
    let file_paths: Vec<String> = results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    // Check for specific files
    assert!(file_paths.iter().any(|path| path.contains("sample.txt")));
    assert!(file_paths.iter().any(|path| path.contains("markdown.md")));
    assert!(file_paths.iter().any(|path| path.contains("config.toml")));
    assert!(file_paths.iter().any(|path| path.contains("file.rs")));

    // We won't test for binary file exclusion in the basic test
    // since file type detection can vary by environment

    // Should not find gitignored files (default respects gitignore)
    assert!(file_paths.iter().all(|path| !path.contains("temp.tmp")));
    assert!(file_paths.iter().all(|path| !path.contains("log.log")));
    assert!(file_paths.iter().all(|path| !path.contains(".hidden")));

    Ok(())
}

#[test]
fn test_traverse_with_binary_files() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        only_text_files: false,
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;

    // Should find text and binary files
    let file_paths: Vec<String> = results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    // Should find both text and binary files
    assert!(file_paths.iter().any(|path| path.contains("sample.txt")));
    assert!(file_paths.iter().any(|path| path.contains("binary.bin")));
    assert!(file_paths.iter().any(|path| path.contains("sample.jpg")));

    Ok(())
}

#[test]
fn test_traverse_without_gitignore_respect() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        respect_gitignore: false,
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;

    // Should find files that would normally be ignored
    let file_paths: Vec<String> = results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    // Should find files normally ignored by gitignore
    assert!(file_paths.iter().any(|path| path.contains("temp.tmp")));
    assert!(file_paths.iter().any(|path| path.contains("log.log")));
    assert!(
        file_paths
            .iter()
            .any(|path| path.contains(".hidden/secret.txt"))
    );

    Ok(())
}

#[test]
fn test_traverse_with_glob_pattern() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        pattern: Some("**/*.txt".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;

    // Should find only .txt files
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .all(|r| r.file_path.to_string_lossy().ends_with(".txt"))
    );

    // Should not find other text files like .md or .rs
    assert!(
        results
            .iter()
            .all(|r| !r.file_path.to_string_lossy().ends_with(".md"))
    );
    assert!(
        results
            .iter()
            .all(|r| !r.file_path.to_string_lossy().ends_with(".rs"))
    );

    Ok(())
}

#[test]
fn test_traverse_with_substring_pattern() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        pattern: Some("level".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;

    // Should find files with "level" in the name
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .all(|r| r.file_path.to_string_lossy().contains("level"))
    );

    // Should include both level1.txt and level2.txt
    let file_paths: Vec<String> = results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    assert!(file_paths.iter().any(|path| path.contains("level1.txt")));
    assert!(file_paths.iter().any(|path| path.contains("level2.txt")));

    Ok(())
}

#[test]
fn test_is_hidden_check() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        respect_gitignore: false,
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;

    // Find hidden files and verify is_hidden() returns true
    let hidden_files: Vec<_> = results
        .iter()
        .filter(|r| r.file_path.to_string_lossy().contains(".hidden"))
        .collect();

    assert!(!hidden_files.is_empty());
    for file in hidden_files {
        assert!(
            file.is_hidden(),
            "File in .hidden directory should be marked as hidden"
        );
    }

    Ok(())
}
