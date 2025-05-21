use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use serial_test::serial;
use std::path::Path;

mod test_helpers;
use test_helpers::{TEST_DIR, TestEnvironment};

/// Tests for combined before-context and after-context functionality
#[cfg(test)]
mod search_combined_context_tests {
    use super::*;

    /// Test searching with both before_context and after_context set
    #[test]
    #[serial]
    fn test_search_with_both_contexts() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn main";
        let mut options = SearchOptions::default();
        options.before_context = 2; // Show 2 lines before each match
        options.after_context = 3;  // Show 3 lines after each match

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Verify that we have both matches and context lines
        let matches: Vec<_> = results.lines.iter().filter(|r| !r.is_context).collect();
        let contexts: Vec<_> = results.lines.iter().filter(|r| r.is_context).collect();

        assert!(!matches.is_empty(), "Should have at least one match");
        assert!(!contexts.is_empty(), "Should have at least one context line");

        // All non-context results should contain the search pattern
        for result in &matches {
            assert!(result.line_content.contains(pattern));
        }

        // For each match, verify that we have the correct context before and after
        for (i, result) in results.lines.iter().enumerate() {
            if !result.is_context {
                // This is a match
                let file_path = &result.file_path;
                let line_num = result.line_number;
                
                // Count context lines before this match
                let mut before_count = 0;
                for j in (0..i).rev() {
                    if results.lines[j].file_path != *file_path || !results.lines[j].is_context {
                        break; // Previous match or different file
                    }
                    // Verify line numbers are consecutive in reverse
                    assert_eq!(results.lines[j].line_number, line_num - (before_count as u64 + 1));
                    before_count += 1;
                    if before_count >= options.before_context {
                        break;
                    }
                }
                
                // Count context lines after this match
                let mut after_count = 0;
                for j in i+1..results.lines.len() {
                    if results.lines[j].file_path != *file_path || !results.lines[j].is_context {
                        break; // Next match or different file
                    }
                    // Verify line numbers are consecutive
                    assert_eq!(results.lines[j].line_number, line_num + (after_count as u64 + 1));
                    after_count += 1;
                    if after_count >= options.after_context {
                        break;
                    }
                }
                
                // Check before context count (unless we're near start of file)
                let file_content = std::fs::read_to_string(file_path)?;
                let file_lines: Vec<_> = file_content.lines().collect();
                let match_line_index = (line_num - 1) as usize; // Convert to 0-based index
                
                if match_line_index >= options.before_context && 
                   (before_count == 0 || 
                    (i > before_count + 1 && !results.lines[i - before_count - 1].is_context)) {
                    assert_eq!(before_count, options.before_context, 
                               "Should have {} lines before the match", options.before_context);
                }
                
                // Check after context count (unless we're near end of file)
                let expected_after_context = std::cmp::min(
                    options.after_context,
                    file_lines.len() - match_line_index - 1
                );
                
                if match_line_index + expected_after_context < file_lines.len() && 
                   (after_count == 0 || i + after_count + 1 < results.lines.len() && !results.lines[i + after_count + 1].is_context) {
                    assert_eq!(after_count, expected_after_context, 
                               "Should have {} lines after the match", expected_after_context);
                }
            }
        }

        Ok(())
    }

    /// Test with overlapping context between matches
    #[test]
    #[serial]
    fn test_search_with_overlapping_contexts() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // First let's find a file with multiple matches close together
        let pattern = "fn";
        let mut options = SearchOptions::default();
        options.before_context = 3;
        options.after_context = 3;

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Group results by file to check for overlapping contexts
        let mut file_results: std::collections::HashMap<_, Vec<_>> = std::collections::HashMap::new();
        for result in &results.lines {
            file_results.entry(result.file_path.clone())
                        .or_insert_with(Vec::new)
                        .push(result);
        }

        // Look for files with multiple matches
        for (_file_path, file_matches) in file_results {
            // Get all non-context matches in this file
            let actual_matches: Vec<_> = file_matches.iter()
                .filter(|r| !r.is_context)
                .collect();
            
            if actual_matches.len() <= 1 {
                continue; // Need at least 2 matches to test overlapping
            }

            // Check for pairs of matches that are close enough for contexts to overlap
            for i in 0..actual_matches.len()-1 {
                let first_match = actual_matches[i];
                let second_match = actual_matches[i+1];
                
                let first_line = first_match.line_number;
                let second_line = second_match.line_number;
                
                // If matches are close enough for contexts to potentially overlap
                if second_line - first_line <= (options.after_context + options.before_context) as u64 + 1 {
                    // Find all results between these two matches
                    let all_lines: Vec<u64> = file_matches.iter()
                        .map(|r| r.line_number)
                        .collect();
                    
                    // Check that every line between the two matches is included in results
                    for line in first_line+1..second_line {
                        assert!(all_lines.contains(&line), 
                                "Line {} should be included as context between matches at {} and {}", 
                                line, first_line, second_line);
                    }
                    
                    // We found a good test case, no need to continue
                    return Ok(());
                }
            }
        }

        // If we reach here, we didn't find any overlapping contexts to test
        // This is acceptable, as not all test files may have matches close enough together
        Ok(())
    }

    /// Test with large combined context values
    #[test]
    #[serial]
    fn test_search_large_combined_contexts() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn main";
        let mut options = SearchOptions::default();
        options.before_context = 100; // Much larger than file sizes
        options.after_context = 100;  // Much larger than file sizes

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Find a match and verify the entire file is included as context
        for (_i, result) in results.lines.iter().enumerate() {
            if !result.is_context && result.line_content.contains(pattern) {
                // Found a match
                let file_path = &result.file_path;
                let file_content = std::fs::read_to_string(file_path)?;
                let file_lines = file_content.lines().count() as u64;
                
                // Count all results for this file
                let file_results_count = results.lines.iter()
                    .filter(|r| r.file_path == *file_path)
                    .count() as u64;
                
                // We should have all lines from the file included
                assert_eq!(file_results_count, file_lines, 
                           "All lines from the file should be included as results");
                
                // Break after finding one good test case
                break;
            }
        }

        Ok(())
    }

    /// Test searching with both contexts and content omission
    #[test]
    #[serial]
    fn test_search_with_combined_contexts_and_omission() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn main";
        let mut options = SearchOptions::default();
        options.before_context = 2; // Show 2 lines before each match
        options.after_context = 2;  // Show 2 lines after each match
        options.match_content_omit_num = Some(10); // Only show 10 chars around matches

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Verify proper flagging of context vs matches and content omission
        let matches_count = results.lines.iter().filter(|r| !r.is_context).count();
        let context_count = results.lines.iter().filter(|r| r.is_context).count();
        
        assert!(matches_count > 0, "Should have at least one match");
        assert!(context_count > 0, "Should have at least one context line");

        // Content omission should only apply to matches, not context lines
        for result in &results.lines {
            if result.is_context {
                assert!(!result.content_omitted, "Context lines should not have content omitted");
            } else if result.line_content.len() > 20 + pattern.len() {
                // For matches with long lines, content should be omitted
                assert!(result.content_omitted, "Long match lines should have content omitted");
            }
        }

        Ok(())
    }
}