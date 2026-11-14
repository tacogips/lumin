use anyhow::Result;
use lumin::tree::{Entry, TreeOptions, generate_tree};
use std::path::Path;

#[test]
fn test_tree_basic() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TreeOptions::default();

    let results = generate_tree(directory, &options)?;

    // Should generate a tree structure
    assert!(!results.is_empty());

    // Root directory should be in the results
    let root_dir = results.iter().find(|d| d.dir.ends_with("fixtures"));
    assert!(root_dir.is_some());

    // Root directory should have entries
    if let Some(root) = root_dir {
        assert!(!root.entries.is_empty());

        // Should contain the "text_files" and "nested" directories
        let dir_names: Vec<_> = root
            .entries
            .iter()
            .filter_map(|e| {
                if let Entry::Directory { name } = e {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(dir_names.contains(&"text_files"));
        assert!(dir_names.contains(&"nested"));
        assert!(dir_names.contains(&"binary_files"));

        // Should not contain ignored or hidden directories
        assert!(!dir_names.contains(&".hidden"));
    }

    // Should include nested directories
    let nested_dir = results.iter().find(|d| d.dir.contains("nested"));
    assert!(nested_dir.is_some());

    // Should include nested/level1 directory
    let level1_dir = results.iter().find(|d| d.dir.contains("nested/level1"));
    assert!(level1_dir.is_some());

    // Should include level2 directories with correct parent
    let level2_dir = results.iter().find(|d| d.dir.contains("level2"));
    if let Some(level2) = level2_dir {
        // The level2 directory should be under level1
        assert!(level2.dir.contains("level1/level2"));
    }

    Ok(())
}

#[test]
fn test_tree_without_gitignore_respect() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TreeOptions {
        respect_gitignore: false,
        ..TreeOptions::default()
    };

    let results = generate_tree(directory, &options)?;

    // Should include .hidden directory when not respecting gitignore
    let contains_hidden_dir = results.iter().any(|d| d.dir.contains(".hidden"));
    assert!(
        contains_hidden_dir,
        ".hidden directory should be included when not respecting gitignore"
    );

    // Should include files from gitignored directories
    let hidden_dir = results.iter().find(|d| d.dir.contains(".hidden"));
    if let Some(dir) = hidden_dir {
        // Should have secret.txt as an entry
        let has_secret_file = dir.entries.iter().any(|e| {
            if let Entry::File { name } = e {
                name == "secret.txt"
            } else {
                false
            }
        });

        assert!(
            has_secret_file,
            "hidden/secret.txt should be included when not respecting gitignore"
        );
    }

    Ok(())
}

#[test]
fn test_tree_structure_integrity() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TreeOptions::default();

    let results = generate_tree(directory, &options)?;

    // Verify the nested directory structure is preserved correctly

    // Should have all levels of nesting
    let has_level1 = results
        .iter()
        .any(|d| d.dir.contains("level1") && !d.dir.contains("level2"));
    let has_level2 = results.iter().any(|d| d.dir.contains("level2"));

    assert!(has_level1, "Should have level1 directory in results");
    assert!(has_level2, "Should have level2 directory in results");

    // Verify level1 directory contains level2 as a subdirectory
    let level1_dir = results
        .iter()
        .find(|d| d.dir.contains("level1") && !d.dir.contains("level2"));
    if let Some(level1) = level1_dir {
        let has_level2_entry = level1.entries.iter().any(|e| {
            if let Entry::Directory { name } = e {
                name == "level2"
            } else {
                false
            }
        });

        assert!(
            has_level2_entry,
            "level1 directory should contain level2 as a subdirectory"
        );
    }

    // Verify text_files directory contains the correct files
    let text_files_dir = results.iter().find(|d| d.dir.contains("text_files"));
    if let Some(dir) = text_files_dir {
        // Should contain sample.txt, markdown.md, and config.toml
        let file_names: Vec<_> = dir
            .entries
            .iter()
            .filter_map(|e| {
                if let Entry::File { name } = e {
                    Some(name.as_str())
                } else {
                    None
                }
            })
            .collect();

        assert!(
            file_names.contains(&"sample.txt"),
            "text_files directory should contain sample.txt"
        );
        assert!(
            file_names.contains(&"markdown.md"),
            "text_files directory should contain markdown.md"
        );
        assert!(
            file_names.contains(&"config.toml"),
            "text_files directory should contain config.toml"
        );
    }

    Ok(())
}
