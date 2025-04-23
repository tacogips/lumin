pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn search_in_text(text: &str, pattern: &str) -> bool {
    text.contains(pattern)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add() {
        assert_eq!(add(2, 3), 5);
    }
    
    #[test]
    fn test_search() {
        assert!(search_in_text("This is a test", "test"));
        assert!(!search_in_text("This is a test", "banana"));
    }
}
