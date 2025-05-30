use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use serial_test::serial;
use std::path::Path;

mod test_helpers;
use test_helpers::{TEST_DIR, TestEnvironment};

/// Tests for the search functionality
#[cfg(test)]
mod search_tests {
    use super::*;

    /// Test searching with default options (case-insensitive, respect_gitignore=true)
    #[test]
    #[serial]
    fn test_search_default_options() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn";
        let options = SearchOptions::default();

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should find "fn" in multiple Rust files but not in hidden files
        assert!(!results.lines.is_empty());

        // All results should contain the pattern
        for result in &results.lines {
            assert!(result.line_content.contains(pattern));
        }

        // Should not find anything in .hidden directory (respects gitignore)
        assert!(
            !results
                .lines
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains(".hidden"))
        );

        Ok(())
    }

    /// Test case-sensitive search
    #[test]
    #[serial]
    fn test_search_case_sensitive() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "Fn"; // Capital F
        let mut options = SearchOptions::default();
        options.case_sensitive = true;

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should not find lowercase "fn" when searching for "Fn" with case sensitivity
        assert!(!results.lines.iter().any(|r| r.line_content.contains("fn ")));

        Ok(())
    }

    /// Test case-insensitive search
    #[test]
    #[serial]
    fn test_search_case_insensitive() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "FN"; // All caps
        let mut options = SearchOptions::default();
        options.case_sensitive = false;

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should find lowercase "fn" when searching for "FN" case-insensitively
        assert!(results.lines.iter().any(|r| r.line_content.contains("fn ")));

        Ok(())
    }

    /// Test searching with respect_gitignore=true (default)
    #[test]
    #[serial]
    fn test_search_respect_gitignore() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // First, make sure .hidden directory exists with the pattern
        let secret_file = Path::new(TEST_DIR).join(".hidden").join("secret.txt");
        assert!(
            secret_file.exists(),
            "Test setup error: .hidden/secret.txt doesn't exist"
        );
        let content = std::fs::read_to_string(&secret_file)?;
        assert!(
            content.contains("API_KEY"),
            "Test setup error: API_KEY not found in secret.txt"
        );

        // Search with default options (should respect gitignore)
        let pattern = "API_KEY";
        let options = SearchOptions::default();

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should NOT find the pattern in .hidden directory
        assert!(
            !results
                .lines
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains(".hidden")),
            "Found .hidden files when respecting gitignore"
        );

        Ok(())
    }

    /// Test searching without respecting gitignore
    #[test]
    #[serial]
    fn test_search_ignore_gitignore() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // Search without respecting gitignore
        let pattern = "API_KEY";
        let mut options = SearchOptions::default();
        options.respect_gitignore = false;

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should find the pattern in .hidden directory
        assert!(
            results
                .lines
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains(".hidden")),
            "Did not find .hidden files when ignoring gitignore"
        );

        Ok(())
    }

    /// Test searching with a pattern that doesn't exist
    #[test]
    #[serial]
    fn test_search_no_matches() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "THIS_PATTERN_SHOULD_NOT_EXIST_ANYWHERE";
        let options = SearchOptions::default();

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should find no matches
        assert!(results.lines.is_empty());

        Ok(())
    }

    /// Test that line_content does not contain trailing newlines
    #[test]
    #[serial]
    fn test_no_trailing_newlines() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // Use a pattern that should exist in the test files
        let pattern = "fn";
        let options = SearchOptions::default();

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should find matches
        assert!(!results.lines.is_empty());

        // None of the results should have trailing newlines
        for result in &results.lines {
            assert!(
                !result.line_content.ends_with('\n'),
                "line_content contains trailing newline: {:?}",
                result.line_content
            );
        }

        Ok(())
    }

    /// Test that context lines do not contain trailing newlines
    #[test]
    #[serial]
    fn test_no_trailing_newlines_in_context() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // Use a pattern that should exist in the test files
        let pattern = "fn";
        let mut options = SearchOptions::default();
        // Set context values to ensure we get context lines
        options.before_context = 2;
        options.after_context = 2;

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Should find matches
        assert!(!results.lines.is_empty());

        // Should have some context lines
        assert!(results.lines.iter().any(|r| r.is_context));

        // None of the results (including context lines) should have trailing newlines
        for result in &results.lines {
            assert!(
                !result.line_content.ends_with('\n'),
                "line_content contains trailing newline: {:?}",
                result.line_content
            );
        }

        Ok(())
    }
}
