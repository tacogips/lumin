use file_searcher::search::{SearchOptions, search_files};
use file_searcher::traverse::{TraverseOptions, traverse_directory};
use file_searcher::view::{ViewOptions, view_file};
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "search" => handle_search(&args),
        "traverse" => handle_traverse(&args),
        "view" => handle_view(&args),
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage(&args[0]);
            std::process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage:");
    eprintln!(
        "  {} search <pattern> <directory> [--case-sensitive] [--no-gitignore]",
        program
    );
    eprintln!(
        "  {} traverse <directory> [--case-sensitive] [--no-gitignore] [--include-binary]",
        program
    );
    eprintln!("  {} view <file>", program);
}

fn handle_search(args: &[String]) {
    if args.len() < 4 {
        eprintln!(
            "Usage: {} search <pattern> <directory> [--case-sensitive] [--no-gitignore]",
            args[0]
        );
        std::process::exit(1);
    }

    let pattern = &args[2];
    let directory = &args[3];

    let mut options = SearchOptions::default();
    for arg in args.iter().skip(4) {
        match arg.as_str() {
            "--case-sensitive" => options.case_sensitive = true,
            "--no-gitignore" => options.respect_gitignore = false,
            _ => {
                eprintln!("Unknown option: {}", arg);
                std::process::exit(1);
            }
        }
    }

    match search_files(pattern, Path::new(directory), &options) {
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

fn handle_traverse(args: &[String]) {
    if args.len() < 3 {
        eprintln!(
            "Usage: {} traverse <directory> [--case-sensitive] [--no-gitignore] [--include-binary]",
            args[0]
        );
        std::process::exit(1);
    }

    let directory = &args[2];

    let mut options = TraverseOptions::default();
    for arg in args.iter().skip(3) {
        match arg.as_str() {
            "--case-sensitive" => options.case_sensitive = true,
            "--no-gitignore" => options.respect_gitignore = false,
            "--include-binary" => options.only_text_files = false,
            _ => {
                eprintln!("Unknown option: {}", arg);
                std::process::exit(1);
            }
        }
    }

    match traverse_directory(Path::new(directory), &options) {
        Ok(results) => {
            if results.is_empty() {
                println!("No files found.");
            } else {
                println!("Found {} files:", results.len());
                for result in results {
                    let hidden_marker = if result.is_hidden { "*" } else { " " };
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

fn handle_view(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: {} view <file>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[2];
    let options = ViewOptions::default();

    match view_file(Path::new(file_path), &options) {
        Ok(json_output) => {
            println!("{}", serde_json::to_string_pretty(&json_output).unwrap());
        }
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(1);
        }
    }
}
