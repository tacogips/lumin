use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use serial_test::serial;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tempfile::tempdir;

/// Tests for the search edge cases
#[cfg(test)]
mod search_edge_case_tests {
    use super::*;

    #[test]
    #[serial]
    fn test_before_context_first_line_match() -> Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("first_line_match.txt");
        
        // Create a test file with a pattern on the first line
        let content = "This line should match the pattern\nSecond line of the file\nThird line of the file\nFourth line of the file\nFifth line of the file";
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", content)?;
        
        // Search with before_context = 3 (shouldn't have any effect since match is on first line)
        let mut options = SearchOptions::default();
        options.before_context = 3;
        
        let results = search_files("match the pattern", &file_path, &options)?;
        
        // Verify results
        assert!(!results.lines.is_empty());
        
        // Find the match result
        let match_result = results.lines.iter().find(|r| !r.is_context).unwrap();
        
        // The match should be on line 1 (1-based indexing)
        assert_eq!(match_result.line_number, 1);
        
        // There should be no context lines before the match (since it's the first line)
        let context_lines_before = results.lines.iter()
            .filter(|r| r.is_context && r.line_number < match_result.line_number)
            .count();
        
        assert_eq!(context_lines_before, 0, "There should be no context lines before the match on the first line");
        
        Ok(())
    }
    
    #[test]
    #[serial]
    fn test_after_context_last_line_match() -> Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("last_line_match.txt");
        
        // Create a test file with a pattern on the last line
        let content = "First line of the file\nSecond line of the file\nThird line of the file\nFourth line of the file\nThis line should match the pattern";
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", content)?;
        
        // Search with after_context = 3 (shouldn't have any effect since match is on last line)
        let mut options = SearchOptions::default();
        options.after_context = 3;
        
        let results = search_files("match the pattern", &file_path, &options)?;
        
        // Verify results
        assert!(!results.lines.is_empty());
        
        // Find the match result
        let match_result = results.lines.iter().find(|r| !r.is_context).unwrap();
        
        // The match should be on line 5 (1-based indexing)
        assert_eq!(match_result.line_number, 5);
        
        // There should be no context lines after the match (since it's the last line)
        let context_lines_after = results.lines.iter()
            .filter(|r| r.is_context && r.line_number > match_result.line_number)
            .count();
        
        assert_eq!(context_lines_after, 0, "There should be no context lines after the match on the last line");
        
        Ok(())
    }
    
    #[test]
    #[serial]
    fn test_combined_context_first_and_last_line_matches() -> Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("first_last_line_match.txt");
        
        // Create a test file with matches on both first and last lines
        let content = "MATCH_THIS first line of file\nSecond line of the file\nThird line of the file\nFourth line of the file\nMATCH_THIS last line of file";
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", content)?;
        
        // Search with both before and after context
        let mut options = SearchOptions::default();
        options.before_context = 2;
        options.after_context = 2;
        
        let results = search_files("MATCH_THIS", &file_path, &options)?;
        
        // Verify results - should have 2 matches
        assert_eq!(results.lines.iter().filter(|r| !r.is_context).count(), 2);
        
        // Get matches by line number
        let first_match = results.lines.iter().find(|r| !r.is_context && r.line_number == 1).unwrap();
        let last_match = results.lines.iter().find(|r| !r.is_context && r.line_number == 5).unwrap();
        
        // Verify first match
        let context_lines_before_first = results.lines.iter()
            .filter(|r| r.is_context && r.line_number < first_match.line_number)
            .count();
        
        // Verify last match
        let context_lines_after_last = results.lines.iter()
            .filter(|r| r.is_context && r.line_number > last_match.line_number)
            .count();
        
        // Check edge case handling
        assert_eq!(context_lines_before_first, 0, "There should be no context lines before the match on the first line");
        assert_eq!(context_lines_after_last, 0, "There should be no context lines after the match on the last line");
        
        // Check that context between matches is present
        let lines_between = results.lines.iter()
            .filter(|r| r.is_context && r.line_number > first_match.line_number && r.line_number < last_match.line_number)
            .count();
        
        assert_eq!(lines_between, 3, "All lines between first and last match should be included as context");
        
        Ok(())
    }
    
    // Test for a single-line file with a match
    #[test]
    #[serial]
    fn test_single_line_file_match() -> Result<()> {
        // Create a temporary directory
        let temp_dir = tempdir()?;
        let file_path = temp_dir.path().join("single_line.txt");
        
        // Create a test file with only one line
        let content = "This is a single line file with a match pattern";
        let mut file = File::create(&file_path)?;
        writeln!(file, "{}", content)?;
        
        // Search with both before and after context
        let mut options = SearchOptions::default();
        options.before_context = 3; // Should be ignored since there are no lines before
        options.after_context = 3; // Should be ignored since there are no lines after
        
        let results = search_files("match pattern", &file_path, &options)?;
        
        // Verify results
        assert_eq!(results.lines.len(), 1, "Should have exactly one result");
        
        // There should be no context lines at all
        assert_eq!(results.lines.iter().filter(|r| r.is_context).count(), 0, 
                   "There should be no context lines in a single-line file");
        
        // The match should be on line 1
        assert_eq!(results.lines[0].line_number, 1);
        assert!(!results.lines[0].is_context);
        
        Ok(())
    }
}