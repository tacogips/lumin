//! Integration test to verify consistent glob pattern behavior across search and traverse modules.
//!
//! This test ensures that all glob pattern usage in the codebase follows consistent
//! relative path matching behavior.

use lumin::search::{search_files, SearchOptions};
use lumin::traverse::common::{collect_files_with_excludes, path_matches_any_glob};
use lumin::traverse::{traverse_directory, TraverseOptions};
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_glob_consistency_across_modules() -> Result<(), Box<dyn std::error::Error>> {
    // Create temporary directory structure:
    // temp_dir/
    //   src/
    //     main.rs
    //     lib.rs
    //   tests/
    //     unit.rs
    //     integration.rs
    //   docs/
    //     readme.md
    //     guide.md
    //   Cargo.toml
    
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    let src_dir = temp_path.join("src");
    let tests_dir = temp_path.join("tests");
    let docs_dir = temp_path.join("docs");
    
    fs::create_dir_all(&src_dir)?;
    fs::create_dir_all(&tests_dir)?;
    fs::create_dir_all(&docs_dir)?;
    
    // Create test files
    File::create(src_dir.join("main.rs"))?.write_all(b"fn main() {}")?;
    File::create(src_dir.join("lib.rs"))?.write_all(b"pub fn lib() {}")?;
    File::create(tests_dir.join("unit.rs"))?.write_all(b"#[test] fn test_unit() {}")?;
    File::create(tests_dir.join("integration.rs"))?.write_all(b"#[test] fn test_integration() {}")?;
    File::create(docs_dir.join("readme.md"))?.write_all(b"# README")?;
    File::create(docs_dir.join("guide.md"))?.write_all(b"# User Guide")?;
    File::create(temp_path.join("Cargo.toml"))?.write_all(b"[package]\nname = \"test\"")?;
    
    // Test 1: Search module include_glob should work with relative paths
    let search_options = SearchOptions {
        include_glob: Some(vec!["**/*.rs".to_string()]),
        ..Default::default()
    };
    
    let search_result = search_files("fn", temp_path, &search_options)?;
    let rust_files_count = search_result.lines.len();
    assert!(rust_files_count >= 4, "Should find at least 4 Rust files with 'fn'");
    
    // Verify all found files are .rs files
    for line in &search_result.lines {
        assert!(line.file_path.extension().map_or(false, |ext| ext == "rs"), 
                "All files should be .rs files, found: {}", line.file_path.display());
    }
    
    // Test 2: Search module exclude_glob should work with relative paths
    let search_options_exclude = SearchOptions {
        exclude_glob: Some(vec!["**/tests/**".to_string()]),
        ..Default::default()
    };
    
    let search_result_exclude = search_files("fn", temp_path, &search_options_exclude)?;
    
    // Should not find files in tests directory
    for line in &search_result_exclude.lines {
        assert!(!line.file_path.to_string_lossy().contains("tests"), 
                "Should not find files in tests directory, found: {}", line.file_path.display());
    }
    
    // Test 3: Traverse module collect_files_with_excludes should work with relative paths
    let exclude_patterns = vec!["**/tests/**".to_string()];
    let traverse_files = collect_files_with_excludes(
        temp_path, 
        true, 
        false, 
        None, 
        Some(&exclude_patterns)
    )?;
    
    // Should not include files from tests directory
    for file_path in &traverse_files {
        assert!(!file_path.to_string_lossy().contains("tests"), 
                "Traverse should not include files from tests directory, found: {}", file_path.display());
    }
    
    // Test 4: Traverse module pattern matching should work with relative paths
    let traverse_options = TraverseOptions {
        pattern: Some("**/*.md".to_string()),
        ..Default::default()
    };
    
    let traverse_result = traverse_directory(temp_path, &traverse_options)?;
    
    // Should find markdown files
    let md_files: Vec<_> = traverse_result.into_iter()
        .filter(|entry| entry.file_path.extension().map_or(false, |ext| ext == "md"))
        .collect();
    
    assert!(md_files.len() >= 2, "Should find at least 2 Markdown files");
    
    // Test 5: Direct path_matches_any_glob function with relative paths
    let patterns = vec!["**/*.rs".to_string()];
    let relative_rust_path = Path::new("src/main.rs");
    let matches = path_matches_any_glob(relative_rust_path, &patterns, false)?;
    assert!(matches, "Relative path 'src/main.rs' should match pattern '**/*.rs'");
    
    let relative_md_path = Path::new("docs/readme.md");
    let md_patterns = vec!["**/*.md".to_string()];
    let md_matches = path_matches_any_glob(relative_md_path, &md_patterns, false)?;
    assert!(md_matches, "Relative path 'docs/readme.md' should match pattern '**/*.md'");
    
    // Test 6: Verify consistency between search include_glob and traverse pattern
    let search_md_options = SearchOptions {
        include_glob: Some(vec!["**/*.md".to_string()]),
        ..Default::default()
    };
    
    let search_md_result = search_files("README", temp_path, &search_md_options)?;
    
    // Both search and traverse should find the same types of files
    assert!(search_md_result.lines.len() >= 1, "Search should find at least 1 Markdown file with 'README'");
    assert!(md_files.len() >= 2, "Traverse should find at least 2 Markdown files");
    
    // Test 7: Verify exclude patterns work the same way in both modules
    let search_exclude_md = SearchOptions {
        exclude_glob: Some(vec!["**/*.md".to_string()]),
        ..Default::default()
    };
    
    let search_no_md = search_files("fn", temp_path, &search_exclude_md)?;
    
    // Should not find any markdown files
    for line in &search_no_md.lines {
        assert!(!line.file_path.extension().map_or(false, |ext| ext == "md"),
                "Search with exclude_glob should not find .md files, found: {}", line.file_path.display());
    }
    
    let traverse_exclude_md = vec!["**/*.md".to_string()];
    let traverse_no_md = collect_files_with_excludes(
        temp_path,
        true,
        false,
        None,
        Some(&traverse_exclude_md)
    )?;
    
    // Traverse should also not include markdown files
    for file_path in &traverse_no_md {
        assert!(!file_path.extension().map_or(false, |ext| ext == "md"),
                "Traverse with exclude should not find .md files, found: {}", file_path.display());
    }
    
    println!("✓ All glob patterns work consistently across search and traverse modules");
    println!("✓ Both modules use relative paths for pattern matching");
    println!("✓ Include and exclude patterns behave consistently");
    println!("✓ Pattern matching results are predictable and unified");
    
    Ok(())
}

