// std.strings module - String manipulation functions
// Requirements: 7.1.3

use std::collections::HashMap;

/// String manipulation utilities
pub struct StringUtils;

impl StringUtils {
    /// Get length of string in characters (not bytes)
    pub fn len(s: &str) -> usize {
        s.chars().count()
    }
    
    /// Get byte length of string
    pub fn byte_len(s: &str) -> usize {
        s.len()
    }
    
    /// Check if string is empty
    pub fn is_empty(s: &str) -> bool {
        s.is_empty()
    }
    
    /// Convert to uppercase
    pub fn to_upper(s: &str) -> String {
        s.to_uppercase()
    }
    
    /// Convert to lowercase
    pub fn to_lower(s: &str) -> String {
        s.to_lowercase()
    }
    
    /// Capitalize first letter
    pub fn capitalize(s: &str) -> String {
        let mut chars = s.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }
    
    /// Title case (capitalize each word)
    pub fn title_case(s: &str) -> String {
        s.split_whitespace()
            .map(|word| Self::capitalize(word))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    /// Trim whitespace from both ends
    pub fn trim(s: &str) -> String {
        s.trim().to_string()
    }
    
    /// Trim whitespace from left
    pub fn trim_left(s: &str) -> String {
        s.trim_start().to_string()
    }
    
    /// Trim whitespace from right
    pub fn trim_right(s: &str) -> String {
        s.trim_end().to_string()
    }
    
    /// Trim specific characters from both ends
    pub fn trim_chars(s: &str, chars: &str) -> String {
        let chars_set: std::collections::HashSet<char> = chars.chars().collect();
        s.trim_matches(|c| chars_set.contains(&c)).to_string()
    }
    
    /// Pad string to specified width with spaces
    pub fn pad_left(s: &str, width: usize) -> String {
        format!("{:>width$}", s, width = width)
    }
    
    /// Pad string to specified width with spaces on right
    pub fn pad_right(s: &str, width: usize) -> String {
        format!("{:<width$}", s, width = width)
    }
    
    /// Pad string to specified width with custom character
    pub fn pad_left_char(s: &str, width: usize, pad_char: char) -> String {
        let current_len = Self::len(s);
        if current_len >= width {
            s.to_string()
        } else {
            let padding = pad_char.to_string().repeat(width - current_len);
            format!("{}{}", padding, s)
        }
    }
    
    /// Pad string to specified width with custom character on right
    pub fn pad_right_char(s: &str, width: usize, pad_char: char) -> String {
        let current_len = Self::len(s);
        if current_len >= width {
            s.to_string()
        } else {
            let padding = pad_char.to_string().repeat(width - current_len);
            format!("{}{}", s, padding)
        }
    }
    
    /// Split string by delimiter
    pub fn split(s: &str, delimiter: &str) -> Vec<String> {
        s.split(delimiter).map(|s| s.to_string()).collect()
    }
    
    /// Split string by whitespace
    pub fn split_whitespace(s: &str) -> Vec<String> {
        s.split_whitespace().map(|s| s.to_string()).collect()
    }
    
    /// Split string into lines
    pub fn split_lines(s: &str) -> Vec<String> {
        s.lines().map(|s| s.to_string()).collect()
    }
    
    /// Join strings with delimiter
    pub fn join(strings: &[String], delimiter: &str) -> String {
        strings.join(delimiter)
    }
    
    /// Replace all occurrences of pattern with replacement
    pub fn replace(s: &str, pattern: &str, replacement: &str) -> String {
        s.replace(pattern, replacement)
    }
    
    /// Replace first occurrence of pattern with replacement
    pub fn replace_first(s: &str, pattern: &str, replacement: &str) -> String {
        if let Some(pos) = s.find(pattern) {
            let mut result = String::new();
            result.push_str(&s[..pos]);
            result.push_str(replacement);
            result.push_str(&s[pos + pattern.len()..]);
            result
        } else {
            s.to_string()
        }
    }
    
    /// Replace last occurrence of pattern with replacement
    pub fn replace_last(s: &str, pattern: &str, replacement: &str) -> String {
        if let Some(pos) = s.rfind(pattern) {
            let mut result = String::new();
            result.push_str(&s[..pos]);
            result.push_str(replacement);
            result.push_str(&s[pos + pattern.len()..]);
            result
        } else {
            s.to_string()
        }
    }
    
    /// Check if string contains substring
    pub fn contains(s: &str, substring: &str) -> bool {
        s.contains(substring)
    }
    
    /// Check if string starts with prefix
    pub fn starts_with(s: &str, prefix: &str) -> bool {
        s.starts_with(prefix)
    }
    
    /// Check if string ends with suffix
    pub fn ends_with(s: &str, suffix: &str) -> bool {
        s.ends_with(suffix)
    }
    
    /// Find first occurrence of substring
    pub fn find(s: &str, substring: &str) -> Option<usize> {
        s.find(substring)
    }
    
    /// Find last occurrence of substring
    pub fn rfind(s: &str, substring: &str) -> Option<usize> {
        s.rfind(substring)
    }
    
    /// Get substring from start to end (exclusive)
    pub fn substring(s: &str, start: usize, end: usize) -> String {
        let chars: Vec<char> = s.chars().collect();
        if start >= chars.len() {
            return String::new();
        }
        let end = end.min(chars.len());
        chars[start..end].iter().collect()
    }
    
    /// Get substring from start with length
    pub fn substr(s: &str, start: usize, length: usize) -> String {
        Self::substring(s, start, start + length)
    }
    
    /// Get character at index
    pub fn char_at(s: &str, index: usize) -> Option<char> {
        s.chars().nth(index)
    }
    
    /// Reverse string
    pub fn reverse(s: &str) -> String {
        s.chars().rev().collect()
    }
    
    /// Repeat string n times
    pub fn repeat(s: &str, n: usize) -> String {
        s.repeat(n)
    }
    
    /// Count occurrences of substring
    pub fn count(s: &str, substring: &str) -> usize {
        if substring.is_empty() {
            return 0;
        }
        
        let mut count = 0;
        let mut start = 0;
        
        while let Some(pos) = s[start..].find(substring) {
            count += 1;
            start += pos + substring.len();
        }
        
        count
    }
    
    /// Check if string is numeric
    pub fn is_numeric(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_numeric())
    }
    
