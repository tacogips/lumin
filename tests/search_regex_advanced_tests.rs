use anyhow::Result;
use defer::defer;
use lumin::search::{SearchOptions, search_files};
use serial_test::serial;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

/// Advanced regex pattern tests that cover the full range of documented patterns

mod search_regex_advanced_tests {
    use super::*;

    /// Test helper to create test files with specific content for regex testing
    fn create_test_file(path: &str, content: &str) -> Result<()> {
        let path = PathBuf::from(path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(&path)?;
        write!(file, "{}", content)?;
        Ok(())
    }

    /// Setup function to create test files with advanced regex test patterns
    fn setup_test_files() -> Result<Vec<String>> {
        let test_files = vec![
            "tests/fixtures/text_files/regex_advanced.txt".to_string(),
            "tests/fixtures/text_files/regex_lookarounds.txt".to_string(),
            "tests/fixtures/text_files/regex_practical.txt".to_string(),
        ];

        // Advanced pattern test file
        create_test_file(
            &test_files[0],
            r#"Regex advanced pattern test file

# Basic Escaping Tests
Literal dot: The file extension is .txt
Literal asterisk: Important* note
Literal plus: C++ programming language
Literal question mark: Do you need help?
Literal parentheses: Function(param1, param2)
Literal brackets: Array[index] access
Literal braces: Define a {scope}
Literal caret: 2^10 = 1024
Literal dollar: Price is $100
Literal backslash: Windows\path\to\file
Literal vertical bar: true | false

# Character Classes
Digits only: 12345
Letters only: abcdef
Mixed: a1b2c3
Custom set: Only vowels: aeiou
Not digits: @#$%^
Word chars: abc_123
Whitespace test:    	    
Non-whitespace: NoSpacesHere

# Repetition Tests
Zero or more: aaaaaa
One or more: bbbbbb
Optional char: color and colour
Exactly 3: exactly123
Two to four: num123
Five or more: repeat12345
Lazy vs greedy: <div>Inner content</div><p>More content</p>

# Boundary Tests
Start of line: ^Beginning of a line
End of line: At the very end$
Word boundaries: the word is here
Not word boundary: thewordishere

# Complex Mixed Patterns
ISO date: 2023-05-15
Phone number: (123) 456-7890
Hex color code: #FF5733
Currency: $1,234.56
Version string: v1.2.3-beta
Semantic version: 2.1.0-alpha.1
MD5 hash: 5d41402abc4b2a76b9719d911017c592
Random UUID: 123e4567-e89b-12d3-a456-426614174000
"#,
        )?;

        // Lookarounds test file  
        create_test_file(
            &test_files[1],
            r#"Regex lookaround pattern test file

# Lookahead Tests
Positive lookahead: foo bar but not foo baz
TODO: Fix this bug
TODO Incomplete item
function myFunc 
function yourFunc()

# Lookbehind Tests
Positive lookbehind: no @ symbol, but @username and @another
Negative lookbehind: 100 123word 456

# Combined Lookarounds
Complex pattern: password123! is strong but password is weak
JS import: import { Component } from 'library';
HTML tag: <div class="container">Content</div>
"#,
        )?;

        // Practical patterns test file
        create_test_file(
            &test_files[2],
            r#"Regex practical pattern test file

# Email Addresses
Contact us at: user@example.com
Alternative email: first.last@subdomain.example.co.uk
Invalid email: not.an.email

# URLs
Website: https://www.example.com
Secure URL: https://api.example.org/v1/users?id=123
Simple URL: http://example.net
Invalid URL: not-a-url.com

# IP Addresses
IPv4: 192.168.1.1
Local IP: 127.0.0.1
Broadcast: 255.255.255.255
Invalid IP: 999.999.999.999

# Programming Patterns
Rust function: fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}

Rust method: impl Calculator {
    pub fn multiply(&self, a: f64, b: f64) -> f64 {
        a * b
    }
}

# Markdown
# Heading 1
## Heading 2
### Heading 3
Not a heading #hashtag

# JSON
{
    "name": "John Doe",
    "age": 30,
    "isActive": true,
    "scores": null
}

# CSS Colors
Background: #fff;
Border: #3a86ff;
Text color: #333;
RGB color: rgb(255, 99, 71);
RGBA color: rgba(255, 99, 71, 0.5);
"#,
        )?;

        Ok(test_files)
    }

