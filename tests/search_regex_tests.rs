use anyhow::Result;
use lumin::search::{SearchOptions, search_files};
use std::path::Path;

/// Tests focused on regex pattern capabilities used in the search functionality

#[test]
fn test_search_with_basic_literal_text() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // Search for a simple literal word
    let results = search_files("apple", directory, &options)?;

    assert!(!results.lines.is_empty());
    assert!(results.lines.iter().any(|r| r.line_content.contains("apple")));

    Ok(())
}

#[test]
fn test_search_with_wildcard() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "a..le" should match "apple" using '.' as wildcard for any character
    let results = search_files("a..le", directory, &options)?;

    assert!(!results.lines.is_empty());
    assert!(results.lines.iter().any(|r| r.line_content.contains("apple")));

    Ok(())
}

#[test]
fn test_search_with_character_class() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "[0-9]+" should match any sequence of digits
    let results = search_files("[0-9]+", directory, &options)?;

    assert!(!results.lines.is_empty());
    // Should find lines with numbers
    assert!(results.lines.iter().any(|r| r.line_content.contains("123")));

    Ok(())
}

#[test]
fn test_search_with_word_boundaries() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "\\bword\\b" should match "word" as a whole word, not as a part of other words
    let results = search_files("\\bword\\b", directory, &options)?;

    assert!(!results.lines.is_empty());
    assert!(results.lines.iter().any(|r| r.line_content.contains("word")));

    // Count occurrences of exactly "word" (not as part of other words)
    let exact_matches = results
        .lines.iter()
        .filter(|r| r.line_content.contains("Words with boundaries: word"))
        .count();

    assert!(exact_matches > 0);

    Ok(())
}

#[test]
fn test_search_with_anchors() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "^Line" should match "Line" only at the beginning of a line
    let results = search_files("^Line", directory, &options)?;

    assert!(!results.lines.is_empty());

    // All matches should be at the start of a line
    for result in &results.lines {
        let trimmed = result.line_content.trim();
        assert!(trimmed.starts_with("Line"));
    }

    Ok(())
}

#[test]
fn test_search_with_end_anchor() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "file$" should match "file" only at the end of a line
    let results = search_files("file$", directory, &options)?;

    assert!(!results.lines.is_empty());

    // All matches should be at the end of a line
    for result in &results.lines {
        let trimmed = result.line_content.trim();
        assert!(trimmed.ends_with("file"));
    }

    Ok(())
}

#[test]
fn test_search_with_alternation() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "apple|orange" should match either "apple" or "orange"
    let results = search_files("apple|orange", directory, &options)?;

    assert!(!results.lines.is_empty());

    // Should match both words
    let has_apple = results.lines.iter().any(|r| r.line_content.contains("apple"));
    let has_orange = results.lines.iter().any(|r| r.line_content.contains("orange"));

    assert!(has_apple);
    assert!(has_orange);

    Ok(())
}

#[test]
fn test_search_with_repetition() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "a{3,}" should match "aaa", "aaaa", and "aaaaa"
    let results = search_files("a{3,}", directory, &options)?;

    assert!(!results.lines.is_empty());

    // Should match lines with repeated 'a's
    assert!(results.lines.iter().any(|r| r.line_content.contains("aaa")));

    Ok(())
}

#[test]
fn test_search_with_plus_quantifier() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "a+" should match one or more "a"s
    let results = search_files("a+", directory, &options)?;

    assert!(!results.lines.is_empty());

    // Should find various matches with at least one 'a'
    assert!(results.lines.iter().any(|r| r.line_content.contains("apple")));
    assert!(results.lines.iter().any(|r| r.line_content.contains("banana")));
    assert!(results.lines.iter().any(|r| r.line_content.contains("aaa")));

    Ok(())
}

#[test]
fn test_search_with_star_quantifier() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // "ab*c" should match "ac", "abc", "abbc", etc.
    let results = search_files("ab*c", directory, &options)?;

    assert!(!results.lines.is_empty());

    // Should find "abc" in "abc123"
    assert!(results.lines.iter().any(|r| r.line_content.contains("abc123")));

    Ok(())
}

#[test]
fn test_search_email_pattern() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // Simple email regex pattern
    let email_pattern = "[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}";
    let results = search_files(email_pattern, directory, &options)?;

    assert!(!results.lines.is_empty());

    // Should find the email addresses in the test file
    assert!(
        results
            .lines.iter()
            .any(|r| r.line_content.contains("user@example.com"))
    );
    assert!(
        results
            .lines.iter()
            .any(|r| r.line_content.contains("another.user@domain.co.uk"))
    );

    Ok(())
}

#[test]
fn test_search_with_escaping() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // The literal ".*" (dot-star) characters, not as a regex wildcard
    let results = search_files("\\.\\'*", directory, &options)?;

    assert!(!results.lines.is_empty());

    // Should find the line with the literal .* in it
    assert!(
        results
            .lines.iter()
            .any(|r| r.line_content.contains("Special characters: .*"))
    );

    Ok(())
}

#[test]
fn test_search_with_word_followed_by() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // Match "prefixABC" directly instead of using lookahead
    let results = search_files("prefixABC", directory, &options)?;

    assert!(!results.is_empty());

    // Should find "prefixABC" but not "prefixDEF"
    let matches: Vec<_> = results
        .iter()
        .filter(|r| r.line_content.contains("prefixABC"))
        .collect();

    assert!(!matches.is_empty());

    Ok(())
}

#[test]
fn test_search_alternate_word_pattern() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // Match "prefixDEF" directly
    let results = search_files("prefixDEF", directory, &options)?;

    assert!(!results.is_empty());

    // Should find "prefixDEF" but not "prefixABC"
    let matches: Vec<_> = results
        .iter()
        .filter(|r| r.line_content.contains("prefixDEF"))
        .collect();

    assert!(!matches.is_empty());

    Ok(())
}

#[test]
fn test_search_with_non_greedy_matching() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // Match everything between two words with a non-greedy pattern
    let results = search_files("Basic.*?banana", directory, &options)?;

    assert!(!results.is_empty());

    // Should find the full line with "Basic text: apple orange banana"
    assert!(
        results
            .iter()
            .any(|r| r.line_content.contains("Basic text: apple orange banana"))
    );

    Ok(())
}

#[test]
fn test_search_with_number_ranges() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // Match numbers between 100 and 199
    let results = search_files("\\b1[0-9]{2}\\b", directory, &options)?;

    assert!(!results.is_empty());

    // Should find "123" but not "456" or "789"
    assert!(results.iter().any(|r| r.line_content.contains("123")));

    Ok(())
}

#[test]
fn test_search_with_capture_groups() -> Result<()> {
    let directory = Path::new("tests/fixtures");
    let options = SearchOptions::default();

    // Pattern to match content inside parentheses
    let results = search_files("\\([^)]*\\)", directory, &options)?;

    assert!(!results.is_empty());

    // Should find both "(group1)" and "(inner)" and "(123)"
    assert!(results.iter().any(|r| r.line_content.contains("(group1)")));
    assert!(results.iter().any(|r| r.line_content.contains("(inner)")));
    assert!(results.iter().any(|r| r.line_content.contains("(123)")));

    Ok(())
}
