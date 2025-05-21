//! Tests for pagination and sorting behavior of search_files

#[cfg(test)]
mod tests {
    use super::super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    /// Creates a temporary directory with test files that will produce inconsistent 
    /// natural ordering results unless properly sorted
    fn create_search_files_for_pagination(dir: &Path) -> Result<()> {
        // Create files with specific content that will produce
        // matches in a predictable order when sorted by path and line number
        let test_data = [
            // Files named in reverse order to test path sorting
            (
                "zfile.txt",
                "Line 1
Line 2 pattern
Line 3
"
            ),
            (
                "yfile.txt",
                "Line 1 pattern
Line 2
Line 3 pattern
"
            ),
            (
                "xfile.txt",
                "No pattern here
Still no pattern
Finally a pattern
"
            ),
            // A nested directory structure to test path sorting
            (
                "subdir/afile.txt",
                "pattern in first line
No pattern
Another pattern here
"
            ),
            (
                "subdir/bfile.txt",
                "Regular line
pattern in second line
pattern in third line
"
            ),
        ];

        // Create subdirectory
        std::fs::create_dir_all(dir.join("subdir"))?;

        // Create each test file
        for (filename, content) in &test_data {
            let file_path = dir.join(filename);
            let mut file = File::create(file_path)?;
            file.write_all(content.as_bytes())?;
        }

        Ok(())
    }

    /// Validates that each subset of results from search_files has the same items
    /// as the corresponding subset from the full sorted results
    fn validate_pagination_consistency(
        full_results: &SearchResult,
        pattern: &str,
        directory: &Path,
        page_size: usize,
    ) -> Result<()> {
        // Total result count for validation
        let total_count = full_results.total_number;
        let total_pages = (total_count + page_size - 1) / page_size;

        println!("Validating pagination with {} total results, page size {}, {} pages",
            total_count, page_size, total_pages);

        // Test each page and verify it matches the corresponding slice of full results
        for page in 0..total_pages {
            let skip = page * page_size;
            let mut options = SearchOptions::default();
            options.skip = Some(skip);
            options.take = Some(page_size);
            
            // Get this page using pagination
            let page_results = search_files(pattern, directory, &options)?;
            
            // Calculate expected results for this page
            let from_idx = skip + 1;  // 1-based index for split
            let to_idx = (skip + page_size).min(total_count);
            let expected_results = full_results.clone().split(from_idx, to_idx);

            // Verify the page has the correct size
            let expected_page_size = to_idx.saturating_sub(from_idx - 1);
            assert_eq!(
                page_results.lines.len(),
                expected_page_size,
                "Page {} should have {} results",
                page,
                expected_page_size
            );
            
            // Verify each result matches the expected result
            for (i, (actual, expected)) in page_results.lines.iter().zip(expected_results.lines.iter()).enumerate() {
                assert_eq!(
                    actual.file_path, expected.file_path,
                    "Page {}, result {} file path mismatch", page, i
                );
                assert_eq!(
                    actual.line_number, expected.line_number,
                    "Page {}, result {} line number mismatch", page, i
                );
                assert_eq!(
                    actual.line_content, expected.line_content,
                    "Page {}, result {} content mismatch", page, i
                );
            }
            
            println!("✓ Page {} validated successfully ({} results)", page, page_results.lines.len());
        }

        Ok(())
    }

