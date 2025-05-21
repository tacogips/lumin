use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use serial_test::serial;
use std::path::Path;

mod test_helpers;
use test_helpers::{TEST_DIR, TestEnvironment, setup_multiple_file_types};

/// Tests for the include_glob feature in search functionality
#[cfg(test)]
mod search_include_glob_tests {
    use super::*;

    /// Test including files using simple glob patterns
    #[test]
    #[serial]
    fn test_include_single_glob() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Search with a pattern that should be in multiple file types
        let pattern = "content";
        let mut options = SearchOptions::default();
        
        // Search without including anything specific first to confirm our test pattern exists in multiple files
        let all_results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Verify we have results in different file types
        assert!(
            all_results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Expected to find the pattern in JSON files"
        );
        assert!(
            all_results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".txt")),
            "Expected to find the pattern in TXT files"
        );

        // Now search with include_glob for JSON files only
        options.include_glob = Some(vec!["*.json".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should still find matches
        assert!(!results.lines.is_empty(), "Expected to find matches in JSON files");
        
        // Should only find JSON files
        assert!(
            results.lines.iter().all(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Found non-JSON files despite only including JSON files"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test including files using multiple glob patterns
    #[test]
    #[serial]
    fn test_include_multiple_globs() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Search with a pattern that should be in multiple file types
        let pattern = "content";
        
        // Include only JSON and YAML files
        let mut options = SearchOptions::default();
        options.include_glob = Some(vec!["*.json".to_string(), "*.yaml".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find matches
        assert!(!results.lines.is_empty(), "Expected to find matches in JSON or YAML files");
        
        // Should only find JSON or YAML files
        assert!(
            results.lines.iter().all(|r| {
                let path = r.file_path.to_string_lossy();
                path.ends_with(".json") || path.ends_with(".yaml")
            }),
            "Found files other than JSON or YAML despite only including those types"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test including files with recursive glob patterns
    #[test]
    #[serial]
    fn test_include_recursive_glob() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Pattern that should be in multiple directories
        let pattern = "content";
        
        // Verify we have files in various directories
        let all_results = search_files(pattern, Path::new(TEST_DIR), &SearchOptions::default())?;
        assert!(
            all_results.lines.iter().any(|r| r.file_path.to_string_lossy().contains("/docs/")),
            "Expected to find the pattern in the docs directory"
        );
        
        // Include only files in the docs directory and subdirectories
        let mut options = SearchOptions::default();
        // Use the path format that would match our test directory structure
        options.include_glob = Some(vec!["**/docs/**".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find matches
        assert!(!results.lines.is_empty(), "Expected to find matches in docs directory");
        
        // Should only find files in the docs directory
        assert!(
            results.lines.iter().all(|r| r.file_path.to_string_lossy().contains("/docs/")),
            "Found files outside the docs directory despite only including it"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test case sensitivity in glob patterns
    #[test]
    #[serial]
    fn test_include_glob_case_sensitivity() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Create files with mixed case extensions in the test directory
        let mixed_case_file1 = Path::new(TEST_DIR).join("test.JsonML");
        let mixed_case_file2 = Path::new(TEST_DIR).join("test.JSON"); // All caps
        std::fs::write(&mixed_case_file1, "This file has content with a mixed case extension")?;
        std::fs::write(&mixed_case_file2, "This file has content with an all caps extension")?;
        
        let pattern = "content";
        
        // Test with case-sensitive mode
        let mut options = SearchOptions::default();
        options.case_sensitive = true;
        options.include_glob = Some(vec!["*.json".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should only find lowercase .json files, not .JSON or .JsonML
        assert!(
            results.iter().all(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Found non-lowercase .json files despite case sensitivity"
        );
        
        // Case-insensitive mode test
        let mut options = SearchOptions::default();
        options.case_sensitive = false;
        options.include_glob = Some(vec!["*.json".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find all json files regardless of case
        assert!(
            results.iter().all(|r| {
                let path = r.file_path.to_string_lossy();
                path.ends_with(".json") || path.ends_with(".JSON") || path.ends_with(".JsonML")
            }),
            "Expected to find all JSON files case-insensitively"
        );

        // Cleanup
        std::fs::remove_file(&mixed_case_file1)?;
        std::fs::remove_file(&mixed_case_file2)?;
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test that an empty include_glob list includes nothing (since no files match)
    #[test]
    #[serial]
    fn test_empty_include_glob() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        let pattern = "content";
        
        // First ensure we have matches with default options
        let default_results = search_files(pattern, Path::new(TEST_DIR), &SearchOptions::default())?;
        assert!(!default_results.is_empty(), "Expected to find matches with default options");
        
        // Create options with an empty include_glob list
        let mut options = SearchOptions::default();
        options.include_glob = Some(vec![]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find no matches as empty include_glob matches nothing
        assert!(results.is_empty(), "Expected to find no matches with empty include_glob");

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }
    
    /// Test include_glob with None vs. empty vec behavior
    #[test]
    #[serial]
    fn test_include_glob_none_vs_empty() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        let pattern = "content";
        
        // With include_glob = None (default), should find all files
        let default_options = SearchOptions::default();
        let default_results = search_files(pattern, Path::new(TEST_DIR), &default_options)?;
        assert!(!default_results.is_empty(), "Expected to find matches with include_glob = None");
        
        // With include_glob = Some(vec![]), should find nothing
        let mut empty_options = SearchOptions::default();
        empty_options.include_glob = Some(vec![]);
        let empty_results = search_files(pattern, Path::new(TEST_DIR), &empty_options)?;
        assert!(empty_results.is_empty(), "Expected to find no matches with include_glob = Some(empty vec)");
        
        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test combining include_glob with gitignore
    #[test]
    #[serial]
    fn test_include_glob_with_gitignore() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        let pattern = "content";
        
        // Use include_glob with respect_gitignore=true (default)
        let mut options = SearchOptions::default();
        options.include_glob = Some(vec!["*.md".to_string(), "*.log".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find markdown files
        assert!(
            results.iter().any(|r| r.file_path.to_string_lossy().ends_with(".md")),
            "Expected to find markdown files"
        );
        
        // Should not find log files despite including them (because of gitignore)
        assert!(
            !results.iter().any(|r| r.file_path.to_string_lossy().ends_with(".log")),
            "Found log files despite them being in gitignore"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }
    
    /// Test combining include_glob with exclude_glob
    #[test]
    #[serial]
    fn test_include_and_exclude_glob_combination() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        let pattern = "content";
        
        // Include all text files but exclude those in the docs directory
        let mut options = SearchOptions::default();
        options.include_glob = Some(vec!["**/*.txt".to_string()]);
        options.exclude_glob = Some(vec!["docs/**".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find matches
        assert!(!results.is_empty(), "Expected to find matches in text files outside docs");
        
        // Should only find .txt files
        assert!(
            results.iter().all(|r| r.file_path.to_string_lossy().ends_with(".txt")),
            "Found non-txt files despite only including txt files"
        );
        
        // Should not find any files in the docs directory
        assert!(
            !results.iter().any(|r| r.file_path.to_string_lossy().contains("/docs/")),
            "Found files in docs directory despite excluding it"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }
    
    /// Test various glob syntax patterns
    #[test]
    #[serial]
    fn test_include_glob_syntax() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Create some additional files for testing glob syntax
        let test_files = [
            ("test1.rs", "fn main() { println!(\"content\"); }"),
            ("test2.rs", "fn test() { println!(\"content\"); }"),
            ("test.py", "print(\"content\")"),
            ("script.py", "print(\"python content\")"),
            ("nested/deep/file.txt", "deep nested content"),
        ];
        
        let mut created_files = Vec::new();
        for (filename, content) in &test_files {
            let file_path = Path::new(TEST_DIR).join(filename);
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            std::fs::write(&file_path, content)?;
            created_files.push(file_path);
        }

        let pattern = "content";
        
        // Test 1: Brace expansion - match multiple extensions
        let mut options = SearchOptions::default();
        options.include_glob = Some(vec!["**/*.{rs,py}".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        assert!(!results.is_empty(), "Expected to find matches with brace expansion");
        assert!(
            results.iter().all(|r| {
                let path = r.file_path.to_string_lossy();
                path.ends_with(".rs") || path.ends_with(".py")
            }),
            "Found files other than .rs or .py with brace expansion"
        );
        
        // Test 2: Character class - match test[digit].rs
        let mut options = SearchOptions::default();
        options.include_glob = Some(vec!["**/test[0-9].rs".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        assert!(!results.is_empty(), "Expected to find matches with character class");
        assert!(
            results.iter().all(|r| {
                let path = r.file_path.to_string_lossy();
                path.ends_with("test1.rs") || path.ends_with("test2.rs")
            }),
            "Found files other than test[digit].rs with character class"
        );
        
        // Test 3: Double asterisk - find files in any directory depth
        let mut options = SearchOptions::default();
        options.include_glob = Some(vec!["**/file.txt".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        assert!(!results.is_empty(), "Expected to find matches with double asterisk");
        assert!(
            results.iter().any(|r| r.file_path.to_string_lossy().contains("nested/deep/file.txt")),
            "Failed to find deeply nested file with double asterisk"
        );

        // Cleanup
        for file in &created_files {
            std::fs::remove_file(file)?;
            // Clean up any created directories too
            if let Some(parent) = file.parent() {
                if parent != Path::new(TEST_DIR) && parent.exists() {
                    let _ = std::fs::remove_dir_all(parent);
                }
            }
        }
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }
}