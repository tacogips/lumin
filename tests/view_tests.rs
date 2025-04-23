use anyhow::Result;
use file_searcher::view::{view_file, ViewOptions};
use std::path::Path;

/// Tests for the view functionality
#[cfg(test)]
mod view_tests {
    use super::*;
    
    /// Test viewing a text file
    #[test]
    fn test_view_text_file() -> Result<()> {
        let file_path = Path::new("tests/test_dir_1/config.toml");
        let options = ViewOptions::default();
        
        let result = view_file(file_path, &options)?;
        
        // Check the structure of the result
        assert_eq!(result.file_path, file_path);
        assert!(result.file_type.starts_with("text/"));
        
        // Convert contents to String for testing
        let contents = result.contents.as_str().unwrap();
        assert!(!contents.is_empty());
        
        // Content should contain configuration data
        assert!(contents.contains("Configuration file for testing"));
        
        Ok(())
    }
    
    /// Test viewing a markdown file
    #[test]
    fn test_view_markdown_file() -> Result<()> {
        let file_path = Path::new("tests/test_dir_1/docs/README.md");
        let options = ViewOptions::default();
        
        let result = view_file(file_path, &options)?;
        
        // Check the result
        assert_eq!(result.file_path, file_path);
        assert!(result.file_type.starts_with("text/"));
        
        // Content should contain markdown
        let contents = result.contents.as_str().unwrap();
        assert!(contents.contains("# Test Documentation"));
        
        Ok(())
    }
    
    /// Test viewing a binary file
    #[test]
    fn test_view_binary_file() -> Result<()> {
        let file_path = Path::new("tests/test_dir_1/images/binary_executable");
        let options = ViewOptions::default();
        
        let result = view_file(file_path, &options)?;
        
        // Check the result
        assert_eq!(result.file_path, file_path);
        
        // For binary files, we return a message about binary detection
        let contents = result.contents.as_str().unwrap();
        assert!(contents.contains("Binary file detected"));
        
        Ok(())
    }
    
    /// Test viewing a file with a size limit
    #[test]
    fn test_view_with_size_limit() -> Result<()> {
        let file_path = Path::new("tests/test_dir_1/images/sample.jpg"); // 5KB file
        let options = ViewOptions {
            max_size: Some(1024),  // 1KB limit
        };
        
        // Should return an error due to size limit
        let result = view_file(file_path, &options);
        assert!(result.is_err());
        
        // Error message should mention file size
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("File is too large"));
        
        Ok(())
    }
    
    /// Test viewing a non-existent file
    #[test]
    fn test_view_nonexistent_file() -> Result<()> {
        let file_path = Path::new("tests/test_dir_1/nonexistent.txt");
        let options = ViewOptions::default();
        
        // Should return an error
        let result = view_file(file_path, &options);
        assert!(result.is_err());
        
        // Error message should mention file not found
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("File not found"));
        
        Ok(())
    }
}