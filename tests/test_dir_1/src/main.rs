fn main() {
    println!("Hello, world!");
    
    // Test searching functionality
    let search_result = search_files("pattern");
    println!("Found {} matches", search_result.len());
}

fn search_files(pattern: &str) -> Vec<String> {
    // This is a dummy function for testing
    vec!["file1.txt".to_string(), "file2.txt".to_string()]
}
