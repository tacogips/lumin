use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use std::path::Path;

#[test]
fn test_search_pattern_case_sensitive() -> Result<()> {
    let pattern = "PATTERN";
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions {
        case_sensitive: true,
        respect_gitignore: true,
        exclude_glob: None,
        match_content_omit_num: None,
    };

    let results = search_files(pattern, directory, &options)?;

    // Should find matches in files where "PATTERN" appears in uppercase
    assert!(!results.is_empty());

    // Now there are more matches with our new test files
    // Just verify that we found the key matches
    let found_sample = results
        .iter()
        .any(|r| r.file_path.to_string_lossy().contains("sample.txt"));
    let found_markdown = results
        .iter()
        .any(|r| r.file_path.to_string_lossy().contains("markdown.md"));
    let found_file_rs = results
        .iter()
        .any(|r| r.file_path.to_string_lossy().contains("file.rs"));

    assert!(found_sample, "Should find PATTERN in sample.txt");
    assert!(found_markdown, "Should find PATTERN in markdown.md");
    assert!(found_file_rs, "Should find PATTERN in file.rs");

    // Verify matches found in expected files
    let file_paths: Vec<String> = results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    assert!(file_paths.iter().any(|path| path.contains("sample.txt")));
    assert!(file_paths.iter().any(|path| path.contains("markdown.md")));
    assert!(file_paths.iter().any(|path| path.contains("file.rs")));

    Ok(())
}

#[test]
fn test_search_pattern_case_insensitive() -> Result<()> {
    let pattern = "pattern";
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions {
        case_sensitive: false,
        respect_gitignore: true,
        exclude_glob: None,
        match_content_omit_num: None,
    };

    let results = search_files(pattern, directory, &options)?;

    // Should find both uppercase and lowercase matches
    assert!(!results.is_empty());
    assert!(results.len() >= 5); // Should find at least 5 matches across all files

    // Verify matches found in expected files
    let file_paths: Vec<String> = results
        .iter()
        .map(|r| r.file_path.to_string_lossy().to_string())
        .collect();

    assert!(file_paths.iter().any(|path| path.contains("sample.txt")));
    assert!(file_paths.iter().any(|path| path.contains("markdown.md")));
    assert!(file_paths.iter().any(|path| path.contains("file.rs")));
    assert!(file_paths.iter().any(|path| path.contains("config.toml")));

    Ok(())
}

#[test]
fn test_search_with_gitignore_respect() -> Result<()> {
    let pattern = "ignored";
    let directory = Path::new("tests/fixtures");

    // First with gitignore respected (default)
    let options = SearchOptions {
        case_sensitive: false,
        respect_gitignore: true,
        exclude_glob: None,
        match_content_omit_num: None,
    };

    let results = search_files(pattern, directory, &options)?;

    // Should not find matches in .hidden/secret.txt, temp.tmp or log.log
    assert!(
        results
            .iter()
            .all(|r| !r.file_path.to_string_lossy().contains(".hidden"))
    );
    assert!(
        results
            .iter()
            .all(|r| !r.file_path.to_string_lossy().ends_with(".tmp"))
    );
    assert!(
        results
            .iter()
            .all(|r| !r.file_path.to_string_lossy().ends_with(".log"))
    );

    Ok(())
}

#[test]
fn test_search_without_gitignore_respect() -> Result<()> {
    let pattern = "ignored";
    let directory = Path::new("tests/fixtures");

    // Now with gitignore bypassed
    let options = SearchOptions {
        case_sensitive: false,
        respect_gitignore: false,
        exclude_glob: None,
        match_content_omit_num: None,
    };

    let results = search_files(pattern, directory, &options)?;

    // Should find matches in temp.tmp and log.log
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .any(|r| r.file_path.to_string_lossy().ends_with(".tmp")
                || r.file_path.to_string_lossy().ends_with(".log"))
    );

    Ok(())
}
