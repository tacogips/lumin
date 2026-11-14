use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use lumin::traverse::{TraverseOptions, traverse_directory};
use lumin::tree::{TreeOptions, generate_tree};
use lumin::view::{ViewOptions, view_file};
use std::path::Path;

// This test exercises each major component together in sequence
#[test]
fn test_full_workflow() -> Result<()> {
    // Initialize telemetry
    lumin::telemetry::init()?;

    // 1. First traverse the directory to discover files
    let directory = Path::new("tests/fixtures");
    let traverse_options = TraverseOptions {
        case_sensitive: false,
        respect_gitignore: true,
        only_text_files: true,
        pattern: Some("**.txt".to_string()),
    };

    let traverse_results = traverse_directory(directory, &traverse_options)?;
    assert!(!traverse_results.is_empty());

    // 2. Search for a pattern in those text files
    let search_pattern = "pattern";
    let search_options = SearchOptions {
        case_sensitive: false,
        respect_gitignore: true,
        exclude_glob: None,
        match_content_omit_num: None,
    };

    let search_results = search_files(search_pattern, directory, &search_options)?;
    assert!(!search_results.is_empty());

    // 3. Generate a tree structure of the directory
    let tree_options = TreeOptions {
        case_sensitive: false,
        respect_gitignore: true,
    };

    let tree_results = generate_tree(directory, &tree_options)?;
    assert!(!tree_results.is_empty());

    // 4. View the first file found in the search results
    if let Some(first_match) = search_results.first() {
        let view_options = ViewOptions::default();
        let view_result = view_file(&first_match.file_path, &view_options)?;

        // Verify the view result contains the search pattern
        match &view_result.contents {
            lumin::view::FileContents::Text { content, .. } => {
                assert!(content.to_lowercase().contains(search_pattern));
            }
            _ => panic!("Expected text content for the matched file"),
        }
    }

    Ok(())
}

// Test the ability to find files at different directory levels
#[test]
fn test_multi_level_search() -> Result<()> {
    let directory = Path::new("tests/fixtures");

    // First find the nested files
    let traverse_options = TraverseOptions {
        pattern: Some("**/level*.txt".to_string()),
        ..TraverseOptions::default()
    };

    let traverse_results = traverse_directory(directory, &traverse_options)?;
    assert!(traverse_results.len() >= 2); // Should find at least level1.txt and level2.txt

    // Verify files at different levels are found
    let paths: Vec<_> = traverse_results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    assert!(paths.iter().any(|p| p.contains("level1/level1.txt")));
    assert!(paths.iter().any(|p| p.contains("level1/level2/level2.txt")));

    // Search for a pattern across those nested files
    let search_options = SearchOptions::default();
    let search_results = search_files("level", directory, &search_options)?;

    // Verify pattern was found in files at different nesting levels
    assert!(!search_results.is_empty());
    let search_paths: Vec<_> = search_results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    assert!(search_paths.iter().any(|p| p.contains("level1.txt")));
    assert!(search_paths.iter().any(|p| p.contains("level2.txt")));

    Ok(())
}
