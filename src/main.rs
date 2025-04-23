use clap::{Parser, Subcommand};
use file_searcher::search::{SearchOptions, search_files};
use file_searcher::traverse::{TraverseOptions, traverse_directory};
use file_searcher::view::{ViewOptions, view_file};
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

    /// View file contents
    View {
        /// File to view
        file: PathBuf,

        /// Maximum file size in bytes
        #[arg(long)]
        max_size: Option<usize>,
    },
}

fn main() {
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

            match search_files(pattern, directory, &options) {
                Ok(results) => {
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
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }

        Commands::Traverse {
            directory,
            case_sensitive,
            no_ignore,
            include_binary,
        } => {
            let options = TraverseOptions {
                case_sensitive: *case_sensitive,
                respect_gitignore: !no_ignore,
                only_text_files: !include_binary,
            };

            match traverse_directory(directory, &options) {
                Ok(results) => {
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
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }

        Commands::View { file, max_size } => {
            let options = ViewOptions {
                max_size: *max_size,
            };

            match view_file(file, &options) {
                Ok(json_output) => {
                    println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}