    #[test]
    fn test_search_pagination_consistency() -> Result<()> {
        // Create a temporary directory with our test files
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        println!("Creating test files in {}", temp_path.display());
        create_search_files_for_pagination(temp_path)?;

        // Search pattern that appears in all files
        let pattern = "pattern";
        
        // First get all results with default sorting
        let full_results = search_files(pattern, temp_path, &SearchOptions::default())?;
        
        println!("Full search found {} results", full_results.total_number);
        println!("Results in sorted order:");
        for (i, line) in full_results.lines.iter().enumerate() {
            println!("{}: {}:{} - {}", 
                i+1, 
                line.file_path.display(), 
                line.line_number,
                line.line_content.trim()
            );
        }
        
        // Test various page sizes to ensure consistent results
        let page_sizes = [1, 2, 3, 5, 7];
        
        for &page_size in &page_sizes {
            println!("\nTesting pagination with page size {}", page_size);
            validate_pagination_consistency(&full_results, pattern, temp_path, page_size)?;
        }

        // Test edge cases
        println!("\nTesting edge cases:");
        
        // Edge case 1: Skip beyond available results
        let mut options_beyond = SearchOptions::default();
        options_beyond.skip = Some(full_results.total_number + 10);
        let beyond_results = search_files(pattern, temp_path, &options_beyond)?;
        assert_eq!(
            beyond_results.lines.len(), 0,
            "Skipping beyond available results should return empty results"
        );
        println!("✓ Edge case: Skip beyond available results - Passed");
        
        // Edge case 2: Take more than available after skip
        let mut options_take_more = SearchOptions::default();
        options_take_more.skip = Some(full_results.total_number - 2);
        options_take_more.take = Some(10); // More than what's left
        let take_more_results = search_files(pattern, temp_path, &options_take_more)?;
        assert_eq!(
            take_more_results.lines.len(), 2,
            "Taking more than available after skip should return all remaining results"
        );
        println!("✓ Edge case: Take more than available - Passed");

        // Edge case 3: Skip 0, take all
        let mut options_all = SearchOptions::default();
        options_all.skip = Some(0);
        options_all.take = Some(full_results.total_number + 10); // More than total
        let all_results = search_files(pattern, temp_path, &options_all)?;
        assert_eq!(
            all_results.lines.len(), full_results.lines.len(),
            "Skip 0, take more than total should return all results"
        );
        println!("✓ Edge case: Skip 0, take all - Passed");

        println!("\nAll pagination tests passed successfully!");
        Ok(())
    }

    #[test]
    fn test_search_sorting_behavior() -> Result<()> {
        // Create a temporary directory with our test files
        let temp_dir = TempDir::new()?;
        let temp_path = temp_dir.path();
        
        println!("Creating test files for sorting test in {}", temp_path.display());
        create_search_files_for_pagination(temp_path)?;

        // Search pattern that appears in all files
        let pattern = "pattern";
        
        // Get all results
        let results = search_files(pattern, temp_path, &SearchOptions::default())?;
        
        println!("Found {} results, verifying sort order", results.total_number);
        
        // Verify results are sorted by file path and line number
        let mut prev_path: Option<&PathBuf> = None;
        let mut prev_line_number: Option<u64> = None;
        
        for (i, line) in results.lines.iter().enumerate() {
            println!("{}: {}:{}", i+1, line.file_path.display(), line.line_number);
            
            // Check if the path has changed
            if let Some(prev) = prev_path {
                if &line.file_path != prev {
                    // Reset line number tracking when path changes
                    prev_line_number = None;
                    
                    // Verify new path is lexicographically greater than previous
                    assert!(
                        line.file_path.to_string_lossy() > prev.to_string_lossy(),
                        "File path at index {} ({}) should sort after previous path ({})",
                        i, line.file_path.display(), prev.display()
                    );
                }
            }
            
            // Check if line numbers within same file are in ascending order
            if prev_path.is_some() && prev_path.unwrap() == &line.file_path {
                if let Some(prev_num) = prev_line_number {
                    assert!(
                        line.line_number > prev_num,
                        "Line number at index {} ({}:{}) should be greater than previous ({})",
                        i, line.file_path.display(), line.line_number, prev_num
                    );
                }
            }
            
            // Update previous values for next iteration
            prev_path = Some(&line.file_path);
            prev_line_number = Some(line.line_number);
        }
        
        println!("✓ Sort order verified - results are properly sorted by path and line number");
        Ok(())
    }
}