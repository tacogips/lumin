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

pub fn view_file(path: &Path, options: &ViewOptions) -> Result<Value, String> {
    // Check if file exists and is a file
    if !path.exists() {
        return Err(format!("File not found: {}", path.display()));
    }

    if !path.is_file() {
        return Err(format!("Not a file: {}", path.display()));
    }

    // Get file metadata
    let metadata = path
        .metadata()
        .map_err(|e| format!("Failed to read file metadata: {}", e))?;

    // Check file size if a limit is set
    if let Some(max_size) = options.max_size {
        if metadata.len() > max_size as u64 {
            return Err(format!(
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
        Err(e) => return Err(format!("Failed to determine file type: {}", e)),
    };

    // Read file content
    let mut file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;

    let mut content = Vec::new();
    file.read_to_end(&mut content)
        .map_err(|e| format!("Failed to read file: {}", e))?;

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

    // Construct the result as JSON
    let result = json!({
        "file_path": path.to_string_lossy(),
        "file_type": file_type,
        "contents": contents
    });

    Ok(result)
}
