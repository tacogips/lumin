use anyhow::Result;
use ignore::WalkBuilder;
use infer::Infer;
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraverseResult {
    pub file_path: PathBuf,
    pub file_type: String,
}

impl TraverseResult {
    pub fn is_hidden(&self) -> bool {
        // Check if the file name starts with a dot
        let file_is_hidden = self
            .file_path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|name| name.starts_with("."));

        // Also check if the file is in a hidden directory
        let path_contains_hidden_dir = self
            .file_path
            .to_string_lossy()
            .split('/')
            .any(|part| part.starts_with(".") && !part.is_empty());

        file_is_hidden || path_contains_hidden_dir
    }
}

pub fn traverse_directory(
    directory: &Path,
    options: &TraverseOptions,
) -> Result<Vec<TraverseResult>> {
    let mut results = Vec::new();
    let infer = Infer::new();

    // Configure the file traversal
    let mut builder = WalkBuilder::new(directory);
    builder.git_ignore(options.respect_gitignore);
    // When respecting gitignore, hidden files are skipped; otherwise they're included
    builder.hidden(options.respect_gitignore);
    if !options.case_sensitive {
        builder.ignore_case_insensitive(true);
    }
    // Additional settings to ensure we fully respect/ignore gitignore as needed
    if !options.respect_gitignore {
        builder.ignore(false); // Turn off all ignore logic
        builder.git_exclude(false); // Don't use git exclude files
        builder.git_global(false); // Don't use global git ignore
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
