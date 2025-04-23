use anyhow::{Context, Result};
use grep::regex::RegexMatcher;
use grep::searcher::sinks::UTF8;
use grep::searcher::{BinaryDetection, SearcherBuilder};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::path::{Path, PathBuf};

pub struct SearchOptions {
    pub case_sensitive: bool,
    pub respect_gitignore: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            respect_gitignore: true,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SearchResult {
    pub file_path: PathBuf,
    pub line_number: u64,
    pub line_content: String,
}

pub fn search_files(
    pattern: &str,
    directory: &Path,
    options: &SearchOptions,
) -> Result<Vec<SearchResult>> {
    // Create the matcher with the appropriate case sensitivity
    let matcher = if options.case_sensitive {
        RegexMatcher::new(pattern)
    } else {
        // For case insensitive search, we add the case-insensitive flag to the regex
        RegexMatcher::new(&format!("(?i){}", pattern))
    }
    .context("Failed to create regular expression matcher")?;

    // Build the list of files to search
    let files =
        collect_files(directory, options).context("Failed to collect files for searching")?;

    let mut results = Vec::new();

    // Set up the searcher
    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .build();

    // Search each file
    for file_path in files {
        let file = match File::open(&file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open file {}: {}", file_path.display(), e);
                continue;
            }
        };

        // Create a sink that collects the results
        let mut matches = Vec::new();
        searcher
            .search_file(
                &matcher,
                &file,
                UTF8(|line_number, line| {
                    matches.push((line_number, line.to_string()));
                    Ok(true)
                }),
            )
            .with_context(|| format!("Error searching file {}", file_path.display()))?;

        // Process the matches
        for (line_number, content) in matches {
            results.push(SearchResult {
                file_path: file_path.clone(),
                line_number,
                line_content: content,
            });
        }
    }

    Ok(results)
}

fn collect_files(directory: &Path, options: &SearchOptions) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    let mut builder = WalkBuilder::new(directory);
    builder.git_ignore(options.respect_gitignore);
    // When not respecting gitignore, explicitly include hidden files and dirs
    builder.hidden(options.respect_gitignore);
    // Additional settings to ensure we fully respect/ignore gitignore as needed
    if !options.respect_gitignore {
        builder.ignore(false); // Turn off all ignore logic
        builder.git_exclude(false); // Don't use git exclude files
        builder.git_global(false); // Don't use global git ignore
    }

    for result in builder.build() {
        match result {
            Ok(entry) => {
                let path = entry.path();
                if path.is_file() {
                    files.push(path.to_path_buf());
                }
            }
            Err(err) => {
                eprintln!("Error walking directory: {}", err);
            }
        }
    }

    Ok(files)
}
