use anyhow::{Context, Result, anyhow};
use infer::Infer;
use serde_json::{Value, json};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct ViewOptions {
    pub max_size: Option<usize>,
}

impl Default for ViewOptions {
    fn default() -> Self {
        Self {
            max_size: Some(10 * 1024 * 1024), // Default to 10MB limit
        }
    }
}
// Construct the result as a struct, then serialize to JSON
#[derive(serde::Serialize, Debug)]
pub struct FileView {
    pub file_path: std::path::PathBuf,
    pub file_type: String,
    pub contents: Value,
}

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

    // Infer file type
    let infer = Infer::new();
    let file_type = match infer.get_from_path(path) {
        Ok(Some(kind)) => kind.mime_type().to_string(),
        Ok(None) => "text/plain".to_string(), // Default to text if can't determine
        Err(e) => return Err(anyhow!("Failed to determine file type: {}", e)),
    };

    // Read file content
    let mut file =
        File::open(path).with_context(|| format!("Failed to open file {}", path.display()))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .with_context(|| format!("Failed to read file {}", path.display()))?;

    // For binary files, return base64 encoding, for text files return as string
    let contents = if file_type.starts_with("text/") {
        match String::from_utf8(content) {
            Ok(text) => json!(text),
            Err(_) => json!(format!(
                "Binary file detected, size: {} bytes",
                metadata.len()
            )),
        }
    } else {
        // For binary files, we could encode as base64, but for simplicity we'll just return a message
        json!(format!(
            "Binary file detected, size: {} bytes",
            metadata.len()
        ))
    };

    let result = FileView {
        file_path: path.to_path_buf(),
        file_type,
        contents,
    };

    Ok(result)
}