    /// Check if string is alphabetic
    pub fn is_alpha(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_alphabetic())
    }
    
    /// Check if string is alphanumeric
    pub fn is_alphanumeric(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_alphanumeric())
    }
    
    /// Check if string is whitespace only
    pub fn is_whitespace(s: &str) -> bool {
        !s.is_empty() && s.chars().all(|c| c.is_whitespace())
    }
    
    /// Convert string to bytes
    pub fn to_bytes(s: &str) -> Vec<u8> {
        s.bytes().collect()
    }
    
    /// Convert bytes to string (UTF-8)
    pub fn from_bytes(bytes: &[u8]) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(bytes.to_vec())
    }
    
    /// Escape special characters for use in regex
    pub fn escape_regex(s: &str) -> String {
        let special_chars = r"\.+*?[^]$(){}=!<>|:-";
        let mut result = String::new();
        
        for ch in s.chars() {
            if special_chars.contains(ch) {
                result.push('\\');
            }
            result.push(ch);
        }
        
        result
    }
    
    /// Simple template substitution
    pub fn template(template: &str, vars: &HashMap<String, String>) -> String {
        let mut result = template.to_string();
        
        for (key, value) in vars {
            let placeholder = format!("${{{}}}", key);
            result = result.replace(&placeholder, value);
        }
        
        result
    }
    
    /// Calculate Levenshtein distance between two strings
    pub fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();
        
        if len1 == 0 { return len2; }
        if len2 == 0 { return len1; }
        
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];
        
        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }
        
        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i-1] == chars2[j-1] { 0 } else { 1 };
                matrix[i][j] = std::cmp::min(
                    std::cmp::min(
                        matrix[i-1][j] + 1,      // deletion
                        matrix[i][j-1] + 1       // insertion
                    ),
                    matrix[i-1][j-1] + cost      // substitution
                );
            }
        }
        
        matrix[len1][len2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        assert_eq!(StringUtils::len("hello"), 5);
        assert_eq!(StringUtils::len("héllo"), 5); // Unicode
        assert_eq!(StringUtils::byte_len("héllo"), 6); // UTF-8 bytes
        
        assert_eq!(StringUtils::to_upper("hello"), "HELLO");
        assert_eq!(StringUtils::to_lower("WORLD"), "world");
        assert_eq!(StringUtils::capitalize("hello world"), "Hello world");
        assert_eq!(StringUtils::title_case("hello world"), "Hello World");
    }
    
    #[test]
    fn test_trimming() {
        assert_eq!(StringUtils::trim("  hello  "), "hello");
        assert_eq!(StringUtils::trim_left("  hello  "), "hello  ");
        assert_eq!(StringUtils::trim_right("  hello  "), "  hello");
        assert_eq!(StringUtils::trim_chars("...hello...", "."), "hello");
    }
    
    #[test]
    fn test_padding() {
        assert_eq!(StringUtils::pad_left("hi", 5), "   hi");
        assert_eq!(StringUtils::pad_right("hi", 5), "hi   ");
        assert_eq!(StringUtils::pad_left_char("hi", 5, '0'), "000hi");
        assert_eq!(StringUtils::pad_right_char("hi", 5, '0'), "hi000");
    }
    
    #[test]
    fn test_splitting_joining() {
        let parts = StringUtils::split("a,b,c", ",");
        assert_eq!(parts, vec!["a", "b", "c"]);
        
        let joined = StringUtils::join(&parts, "|");
        assert_eq!(joined, "a|b|c");
        
        let words = StringUtils::split_whitespace("hello  world\ttest");
        assert_eq!(words, vec!["hello", "world", "test"]);
    }
    
    #[test]
    fn test_replacement() {
        assert_eq!(StringUtils::replace("hello world", "world", "rust"), "hello rust");
        assert_eq!(StringUtils::replace_first("test test test", "test", "demo"), "demo test test");
        assert_eq!(StringUtils::replace_last("test test test", "test", "demo"), "test test demo");
    }
    
    #[test]
    fn test_searching() {
        assert!(StringUtils::contains("hello world", "world"));
        assert!(StringUtils::starts_with("hello world", "hello"));
        assert!(StringUtils::ends_with("hello world", "world"));
        
        assert_eq!(StringUtils::find("hello world", "world"), Some(6));
        assert_eq!(StringUtils::rfind("test test", "test"), Some(5));
    }
    
    #[test]
    fn test_substring() {
        assert_eq!(StringUtils::substring("hello world", 0, 5), "hello");
        assert_eq!(StringUtils::substr("hello world", 6, 5), "world");
        assert_eq!(StringUtils::char_at("hello", 1), Some('e'));
    }
    
    #[test]
    fn test_utilities() {
        assert_eq!(StringUtils::reverse("hello"), "olleh");
        assert_eq!(StringUtils::repeat("hi", 3), "hihihi");
        assert_eq!(StringUtils::count("hello world hello", "hello"), 2);
        
        assert!(StringUtils::is_numeric("12345"));
        assert!(StringUtils::is_alpha("hello"));
        assert!(StringUtils::is_alphanumeric("hello123"));
        assert!(StringUtils::is_whitespace("   \t\n"));
    }
    
    #[test]
    fn test_template() {
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "Alice".to_string());
        vars.insert("age".to_string(), "30".to_string());
        
        let result = StringUtils::template("Hello ${name}, you are ${age} years old", &vars);
        assert_eq!(result, "Hello Alice, you are 30 years old");
    }
    
    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(StringUtils::levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(StringUtils::levenshtein_distance("hello", "hello"), 0);
        assert_eq!(StringUtils::levenshtein_distance("", "hello"), 5);
    }
}