use ignore::WalkBuilder;
use infer::Infer;
use std::path::{Path, PathBuf};

pub struct TraverseOptions {
    pub case_sensitive: bool,
    pub respect_gitignore: bool,
    pub only_text_files: bool,
}

impl Default for TraverseOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
            only_text_files: true,
        }
    }
}

pub struct TraverseResult {
    pub file_path: PathBuf,
    pub file_type: String,
    pub is_hidden: bool,
}

pub fn traverse_directory(
    directory: &Path,
    options: &TraverseOptions,
) -> Result<Vec<TraverseResult>, String> {
    let mut results = Vec::new();
    let infer = Infer::new();

    // Configure the file traversal
    let mut builder = WalkBuilder::new(directory);
    builder.git_ignore(options.respect_gitignore);
    builder.hidden(!options.respect_gitignore);
    if !options.case_sensitive {
        builder.ignore_case_insensitive(true);
    }

    // Walk the directory
    for result in builder.build() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    // Check if we should include this file
                    let include = if options.only_text_files {
                        // Read a small amount of the file to determine its type
                        match std::fs::read(path) {
                            Ok(_) => {
                                // If infer can determine a type, it's probably not a text file
                                match infer.get_from_path(path) {
                                    Ok(Some(kind)) => kind.mime_type().starts_with("text/"),
                                    Ok(None) => true, // Consider as text if infer couldn't determine a type
                                    Err(_) => false,  // Skip files with errors
                                }
                            }
                            Err(_) => false, // Skip files we can't read
                        }
                    } else {
                        true
                    };

                    if include {
                        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                        let is_hidden = file_name.starts_with(".");

                        // Get file type (simplified)
                        let file_type = if let Some(ext) = path.extension().and_then(|e| e.to_str())
                        {
                            ext.to_lowercase()
                        } else {
                            "unknown".to_string()
                        };

                        results.push(TraverseResult {
                            file_path: path.to_path_buf(),
                            file_type,
                            is_hidden,
                        });
                    }
                }
            }
            Err(err) => {
                eprintln!("Error walking directory: {}", err);
            }
        }
    }

    // Sort results by path
    results.sort_by(|a, b| a.file_path.cmp(&b.file_path));

    Ok(results)
}
