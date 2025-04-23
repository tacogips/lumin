//! File content viewing with type detection and formatting.
//!
//! This module provides tools to view file contents with automatic type detection,
//! handling different file types (text, binary, image) appropriately with metadata.

use anyhow::{Context, Result, anyhow};
use infer::Infer;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Configuration options for file viewing operations.
pub struct ViewOptions {
    /// Maximum file size to read in bytes.
    /// Files larger than this will be rejected to prevent excessive memory usage.
    /// A value of None means no limit.
    pub max_size: Option<usize>,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            max_size: Some(10 * 1024 * 1024), // Default to 10MB limit
        }
    }
}

/// Represents the contents of a file with type-specific information.
///
/// This enum has different variants based on the detected file type:
/// - `Text` for text files with content and metadata
/// - `Binary` for binary files with a description message and metadata
/// - `Image` for image files with a description message and metadata
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum FileContents {
    /// Text file contents with the actual content and metadata
    #[serde(rename = "text")]
    Text {
        /// The actual text content of the file
        content: String,
        /// Metadata about the text content
        metadata: TextMetadata,
    },
    
    /// Binary file representation with a descriptive message
    #[serde(rename = "binary")]
    Binary {
        /// A descriptive message about the binary file
        message: String,
        /// Metadata about the binary file
        metadata: BinaryMetadata,
    },
    
    /// Image file representation with a descriptive message
    #[serde(rename = "image")]
    Image {
        /// A descriptive message about the image file
        message: String,
        /// Metadata about the image file
        metadata: ImageMetadata,
    },
}

/// Metadata for text files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMetadata {
    /// Number of lines in the text file
    pub line_count: usize,
    /// Number of characters in the text file
    pub char_count: usize,
}

/// Metadata for binary files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BinaryMetadata {
    /// Whether the file is binary (always true for this struct)
    pub binary: bool,
    /// Size of the file in bytes
    pub size_bytes: u64,
    /// MIME type of the file, if it could be determined
    pub mime_type: Option<String>,
}

/// Metadata for image files.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageMetadata {
    /// Whether the file is binary (always true for images)
    pub binary: bool,
    /// Size of the image file in bytes
    pub size_bytes: u64,
    /// Media type descriptor (typically "image")
    pub media_type: String,
}

/// Main result structure for file viewing, containing the file path, type, and contents.
#[derive(Serialize, Debug)]
pub struct FileView {
    /// Path to the viewed file
    pub file_path: PathBuf,
    /// MIME type or file type descriptor
    pub file_type: String,
    /// The contents of the file, represented as an appropriate variant of FileContents
    pub contents: FileContents,
}

/// Reads and processes a file, detecting its type and returning an appropriate representation.
///
/// # Arguments
///
/// * `path` - Path to the file to view
/// * `options` - Configuration options for the viewing operation
///
/// # Returns
///
/// A FileView struct containing the file path, detected type, and contents with metadata
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist or is not a regular file
/// - The file is larger than the maximum size specified in options
/// - Failed to read file metadata or content
/// - Failed to determine the file type
pub fn view_file(path: &Path, options: &ViewOptions) -> Result<FileView> {
    // Check if file exists and is a file
    if !path.exists() {
        return Err(anyhow!("File not found: {}", path.display()));
    }

    if !path.is_file() {
        return Err(anyhow!("Not a file: {}", path.display()));
    }

    // Get file metadata
    let metadata = path
        .metadata()
        .with_context(|| format!("Failed to read file metadata for {}", path.display()))?;

    // Check file size if a limit is set
    if let Some(max_size) = options.max_size {
        if metadata.len() > max_size as u64 {
            return Err(anyhow!(
                "File is too large: {} (size: {}, limit: {})",
                path.display(),
                metadata.len(),
                max_size
            ));
        }
    }

    // Infer file type using both extension and content analysis
    let infer = Infer::new();

    // First try to get a type hint from the extension
    let extension_type = path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| match ext.to_lowercase().as_str() {
            "txt" | "md" | "rs" | "toml" | "yml" | "yaml" | "json" => Some("text/plain"),
            "py" => Some("text/x-python"),
            "js" => Some("text/javascript"),
            "html" => Some("text/html"),
            "css" => Some("text/css"),
            _ => None,
        })
        .unwrap_or(None);

    // Then try content-based detection
    let file_type = match infer.get_from_path(path) {
        Ok(Some(kind)) => kind.mime_type().to_string(),
        Ok(None) => {
            // If infer couldn't determine type but we have an extension hint, use that
            if let Some(ext_type) = extension_type {
                ext_type.to_string()
            } else {
                // Read a small sample to check if it's probably text
                match std::fs::read(path) {
                    Ok(bytes) if bytes.len() <= 1024 => {
                        // Check if the content looks like text (mostly ASCII or UTF-8)
                        let text_likelihood = bytes
                            .iter()
                            .filter(|b| {
                                **b >= 32 && **b <= 126
                                    || **b == b'\n'
                                    || **b == b'\r'
                                    || **b == b'\t'
                            })
                            .count() as f64
                            / bytes.len() as f64;

                        if text_likelihood > 0.8 {
                            "text/plain".to_string()
                        } else {
                            "application/octet-stream".to_string()
                        }
                    }
                    _ => "application/octet-stream".to_string(), // Default to binary for larger files or errors
                }
            }
        }
        Err(e) => return Err(anyhow!("Failed to determine file type: {}", e)),
    };

    // Read file content
    let mut file =
        File::open(path).with_context(|| format!("Failed to open file {}", path.display()))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .with_context(|| format!("Failed to read file {}", path.display()))?;

    // Process contents based on file type
    let contents = if file_type.starts_with("text/") {
        // Handle text files
        match String::from_utf8(content.clone()) {
            Ok(text) => {
                // Count lines for information
                let line_count = text.lines().count();
                let char_count = text.chars().count();

                // Create structured text content
                FileContents::Text {
                    content: text,
                    metadata: TextMetadata {
                        line_count,
                        char_count,
                    },
                }
            }
            Err(_) => {
                // Text detection was wrong, it's actually binary
                FileContents::Binary {
                    message: format!("Binary file detected, size: {} bytes", metadata.len()),
                    metadata: BinaryMetadata {
                        binary: true,
                        size_bytes: metadata.len(),
                        mime_type: None,
                    },
                }
            }
        }
    } else if file_type.starts_with("image/") {
        // Special handling for images
        FileContents::Image {
            message: format!("Image file detected: {}", file_type),
            metadata: ImageMetadata {
                binary: true,
                size_bytes: metadata.len(),
                media_type: "image".to_string(),
            },
        }
    } else {
        // For other binary files
        FileContents::Binary {
            message: format!(
                "Binary file detected, size: {} bytes, type: {}",
                metadata.len(),
                file_type
            ),
            metadata: BinaryMetadata {
                binary: true,
                size_bytes: metadata.len(),
                mime_type: Some(file_type.clone()),
            },
        }
    };

    let result = FileView {
        file_path: path.to_path_buf(),
        file_type,
        contents,
    };

    Ok(result)
}
