use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use serial_test::serial;
use std::path::Path;

mod test_helpers;
use test_helpers::{TEST_DIR, TestEnvironment, setup_multiple_file_types};

/// Tests for the exclude_glob feature in search functionality
#[cfg(test)]
mod search_exclude_glob_tests {
    use super::*;

    /// Test excluding files using simple glob patterns
    #[test]
    #[serial]
    fn test_exclude_single_glob() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Search with a pattern that should be in multiple file types
        let pattern = "content";
        let mut options = SearchOptions::default();
        
        // Search without excluding anything first to confirm our test pattern exists
        let all_results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // There should be some JSON files in the results
        assert!(
            all_results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Expected to find the pattern in JSON files before exclusion"
        );

        // Now exclude JSON files
        options.exclude_glob = Some(vec!["*.json".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should still find matches
        assert!(!results.lines.is_empty(), "Expected to find matches in non-JSON files");
        
        // Should not find any JSON files
        assert!(
            !results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Found JSON files despite excluding them"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test excluding files using multiple glob patterns
    #[test]
    #[serial]
    fn test_exclude_multiple_globs() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Search with a pattern that should be in multiple file types
        let pattern = "content";
        
        // Exclude both JSON and YAML files
        let mut options = SearchOptions::default();
        options.exclude_glob = Some(vec!["*.json".to_string(), "*.yaml".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should still find matches
        assert!(!results.lines.is_empty(), "Expected to find matches in non-excluded files");
        
        // Should not find any JSON or YAML files
        assert!(
            !results.lines.iter().any(|r| {
                let path = r.file_path.to_string_lossy();
                path.ends_with(".json") || path.ends_with(".yaml")
            }),
            "Found JSON or YAML files despite excluding them"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test excluding files with recursive glob patterns
    #[test]
    #[serial]
    fn test_exclude_recursive_glob() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        // Pattern that should be in multiple directories
        let pattern = "content";
        
        // Verify we have files in the 'docs' directory
        let all_results = search_files(pattern, Path::new(TEST_DIR), &SearchOptions::default())?;
        assert!(
            all_results.lines.iter().any(|r| r.file_path.to_string_lossy().contains("/docs/")),
            "Expected to find the pattern in the docs directory"
        );
        
        // Exclude all files in the docs directory and subdirectories
        let mut options = SearchOptions::default();
        options.exclude_glob = Some(vec!["docs/**".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should still find matches
        assert!(!results.lines.is_empty(), "Expected to find matches in non-excluded directories");
        
        // Should not find anything in the docs directory
        assert!(
            !results.lines.iter().any(|r| r.file_path.to_string_lossy().contains("/docs/")),
            "Found files in the docs directory despite excluding it"
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
    fn test_exclude_glob_case_sensitivity() -> Result<()> {
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
        options.exclude_glob = Some(vec!["*.json".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should not find lowercase .json files
        assert!(
            !results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Found .json files despite excluding them with case sensitivity"
        );
        
        // Case-insensitive mode test
        let mut options = SearchOptions::default();
        options.case_sensitive = false;
        options.exclude_glob = Some(vec!["*.json".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should not find .json files
        assert!(
            !results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Found .json files despite excluding them case-insensitively"
        );
        
        // Test with explicit patterns for both uppercase and mixed case
        let mut options = SearchOptions::default();
        options.exclude_glob = Some(vec!["*.JSON".to_string(), "*.JsonML".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should not find files with these specific extensions
        assert!(
            !results.lines.iter().any(|r| {
                let path = r.file_path.to_string_lossy();
                path.ends_with(".JSON") || path.ends_with(".JsonML")
            }),
            "Found excluded files with specific case patterns"
        );

        // Cleanup
        std::fs::remove_file(&mixed_case_file1)?;
        std::fs::remove_file(&mixed_case_file2)?;
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test that an empty exclude_glob list doesn't exclude anything
    #[test]
    #[serial]
    fn test_empty_exclude_glob() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        let pattern = "content";
        
        // Create options with an empty exclude_glob list
        let mut options = SearchOptions::default();
        options.exclude_glob = Some(vec![]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should still find matches in all file types
        assert!(!results.lines.is_empty(), "Expected to find matches");
        
        // Should find JSON files since exclusion list is empty
        assert!(
            results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".json")),
            "Did not find JSON files despite empty exclusion list"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }

    /// Test combining exclude_glob with gitignore
    #[test]
    #[serial]
    fn test_exclude_glob_with_gitignore() -> Result<()> {
        let _env = TestEnvironment::setup()?;
        let additional_files = setup_multiple_file_types()?;

        let pattern = "content";
        
        // Use exclude_glob with respect_gitignore=true (default)
        let mut options = SearchOptions::default();
        options.exclude_glob = Some(vec!["*.md".to_string()]);
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should not find markdown files
        assert!(
            !results.lines.iter().any(|r| r.file_path.to_string_lossy().ends_with(".md")),
            "Found markdown files despite excluding them"
        );
        
        // Should not find files in .hidden directory (gitignore)
        assert!(
            !results.lines.iter().any(|r| r.file_path.to_string_lossy().contains(".hidden")),
            "Found hidden files despite respecting gitignore"
        );

        // Cleanup
        for file in &additional_files {
            std::fs::remove_file(file)?;
        }

        Ok(())
    }
}