use anyhow::Result;
use file_searcher::search::{search_files, SearchOptions};
use std::path::Path;

/// Tests for the search functionality
#[cfg(test)]
mod search_tests {
    use super::*;
    
    const TEST_DIR: &str = "tests/test_dir_1";
    
    /// Test searching with default options (case-insensitive, respect_gitignore=true)
    #[test]
    fn test_search_default_options() -> Result<()> {
        let pattern = "fn";
        let options = SearchOptions::default();
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find "fn" in multiple Rust files but not in hidden files
        assert!(!results.is_empty());
        
        // All results should contain the pattern
        for result in &results {
            assert!(result.line_content.contains(pattern));
        }
        
        // Should not find anything in .hidden directory (respects gitignore)
        assert!(!results.iter().any(|r| r.file_path.to_string_lossy().contains(".hidden")));
        
        Ok(())
    }
    
    /// Test case-sensitive search
    #[test]
    fn test_search_case_sensitive() -> Result<()> {
        let pattern = "Fn";  // Capital F
        let mut options = SearchOptions::default();
        options.case_sensitive = true;
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should not find lowercase "fn" when searching for "Fn" with case sensitivity
        assert!(!results.iter().any(|r| r.line_content.contains("fn ")));
        
        Ok(())
    }
    
    /// Test case-insensitive search
    #[test]
    fn test_search_case_insensitive() -> Result<()> {
        let pattern = "FN";  // All caps
        let mut options = SearchOptions::default();
        options.case_sensitive = false;
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find lowercase "fn" when searching for "FN" case-insensitively
        assert!(results.iter().any(|r| r.line_content.contains("fn ")));
        
        Ok(())
    }
    
    /// Test searching without respecting gitignore
    #[test]
    fn test_search_ignore_gitignore() -> Result<()> {
        let pattern = "API_KEY";
        let mut options = SearchOptions::default();
        options.respect_gitignore = false;
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find the pattern in .hidden directory
        assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains(".hidden")));
        
        Ok(())
    }
    
    /// Test searching with a pattern that doesn't exist
    #[test]
    fn test_search_no_matches() -> Result<()> {
        let pattern = "THIS_PATTERN_SHOULD_NOT_EXIST_ANYWHERE";
        let options = SearchOptions::default();
        
        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;
        
        // Should find no matches
        assert!(results.is_empty());
        
        Ok(())
    }
}