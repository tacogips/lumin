//! Tests for path prefix removal in search results.

#[cfg(test)]
mod tests {
    use anyhow::Result;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;
    use tempfile::TempDir;

    use crate::search::{SearchOptions, search_files};

    #[test]
    fn test_path_prefix_removal() -> Result<()> {
        // Create a temporary directory for our test files
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();

        // Create a simple test file
        let file_path = temp_path.join("test.txt");
        let mut file = File::create(&file_path)?;
        file.write_all(b"This is a test file with a pattern inside.\n")?;

        // The pattern to search for
        let pattern = "pattern";

        // Test case 1: Without path prefix removal
        let options = SearchOptions::default();
        let results = search_files(pattern, temp_path, &options)?;
        assert_eq!(results.total_number, 1, "Should find one match");
        assert_eq!(results.lines[0].file_path, file_path, "File path should be preserved as-is");

        // Test case 2: With path prefix removal
        let mut options_with_prefix = SearchOptions::default();
        options_with_prefix.omit_path_prefix = Some(temp_path.to_path_buf());
        let results_with_prefix = search_files(pattern, temp_path, &options_with_prefix)?;
        assert_eq!(results_with_prefix.total_number, 1, "Should find one match");
        assert_eq!(
            results_with_prefix.lines[0].file_path,
            PathBuf::from("test.txt"),
            "Path prefix should be removed"
        );

        // Test case 3: With non-matching path prefix
        let mut options_with_nonmatching_prefix = SearchOptions::default();
        options_with_nonmatching_prefix.omit_path_prefix = Some(PathBuf::from("/non/existing/path"));
        let results_nonmatching = search_files(pattern, temp_path, &options_with_nonmatching_prefix)?;
        assert_eq!(results_nonmatching.total_number, 1, "Should find one match");
        assert_eq!(
            results_nonmatching.lines[0].file_path, 
            file_path, 
            "File path should be preserved when prefix doesn't match"
        );

        Ok(())
    }
}