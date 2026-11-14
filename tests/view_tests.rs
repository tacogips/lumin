use anyhow::Result;
use lumin::view::{FileContents, ViewOptions, view_file};
use serial_test::serial;
use std::path::Path;

mod test_helpers;
use test_helpers::{TEST_DIR, TestEnvironment};

/// Tests for the view functionality
#[cfg(test)]
mod view_tests {
    use super::*;

    /// Test viewing a text file
    #[test]
    #[serial]
    fn test_view_text_file() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // Make sure we're actually creating the file in our test environment
        std::fs::copy("tests/fixtures/text_files/config.toml", Path::new(TEST_DIR).join("config.toml"))?;
        let file_path = Path::new(TEST_DIR).join("config.toml");
        let options = ViewOptions::default();

        let result = view_file(&file_path, &options)?;

        // Check the structure of the result
        assert_eq!(result.file_path, file_path);
        assert!(result.file_type.starts_with("text/"));

        // Check content based on enum variant
        match &result.contents {
            FileContents::Text { content, metadata } => {
                assert!(!content.is_empty());
                assert!(content.contains("server"));
                assert!(content.contains("database"));
                assert!(content.contains("port = 8080"));
                assert!(metadata.line_count > 0);
                assert!(metadata.char_count > 0);
            }
            _ => panic!("Expected text content, got a different variant"),
        }

        Ok(())
    }

    /// Test viewing a markdown file
    #[test]
    #[serial]
    fn test_view_markdown_file() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let file_path = Path::new(TEST_DIR).join("docs").join("README.md");
        let options = ViewOptions::default();

        let result = view_file(&file_path, &options)?;

        // Check the result
        assert_eq!(result.file_path, file_path);
        assert!(result.file_type.starts_with("text/"));

        // Check content based on enum variant
        match &result.contents {
            FileContents::Text { content, metadata } => {
                assert!(content.contains("# Test Documentation"));
                assert!(metadata.line_count > 0);
            }
            _ => panic!("Expected text content, got a different variant"),
        }

        Ok(())
    }

    /// Test viewing a binary file
    #[test]
    #[serial]
    fn test_view_binary_file() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let file_path = Path::new(TEST_DIR).join("images").join("binary_executable");
        let options = ViewOptions::default();

        let result = view_file(&file_path, &options)?;

        // Check the result
        assert_eq!(result.file_path, file_path);

        // Check binary content based on enum variant
        match &result.contents {
            FileContents::Binary { message, metadata } => {
                assert!(message.contains("Binary file detected"));
                assert!(metadata.binary);
                assert!(metadata.size_bytes > 0);
            }
            _ => panic!("Expected binary content, got a different variant"),
        }

        Ok(())
    }

    /// Test viewing a file with a size limit
    #[test]
    #[serial]
    fn test_view_with_size_limit() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let file_path = Path::new(TEST_DIR).join("images").join("sample.jpg"); // 5KB file
        let options = ViewOptions {
            max_size: Some(1024), // 1KB limit
        };

        // Should return an error due to size limit
        let result = view_file(&file_path, &options);
        assert!(result.is_err());

        // Error message should mention file size
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("File is too large"));

        Ok(())
    }

    /// Test viewing a non-existent file
    #[test]
    #[serial]
    fn test_view_nonexistent_file() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let file_path = Path::new(TEST_DIR).join("nonexistent.txt");
        let options = ViewOptions::default();

        // Should return an error
        let result = view_file(&file_path, &options);
        assert!(result.is_err());

        // Error message should mention file not found
        let err = format!("{}", result.unwrap_err());
        assert!(err.contains("File not found"));

        Ok(())
    }

    /// Test viewing a file that's ignored by gitignore
    #[test]
    #[serial]
    fn test_view_ignored_file() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // Test hidden file
        let hidden_file = Path::new(TEST_DIR).join(".hidden").join("secret.txt");
        let options = ViewOptions::default();

        // We should still be able to view the file directly even if it's ignored by gitignore
        let result = view_file(&hidden_file, &options)?;

        // Check content based on enum variant
        match &result.contents {
            FileContents::Text { content, metadata } => {
                assert!(content.contains("API_KEY=test_key_12345"));
                assert!(metadata.line_count > 0);
            }
            _ => panic!("Expected text content, got a different variant"),
        }

        Ok(())
    }
}
