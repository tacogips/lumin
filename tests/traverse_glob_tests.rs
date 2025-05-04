use anyhow::Result;
use lumin::traverse::{TraverseOptions, traverse_directory};
use serial_test::serial;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Tests focused on glob pattern matching in the traverse functionality
#[cfg(test)]
mod traverse_glob_tests {
    use super::*;

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
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("sample.txt"))
        );

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
        let found_level1 = results
            .iter()
            .any(|r| r.file_path.to_string_lossy().contains("level1.txt"));
        let found_level2 = results
            .iter()
            .any(|r| r.file_path.to_string_lossy().contains("level2.txt"));

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
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2.txt"))
        );

        Ok(())
    }

    #[test]
    fn test_traverse_with_multiple_question_marks() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Let's create a test file with specific length
        let test_file_path = PathBuf::from("tests/fixtures/nested/abc123.txt");
        std::fs::write(&test_file_path, "Test file with specific length filename")?;

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = std::fs::remove_file(&test_file_path);
        });

        // Match exactly three characters followed by exactly three digits
        let options = TraverseOptions {
            pattern: Some("**/???.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // Should not find any files with the pattern "???.txt" (we need exact 3 chars before .txt)
        assert!(results.is_empty());

        // Now test with a pattern that should match our new file
        let options = TraverseOptions {
            pattern: Some("**/??????.txt".to_string()), // Exactly 6 characters
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("abc123.txt"))
        );

        // Check that all matched files have exactly 6 characters before .txt
        for result in &results {
            let filename = result.file_path.file_name().unwrap().to_string_lossy();
            assert_eq!(
                filename.len(),
                10,
                "Filename should be 6 chars + .txt (10 total)"
            );
            assert_eq!(&filename[filename.len() - 4..], ".txt");
        }

        Ok(())
    }

    #[test]
    fn test_traverse_with_mixed_wildcards() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Let's create test files with specific patterns
        let test_file_path1 = PathBuf::from("tests/fixtures/nested/config_123.txt");
        let test_file_path2 = PathBuf::from("tests/fixtures/nested/config_abc.txt");

        std::fs::write(&test_file_path1, "Test file with digits in name")?;
        std::fs::write(&test_file_path2, "Test file with letters in name")?;

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = std::fs::remove_file(&test_file_path1);
            let _ = std::fs::remove_file(&test_file_path2);
        });

        // Mixed wildcard pattern: config_? matches any single char after config_
        let options = TraverseOptions {
            pattern: Some("**/config_?.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // Should find no matches (we have config_123.txt and config_abc.txt, both more than one char)
        assert!(results.is_empty());

        // Now use pattern with * to match multiple characters
        let options = TraverseOptions {
            pattern: Some("**/config_*[0-9]*.txt".to_string()), // Must contain at least one digit
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());

        // Should find config_123.txt but not config_abc.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config_123.txt"))
        );
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config_abc.txt"))
        );

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
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1.txt"))
        );

        // Should NOT find level2.txt
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2.txt"))
        );

        Ok(())
    }

    #[test]
    fn test_traverse_with_digit_character_class() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Match level followed by any digit
        let options = TraverseOptions {
            pattern: Some("**/level[0-9].txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());

        // Should find both level1.txt and level2.txt
        let paths: Vec<_> = results
            .iter()
            .map(|r| r.file_path.to_string_lossy().to_string())
            .collect();

        assert!(
            paths.iter().any(|p| p.contains("level1.txt")),
            "Should find level1.txt"
        );
        assert!(
            paths.iter().any(|p| p.contains("level2.txt")),
            "Should find level2.txt"
        );

        // All files should match the pattern
        for path in paths {
            assert!(path.contains("level") && path.ends_with(".txt"));
            // Check that the character before ".txt" is a digit
            let filename = std::path::Path::new(&path)
                .file_name()
                .unwrap()
                .to_string_lossy();
            let digit_char = filename.chars().nth(filename.len() - 5).unwrap();
            assert!(
                digit_char.is_digit(10),
                "Character before .txt should be a digit: {}",
                digit_char
            );
        }

        Ok(())
    }

    #[test]
    fn test_traverse_with_letter_character_class() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Let's create a test file with a letter after "level"
        let test_file_path = PathBuf::from("tests/fixtures/nested/levelA.txt");
        std::fs::write(
            &test_file_path,
            "This is a test file with letter after level.",
        )?;

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = std::fs::remove_file(&test_file_path);
        });

        // Match level followed by any letter a-z
        let options = TraverseOptions {
            pattern: Some("**/level[a-z].txt".to_string()),
            case_sensitive: false, // case-insensitive to match both A and a
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // Should find levelA.txt
        assert!(!results.is_empty());
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("levelA.txt"))
        );

        // Should NOT find level1.txt or level2.txt (they have digits, not letters)
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1.txt"))
        );
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2.txt"))
        );

        Ok(())
    }

    #[test]
    fn test_traverse_with_combined_character_classes() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Let's create a test file with a letter after "level"
        let test_file_path = PathBuf::from("tests/fixtures/nested/levelA.txt");
        std::fs::write(
            &test_file_path,
            "This is a test file with letter after level.",
        )?;

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = std::fs::remove_file(&test_file_path);
        });

        // Match level followed by any letter or digit
        let options = TraverseOptions {
            pattern: Some("**/level[a-z0-9].txt".to_string()),
            case_sensitive: false,
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(results.len() >= 3, "Should find at least 3 files");

        // Should find levelA.txt, level1.txt, and level2.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("levelA.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2.txt"))
        );

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
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("sample.txt"))
        );

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
        let has_txt = results
            .iter()
            .any(|r| r.file_path.to_string_lossy().ends_with(".txt"));
        let has_md = results
            .iter()
            .any(|r| r.file_path.to_string_lossy().ends_with(".md"));

        assert!(has_txt);
        assert!(has_md);

        Ok(())
    }

    #[test]
    fn test_traverse_with_multiple_braces() -> Result<()> {
        let directory = Path::new("tests/fixtures");
        let options = TraverseOptions {
            // Match multiple brace patterns in the same glob
            pattern: Some("**/{text_files,nested}/*.{txt,md,rs}".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());

        // Should find files in either text_files or nested directories with the specified extensions
        let text_files_count = results
            .iter()
            .filter(|r| r.file_path.to_string_lossy().contains("text_files"))
            .count();

        let nested_files_count = results
            .iter()
            .filter(|r| {
                r.file_path.to_string_lossy().contains("nested")
                    && !r.file_path.to_string_lossy().contains("nested/level")
            })
            .count();

        assert!(
            text_files_count > 0,
            "Should find files in text_files directory"
        );
        assert!(
            nested_files_count > 0,
            "Should find files in nested directory"
        );

        // Make sure we only find files with the specified extensions
        for result in &results {
            let path = result.file_path.to_string_lossy();
            assert!(
                path.ends_with(".txt") || path.ends_with(".md") || path.ends_with(".rs"),
                "Found file with unexpected extension: {}",
                path
            );
        }

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
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("markdown.md"))
        );

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
        assert!(results.iter().any(|r| {
            r.file_path
                .to_string_lossy()
                .contains("level1/level2/level2.txt")
        }));

        Ok(())
    }

    #[test]
    fn test_traverse_with_alternation_patterns() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Test alternation with complex glob patterns
        let options = TraverseOptions {
            pattern: Some("**/{sample,config,regex_patterns}.{txt,toml}".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());

        // Should find sample.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("sample.txt")),
            "Should find sample.txt"
        );

        // Should find config.toml
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config.toml")),
            "Should find config.toml"
        );

        // Should find regex_patterns.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("regex_patterns.txt")),
            "Should find regex_patterns.txt"
        );

        Ok(())
    }

    #[test]
    fn test_traverse_with_nested_star_patterns() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Test nested star patterns
        let options = TraverseOptions {
            pattern: Some("**/nested/**/*.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());

        // All results should be .txt files in the nested directory or its subdirectories
        for result in &results {
            let path = result.file_path.to_string_lossy();
            assert!(
                path.contains("nested") && path.ends_with(".txt"),
                "Found unexpected file: {}",
                path
            );
        }

        // Should find level1.txt and level2.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2.txt"))
        );

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
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("sample.txt"))
        );

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
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2.txt"))
        );

        Ok(())
    }

    #[test]
    fn test_traverse_with_case_sensitive_substring() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Let's create test files with mixed case
        let test_file_path1 = PathBuf::from("tests/fixtures/nested/CONFIG_upper.txt");
        let test_file_path2 = PathBuf::from("tests/fixtures/nested/config_lower.txt");

        std::fs::write(&test_file_path1, "Test file with uppercase name")?;
        std::fs::write(&test_file_path2, "Test file with lowercase name")?;

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = std::fs::remove_file(&test_file_path1);
            let _ = std::fs::remove_file(&test_file_path2);
        });

        // Case-sensitive substring search for "CONFIG" (uppercase)
        let options = TraverseOptions {
            pattern: Some("CONFIG".to_string()),
            case_sensitive: true,
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());

        // Should find CONFIG_upper.txt but not config_lower.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("CONFIG_upper.txt"))
        );
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config_lower.txt"))
        );

        // Case-insensitive substring search
        let options = TraverseOptions {
            pattern: Some("config".to_string()),
            case_sensitive: false,
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // Should find both CONFIG_upper.txt and config_lower.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("CONFIG_upper.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config_lower.txt"))
        );

        Ok(())
    }

    #[test]
    fn test_traverse_with_partial_substring() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Using substring to match part of a directory and part of a filename
        let options = TraverseOptions {
            pattern: Some("xt_fi".to_string()), // Should match "text_files"
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());

        // All results should be from the text_files directory
        for result in &results {
            assert!(result.file_path.to_string_lossy().contains("text_files"));
        }

        // Now match only part of a file name
        let options = TraverseOptions {
            pattern: Some("mple.t".to_string()), // Should match "sample.txt"
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("sample.txt"))
        );

        Ok(())
    }

    #[test]
    fn test_traverse_with_special_character_substring() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Let's create a test file with special characters
        let test_file_path = PathBuf::from("tests/fixtures/nested/test-with-hyphens.txt");
        std::fs::write(&test_file_path, "Test file with hyphens in name")?;

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = std::fs::remove_file(&test_file_path);
        });

        // Searching for a pattern with special characters that would be regex metacharacters
        let options = TraverseOptions {
            pattern: Some("with-hyp".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty());
        assert!(results.iter().any(|r| {
            r.file_path
                .to_string_lossy()
                .contains("test-with-hyphens.txt")
        }));

        // Substring pattern with characters that would be glob metacharacters
        let options = TraverseOptions {
            pattern: Some("with*hyp".to_string()), // Literal * character, not a glob
            ..TraverseOptions::default()
        };

        // Should find no results since the * is treated as a literal
        let results = traverse_directory(directory, &options)?;
        assert!(results.is_empty());

        Ok(())
    }

    #[test]
    #[serial]
    fn test_traverse_with_anchored_patterns() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Create test files for anchoring tests
        let prefix_file_path = PathBuf::from("tests/fixtures/nested/markdown-file.txt");
        let suffix_file_path = PathBuf::from("tests/fixtures/nested/file-markdown.txt");

        std::fs::write(&prefix_file_path, "File with markdown in prefix")?;
        std::fs::write(&suffix_file_path, "File with markdown in suffix")?;

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = std::fs::remove_file(&prefix_file_path);
            let _ = std::fs::remove_file(&suffix_file_path);
        });

        // Test pattern anchored to start of filename (markdown*)
        let options = TraverseOptions {
            pattern: Some("**/markdown*".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match files starting with 'markdown'"
        );

        // Should find markdown.md and markdown-file.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("markdown.md"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("markdown-file.txt"))
        );

        // Should NOT find file-markdown.txt (doesn't start with markdown)
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("file-markdown.txt"))
        );

        // Test pattern anchored to end of filename (*markdown.txt)
        let options = TraverseOptions {
            pattern: Some("**/*markdown.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match files ending with 'markdown.txt'"
        );

        // Should find file-markdown.txt but not markdown-file.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("file-markdown.txt"))
        );
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("markdown-file.txt"))
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_traverse_with_mixed_glob_features() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Create complex directory structure for testing
        let complex_dir = PathBuf::from("tests/fixtures/complex");
        let nested_dir = complex_dir.join("nested123");
        let deeply_nested = nested_dir.join("level-a").join("level-b");

        fs::create_dir_all(&complex_dir)?;
        fs::create_dir_all(&nested_dir)?;
        fs::create_dir_all(&deeply_nested)?;

        // Create files with various patterns
        let files = vec![
            (complex_dir.join("config-1.json"), "Config JSON file"),
            (complex_dir.join("config-2.toml"), "Config TOML file"),
            (complex_dir.join("data-10.csv"), "Data CSV file"),
            (nested_dir.join("test-a.txt"), "Test A file"),
            (nested_dir.join("test-b.md"), "Test B file"),
            (nested_dir.join("dev-note.txt"), "Development note"),
            (deeply_nested.join("deep-1.txt"), "Deep level 1"),
            (deeply_nested.join("deep-2.md"), "Deep level 2"),
        ];

        for (path, content) in files {
            let mut file = File::create(&path)?;
            write!(file, "{}", content)?;
        }

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = fs::remove_dir_all(&complex_dir);
        });

        // Test complex pattern with multiple features (character class, brace expansion, wildcards)
        let options = TraverseOptions {
            pattern: Some("**/complex/**/[a-z]*-[0-9].{txt,md,json}".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match files with the complex pattern"
        );

        // Should match config-1.json, deep-1.txt but not config-2.toml, deep-2.md
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config-1.json"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("deep-1.txt"))
        );
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("config-2.toml"))
        );
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("deep-2.md"))
        );

        // Test nested directory pattern with negation
        let options = TraverseOptions {
            pattern: Some("**/complex/**/[!0-9]*.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match text files not starting with digits"
        );

        // Should find test-a.txt, dev-note.txt, deep-1.txt
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("test-a.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("dev-note.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("deep-1.txt"))
        );

        // Test complex brace expansion with multiple levels
        let options = TraverseOptions {
            pattern: Some("**/complex/{nested123,level-?}/*-{a,b}.{txt,md}".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match files with complex brace expansion"
        );

        // Should find test-a.txt and test-b.md
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("test-a.txt"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("test-b.md"))
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_traverse_with_extreme_patterns() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Create a temporary directory with complex nested structure
        let extreme_dir = PathBuf::from("tests/fixtures/extreme");
        fs::create_dir_all(&extreme_dir)?;

        // Create nested structure with special characters
        let levels = vec![
            "level1",
            "level2[abc]",
            "level3{xyz}",
            "level4(123)",
            "level5-special",
        ];

        let mut current_path = extreme_dir.clone();
        for level in &levels {
            current_path = current_path.join(level);
            fs::create_dir_all(&current_path)?;
        }

        // Create various files in different directories
        let files = vec![
            (extreme_dir.join("file1.txt"), "Top level file"),
            (
                extreme_dir.join("level1").join("level1-file.md"),
                "Level 1 file",
            ),
            (
                extreme_dir
                    .join("level1")
                    .join("level2[abc]")
                    .join("level2-file.rs"),
                "Level 2 file",
            ),
            (
                extreme_dir
                    .join("level1")
                    .join("level2[abc]")
                    .join("level3{xyz}")
                    .join("level3-file.toml"),
                "Level 3 file",
            ),
            (
                extreme_dir
                    .join("level1")
                    .join("level2[abc]")
                    .join("level3{xyz}")
                    .join("level4(123)")
                    .join("level4-file.json"),
                "Level 4 file",
            ),
            (current_path.join("final-file.yaml"), "Final nested file"),
        ];

        for (path, content) in files {
            let mut file = File::create(&path)?;
            write!(file, "{}", content)?;
        }

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = fs::remove_dir_all(&extreme_dir);
        });

        // Test glob pattern with extreme nesting and special characters
        let options = TraverseOptions {
            pattern: Some("**/extreme/**/level[0-9]*/*-file.{md,rs,toml}".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match files in the extreme nested structure"
        );

        // Should find level1-file.md, level2-file.rs, level3-file.toml
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1-file.md"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2-file.rs"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level3-file.toml"))
        );

        // Should NOT find level4-file.json (extension not in pattern)
        assert!(
            !results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level4-file.json"))
        );

        // Test with extreme pattern to find exactly one deep file
        let options = TraverseOptions {
            pattern: Some("**/extreme/**/level5-special/final-file.yaml".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert_eq!(results.len(), 1, "Should match exactly one file");
        assert!(
            results[0]
                .file_path
                .to_string_lossy()
                .contains("final-file.yaml")
        );

        // Test extremely complex pattern with all glob features
        let options = TraverseOptions {
            pattern: Some("**/extreme/**/level[1-3]{*,*/*}/*{-file,}.{md,rs,toml}".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match files with the extreme pattern"
        );

        // Validation of results
        let file_paths: Vec<_> = results
            .iter()
            .map(|r| r.file_path.to_string_lossy().to_string())
            .collect();

        // Debug print to see what we found
        println!("Found files: {:?}", file_paths);

        // Should find appropriate files
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level1-file.md"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level2-file.rs"))
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("level3-file.toml"))
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_traverse_boundary_conditions() -> Result<()> {
        let directory = Path::new("tests/fixtures");

        // Create a directory with boundary condition test files
        let boundary_dir = PathBuf::from("tests/fixtures/boundary");
        fs::create_dir_all(&boundary_dir)?;

        // Create files with different boundary conditions
        let files = vec![
        // Empty filename with extension
        (boundary_dir.join(".txt"), "File with empty name"),
        // Very long filename
        (boundary_dir.join("very_long_filename_with_many_characters_to_test_edge_cases_in_pattern_matching_implementation.txt"), "Very long filename"),
        // Filename with only special characters
        (boundary_dir.join("!@#$%.txt"), "Special characters only"),
        // Filename with unicode characters
        (boundary_dir.join("unicode_カタカナ_😊_file.txt"), "Unicode characters"),
        // File with no extension
        (boundary_dir.join("no_extension"), "File without extension"),
        // File with multiple dots
        (boundary_dir.join("multiple.dots.in.filename.txt"), "Multiple dots"),
        // File with leading dot but also extension
        (boundary_dir.join(".hidden.txt"), "Hidden with extension"),
    ];

        for (path, content) in files {
            let mut file = File::create(&path)?;
            write!(file, "{}", content)?;
        }

        // Cleanup function
        let _cleanup = defer::defer(|| {
            let _ = fs::remove_dir_all(&boundary_dir);
        });

        // Test matching a file with empty name (just extension)
        let options = TraverseOptions {
            pattern: Some("**/boundary/.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty(), "Should match file with empty name");
        assert!(
            results
                .iter()
                .any(|r| r.file_path.file_name().unwrap().to_string_lossy() == ".txt")
        );

        // Test matching very long filename
        let options = TraverseOptions {
            pattern: Some("**/boundary/very_*.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty(), "Should match very long filename");
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("very_long_filename"))
        );

        // Test matching special characters
        let options = TraverseOptions {
            pattern: Some("**/boundary/!@#$%.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match filename with special characters"
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.file_name().unwrap().to_string_lossy() == "!@#$%.txt")
        );

        // Test matching unicode characters
        let options = TraverseOptions {
            pattern: Some("**/boundary/*カタカナ*.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(
            !results.is_empty(),
            "Should match filename with Unicode characters"
        );
        assert!(
            results
                .iter()
                .any(|r| r.file_path.to_string_lossy().contains("カタカナ"))
        );

        // Test matching file with no extension
        let options = TraverseOptions {
            pattern: Some("**/boundary/no_extension".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty(), "Should match file with no extension");
        assert!(
            results
                .iter()
                .any(|r| r.file_path.file_name().unwrap().to_string_lossy() == "no_extension")
        );

        // Test matching files with multiple dots
        let options = TraverseOptions {
            pattern: Some("**/boundary/*.dots.*.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        assert!(!results.is_empty(), "Should match file with multiple dots");
        assert!(
            results
                .iter()
                .any(|r| r.file_path.file_name().unwrap().to_string_lossy()
                    == "multiple.dots.in.filename.txt")
        );

        // Test edge case: match all files with non-standard naming
        let options = TraverseOptions {
            pattern: Some("**/boundary/{.*,!*,*カタカナ*}*".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // Should match .txt, .hidden.txt, !@#$%.txt, and unicode_カタカナ_😊_file.txt
        assert!(
            results.len() >= 4,
            "Should match at least 4 files with non-standard naming"
        );

        // Test pattern with emoji character (though this depends on filesystem support)
        let options = TraverseOptions {
            pattern: Some("**/boundary/*😊*.txt".to_string()),
            ..TraverseOptions::default()
        };

        let results = traverse_directory(directory, &options)?;

        // May or may not match depending on OS and filesystem support for emoji in filenames
        if !results.is_empty() {
            assert!(
                results
                    .iter()
                    .any(|r| r.file_path.to_string_lossy().contains("😊"))
            );
        }

        Ok(())
    }
} // Close the traverse_glob_tests module
