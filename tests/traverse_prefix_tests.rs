use anyhow::Result;
use lumin::traverse::{traverse_directory, TraverseOptions};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Tests for prefix matching in traverse functionality
#[cfg(test)]
mod traverse_prefix_tests {
    use super::*;

    /// Test pure prefix matching at the root level
    #[test]
    fn test_traverse_root_prefix_matching() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Create test files for root prefix matching
        let test_files = [
            "tests/fixtures/prefix_test1.txt",
            "tests/fixtures/prefix_test2.md",
            "tests/fixtures/not_matching.txt",
            "tests/fixtures/nested/prefix_test3.txt",
        ];

        // Create the test files
        for file_path in &test_files {
            let path = PathBuf::from(file_path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = File::create(path)?;
            writeln!(file, "Test content for {}", file_path)?;
        }

        // Cleanup function
        let _cleanup = defer::defer(|| {
            for file_path in &test_files {
                let _ = std::fs::remove_file(file_path);
            }
        });

        // Test prefix pattern at root level
        let options = TraverseOptions {
            pattern: Some("prefix_*".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // Should only match prefix_test1.txt and prefix_test2.md at the root level
        // It should not match nested/prefix_test3.txt
        assert_eq!(results.len(), 2, "Should match exactly 2 files at the root level");

        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().ends_with("prefix_test1.txt")),
            "Should find prefix_test1.txt"
        );

        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().ends_with("prefix_test2.md")),
            "Should find prefix_test2.md"
        );

        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("nested/prefix_test3.txt")),
            "Should NOT find nested/prefix_test3.txt"
        );

        // Now test prefix pattern with recursion
        let options = TraverseOptions {
            pattern: Some("**/prefix_*".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // Should match all 3 prefix_* files in any directory
        assert_eq!(results.len(), 3, "Should match all 3 prefix_* files");

        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().ends_with("nested/prefix_test3.txt")),
            "Should find nested/prefix_test3.txt with **/ pattern"
        );

        Ok(())
    }
}