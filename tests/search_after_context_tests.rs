use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use serial_test::serial;
use std::path::Path;

mod test_helpers;
use test_helpers::{TEST_DIR, TestEnvironment};

/// Tests for the search after-context functionality
#[cfg(test)]
mod search_after_context_tests {
    use super::*;

    /// Test searching with after_context=0 (default)
    #[test]
    #[serial]
    fn test_search_no_after_context() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn";
        let options = SearchOptions::default();

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Verify that no results are marked as context
        assert!(!results.lines.iter().any(|r| r.is_context));

        // All results should contain the search pattern
        for result in &results.lines {
            assert!(result.line_content.contains(pattern));
        }

        Ok(())
    }

    /// Test searching with after_context=3
    #[test]
    #[serial]
    fn test_search_with_after_context() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn main";
        let mut options = SearchOptions::default();
        options.after_context = 3; // Show 3 lines after each match

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Verify that we have both matches and context lines
        let matches: Vec<_> = results.lines.iter().filter(|r| !r.is_context).collect();
        let contexts: Vec<_> = results.lines.iter().filter(|r| r.is_context).collect();

        assert!(!matches.is_empty(), "Should have at least one match");
        assert!(
            !contexts.is_empty(),
            "Should have at least one context line"
        );

        // All non-context results should contain the search pattern
        for result in &matches {
            assert!(result.line_content.contains(pattern));
        }

        // Verify that we have the right amount of context for each match
        // In our case, we're looking for "fn main" which should have at least 3 lines after it
        for (i, result) in results.lines.iter().enumerate() {
            if !result.is_context {
                // This is a match, check if it has context lines following it
                let mut context_count: usize = 0;
                for j in i + 1..results.lines.len() {
                    if !results.lines[j].is_context {
                        break; // Next match found
                    }
                    // Should be the same file
                    assert_eq!(results.lines[j].file_path, result.file_path);
                    // Should be consecutive line numbers
                    assert_eq!(
                        results.lines[j].line_number,
                        result.line_number + context_count as u64 + 1
                    );
                    context_count += 1;
                    if context_count >= options.after_context {
                        break;
                    }
                }
                // Only verify exact context count if we're not at the end of the file
                // and if this match doesn't immediately precede another match
                if i + context_count + 1 < results.lines.len()
                    && results.lines[i + context_count + 1].is_context == false
                {
                    assert_eq!(context_count, options.after_context);
                }
            }
        }

        Ok(())
    }

    /// Test searching with after_context when matches are adjacent
    #[test]
    #[serial]
    fn test_search_adjacent_matches() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        // Search for pattern that might have adjacent matches
        let pattern = "#";
        let mut options = SearchOptions::default();
        options.after_context = 2; // Show 2 lines after each match

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Create a mapping of file paths to line numbers with their is_context flag
        let mut file_lines = std::collections::HashMap::new();

        for result in &results.lines {
            let entries = file_lines
                .entry(result.file_path.clone())
                .or_insert_with(Vec::new);
            entries.push((result.line_number, result.is_context));
        }

        // For each file, verify that all lines are accounted for correctly
        for (_, lines) in file_lines {
            // Sort by line number for consistent checking
            let mut sorted_lines = lines.clone();
            sorted_lines.sort_by_key(|(line_num, _)| *line_num);

            for i in 0..sorted_lines.len() {
                let (line_num, is_context) = sorted_lines[i];

                // If this is a context line, check that it's properly attributed
                if is_context {
                    // Find the match that this context line belongs to
                    let mut found_parent = false;
                    for j in (0..i).rev() {
                        let (parent_line, parent_is_context) = sorted_lines[j];
                        if !parent_is_context {
                            // This is a match, check if our context line is within range
                            if line_num <= parent_line + options.after_context as u64 {
                                found_parent = true;
                                break;
                            }
                        }
                    }
                    assert!(
                        found_parent,
                        "Context line {} has no matching parent",
                        line_num
                    );
                }
            }
        }

        Ok(())
    }

    /// Test searching with a large after_context value
    #[test]
    #[serial]
    fn test_search_large_after_context() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn main";
        let mut options = SearchOptions::default();
        options.after_context = 100; // Much larger than file sizes

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Find a match and verify all following lines are included as context
        for (i, result) in results.lines.iter().enumerate() {
            if !result.is_context && result.line_content.contains(pattern) {
                // Found a match, check all following lines in the same file
                let file_path = &result.file_path;
                let line_num = result.line_number;

                // Count actual lines in the file after the match
                let file_content = std::fs::read_to_string(file_path)?;
                let file_lines: Vec<_> = file_content.lines().collect();
                let expected_context_lines = file_lines.len() as u64 - line_num;

                // Count context lines in the results
                let mut context_count = 0;
                for j in i + 1..results.lines.len() {
                    if results.lines[j].file_path != *file_path || !results.lines[j].is_context {
                        break;
                    }
                    context_count += 1;
                }

                // We should have all lines until the end of the file as context
                assert_eq!(context_count as u64, expected_context_lines);
                break;
            }
        }

        Ok(())
    }

    /// Test searching with after_context while also applying content omission
    #[test]
    #[serial]
    fn test_search_with_after_context_and_omission() -> Result<()> {
        let _env = TestEnvironment::setup()?;

        let pattern = "fn main";
        let mut options = SearchOptions::default();
        options.after_context = 3; // Show 3 lines after each match
        options.match_content_omit_num = Some(10); // Only show 10 chars around matches

        let results = search_files(pattern, Path::new(TEST_DIR), &options)?;

        // Verify that we have results
        assert!(!results.lines.is_empty());

        // Check that matches have content_omitted=true (if long enough)
        // and context lines have content_omitted=false
        for result in &results.lines {
            if !result.is_context {
                // This is a match - may have content omitted if the line is long enough
                if result.line_content.len() > 20 + pattern.len() {
                    // rough estimate
                    assert!(
                        result.content_omitted,
                        "Long match line should have content omitted"
                    );
                }
            } else {
                // Context lines should never have content omitted
                assert!(
                    !result.content_omitted,
                    "Context lines should not have content omitted"
                );
            }
        }

        Ok(())
    }
}
