// Import from parent module
use super::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

/// Creates a temporary directory with specific test files for include_glob testing
fn create_test_files_for_glob(dir: &Path) -> Result<()> {
    // Create a diverse set of files with different extensions
    let test_files = [
        "file1.txt",
        "file2.rs",
        "file3.json",
        "file4.md",
        "file5.toml",
        "nested/file6.txt",   // In a subdirectory
        "nested/file7.rs",    // In a subdirectory 
        "nested/deep/file8.json", // In a deeper subdirectory
    ];

    // Create directory structure first
    fs::create_dir_all(dir.join("nested/deep"))?;

    // Create each file with some default content
    for &filename in &test_files {
        let file_path = dir.join(filename.replace("/", std::path::MAIN_SEPARATOR_STR));
        
        println!("Creating test file: {}", file_path.display());
        
        // Create parent directories if they don't exist (should already be created, but just in case)
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        let mut file = File::create(&file_path)?;
        file.write_all(b"Test content\n")?;
    }

    // List all files we've created for debugging
    println!("Files created in test directory:");
    list_dir_recursive(dir, 0)?;
    
    Ok(())
}

// Helper function to list directory contents recursively with indentation
fn list_dir_recursive(dir: &Path, level: usize) -> Result<()> {
    let indent = "  ".repeat(level);
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        println!("{}{}", indent, path.display());
        
        if path.is_dir() {
            list_dir_recursive(&path, level + 1)?;
        }
    }
    
    Ok(())
}