    /// Cleanup function to remove test files
    fn cleanup_test_files(test_files: &[String]) -> Result<()> {
        for file_path in test_files {
            let _ = std::fs::remove_file(file_path);
        }
        Ok(())
    }

    #[test]
    #[serial]
    fn test_basic_escaping_patterns() -> Result<()> {
        let test_files = setup_test_files()?;
        let _cleanup = defer::defer(|| {
            let _ = cleanup_test_files(&test_files);
        });

        let directory = Path::new("tests/fixtures");
        let options = SearchOptions::default();

        // Test escaping of special regex characters
        let patterns_and_expected = [
            (r"\\.txt", "extension is .txt"),
            (r"\\*", "Important* note"),
            (r"\\+", "C++ programming"),
            (r"\\?", "need help?"),
            (r"\\(", "Function(param1"),
            (r"\\[", "Array[index"),
            (r"\\{", "Define a {scope}"),
            (r"\\^", "2^10 = 1024"),
            (r"\\$", "Price is $100"),
            (r"\\\\", "Windows\\path"),
            (r"\\|", "true | false"),
        ];

        for (pattern, expected_match) in patterns_and_expected {
            let results = search_files(pattern, directory, &options)?;
            assert!(!results.is_empty(), "No results for pattern: {}", pattern);
            assert!(
                results.iter().any(|r| r.line_content.contains(expected_match)),
                "Failed to find '{}' with pattern: {}",
                expected_match,
                pattern
            );
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_character_classes() -> Result<()> {
        let test_files = setup_test_files()?;
        let _cleanup = defer::defer(|| {
            let _ = cleanup_test_files(&test_files);
        });

        let directory = Path::new("tests/fixtures");
        let options = SearchOptions::default();

        // Test various character class patterns
        let patterns_and_expected = [
            (r"\\d+", "Digits only: 12345"),
            (r"[a-z]+", "Letters only: abcdef"),
            (r"[a-z][0-9][a-z][0-9][a-z][0-9]", "Mixed: a1b2c3"),
            (r"[aeiou]+", "Only vowels: aeiou"),
            (r"[^0-9]+", "Not digits: @#$%^"),
            (r"\\w+", "Word chars: abc_123"),
            (r"\\s+", "Whitespace test:"),
            (r"\\S+", "NoSpacesHere"),
        ];

        for (pattern, expected_match) in patterns_and_expected {
            let results = search_files(pattern, directory, &options)?;
            assert!(!results.is_empty(), "No results for pattern: {}", pattern);
            assert!(
                results.iter().any(|r| r.line_content.contains(expected_match)),
                "Failed to find '{}' with pattern: {}",
                expected_match,
                pattern
            );
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_repetition_and_quantifiers() -> Result<()> {
        let test_files = setup_test_files()?;
        let _cleanup = defer::defer(|| {
            let _ = cleanup_test_files(&test_files);
        });

        let directory = Path::new("tests/fixtures");
        let options = SearchOptions::default();

        // Test repetition and quantifier patterns
        let patterns_and_expected = [
            (r"a*", "Zero or more: aaaaaa"),
            (r"b+", "One or more: bbbbbb"),
            (r"colou?r", "Optional char: color and colour"),
            (r"exactly\\d{3}", "Exactly 3: exactly123"),
            (r"num\\d{2,4}", "Two to four: num123"),
            (r"repeat\\d{5,}", "Five or more: repeat12345"),
            (r"<div>.*?</div>", "<div>Inner content</div>"),
        ];

        for (pattern, expected_match) in patterns_and_expected {
            let results = search_files(pattern, directory, &options)?;
            assert!(!results.is_empty(), "No results for pattern: {}", pattern);
            assert!(
                results.iter().any(|r| r.line_content.contains(expected_match)),
                "Failed to find '{}' with pattern: {}",
                expected_match,
                pattern
            );
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_anchors_and_boundaries() -> Result<()> {
        let test_files = setup_test_files()?;
        let _cleanup = defer::defer(|| {
            let _ = cleanup_test_files(&test_files);
        });

        let directory = Path::new("tests/fixtures");
        let options = SearchOptions::default();

        // Test anchor and boundary patterns
        let patterns_and_expected = [
            (r"^Start", "Start of line: ^Beginning"),
            (r"end$", "End of line: At the very end$"),
            (r"\\bword\\b", "the word is"),
            (r"\\Bword\\B", "thewordishere"),
        ];

        for (pattern, expected_match) in patterns_and_expected {
            let results = search_files(pattern, directory, &options)?;
            assert!(!results.is_empty(), "No results for pattern: {}", pattern);
            assert!(
                results.iter().any(|r| r.line_content.contains(expected_match)),
                "Failed to find '{}' with pattern: {}",
                expected_match,
                pattern
            );
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_alternation_and_grouping() -> Result<()> {
        let test_files = setup_test_files()?;
        let _cleanup = defer::defer(|| {
            let _ = cleanup_test_files(&test_files);
        });

        let directory = Path::new("tests/fixtures");
        let options = SearchOptions::default();

        // Test alternation and grouping patterns
        let patterns_and_expected = [
            (r"color|colour", "Optional char: color and colour"),
            (r"(RGB|RGBA) color", "RGB color: rgb(255, 99, 71);"),
            (r"(v\\d+\\.\\d+\\.\\d+)(-[a-z]+)?", "Version string: v1.2.3-beta"),
            (r"(api|www)\\.example\\.(com|org)", "https://www.example.com"),
        ];

        for (pattern, expected_match) in patterns_and_expected {
            let results = search_files(pattern, directory, &options)?;
            assert!(!results.is_empty(), "No results for pattern: {}", pattern);
            assert!(
                results.iter().any(|r| r.line_content.contains(expected_match)),
                "Failed to find '{}' with pattern: {}",
                expected_match,
                pattern
            );
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_lookaround_patterns() -> Result<()> {
        let test_files = setup_test_files()?;
        let _cleanup = defer::defer(|| {
            let _ = cleanup_test_files(&test_files);
        });

        let directory = Path::new("tests/fixtures");
        let options = SearchOptions::default();

        // Test lookaround patterns
        let patterns_and_expected = [
            (r"TODO(?=:)", "TODO: Fix this"),
            (r"function\\s+\\w+(?=\\()", "function yourFunc()"),
            (r"function\\s+\\w+(?!\\()", "function myFunc"),
            (r"(?<=@)\\w+", "@username and @another"),
            (r"(?<!\\w)\\d+(?!\\w)", "100 123word"),
        ];

        for (pattern, expected_match) in patterns_and_expected {
            let results = search_files(pattern, directory, &options)?;
            assert!(!results.is_empty(), "No results for pattern: {}", pattern);
            assert!(
                results.iter().any(|r| r.line_content.contains(expected_match)),
                "Failed to find '{}' with pattern: {}",
                expected_match,
                pattern
            );
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_practical_patterns() -> Result<()> {
        let test_files = setup_test_files()?;
        let _cleanup = defer::defer(|| {
            let _ = cleanup_test_files(&test_files);
        });

        let directory = Path::new("tests/fixtures");
        let options = SearchOptions::default();

        // Test practical real-world patterns
        let patterns_and_expected = [
            // Email pattern
            (r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}", "user@example.com"),
            
            // URL pattern
            (r"https?://[\\w.-]+\\.[a-zA-Z]{2,}(?:/[\\w.-]*)*", "https://www.example.com"),
            
            // IP address pattern
            (r"\\b(?:\\d{1,3}\\.){3}\\d{1,3}\\b", "IPv4: 192.168.1.1"),
            
            // Function definition pattern
            (r"fn\\s+\\w+\\s*\\([^)]*\\)", "fn calculate_sum(a: i32, b: i32)"),
            
            // Markdown heading pattern
            (r"^#{1,6}\\s+.*", "# Heading 1"),
            
            // JSON key-value pattern
            (r"\"([\\w.-]+)\"\\s*:\\s*\"([^\"]*)\"", "\"name\": \"John Doe\""),
            
            // CSS color code pattern
            (r"#[a-fA-F0-9]{3,6}", "Background: #fff;"),
            
            // ISO date pattern
            (r"\\d{4}-\\d{2}-\\d{2}", "ISO date: 2023-05-15"),
        ];

        for (pattern, expected_match) in patterns_and_expected {
            let results = search_files(pattern, directory, &options)?;
            assert!(!results.is_empty(), "No results for pattern: {}", pattern);
            assert!(
                results.iter().any(|r| r.line_content.contains(expected_match)),
                "Failed to find '{}' with pattern: {}",
                expected_match,
                pattern
            );
        }

        Ok(())
    }
}