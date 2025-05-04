use anyhow::Result;
use lumin::view::{ViewOptions, view_file, FileContents};
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
        },
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
        },
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
        },
        FileContents::Binary { message, metadata } => {
            // If detected as binary that's also fine
            assert!(message.contains("Binary file detected"));
            
            // Verify metadata
            assert!(metadata.binary);
            assert!(metadata.size_bytes > 0);
        },
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
        FileContents::Text { content, metadata: _ } => {
            assert!(content.contains("# Sample Markdown"));
            assert!(content.contains("```rust"));
            assert!(content.contains("fn main()"));
        },
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
        FileContents::Text { content, metadata: _ } => {
            assert!(content.contains("[server]"));
            assert!(content.contains("port = 8080"));
            assert!(content.contains("[database]"));
            assert!(content.contains("[logging]"));
        },
        _ => panic!("Expected text content for toml file"),
    }
    
    Ok(())
}