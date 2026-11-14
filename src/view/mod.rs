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

    /// Starting line number to include in text file content (1-based, inclusive).
    /// Only applied for text files. If None, starts from the first line.
    /// If the specified line is beyond the file's content, an empty result will be returned.
    pub line_from: Option<usize>,

    /// Ending line number to include in text file content (1-based, inclusive).
    /// Only applied for text files. If None, includes until the last line.
    /// If the specified line is beyond the file's content, only available lines up to the end will be included.
    pub line_to: Option<usize>,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            max_size: Some(10 * 1024 * 1024), // Default to 10MB limit
            line_from: None,
            line_to: None,
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
        content: TextContent,
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

/// Text content with line-by-line structure.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextContent {
    /// Collection of individual lines with their content
    pub line_contents: Vec<LineContent>,
}

impl TextContent {
    /// Check if the content contains the given string
    pub fn contains(&self, s: &str) -> bool {
        self.line_contents.iter().any(|line| line.line.contains(s))
    }

    /// Check if the content is empty
    pub fn is_empty(&self) -> bool {
        self.line_contents.is_empty()
    }

    /// Convert the content to lowercase
    pub fn to_lowercase(&self) -> String {
        self.line_contents
            .iter()
            .map(|line| line.line.to_lowercase())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Convert the content to a string
    pub fn to_string(&self) -> String {
        self.line_contents
            .iter()
            .map(|line| line.line.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Represents a single line in a text file.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LineContent {
    /// Line number (1-based index)
    pub line_number: usize,
    /// The content of the line without trailing newlines
    pub line: String,
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
    /// Total number of lines in the file, only present for text files
    pub total_line_num: Option<usize>,
}

/// Reads and processes a file, detecting its type and returning an appropriate representation.
/// For text files, can optionally filter to include only specific line ranges.
///
/// # Arguments
///
/// * `path` - Path to the file to view
/// * `options` - Configuration options for the viewing operation, including:
///   - `max_size`: Optional maximum file size limit
///   - `line_from`: Optional starting line number (1-based, inclusive)
///   - `line_to`: Optional ending line number (1-based, inclusive)
///
/// # Returns
///
/// A FileView struct containing the file path, detected type, contents with metadata, and
/// total line count information (for text files only).
/// For text files, the content is structured as a collection of lines with line numbers.
///
/// When line filtering is applied:
/// - Only lines within the specified range (inclusive) are included
/// - Size checking is optimized to check only the filtered content size, not the entire file
/// - If the range is out of bounds, no error is returned:
///   - If `line_from` is beyond the file size, an empty content list is returned
///   - If `line_to` exceeds the file size, only available lines are included
///   - If `line_from` > `line_to`, an empty content list is returned
/// - Metadata still represents the whole file regardless of filtering
/// - The `total_line_num` field provides the total number of lines in the original file
///
/// # Errors
///
/// Returns an error if:
/// - The file does not exist or is not a regular file
/// - The file is larger than the maximum size specified in options (when not using line filters)
/// - The filtered content is larger than the maximum size (when using line filters)
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

    // Check file size if a limit is set and no line filters are applied
    // When line filters are applied, we'll only process a subset of the file,
    // so we skip the initial size check and validate the filtered content size later
    let using_line_filters = options.line_from.is_some() || options.line_to.is_some();

    if let Some(max_size) = options.max_size {
        if !using_line_filters && metadata.len() > max_size as u64 {
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

    // We'll handle size checks for each file type separately when line filters are applied

    // Process contents based on file type
    let contents = if file_type.starts_with("text/") {
        // Handle text files
        match String::from_utf8(content.clone()) {
            Ok(text) => {
                // Count lines for information
                let all_lines: Vec<&str> = text.lines().collect();
                let line_count = all_lines.len();
                let char_count = text.chars().count();

                // Apply line filtering if requested, silently adjusting for boundaries
                let from_line = options.line_from.unwrap_or(1).max(1);
                let to_line = options.line_to.unwrap_or(line_count).min(line_count);

                // If from_line is beyond file content or greater than to_line, adjust silently
                let (effective_from, effective_to) =
                    if from_line > line_count || from_line > to_line {
                        // If range is completely invalid, return empty content
                        (1, 0) // This will create an empty collection as from > to
                    } else {
                        (from_line, to_line)
                    };

                // Create line contents with line numbers and filtered text
                let line_contents = all_lines
                    .iter()
                    .enumerate()
                    .filter(|(idx, _)| {
                        let line_num = idx + 1; // Convert to 1-based index
                        line_num >= effective_from && line_num <= effective_to
                    })
                    .map(|(idx, line)| LineContent {
                        line_number: idx + 1, // Convert to 1-based index
                        line: line.to_string().trim_end_matches('\n').to_string(),
                    })
                    .collect();

                // Create structured text content
                let content = TextContent { line_contents };

                // If we're using line filters and have a max size, check the filtered content size
                if using_line_filters && options.max_size.is_some() {
                    let max_size = options.max_size.unwrap();
                    // Estimate the size of filtered content by summing up the lengths of included lines
                    // Also account for newline characters (\n) that would be present when reconstructing the content
                    let filtered_size = content
                        .line_contents
                        .iter()
                        .map(|line| line.line.len() + 1) // +1 for the newline character
                        .sum::<usize>();

                    if filtered_size > max_size {
                        return Err(anyhow!(
                            "Filtered content is too large: {} (filtered size: {}, limit: {})",
                            path.display(),
                            filtered_size,
                            max_size
                        ));
                    }
                }

                FileContents::Text {
                    content,
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
        // If using line filters and we have a max size, check file size (since we skipped initial check)
        if using_line_filters && options.max_size.is_some() {
            let max_size = options.max_size.unwrap();
            if metadata.len() > max_size as u64 {
                return Err(anyhow!(
                    "Image file is too large when using line filters: {} (size: {}, limit: {})",
                    path.display(),
                    metadata.len(),
                    max_size
                ));
            }
        }

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
        // If using line filters and we have a max size, check file size (since we skipped initial check)
        if using_line_filters && options.max_size.is_some() {
            let max_size = options.max_size.unwrap();
            if metadata.len() > max_size as u64 {
                return Err(anyhow!(
                    "Binary file is too large when using line filters: {} (size: {}, limit: {})",
                    path.display(),
                    metadata.len(),
                    max_size
                ));
            }
        }

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

    // Set total_line_num based on file content type
    let total_line_num = match &contents {
        FileContents::Text { metadata, .. } => Some(metadata.line_count),
        _ => None,
    };

    let result = FileView {
        file_path: path.to_path_buf(),
        file_type,
        contents,
        total_line_num,
    };

    Ok(result)
}
