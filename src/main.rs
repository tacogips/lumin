use anyhow::Result;
use clap::{Parser, Subcommand};
use lumin::search::{SearchOptions, search_files};
use lumin::traverse::{TraverseOptions, traverse_directory};
use lumin::tree::{TreeOptions, generate_tree};
use lumin::view::{FileContents, ViewOptions, view_file};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    author,
    version,
    about = "A utility for searching and traversing files"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Search for patterns in files
    Search {
        /// Pattern to search for
        pattern: String,

        /// Directory to search in
        directory: PathBuf,

        /// Case sensitive search
        #[arg(long)]
        case_sensitive: bool,

        /// Ignore gitignore files
        #[arg(long)]
        no_ignore: bool,

        /// Maximum directory traversal depth (0 for unlimited)
        #[arg(long = "max-depth", default_value = "20")]
        max_depth: usize,

        /// Limit context around matches (number of characters before and after)
        /// While context is limited, the full matched pattern is always preserved
        #[arg(long)]
        omit_context: Option<usize>,

        /// Number of lines to show before each match (similar to grep's -B option)
        #[arg(short = 'B', long = "before-context", default_value = "0")]
        before_context: usize,

        /// Number of lines to show after each match (similar to grep's -A option)
        #[arg(short = 'A', long = "after-context", default_value = "0")]
        after_context: usize,
    },

    /// Traverse directories and list files
    Traverse {
        /// Directory to traverse
        directory: PathBuf,

        /// Pattern to filter files (optional)
        pattern: Option<String>,

        /// Case sensitive matching
        #[arg(long)]
        case_sensitive: bool,

        /// Ignore gitignore files
        #[arg(long)]
        no_ignore: bool,

        /// Include binary files
        #[arg(long)]
        include_binary: bool,

        /// Maximum directory traversal depth (0 for unlimited)
        #[arg(long = "max-depth", default_value = "20")]
        max_depth: usize,
    },

    /// Display directory structure as a tree
    Tree {
        /// Directory to display as tree
        directory: PathBuf,

        /// Case sensitive matching
        #[arg(long)]
        case_sensitive: bool,

        /// Ignore gitignore files
        #[arg(long)]
        no_ignore: bool,

        /// Maximum directory traversal depth (0 for unlimited)
        #[arg(long = "max-depth", default_value = "20")]
        max_depth: usize,
    },

    /// View file contents
    View {
        /// File to view
        file: PathBuf,

        /// Maximum file size in bytes
        #[arg(long)]
        max_size: Option<usize>,
        
        /// Start viewing from this line number (1-based, inclusive)
        #[arg(long)]
        line_from: Option<usize>,
        
        /// End viewing at this line number (1-based, inclusive)
        #[arg(long)]
        line_to: Option<usize>,
    },
}

fn main() -> Result<()> {
    // Initialize structured logging
    lumin::telemetry::init()?;
    let cli = Cli::parse();

    match &cli.command {
        Commands::Search {
            pattern,
            directory,
            case_sensitive,
            no_ignore,
            omit_context,
            before_context,
            after_context,
            max_depth,
        } => {
            let options = SearchOptions {
                case_sensitive: *case_sensitive,
                respect_gitignore: !no_ignore,
                exclude_glob: None,
                include_glob: None,
                omit_path_prefix: None,
                match_content_omit_num: *omit_context,
                depth: if *max_depth == 0 { None } else { Some(*max_depth) },
                before_context: *before_context,
                after_context: *after_context,
                skip: None,
                take: None,
            };

            let results = search_files(pattern, directory, &options)?;

            if results.lines.is_empty() {
                println!("No matches found.");
            } else {
                // Count actual matches (not context lines)
                let match_count = results.lines.iter().filter(|r| !r.is_context).count();
                println!("Found {} matches:", match_count);
                
                let mut last_file = None;
                let mut last_line_number = 0;
                
                for result in results.lines {
                    // Print separator between discontinuous results
                    if let Some(last) = &last_file {
                        if &result.file_path != last || result.line_number > last_line_number + 1 {
                            println!("--");
                        }
                    }
                    
                    // Update tracking variables
                    last_file = Some(result.file_path.clone());
                    last_line_number = result.line_number;
                    
                    // Print result with different formatting for matches vs context
                    if result.is_context {
                        // Context line (grey/dimmed if terminal supports it)
                        println!(
                            "{}:{}- {}",
                            result.file_path.display(),
                            result.line_number,
                            result.line_content.trim()
                        );
                    } else {
                        // Matched line (regular text)
                        println!(
                            "{}:{}: {}",
                            result.file_path.display(),
                            result.line_number,
                            result.line_content.trim()
                        );
                    }
                }
            }
        }

        Commands::Traverse {
            directory,
            pattern,
            case_sensitive,
            no_ignore,
            include_binary,
            max_depth,
        } => {
            let options = TraverseOptions {
                case_sensitive: *case_sensitive,
                respect_gitignore: !no_ignore,
                only_text_files: !include_binary,
                pattern: pattern.clone(),
                depth: if *max_depth == 0 { None } else { Some(*max_depth) },
                omit_path_prefix: None,
            };

            let results = traverse_directory(directory, &options)?;

            if results.is_empty() {
                println!("No files found.");
            } else {
                println!("Found {} files:", results.len());
                for result in results {
                    let hidden_marker = if result.is_hidden() { "*" } else { " " };
                    println!(
                        "{} {:<10} {}",
                        hidden_marker,
                        result.file_type,
                        result.file_path.display()
                    );
                }
            }
        }

        Commands::Tree {
            directory,
            case_sensitive,
            no_ignore,
            max_depth,
        } => {
            let options = TreeOptions {
                case_sensitive: *case_sensitive,
                respect_gitignore: !no_ignore,
                depth: if *max_depth == 0 { None } else { Some(*max_depth) },
                omit_path_prefix: None,
            };

            let results = generate_tree(directory, &options)?;

            if results.is_empty() {
                println!("No directories found.");
            } else {
                // Output as JSON
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
        }

        Commands::View { file, max_size, line_from, line_to } => {
            let options = ViewOptions {
                max_size: *max_size,
                line_from: *line_from,
                line_to: *line_to,
            };

            let view_result = view_file(file, &options)?;
            
            // Format output as {filepath}:{line_num}:{line_contents}
            match view_result.contents {
                FileContents::Text { content, .. } => {
                    let file_path = view_result.file_path.to_string_lossy();
                    for line_content in content.line_contents {
                        println!("{file_path}:{}:{}", line_content.line_number, line_content.line);
                    }
                },
                FileContents::Binary { message, .. } => {
                    println!("{}: {}", view_result.file_path.to_string_lossy(), message);
                },
                FileContents::Image { message, .. } => {
                    println!("{}: {}", view_result.file_path.to_string_lossy(), message);
                },
            }
        }
    }

    Ok(())
}