#[test]
fn test_complex_glob_patterns_consistency() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    
    // Create nested structure
    let nested_dir = temp_path.join("project").join("src").join("modules");
    fs::create_dir_all(&nested_dir)?;
    
    File::create(nested_dir.join("auth.rs"))?.write_all(b"mod auth;")?;
    File::create(nested_dir.join("user.rs"))?.write_all(b"mod user;")?;
    File::create(nested_dir.join("config.toml"))?.write_all(b"[config]")?;
    
    // Test complex patterns work the same in both modules
    let complex_patterns = vec![
        "project/src/**/*.rs".to_string(),
        "**/modules/*.{rs,toml}".to_string(),
    ];
    
    // Test with search module
    let search_options = SearchOptions {
        include_glob: Some(complex_patterns.clone()),
        ..Default::default()
    };
    
    let search_result = search_files("mod", temp_path, &search_options)?;
    assert!(search_result.lines.len() >= 2, "Should find files with complex patterns");
    
    // Test with path_matches_any_glob directly
    let test_path = Path::new("project/src/modules/auth.rs");
    let matches = path_matches_any_glob(test_path, &complex_patterns, false)?;
    assert!(matches, "Complex pattern should match nested file");
    
    println!("✓ Complex glob patterns work consistently across modules");
    
    Ok(())
}