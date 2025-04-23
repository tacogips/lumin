use anyhow::Result;
use lumin::tree::{TreeOptions, generate_tree, Entry, DirectoryTree};
use serial_test::serial;
use std::path::Path;

mod test_helpers;
use test_helpers::{TEST_DIR, TestEnvironment};

/// Tests for the tree functionality
#[cfg(test)]
mod tree_tests {
    use super::*;

    /// Test tree generation with default options
    #[test]
    #[serial]
    fn test_tree_default_options() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let options = TreeOptions::default();
        let result = generate_tree(Path::new(TEST_DIR), &options)?;

        // Should find multiple directories
        assert!(!result.is_empty());

        // Check the structure format
        for dir_tree in &result {
            // Dir should be a valid path string
            assert!(!dir_tree.dir.is_empty());
            
            // Each directory should have entries
            assert!(!dir_tree.entries.is_empty());
            
            // Entries should be either files or directories
            for entry in &dir_tree.entries {
                match entry {
                    Entry::File { name } => {
                        assert!(!name.is_empty());
                    },
                    Entry::Directory { name } => {
                        assert!(!name.is_empty());
                    },
                }
            }
        }

        // Should not find directories in .hidden directory (respects gitignore)
        assert!(
            !result
                .iter()
                .any(|r| r.dir.contains(".hidden"))
        );

        Ok(())
    }

    /// Test tree generation without respecting gitignore
    #[test]
    #[serial]
    fn test_tree_ignore_gitignore() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // Configure to ignore gitignore
        let mut options = TreeOptions::default();
        options.respect_gitignore = false;

        let result = generate_tree(Path::new(TEST_DIR), &options)?;

        // Should find .hidden directories
        assert!(
            result
                .iter()
                .any(|r| r.dir.contains(".hidden")),
            "Did not find .hidden directories when ignoring gitignore"
        );

        Ok(())
    }

    /// Test tree with text files only
    #[test]
    #[serial]
    fn test_tree_text_files_only() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let options = TreeOptions::default();
        let result = generate_tree(Path::new(TEST_DIR), &options)?;

        // In default mode (text files only), should not find binary files in entries
        let has_binary_files = result.iter().any(|dir_tree| {
            dir_tree.entries.iter().any(|entry| {
                match entry {
                    Entry::File { name } => {
                        name.ends_with(".jpg") || 
                        name.ends_with(".png") || 
                        name == "binary_executable"
                    },
                    _ => false,
                }
            })
        });

        assert!(!has_binary_files, "Found binary files when only_text_files is true");

        Ok(())
    }

    /// Test tree with all files (including binary)
    #[test]
    #[serial]
    fn test_tree_include_binary() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let mut options = TreeOptions::default();
        options.only_text_files = false;

        let result = generate_tree(Path::new(TEST_DIR), &options)?;

        // Should find binary files in entries
        let has_binary_files = result.iter().any(|dir_tree| {
            dir_tree.entries.iter().any(|entry| {
                match entry {
                    Entry::File { name } => {
                        name.ends_with(".jpg") || 
                        name.ends_with(".png") || 
                        name == "binary_executable"
                    },
                    _ => false,
                }
            })
        });

        assert!(has_binary_files, "Did not find binary files when only_text_files is false");

        Ok(())
    }
}