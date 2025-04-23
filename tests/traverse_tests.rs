use anyhow::Result;
use file_searcher::traverse::{traverse_directory, TraverseOptions};
use std::path::Path;

/// Tests for the traverse functionality
#[cfg(test)]
mod traverse_tests {
    use super::*;
    
    const TEST_DIR: &str = "tests/test_dir_1";
    
    /// Test directory traversal with default options
    #[test]
    fn test_traverse_default_options() -> Result<()> {
        let options = TraverseOptions::default();
        
        let results = traverse_directory(Path::new(TEST_DIR), &options)?;
        
        // Should find multiple files
        assert!(!results.is_empty());
        
        // Should find Rust files
        assert!(results.iter().any(|r| r.file_type == "rs"));
        
        // Should find Markdown files
        assert!(results.iter().any(|r| r.file_type == "md"));
        
        // Should find Python files
        assert!(results.iter().any(|r| r.file_type == "py"));
        
        // Should not include binary files by default
        assert!(!results.iter().any(|r| r.file_type == "jpg" || r.file_type == "png"));
        
        // Should not find files in .hidden directory (respects gitignore)
        assert!(!results.iter().any(|r| r.file_path.to_string_lossy().contains(".hidden")));
        
        Ok(())
    }
    
    /// Test traversal including binary files
    #[test]
    fn test_traverse_include_binary() -> Result<()> {
        let mut options = TraverseOptions::default();
        options.only_text_files = false;
        
        let results = traverse_directory(Path::new(TEST_DIR), &options)?;
        
        // Should find binary files
        assert!(results.iter().any(|r| r.file_type == "jpg" || r.file_type == "png"));
        
        Ok(())
    }
    
    /// Test traversal without respecting gitignore
    #[test]
    fn test_traverse_ignore_gitignore() -> Result<()> {
        let mut options = TraverseOptions::default();
        options.respect_gitignore = false;
        
        let results = traverse_directory(Path::new(TEST_DIR), &options)?;
        
        // Should find files in .hidden directory
        assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains(".hidden")));
        
        Ok(())
    }
    
    /// Test the is_hidden method
    #[test]
    fn test_is_hidden() -> Result<()> {
        let mut options = TraverseOptions::default();
        options.respect_gitignore = false;  // To include hidden files
        
        let results = traverse_directory(Path::new(TEST_DIR), &options)?;
        
        // Files in .hidden directory should be marked as hidden
        for result in &results {
            if result.file_path.to_string_lossy().contains(".hidden") {
                assert!(result.is_hidden());
            }
        }
        
        Ok(())
    }
    
    /// Test traversal with case-sensitive option
    #[test]
    fn test_traverse_case_sensitive() -> Result<()> {
        let mut options = TraverseOptions::default();
        options.case_sensitive = true;
        
        let results = traverse_directory(Path::new(TEST_DIR), &options)?;
        
        // Should still find files regardless of case sensitivity
        assert!(!results.is_empty());
        
        Ok(())
    }
}