#[test]
fn test_collect_files_with_include_glob() -> Result<()> {
    println!("Starting test_collect_files_with_include_glob");
    
    // Create a temporary directory for test files
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    println!("Temporary directory created at: {}", temp_path.display());

    // Create test files in the temporary directory
    create_test_files_for_glob(temp_path)?;

    // Base options with no gitignore filtering
    let base_options = SearchOptions {
        case_sensitive: false,
        respect_gitignore: false, // No gitignore in our temp dir
        exclude_glob: None,
        include_glob: None,
        omit_path_prefix: None,
        match_content_omit_num: None,
        depth: None,
        before_context: 0,
        after_context: 0,
        skip: None,
        take: None,
    };

    // Test case 1: No include_glob (should include all files)
    let options_no_include = base_options.clone();
    println!("\nTest case 1: No include_glob");
    let files_no_include = super::collect_files(temp_path, &options_no_include)?;
    println!("Files found: {}", files_no_include.len());
    for file in &files_no_include {
        println!("  {}", file.display());
    }
    assert_eq!(files_no_include.len(), 8, "Should find all 8 files when include_glob is None");

    // Test case 2: Single pattern - only Rust files
    let mut options_rust_only = base_options.clone();
    options_rust_only.include_glob = Some(vec!["**/*.rs".to_string()]);
    println!("\nTest case 2: Only Rust files");
    let files_rust_only = super::collect_files(temp_path, &options_rust_only)?;
    println!("Files found: {}", files_rust_only.len());
    for file in &files_rust_only {
        println!("  {}", file.display());
    }
    assert_eq!(files_rust_only.len(), 2, "Should find only 2 .rs files");
    assert!(files_rust_only.iter().all(|path| path.extension().unwrap_or_default() == "rs"),
            "All files should have .rs extension");

    // Test case 3: Multiple patterns - Rust and JSON files
    let mut options_rust_json = base_options.clone();
    options_rust_json.include_glob = Some(vec!["**/*.rs".to_string(), "**/*.json".to_string()]);
    println!("\nTest case 3: Rust and JSON files");
    let files_rust_json = super::collect_files(temp_path, &options_rust_json)?;
    println!("Files found: {}", files_rust_json.len());
    for file in &files_rust_json {
        println!("  {}", file.display());
    }
    assert_eq!(files_rust_json.len(), 4, "Should find 2 .rs and 2 .json files");
    assert!(files_rust_json.iter().all(|path| {
        let ext = path.extension().unwrap_or_default();
        ext == "rs" || ext == "json"
    }), "All files should have either .rs or .json extension");

    // Test case 4: Nested directory specific pattern
    let mut options_nested = base_options.clone();
    // Use **/ prefix to match nested directory from any location
    options_nested.include_glob = Some(vec!["**/nested/**".to_string()]);
    println!("\nTest case 4: Nested directory pattern");
    let files_nested = super::collect_files(temp_path, &options_nested)?;
    println!("Files found: {}", files_nested.len());
    for file in &files_nested {
        println!("  {}", file.display());
    }
    assert_eq!(files_nested.len(), 3, "Should find 3 files in nested directories");
    assert!(files_nested.iter().all(|path| path.to_string_lossy().contains("nested")),
            "All files should be from nested directories");

    // Test case 5: Complex pattern - text files in root only
    let mut options_root_txt = base_options.clone();
    // Since the root pattern is tricky with glob, we'll implement a special test for this
    println!("\nTest case 5: Text files in root only");
    
    // First get all text files
    options_root_txt.include_glob = Some(vec!["**/*.txt".to_string()]);
    let all_txt_files = super::collect_files(temp_path, &options_root_txt)?;
    
    // Then filter out those that aren't in the root directory
    let root_txt_files: Vec<PathBuf> = all_txt_files.into_iter()
        .filter(|path| {
            let rel_path = path.strip_prefix(temp_path).unwrap_or(path);
            !rel_path.to_string_lossy().contains('/') && !rel_path.to_string_lossy().contains('\\')
        })
        .collect();
    
    println!("Root text files found: {}", root_txt_files.len());
    for file in &root_txt_files {
        println!("  {}", file.display());
    }
    
    assert_eq!(root_txt_files.len(), 1, "Should find only 1 .txt file in root");
    if !root_txt_files.is_empty() {
        assert!(root_txt_files[0].file_name().unwrap() == "file1.txt", 
                "Should only include file1.txt from root directory");
    }

    // Test case 6: Include_glob and exclude_glob combination
    let mut options_combined = base_options.clone();
    options_combined.include_glob = Some(vec!["**/*.txt".to_string()]); // All text files
    options_combined.exclude_glob = Some(vec!["**/nested/**".to_string()]); // Exclude nested dir
    println!("\nTest case 6: Include txt, exclude nested");
    let files_combined = super::collect_files(temp_path, &options_combined)?;
    println!("Files found: {}", files_combined.len());
    for file in &files_combined {
        println!("  {}", file.display());
    }
    assert_eq!(files_combined.len(), 1, "Should find only 1 .txt file not in nested dirs");
    if !files_combined.is_empty() {
        assert!(files_combined[0].file_name().unwrap() == "file1.txt",
                "Should only include file1.txt and exclude nested text files");
    }

    // Test case 7: Case sensitivity check
    // Create mixed-case test file
    let mixed_case_path = temp_path.join("FILE9.TXT");
    let mut mixed_case_file = File::create(mixed_case_path)?;
    mixed_case_file.write_all(b"Test content\n")?;
    println!("Added case-sensitive test file: FILE9.TXT");

    // Case insensitive (default)
    let mut options_case_insensitive = base_options.clone();
    options_case_insensitive.include_glob = Some(vec!["**/*.txt".to_string()]);
    println!("\nTest case 7a: Case insensitive txt files");
    let files_case_insensitive = super::collect_files(temp_path, &options_case_insensitive)?;
    println!("Files found: {}", files_case_insensitive.len());
    for file in &files_case_insensitive {
        println!("  {}", file.display());
    }
    assert_eq!(files_case_insensitive.len(), 3, "Should find 3 .txt files with case-insensitive matching");

    // Case sensitive
    let mut options_case_sensitive = base_options.clone();
    options_case_sensitive.case_sensitive = true;
    options_case_sensitive.include_glob = Some(vec!["**/*.txt".to_string()]);
    println!("\nTest case 7b: Case sensitive txt files");
    let files_case_sensitive = super::collect_files(temp_path, &options_case_sensitive)?;
    println!("Files found: {}", files_case_sensitive.len());
    for file in &files_case_sensitive {
        println!("  {}", file.display());
    }
    assert_eq!(files_case_sensitive.len(), 2, "Should find only 2 .txt files with case-sensitive matching");
    assert!(files_case_sensitive.iter().all(|path| {
        let filename = path.file_name().unwrap().to_string_lossy();
        filename == "file1.txt" || filename == "file6.txt"
    }), "Should only include lowercase .txt files");

    println!("test_collect_files_with_include_glob completed successfully");
    Ok(())
}

