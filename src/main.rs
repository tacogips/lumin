use anyhow::Result;
use clap::{Parser, Subcommand};
use lumin::search::{SearchOptions, search_files};
use lumin::traverse::{TraverseOptions, traverse_directory};
use lumin::tree::{TreeOptions, generate_tree};
use lumin::view::{ViewOptions, view_file};
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
    },

    /// View file contents
    View {
        /// File to view
        file: PathBuf,

        /// Maximum file size in bytes
        #[arg(long)]
        max_size: Option<usize>,
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
        } => {
            let options = SearchOptions {
                case_sensitive: *case_sensitive,
                respect_gitignore: !no_ignore,
            };

            let results = search_files(pattern, directory, &options)?;

            if results.is_empty() {
                println!("No matches found.");
            } else {
                println!("Found {} matches:", results.len());
                for result in results {
                    println!(
                        "{}:{}: {}",
                        result.file_path.display(),
                        result.line_number,
                        result.line_content.trim()
                    );
                }
            }
        }

        Commands::Traverse {
            directory,
            pattern,
            case_sensitive,
            no_ignore,
            include_binary,
        } => {
            let options = TraverseOptions {
                case_sensitive: *case_sensitive,
                respect_gitignore: !no_ignore,
                only_text_files: !include_binary,
                pattern: pattern.clone(),
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
        } => {
            let options = TreeOptions {
                case_sensitive: *case_sensitive,
                respect_gitignore: !no_ignore,
            };

            let results = generate_tree(directory, &options)?;

            if results.is_empty() {
                println!("No directories found.");
            } else {
                // Output as JSON
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
        }

        Commands::View { file, max_size } => {
            let options = ViewOptions {
                max_size: *max_size,
            };

            let json_output = view_file(file, &options)?;
            println!("{}", serde_json::to_string_pretty(&json_output)?);
        }
    }

    Ok(())
}
