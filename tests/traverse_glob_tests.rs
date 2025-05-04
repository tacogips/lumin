use anyhow::Result;
use lumin::traverse::{TraverseOptions, traverse_directory};
use std::path::Path;

/// Tests focused on glob pattern matching in the traverse functionality

#[test]
fn test_traverse_with_star_wildcard() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        pattern: Some("*.txt".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // All results should be .txt files
    for result in &results {
        assert!(result.file_path.to_string_lossy().ends_with(".txt"));
    }
    
    // Should find sample.txt
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("sample.txt")));
    
    Ok(())
}

#[test]
fn test_traverse_with_double_star_recursive() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        pattern: Some("**/level*.txt".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // Should find both level1.txt and level2.txt in nested directories
    let found_level1 = results.iter().any(|r| r.file_path.to_string_lossy().contains("level1.txt"));
    let found_level2 = results.iter().any(|r| r.file_path.to_string_lossy().contains("level2.txt"));
    
    assert!(found_level1);
    assert!(found_level2);
    
    Ok(())
}

#[test]
fn test_traverse_with_question_mark_wildcard() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    
    // Match any single character in our known files
    let options = TraverseOptions {
        pattern: Some("**/level?.txt".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // Should match level1.txt and level2.txt (single character after "level")
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("level1.txt")));
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("level2.txt")));
    
    Ok(())
}

#[test]
fn test_traverse_with_character_class() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        // Match level1.txt but not level2.txt
        pattern: Some("**/level[1].txt".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // Should find level1.txt
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("level1.txt")));
    
    // Should NOT find level2.txt
    assert!(!results.iter().any(|r| r.file_path.to_string_lossy().contains("level2.txt")));
    
    Ok(())
}

#[test]
fn test_traverse_with_negated_character_class() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        // Match any file with a name that doesn't end with a digit
        pattern: Some("**/[!0-9]*.txt".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // Should find sample.txt (starts with 's', not a digit)
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("sample.txt")));
    
    Ok(())
}

#[test]
fn test_traverse_with_braces() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = TraverseOptions {
        // Match either .txt or .md files
        pattern: Some("*.{txt,md}".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // Should find both .txt and .md files
    let has_txt = results.iter().any(|r| r.file_path.to_string_lossy().ends_with(".txt"));
    let has_md = results.iter().any(|r| r.file_path.to_string_lossy().ends_with(".md"));
    
    assert!(has_txt);
    assert!(has_md);
    
    Ok(())
}

#[test]
fn test_traverse_with_extension_match() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    
    // Match all markdown files
    let options = TraverseOptions {
        pattern: Some("**/*.md".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // All results should be .md files
    for result in &results {
        assert!(result.file_path.to_string_lossy().ends_with(".md"));
    }
    
    // Should find markdown.md
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("markdown.md")));
    
    Ok(())
}

#[test]
fn test_traverse_with_complex_pattern() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    
    // Complex pattern: nested text or markdown files with level in the name
    let options = TraverseOptions {
        pattern: Some("**/level*/*.{txt,md}".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    // Should find level2.txt in the level1 directory
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("level1/level2/level2.txt")));
    
    Ok(())
}

#[test]
fn test_traverse_with_directory_specific_pattern() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    
    // Match only files directly in the text_files directory
    let options = TraverseOptions {
        pattern: Some("**/text_files/*.txt".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // All results should be from the text_files directory
    for result in &results {
        assert!(result.file_path.to_string_lossy().contains("text_files"));
        assert!(result.file_path.to_string_lossy().ends_with(".txt"));
    }
    
    Ok(())
}

#[test]
fn test_traverse_with_filename_prefix() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    
    // Match files starting with "sample"
    let options = TraverseOptions {
        pattern: Some("**/sample*".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // All results should have filenames starting with "sample"
    for result in &results {
        let filename = result.file_path.file_name().unwrap().to_string_lossy();
        assert!(filename.starts_with("sample"));
    }
    
    // Should find sample.txt
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("sample.txt")));
    
    Ok(())
}

#[test]
fn test_traverse_with_substring_pattern() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    
    // Use a normal substring pattern (not glob)
    let options = TraverseOptions {
        pattern: Some("level".to_string()),
        ..TraverseOptions::default()
    };

    let results = traverse_directory(directory, &options)?;
    
    assert!(!results.is_empty());
    
    // All results should contain "level" in the path
    for result in &results {
        assert!(result.file_path.to_string_lossy().contains("level"));
    }
    
    // Should find both level1.txt and level2.txt
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("level1.txt")));
    assert!(results.iter().any(|r| r.file_path.to_string_lossy().contains("level2.txt")));
    
    Ok(())
}