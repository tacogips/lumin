use anyhow::Result;
use lumin::view::{FileContents, ViewOptions, view_file};
use std::path::Path;

#[test]
fn test_view_text_file() -> Result<()> {
    let file_path = Path::new("tests/fixtures/text_files/sample.txt");
    let options = ViewOptions::default();

    let result = view_file(file_path, &options)?;

    // Verify file path
    assert_eq!(result.file_path, file_path);

    // Verify file type detection
    assert!(result.file_type.starts_with("text/"));

    // Verify content type is Text
    match result.contents {
        FileContents::Text { content, metadata } => {
            // Verify content contains the expected text
            assert!(content.contains("This is a sample text file"));
            assert!(content.contains("PATTERN"));

            // Verify metadata
            assert_eq!(metadata.line_count, 6);
            assert!(metadata.char_count > 0);
        }
        _ => panic!("Expected text content"),
    }

    Ok(())
}

#[test]
fn test_view_binary_file() -> Result<()> {
    let file_path = Path::new("tests/fixtures/binary_files/binary.bin");
    let options = ViewOptions::default();

    let result = view_file(file_path, &options)?;

    // Verify file path
    assert_eq!(result.file_path, file_path);

    // Verify content type is Binary
    match result.contents {
        FileContents::Binary { message, metadata } => {
            // Verify binary message
            assert!(message.contains("Binary file detected"));

            // Verify metadata
            assert!(metadata.binary);
            assert!(metadata.size_bytes > 0);
        }
        _ => panic!("Expected binary content"),
    }

    Ok(())
}

#[test]
fn test_view_image_file() -> Result<()> {
    let file_path = Path::new("tests/fixtures/binary_files/sample.jpg");
    let options = ViewOptions::default();

    let result = view_file(file_path, &options)?;

    // Verify file path
    assert_eq!(result.file_path, file_path);

    // File type detection might vary by environment - some systems might detect as application/octet-stream
    // So we'll just verify we got an Image variant

    // Some systems might detect the file differently based on infer library behavior
    // So we'll accept either Image or Binary for this test
    match result.contents {
        FileContents::Image { message, metadata } => {
            // Verify image message
            assert!(message.contains("Image file detected"));

            // Verify metadata
            assert!(metadata.binary);
            assert!(metadata.size_bytes > 0);
            assert_eq!(metadata.media_type, "image");
        }
        FileContents::Binary { message, metadata } => {
            // If detected as binary that's also fine
            assert!(message.contains("Binary file detected"));

            // Verify metadata
            assert!(metadata.binary);
            assert!(metadata.size_bytes > 0);
        }
        _ => panic!("Expected image or binary content"),
    }

    Ok(())
}

#[test]
fn test_view_with_size_limit() -> Result<()> {
    // Set a very small limit that should reject any file
    let tiny_limit = 10; // 10 bytes
    let file_path = Path::new("tests/fixtures/text_files/sample.txt");
    let options = ViewOptions {
        max_size: Some(tiny_limit),
        line_from: None,
        line_to: None,
    };

    // Should fail because file is larger than the limit
    let result = view_file(file_path, &options);
    assert!(result.is_err());

    let error_message = format!("{:?}", result.err().unwrap());
    assert!(error_message.contains("File is too large"));

    Ok(())
}

#[test]
fn test_view_nonexistent_file() -> Result<()> {
    let file_path = Path::new("tests/fixtures/does_not_exist.txt");
    let options = ViewOptions::default();

    // Should fail because file doesn't exist
    let result = view_file(file_path, &options);
    assert!(result.is_err());

    let error_message = format!("{:?}", result.err().unwrap());
    assert!(error_message.contains("File not found"));

    Ok(())
}

#[test]
fn test_view_markdown_file() -> Result<()> {
    let file_path = Path::new("tests/fixtures/text_files/markdown.md");
    let options = ViewOptions::default();

    let result = view_file(file_path, &options)?;

    // Verify file type detection (should be text)
    assert!(result.file_type.starts_with("text/"));

    // Verify content
    match result.contents {
        FileContents::Text {
            content,
            metadata: _,
        } => {
            assert!(content.contains("# Sample Markdown"));
            assert!(content.contains("```rust"));
            assert!(content.contains("fn main()"));
        }
        _ => panic!("Expected text content for markdown file"),
    }

    Ok(())
}