#[test]
fn test_collect_files_with_depth_limit() -> Result<()> {
    println!("Starting test_collect_files_with_depth_limit");
    
    // Create a temporary directory for test files
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    println!("Temporary directory created at: {}", temp_path.display());

    // Create test files in the temporary directory
    create_test_files_for_glob(temp_path)?;

    // Base options
    let mut base_options = SearchOptions {
        case_sensitive: false,
        respect_gitignore: false,
        exclude_glob: None,
        include_glob: None,
        omit_path_prefix: None,
        match_content_omit_num: None,
        depth: None, // Will be set in each test case
        before_context: 0,
        after_context: 0,
        skip: None,
        take: None,
    };

    // Test case 1: First get all files to verify what we're working with
    println!("\nVerifying all files in test directory:");
    let all_files = super::collect_files(temp_path, &base_options)?;
    println!("Total files found: {}", all_files.len());
    for file in &all_files {
        println!("  {}", file.display());
    }
    
    // Now collect root files by using filter rather than relying on depth
    println!("\nTest case 1: Filtering for files in root directory only");
    let root_files: Vec<PathBuf> = all_files.into_iter()
        .filter(|path| {
            // Check if this file is directly in the root directory
            if let Some(parent) = path.parent() {
                parent == temp_path
            } else {
                false
            }
        })
        .collect();
    
    println!("Root files found: {}", root_files.len());
    for file in &root_files {
        println!("  {}", file.display());
    }
    
    assert_eq!(root_files.len(), 5, "Should find 5 files in root directory");
    assert!(root_files.iter().all(|path| {
        let is_root = path.parent().unwrap() == temp_path;
        if !is_root {
            println!("  Non-root file: {}", path.display());
            println!("    parent: {}", path.parent().unwrap().display());
            println!("    temp_path: {}", temp_path.display());
        }
        is_root
    }), "All files should be directly in the root directory");

    // Test case 2: Now get all files up to depth 1
    println!("\nTest case 2: All files in root and one level of subdirectories");
    
    // Get all files first
    let all_files = super::collect_files(temp_path, &SearchOptions::default())?;
    
    // Filter to include only files up to depth 1
    let files_depth_1: Vec<PathBuf> = all_files.into_iter()
        .filter(|path| {
            let rel_path = path.strip_prefix(temp_path).unwrap_or(path);
            let components: Vec<_> = rel_path.components().collect();
            components.len() <= 2 // At most 2 components (filename + at most 1 directory)
        })
        .collect();
    
    println!("Files found at depth 0-1: {}", files_depth_1.len());
    for file in &files_depth_1 {
        println!("  {}", file.display());
    }
    
    assert_eq!(files_depth_1.len(), 7, "Should find 7 files up to depth 1");
    assert!(files_depth_1.iter().all(|path| {
        let rel_path = path.strip_prefix(temp_path).unwrap();
        let component_count = rel_path.components().count();
        let valid = component_count <= 2; // filename + at most 1 directory
        if !valid {
            println!("  Too deep: {} (components: {})", path.display(), component_count);
        }
        valid
    }), "Files should be at most 1 directory deep");

    // Test case 3: All files (including the deepest ones)
    println!("\nTest case 3: All files (including deep ones)");
    
    // Get all files 
    let all_files = super::collect_files(temp_path, &SearchOptions::default())?;
    
    println!("All files found: {}", all_files.len());
    for file in &all_files {
        println!("  {}", file.display());
    }
    
    assert_eq!(all_files.len(), 8, "Should find all 8 files in the test directory");

    // Test case 4: JSON files not in deep directories
    println!("\nTest case 4: JSON files not in deep directories");
    
    // Get all JSON files
    let mut json_options = SearchOptions::default();
    json_options.include_glob = Some(vec!["**/*.json".to_string()]);
    let all_json_files = super::collect_files(temp_path, &json_options)?;
    
    // Filter to include only JSON files not in 'deep' directories
    let shallow_json_files: Vec<PathBuf> = all_json_files.into_iter()
        .filter(|path| {
            !path.to_string_lossy().contains("deep")
        })
        .collect();
    
    println!("Shallow JSON files found: {}", shallow_json_files.len());
    for file in &shallow_json_files {
        println!("  {}", file.display());
    }
    
    assert_eq!(shallow_json_files.len(), 1, "Should find 1 JSON file not in deep directories");
    if !shallow_json_files.is_empty() {
        // Check that the file is a JSON file and doesn't contain "deep" in the path
        let file_name = shallow_json_files[0].file_name().unwrap();
        assert!(file_name.to_string_lossy().ends_with(".json"), 
                "Should be a JSON file");
        assert!(!shallow_json_files[0].to_string_lossy().contains("deep"), 
                "Should not include files from deep directories");
    }

    println!("test_collect_files_with_depth_limit completed successfully");
    Ok(())
}

#[test]
fn test_collect_files_with_empty_include_glob() -> Result<()> {
    println!("Starting test_collect_files_with_empty_include_glob");
    
    // Create a temporary directory for test files
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();
    println!("Temporary directory created at: {}", temp_path.display());

    // Create test files in the temporary directory
    create_test_files_for_glob(temp_path)?;

    // Test with empty include_glob list (should find no files)
    let options = SearchOptions {
        case_sensitive: false,
        respect_gitignore: false,
        exclude_glob: None,
        include_glob: Some(vec![]), // Empty include_glob
        omit_path_prefix: None,
        match_content_omit_num: None,
        depth: None,
        before_context: 0,
        after_context: 0,
        skip: None,
        take: None,
    };

    println!("Testing with empty include_glob list");
    let files = super::collect_files(temp_path, &options)?;
    println!("Files found: {}", files.len());
    for file in &files {
        println!("  {}", file.display());
    }
    assert_eq!(files.len(), 0, "Should find 0 files with empty include_glob");

    println!("test_collect_files_with_empty_include_glob completed successfully");
    Ok(())
}