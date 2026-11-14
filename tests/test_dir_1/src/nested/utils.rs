// Utility functions for nested module

pub fn format_string(s: &str) -> String {
    format!("[{}]", s.to_uppercase())
}

pub fn count_words(s: &str) -> usize {
    s.split_whitespace().count()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_format_string() {
        assert_eq!(format_string("test"), "[TEST]");
    }
    
    #[test]
    fn test_count_words() {
        assert_eq!(count_words("This is a test"), 4);
    }
}