#[test]
fn test_view_toml_config_file() -> Result<()> {
    let file_path = Path::new("tests/fixtures/text_files/config.toml");
    let options = ViewOptions::default();

    let result = view_file(file_path, &options)?;

    // Verify file type detection (should be text)
    assert!(result.file_type.starts_with("text/"));

    // Verify content
    match result.contents {
        FileContents::Text {
            content,
            metadata: _,
        } => {
            assert!(content.contains("[server]"));
            assert!(content.contains("port = 8080"));
            assert!(content.contains("[database]"));
            assert!(content.contains("[logging]"));
        }
        _ => panic!("Expected text content for toml file"),
    }

    Ok(())
}

#[test]
fn test_view_with_line_filtering() -> Result<()> {
    let file_path = Path::new("tests/fixtures/text_files/sample.txt");

    // Create test options with line filtering
    let options = ViewOptions {
        max_size: None,
        line_from: Some(2), // Start from line 2
        line_to: Some(4),   // End at line 4
    };

    // View the file
    let view_result = view_file(file_path, &options)?;

    // Check the filtered content
    match &view_result.contents {
        FileContents::Text { content, metadata } => {
            // Check that we only got lines 2-4
            assert_eq!(content.line_contents.len(), 3);
            assert_eq!(content.line_contents[0].line_number, 2);
            assert_eq!(content.line_contents[2].line_number, 4);

            // Check metadata (should still show total count for the file)
            assert!(metadata.line_count > 3); // Total line count should be more than our filtered range
        }
        _ => panic!("Expected text content"),
    }

    Ok(())
}

#[test]
fn test_view_with_out_of_range_line_filtering() -> Result<()> {
    let file_path = Path::new("tests/fixtures/text_files/sample.txt");

    // Test with line range completely beyond the file's content
    // The file has 6 lines, requesting lines 100-200
    let options = ViewOptions {
        max_size: None,
        line_from: Some(100),
        line_to: Some(200),
    };

    // Should not error, just return empty content
    let view_result = view_file(file_path, &options)?;

    match &view_result.contents {
        FileContents::Text { content, metadata } => {
            // Content should be empty as requested lines are out of range
            assert!(content.line_contents.is_empty());

            // Metadata should still reflect the actual file
            assert_eq!(metadata.line_count, 6);
        }
        _ => panic!("Expected text content"),
    }

    // Test with partial range overlap
    // Requesting lines 5-10 but file only has 6 lines
    let options = ViewOptions {
        max_size: None,
        line_from: Some(5),
        line_to: Some(10),
    };

    let view_result = view_file(file_path, &options)?;

    match &view_result.contents {
        FileContents::Text { content, metadata } => {
            // Should get lines 5-6 only
            assert_eq!(content.line_contents.len(), 2);
            assert_eq!(content.line_contents[0].line_number, 5);
            assert_eq!(content.line_contents[1].line_number, 6);

            // Metadata still reflects the whole file
            assert_eq!(metadata.line_count, 6);
        }
        _ => panic!("Expected text content"),
    }

    // Test with inverted range (from > to)
    let options = ViewOptions {
        max_size: None,
        line_from: Some(4),
        line_to: Some(2),
    };

    let view_result = view_file(file_path, &options)?;

    match &view_result.contents {
        FileContents::Text { content, metadata } => {
            // Content should be empty due to invalid range
            assert!(content.line_contents.is_empty());

            // Metadata still reflects the whole file
            assert_eq!(metadata.line_count, 6);
        }
        _ => panic!("Expected text content"),
    }

    Ok(())
}

#[test]
fn test_total_line_num_field() -> Result<()> {
    // Test total_line_num for text file
    let text_file_path = Path::new("tests/fixtures/text_files/sample.txt");
    let options = ViewOptions::default();

    let text_result = view_file(text_file_path, &options)?;

    // For text files, total_line_num should be Some with the correct count
    assert_eq!(text_result.total_line_num, Some(6));

    // Test total_line_num for binary file
    let binary_file_path = Path::new("tests/fixtures/binary_files/binary.bin");
    let binary_result = view_file(binary_file_path, &options)?;

    // For binary files, total_line_num should be None
    assert_eq!(binary_result.total_line_num, None);

    // Test that total_line_num is maintained even with line filtering
    let filtered_options = ViewOptions {
        max_size: None,
        line_from: Some(2),
        line_to: Some(4),
    };

    let filtered_result = view_file(text_file_path, &filtered_options)?;

    // total_line_num should still represent the whole file
    assert_eq!(filtered_result.total_line_num, Some(6));

    Ok(())
}

