use anyhow::Result;
use lumin::search::{SearchOptions, SearchResult, SearchResultLine, search_files};
use serial_test::serial;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use std::fs::File;
use std::io::Write;

mod test_helpers;
use test_helpers::{TestEnvironment, TEST_DIR};

#[cfg(test)]
mod search_sort_tests {
    use super::*;
    
    /// Test that search results are sorted by file path and line number
    #[test]
    #[serial]
    fn test_search_results_sorted() -> Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        
        // Create test files with patterns that will match
        let file_paths = [
            "b_file.txt", // Intentionally out of alphabetical order
            "a_file.txt",
            "c_file.txt",
        ];
        
        for file_name in &file_paths {
            let file_path = temp_dir.path().join(file_name);
            let mut file = File::create(&file_path)?;
            
            // Write content with line numbers out of order to test sorting
            writeln!(file, "Line 1 - no match")?;
            writeln!(file, "Line 2 - pattern to find")?; // Match on line 2
            writeln!(file, "Line 3 - no match")?;
            writeln!(file, "Line 4 - pattern to find")?; // Match on line 4
            writeln!(file, "Line 5 - no match")?;
            writeln!(file, "Line 6 - pattern to find")?; // Match on line 6
        }
        
        // Search for the pattern
        let pattern = "pattern to find";
        let options = SearchOptions::default();
        let result = search_files(pattern, temp_dir.path(), &options)?;
        
        // Check that we have the expected number of matches
        // 3 files × 3 matches per file = 9 matches
        assert_eq!(result.total_number, 9, "Expected 9 matches in total");
        
        // Check that the results are sorted by file path
        // First check file paths
        let mut prev_path: Option<&PathBuf> = None;
        let mut prev_line_number: Option<u64> = None;
        
        for (i, line) in result.lines.iter().enumerate() {
            // Check file path sorting
            if let Some(prev) = prev_path {
                assert!(prev <= &line.file_path, "Results not sorted by file path at index {}", i);
                
                // If same file, check line number sorting
                if prev == &line.file_path {
                    let prev_num = prev_line_number.unwrap();
                    assert!(prev_num < line.line_number, "Results not sorted by line number at index {}", i);
                }
            }
            
            prev_path = Some(&line.file_path);
            prev_line_number = Some(line.line_number);
        }
        
        // Assert that the first result is from a_file.txt (alphabetically first)
        assert!(result.lines[0].file_path.to_string_lossy().contains("a_file.txt"), 
                "First result should be from a_file.txt, got: {}", result.lines[0].file_path.display());
        
        // Check the expected order of line numbers for the first file
        assert_eq!(result.lines[0].line_number, 2, "First match in a_file.txt should be on line 2");
        assert_eq!(result.lines[1].line_number, 4, "Second match in a_file.txt should be on line 4");
        assert_eq!(result.lines[2].line_number, 6, "Third match in a_file.txt should be on line 6");
        
        // Verify the sort_by_path_and_line method on an artificially unsorted result
        let mut unsorted_result = SearchResult {
            total_number: 6,
            lines: vec![
                SearchResultLine {
                    file_path: temp_dir.path().join("z_file.txt"),
                    line_number: 10,
                    line_content: "test".to_string(),
                    content_omitted: false,
                    is_context: false,
                },
                SearchResultLine {
                    file_path: temp_dir.path().join("a_file.txt"),
                    line_number: 5,
                    line_content: "test".to_string(),
                    content_omitted: false,
                    is_context: false,
                },
                SearchResultLine {
                    file_path: temp_dir.path().join("a_file.txt"),
                    line_number: 1,
                    line_content: "test".to_string(),
                    content_omitted: false,
                    is_context: false,
                },
                SearchResultLine {
                    file_path: temp_dir.path().join("z_file.txt"),
                    line_number: 3,
                    line_content: "test".to_string(),
                    content_omitted: false,
                    is_context: false,
                },
                SearchResultLine {
                    file_path: temp_dir.path().join("m_file.txt"),
                    line_number: 7,
                    line_content: "test".to_string(),
                    content_omitted: false,
                    is_context: false,
                },
                SearchResultLine {
                    file_path: temp_dir.path().join("m_file.txt"),
                    line_number: 2,
                    line_content: "test".to_string(),
                    content_omitted: false,
                    is_context: false,
                },
            ],
        };
        
        // Sort the results
        unsorted_result.sort_by_path_and_line();
        
        // Verify that the sort worked correctly
        // First check for file path order: a_file -> m_file -> z_file
        assert!(unsorted_result.lines[0].file_path.to_string_lossy().contains("a_file.txt"));
        assert!(unsorted_result.lines[2].file_path.to_string_lossy().contains("m_file.txt"));
        assert!(unsorted_result.lines[4].file_path.to_string_lossy().contains("z_file.txt"));
        
        // Then check for line number order within each file
        assert_eq!(unsorted_result.lines[0].line_number, 1); // a_file.txt line 1
        assert_eq!(unsorted_result.lines[1].line_number, 5); // a_file.txt line 5
        assert_eq!(unsorted_result.lines[2].line_number, 2); // m_file.txt line 2
        assert_eq!(unsorted_result.lines[3].line_number, 7); // m_file.txt line 7
        assert_eq!(unsorted_result.lines[4].line_number, 3); // z_file.txt line 3
        assert_eq!(unsorted_result.lines[5].line_number, 10); // z_file.txt line 10
        
        Ok(())
    }
}