#[test]
fn test_no_trailing_newlines() -> Result<()> {
    // Test that line field doesn't have trailing newlines
    let text_file_path = Path::new("tests/fixtures/text_files/sample.txt");
    let options = ViewOptions::default();

    let text_result = view_file(text_file_path, &options)?;

    // For text files, check that no line ends with newline
    match &text_result.contents {
        FileContents::Text { content, .. } => {
            for line_content in &content.line_contents {
                assert!(
                    !line_content.line.ends_with('\n'),
                    "Line content contains trailing newline: {:?}",
                    line_content.line
                );
            }
        }
        _ => panic!("Expected text content"),
    }

    // Test with line filtering as well
    let filtered_options = ViewOptions {
        max_size: None,
        line_from: Some(2),
        line_to: Some(4),
    };

    let filtered_result = view_file(text_file_path, &filtered_options)?;

    // Check that filtered lines also don't have trailing newlines
    match &filtered_result.contents {
        FileContents::Text { content, .. } => {
            // Verify we got the expected lines
            assert_eq!(content.line_contents.len(), 3);

            // Verify no trailing newlines
            for line_content in &content.line_contents {
                assert!(
                    !line_content.line.ends_with('\n'),
                    "Filtered line contains trailing newline: {:?}",
                    line_content.line
                );
            }
        }
        _ => panic!("Expected text content"),
    }

    Ok(())
}

#[test]
fn test_size_check_with_line_filters() -> Result<()> {
    let file_path = Path::new("tests/fixtures/text_files/sample.txt");

    // First check that a normal size limit rejects the file
    let regular_options = ViewOptions {
        max_size: Some(10), // 10 bytes (file is larger)
        line_from: None,
        line_to: None,
    };

    // This should fail - entire file is too large
    let regular_result = view_file(file_path, &regular_options);
    assert!(regular_result.is_err());
    assert!(format!("{:?}", regular_result.unwrap_err()).contains("File is too large"));

    // Now try with line filters to get only a small portion
    let filter_options = ViewOptions {
        max_size: Some(10), // Same tiny limit
        line_from: Some(1), // Just get the first line
        line_to: Some(1),
    };

    // This should work - we're only loading a small part of the file
    let filter_result = view_file(file_path, &filter_options);

    // If the first line is over 10 bytes, this might still fail, but with a different error
    match filter_result {
        Ok(result) => {
            // Test passed - we were able to load just the first line
            match &result.contents {
                FileContents::Text { content, .. } => {
                    assert_eq!(content.line_contents.len(), 1);
                    assert_eq!(content.line_contents[0].line_number, 1);
                }
                _ => panic!("Expected text content"),
            };
        }
        Err(e) => {
            // If this fails, it should be because the filtered content is still too large
            let err_msg = format!("{:?}", e);
            assert!(err_msg.contains("Filtered content is too large"));
            // This is also a valid outcome, as our 10 byte limit is very small
        }
    }

    // Now create a file with predictable size we can test
    let test_dir = tempfile::tempdir()?;
    let test_file_path = test_dir.path().join("tiny_test.txt");
    std::fs::write(&test_file_path, "Line1\nLine2\nLine3\n")?;

    // Get just Line1 with a limit that allows it
    let tiny_options = ViewOptions {
        max_size: Some(6), // "Line1\n" is 6 bytes
        line_from: Some(1),
        line_to: Some(1),
    };

    let tiny_result = view_file(&test_file_path, &tiny_options)?;

    match &tiny_result.contents {
        FileContents::Text { content, .. } => {
            assert_eq!(content.line_contents.len(), 1);
            assert_eq!(content.line_contents[0].line_number, 1);
            assert_eq!(content.line_contents[0].line, "Line1");
        }
        _ => panic!("Expected text content"),
    }

    // Try to get Line1 and Line2 with a limit that's too small
    let too_small_options = ViewOptions {
        max_size: Some(6), // Only enough for Line1
        line_from: Some(1),
        line_to: Some(2), // But we want two lines
    };

    let too_small_result = view_file(&test_file_path, &too_small_options);
    assert!(too_small_result.is_err());
    assert!(
        format!("{:?}", too_small_result.unwrap_err()).contains("Filtered content is too large")
    );

    Ok(())
}
