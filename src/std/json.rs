// JSON encoding/decoding functionality for the Bulu programming language
// Requirements: 7.3.1, 7.3.4, 7.3.5

use std::collections::HashMap;
use std::fmt;

/// JSON value types
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(HashMap<String, JsonValue>),
}

impl JsonValue {
    /// Check if the value is null
    pub fn is_null(&self) -> bool {
        matches!(self, JsonValue::Null)
    }

    /// Check if the value is a boolean
    pub fn is_bool(&self) -> bool {
        matches!(self, JsonValue::Bool(_))
    }

    /// Check if the value is a number
    pub fn is_number(&self) -> bool {
        matches!(self, JsonValue::Number(_))
    }

    /// Check if the value is a string
    pub fn is_string(&self) -> bool {
        matches!(self, JsonValue::String(_))
    }

    /// Check if the value is an array
    pub fn is_array(&self) -> bool {
        matches!(self, JsonValue::Array(_))
    }

    /// Check if the value is an object
    pub fn is_object(&self) -> bool {
        matches!(self, JsonValue::Object(_))
    }

    /// Get the value as a boolean
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            JsonValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get the value as a number
    pub fn as_number(&self) -> Option<f64> {
        match self {
            JsonValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// Get the value as an integer
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            JsonValue::Number(n) => {
                if n.fract() == 0.0 && *n >= i64::MIN as f64 && *n <= i64::MAX as f64 {
                    Some(*n as i64)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get the value as a string
    pub fn as_str(&self) -> Option<&str> {
        match self {
            JsonValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get the value as an array
    pub fn as_array(&self) -> Option<&Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get the value as a mutable array
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<JsonValue>> {
        match self {
            JsonValue::Array(arr) => Some(arr),
            _ => None,
        }
    }

    /// Get the value as an object
    pub fn as_object(&self) -> Option<&HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get the value as a mutable object
    pub fn as_object_mut(&mut self) -> Option<&mut HashMap<String, JsonValue>> {
        match self {
            JsonValue::Object(obj) => Some(obj),
            _ => None,
        }
    }

    /// Get a value from an object by key
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.get(key),
            _ => None,
        }
    }

    /// Get a value from an array by index
    pub fn get_index(&self, index: usize) -> Option<&JsonValue> {
        match self {
            JsonValue::Array(arr) => arr.get(index),
            _ => None,
        }
    }

    /// Insert a value into an object
    pub fn insert(&mut self, key: String, value: JsonValue) -> Option<JsonValue> {
        match self {
            JsonValue::Object(obj) => obj.insert(key, value),
            _ => None,
        }
    }

    /// Push a value to an array
    pub fn push(&mut self, value: JsonValue) -> Result<(), JsonError> {
        match self {
            JsonValue::Array(arr) => {
                arr.push(value);
                Ok(())
            }
            _ => Err(JsonError::TypeError("Cannot push to non-array".to_string())),
        }
    }

    /// Get the length of an array or object
    pub fn len(&self) -> Option<usize> {
        match self {
            JsonValue::Array(arr) => Some(arr.len()),
            JsonValue::Object(obj) => Some(obj.len()),
            JsonValue::String(s) => Some(s.len()),
            _ => None,
        }
    }

    /// Check if an array or object is empty
    pub fn is_empty(&self) -> bool {
        self.len().map_or(false, |len| len == 0)
    }
}

impl fmt::Display for JsonValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonValue::Null => write!(f, "null"),
            JsonValue::Bool(b) => write!(f, "{}", b),
            JsonValue::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            JsonValue::String(s) => write!(f, "\"{}\"", escape_string(s)),
            JsonValue::Array(arr) => {
                write!(f, "[")?;
                for (i, item) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            JsonValue::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"{}\":{}", escape_string(key), value)?;
                }
                write!(f, "}}")
            }
        }
    }
}

/// JSON parsing and serialization errors
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    ParseError(String),
    TypeError(String),
    IndexError(String),
    KeyError(String),
}

impl fmt::Display for JsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JsonError::ParseError(msg) => write!(f, "JSON Parse Error: {}", msg),
            JsonError::TypeError(msg) => write!(f, "JSON Type Error: {}", msg),
            JsonError::IndexError(msg) => write!(f, "JSON Index Error: {}", msg),
            JsonError::KeyError(msg) => write!(f, "JSON Key Error: {}", msg),
        }
    }
}

impl std::error::Error for JsonError {}

/// JSON parser
pub struct JsonParser {
    input: Vec<char>,
    position: usize,
}

impl JsonParser {
    pub fn new(input: &str) -> Self {
        JsonParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        let value = self.parse_value()?;
        self.skip_whitespace();
        
        if self.position < self.input.len() {
            return Err(JsonError::ParseError("Unexpected characters after JSON value".to_string()));
        }
        
        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        
        if self.position >= self.input.len() {
            return Err(JsonError::ParseError("Unexpected end of input".to_string()));
        }

        match self.current_char() {
            'n' => self.parse_null(),
            't' | 'f' => self.parse_bool(),
            '"' => self.parse_string(),
            '[' => self.parse_array(),
            '{' => self.parse_object(),
            c if c.is_ascii_digit() || c == '-' => self.parse_number(),
            _ => Err(JsonError::ParseError(format!("Unexpected character: {}", self.current_char()))),
        }
    }

    fn parse_null(&mut self) -> Result<JsonValue, JsonError> {
        if self.consume_literal("null") {
            Ok(JsonValue::Null)
        } else {
            Err(JsonError::ParseError("Invalid null literal".to_string()))
        }
    }

    fn parse_bool(&mut self) -> Result<JsonValue, JsonError> {
        if self.consume_literal("true") {
            Ok(JsonValue::Bool(true))
        } else if self.consume_literal("false") {
            Ok(JsonValue::Bool(false))
        } else {
            Err(JsonError::ParseError("Invalid boolean literal".to_string()))
        }
    }

    fn parse_string(&mut self) -> Result<JsonValue, JsonError> {
        if self.current_char() != '"' {
            return Err(JsonError::ParseError("Expected '\"' at start of string".to_string()));
        }
        
        self.advance(); // Skip opening quote
        let mut result = String::new();
        
        while self.position < self.input.len() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                if self.position >= self.input.len() {
                    return Err(JsonError::ParseError("Unexpected end of input in string escape".to_string()));
                }
                
                match self.current_char() {
                    '"' => result.push('"'),
                    '\\' => result.push('\\'),
                    '/' => result.push('/'),
                    'b' => result.push('\u{0008}'),
                    'f' => result.push('\u{000C}'),
                    'n' => result.push('\n'),
                    'r' => result.push('\r'),
                    't' => result.push('\t'),
                    'u' => {
                        // Unicode escape sequence
                        self.advance();
                        let mut hex = String::new();
                        for _ in 0..4 {
                            if self.position >= self.input.len() || !self.current_char().is_ascii_hexdigit() {
                                return Err(JsonError::ParseError("Invalid unicode escape sequence".to_string()));
                            }
                            hex.push(self.current_char());
                            self.advance();
                        }
                        self.position -= 1; // Back up one since we'll advance at the end of the loop
                        
                        if let Ok(code_point) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code_point) {
                                result.push(ch);
                            } else {
                                return Err(JsonError::ParseError("Invalid unicode code point".to_string()));
                            }
                        } else {
                            return Err(JsonError::ParseError("Invalid unicode escape sequence".to_string()));
                        }
                    }
                    _ => return Err(JsonError::ParseError(format!("Invalid escape sequence: \\{}", self.current_char()))),
                }
            } else {
                result.push(self.current_char());
            }
            self.advance();
        }
        
        if self.position >= self.input.len() {
            return Err(JsonError::ParseError("Unterminated string".to_string()));
        }
        
        self.advance(); // Skip closing quote
        Ok(JsonValue::String(result))
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.position;
        
        // Handle negative sign
        if self.current_char() == '-' {
            self.advance();
        }
        
        // Parse integer part
        if self.current_char() == '0' {
            self.advance();
        } else if self.current_char().is_ascii_digit() {
            while self.position < self.input.len() && self.current_char().is_ascii_digit() {
                self.advance();
            }
        } else {
            return Err(JsonError::ParseError("Invalid number format".to_string()));
        }
        
        // Parse fractional part
        if self.position < self.input.len() && self.current_char() == '.' {
            self.advance();
            if self.position >= self.input.len() || !self.current_char().is_ascii_digit() {
                return Err(JsonError::ParseError("Invalid number format: missing digits after decimal point".to_string()));
            }
            while self.position < self.input.len() && self.current_char().is_ascii_digit() {
                self.advance();
            }
        }
        
        // Parse exponent part
        if self.position < self.input.len() && (self.current_char() == 'e' || self.current_char() == 'E') {
            self.advance();
            if self.position < self.input.len() && (self.current_char() == '+' || self.current_char() == '-') {
                self.advance();
            }
            if self.position >= self.input.len() || !self.current_char().is_ascii_digit() {
                return Err(JsonError::ParseError("Invalid number format: missing digits in exponent".to_string()));
            }
            while self.position < self.input.len() && self.current_char().is_ascii_digit() {
                self.advance();
            }
        }
        
        let number_str: String = self.input[start..self.position].iter().collect();
        match number_str.parse::<f64>() {
            Ok(num) => Ok(JsonValue::Number(num)),
            Err(_) => Err(JsonError::ParseError(format!("Invalid number: {}", number_str))),
        }
    }

    fn parse_array(&mut self) -> Result<JsonValue, JsonError> {
        if self.current_char() != '[' {
            return Err(JsonError::ParseError("Expected '[' at start of array".to_string()));
        }
        
        self.advance(); // Skip opening bracket
        self.skip_whitespace();
        
        let mut array = Vec::new();
        
        // Handle empty array
        if self.position < self.input.len() && self.current_char() == ']' {
            self.advance();
            return Ok(JsonValue::Array(array));
        }
        
        loop {
            let value = self.parse_value()?;
            array.push(value);
            
            self.skip_whitespace();
            if self.position >= self.input.len() {
                return Err(JsonError::ParseError("Unterminated array".to_string()));
            }
            
            match self.current_char() {
                ',' => {
                    self.advance();
                    self.skip_whitespace();
                }
                ']' => {
                    self.advance();
                    break;
                }
                _ => return Err(JsonError::ParseError("Expected ',' or ']' in array".to_string())),
            }
        }
        
        Ok(JsonValue::Array(array))
    }

    fn parse_object(&mut self) -> Result<JsonValue, JsonError> {
        if self.current_char() != '{' {
            return Err(JsonError::ParseError("Expected '{' at start of object".to_string()));
        }
        
        self.advance(); // Skip opening brace
        self.skip_whitespace();
        
        let mut object = HashMap::new();
        
        // Handle empty object
        if self.position < self.input.len() && self.current_char() == '}' {
            self.advance();
            return Ok(JsonValue::Object(object));
        }
        
        loop {
            // Parse key
            let key = match self.parse_string()? {
                JsonValue::String(s) => s,
                _ => return Err(JsonError::ParseError("Object key must be a string".to_string())),
            };
            
            self.skip_whitespace();
            if self.position >= self.input.len() || self.current_char() != ':' {
                return Err(JsonError::ParseError("Expected ':' after object key".to_string()));
            }
            self.advance(); // Skip colon
            
            // Parse value
            let value = self.parse_value()?;
            object.insert(key, value);
            
            self.skip_whitespace();
            if self.position >= self.input.len() {
                return Err(JsonError::ParseError("Unterminated object".to_string()));
            }
            
            match self.current_char() {
                ',' => {
                    self.advance();
                    self.skip_whitespace();
                }
                '}' => {
                    self.advance();
                    break;
                }
                _ => return Err(JsonError::ParseError("Expected ',' or '}' in object".to_string())),
            }
        }
        
        Ok(JsonValue::Object(object))
    }

    fn current_char(&self) -> char {
        if self.position < self.input.len() {
            self.input[self.position]
        } else {
            '\0' // Return null character if out of bounds
        }
    }

    fn advance(&mut self) {
        self.position += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.current_char().is_whitespace() {
            self.advance();
        }
    }

    fn consume_literal(&mut self, literal: &str) -> bool {
        let literal_chars: Vec<char> = literal.chars().collect();
        
        if self.position + literal_chars.len() > self.input.len() {
            return false;
        }
        
        for (i, &ch) in literal_chars.iter().enumerate() {
            if self.input[self.position + i] != ch {
                return false;
            }
        }
        
        self.position += literal_chars.len();
        true
    }
}

/// JSON serializer with pretty printing support
pub struct JsonSerializer {
    pretty: bool,
    indent: usize,
}

impl JsonSerializer {
    pub fn new() -> Self {
        JsonSerializer {
            pretty: false,
            indent: 0,
        }
    }

    pub fn pretty() -> Self {
        JsonSerializer {
            pretty: true,
            indent: 0,
        }
    }

    pub fn serialize(&self, value: &JsonValue) -> String {
        self.serialize_value(value, 0)
    }

    fn serialize_value(&self, value: &JsonValue, depth: usize) -> String {
        match value {
            JsonValue::Null => "null".to_string(),
            JsonValue::Bool(b) => b.to_string(),
            JsonValue::Number(n) => {
                if n.fract() == 0.0 {
                    format!("{}", *n as i64)
                } else {
                    n.to_string()
                }
            }
            JsonValue::String(s) => format!("\"{}\"", escape_string(s)),
            JsonValue::Array(arr) => self.serialize_array(arr, depth),
            JsonValue::Object(obj) => self.serialize_object(obj, depth),
        }
    }

    fn serialize_array(&self, array: &[JsonValue], depth: usize) -> String {
        if array.is_empty() {
            return "[]".to_string();
        }

        let mut result = String::from("[");
        
        if self.pretty {
            result.push('\n');
        }

        for (i, item) in array.iter().enumerate() {
            if i > 0 {
                result.push(',');
                if self.pretty {
                    result.push('\n');
                }
            }

            if self.pretty {
                result.push_str(&"  ".repeat(depth + 1));
            }

            result.push_str(&self.serialize_value(item, depth + 1));
        }

        if self.pretty {
            result.push('\n');
            result.push_str(&"  ".repeat(depth));
        }

        result.push(']');
        result
    }

    fn serialize_object(&self, object: &HashMap<String, JsonValue>, depth: usize) -> String {
        if object.is_empty() {
            return "{}".to_string();
        }

        let mut result = String::from("{");
        
        if self.pretty {
            result.push('\n');
        }

        let mut keys: Vec<&String> = object.keys().collect();
        keys.sort(); // Sort keys for consistent output

        for (i, key) in keys.iter().enumerate() {
            if i > 0 {
                result.push(',');
                if self.pretty {
                    result.push('\n');
                }
            }

            if self.pretty {
                result.push_str(&"  ".repeat(depth + 1));
            }

            result.push_str(&format!("\"{}\":", escape_string(key)));
            
            if self.pretty {
                result.push(' ');
            }

            if let Some(value) = object.get(*key) {
                result.push_str(&self.serialize_value(value, depth + 1));
            }
        }

        if self.pretty {
            result.push('\n');
            result.push_str(&"  ".repeat(depth));
        }

        result.push('}');
        result
    }
}

/// JSON utility functions
pub struct Json;

impl Json {
    /// Parse a JSON string into a JsonValue
    pub fn parse(input: &str) -> Result<JsonValue, JsonError> {
        let mut parser = JsonParser::new(input);
        parser.parse()
    }

    /// Serialize a JsonValue to a JSON string
    pub fn stringify(value: &JsonValue) -> String {
        let serializer = JsonSerializer::new();
        serializer.serialize(value)
    }

    /// Serialize a JsonValue to a pretty-printed JSON string
    pub fn stringify_pretty(value: &JsonValue) -> String {
        let serializer = JsonSerializer::pretty();
        serializer.serialize(value)
    }

    /// Create a JsonValue from a Rust value (simplified type-safe decoding)
    pub fn from_bool(value: bool) -> JsonValue {
        JsonValue::Bool(value)
    }

    pub fn from_i64(value: i64) -> JsonValue {
        JsonValue::Number(value as f64)
    }

    pub fn from_f64(value: f64) -> JsonValue {
        JsonValue::Number(value)
    }

    pub fn from_str(value: &str) -> JsonValue {
        JsonValue::String(value.to_string())
    }

    pub fn from_string(value: String) -> JsonValue {
        JsonValue::String(value)
    }

    /// Create an empty JSON object
    pub fn object() -> JsonValue {
        JsonValue::Object(HashMap::new())
    }

    /// Create an empty JSON array
    pub fn array() -> JsonValue {
        JsonValue::Array(Vec::new())
    }

    /// Create a JSON null value
    pub fn null() -> JsonValue {
        JsonValue::Null
    }
}

/// Escape special characters in a string for JSON
fn escape_string(s: &str) -> String {
    let mut result = String::new();
    
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\u{0008}' => result.push_str("\\b"),
            '\u{000C}' => result.push_str("\\f"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_json_value_creation() {
        let null_val = JsonValue::Null;
        assert!(null_val.is_null());

        let bool_val = JsonValue::Bool(true);
        assert!(bool_val.is_bool());
        assert_eq!(bool_val.as_bool(), Some(true));

        let num_val = JsonValue::Number(42.0);
        assert!(num_val.is_number());
        assert_eq!(num_val.as_number(), Some(42.0));
        assert_eq!(num_val.as_i64(), Some(42));

        let str_val = JsonValue::String("hello".to_string());
        assert!(str_val.is_string());
        assert_eq!(str_val.as_str(), Some("hello"));
    }

    #[test]
    fn test_json_parse_primitives() {
        assert_eq!(Json::parse("null").unwrap(), JsonValue::Null);
        assert_eq!(Json::parse("true").unwrap(), JsonValue::Bool(true));
        assert_eq!(Json::parse("false").unwrap(), JsonValue::Bool(false));
        assert_eq!(Json::parse("42").unwrap(), JsonValue::Number(42.0));
        assert_eq!(Json::parse("3.14").unwrap(), JsonValue::Number(3.14));
        assert_eq!(Json::parse("\"hello\"").unwrap(), JsonValue::String("hello".to_string()));
    }

    #[test]
    fn test_json_parse_array() {
        let json = "[1, 2, 3]";
        let parsed = Json::parse(json).unwrap();
        
        assert!(parsed.is_array());
        let array = parsed.as_array().unwrap();
        assert_eq!(array.len(), 3);
        assert_eq!(array[0], JsonValue::Number(1.0));
        assert_eq!(array[1], JsonValue::Number(2.0));
        assert_eq!(array[2], JsonValue::Number(3.0));
    }

    #[test]
    fn test_json_parse_object() {
        let json = r#"{"name": "Alice", "age": 30, "active": true}"#;
        let parsed = Json::parse(json).unwrap();
        
        assert!(parsed.is_object());
        let obj = parsed.as_object().unwrap();
        assert_eq!(obj.len(), 3);
        assert_eq!(obj.get("name"), Some(&JsonValue::String("Alice".to_string())));
        assert_eq!(obj.get("age"), Some(&JsonValue::Number(30.0)));
        assert_eq!(obj.get("active"), Some(&JsonValue::Bool(true)));
    }

    #[test]
    fn test_json_parse_nested() {
        let json = r#"{"users": [{"name": "Alice", "age": 30}, {"name": "Bob", "age": 25}]}"#;
        let parsed = Json::parse(json).unwrap();
        
        let users = parsed.get("users").unwrap().as_array().unwrap();
        assert_eq!(users.len(), 2);
        
        let alice = users[0].as_object().unwrap();
        assert_eq!(alice.get("name").unwrap().as_str(), Some("Alice"));
        assert_eq!(alice.get("age").unwrap().as_i64(), Some(30));
    }

    #[test]
    fn test_json_stringify() {
        let value = JsonValue::Object({
            let mut obj = HashMap::new();
            obj.insert("name".to_string(), JsonValue::String("Alice".to_string()));
            obj.insert("age".to_string(), JsonValue::Number(30.0));
            obj.insert("active".to_string(), JsonValue::Bool(true));
            obj
        });

        let json = Json::stringify(&value);
        assert!(json.contains("\"name\":\"Alice\""));
        assert!(json.contains("\"age\":30"));
        assert!(json.contains("\"active\":true"));
    }

    #[test]
    fn test_json_stringify_pretty() {
        let value = JsonValue::Object({
            let mut obj = HashMap::new();
            obj.insert("name".to_string(), JsonValue::String("Alice".to_string()));
            obj.insert("age".to_string(), JsonValue::Number(30.0));
            obj
        });

        let json = Json::stringify_pretty(&value);
        assert!(json.contains("{\n"));
        assert!(json.contains("  \""));
        assert!(json.contains("\n}"));
    }

    #[test]
    fn test_json_escape_string() {
        assert_eq!(escape_string("hello"), "hello");
        assert_eq!(escape_string("hello\nworld"), "hello\\nworld");
        assert_eq!(escape_string("say \"hello\""), "say \\\"hello\\\"");
        assert_eq!(escape_string("path\\to\\file"), "path\\\\to\\\\file");
    }

    #[test]
    fn test_json_parse_string_escapes() {
        let json = r#""hello\nworld\t\"quoted\"""#;
        let parsed = Json::parse(json).unwrap();
        assert_eq!(parsed.as_str(), Some("hello\nworld\t\"quoted\""));
    }

    #[test]
    fn test_json_parse_unicode() {
        let json = r#""\u0048\u0065\u006c\u006c\u006f""#;
        let parsed = Json::parse(json).unwrap();
        assert_eq!(parsed.as_str(), Some("Hello"));
    }

    #[test]
    fn test_json_parse_errors() {
        assert!(Json::parse("").is_err());
        assert!(Json::parse("{").is_err());
        assert!(Json::parse("[1,]").is_err());
        assert!(Json::parse(r#"{"key": }"#).is_err());
        assert!(Json::parse("invalid").is_err());
    }

    #[test]
    fn test_json_value_manipulation() {
        let mut obj = Json::object();
        obj.insert("name".to_string(), Json::from_str("Alice"));
        obj.insert("age".to_string(), Json::from_i64(30));

        assert_eq!(obj.get("name").unwrap().as_str(), Some("Alice"));
        assert_eq!(obj.get("age").unwrap().as_i64(), Some(30));

        let mut arr = Json::array();
        arr.push(Json::from_i64(1)).unwrap();
        arr.push(Json::from_i64(2)).unwrap();
        arr.push(Json::from_i64(3)).unwrap();

        assert_eq!(arr.len(), Some(3));
        assert_eq!(arr.get_index(0).unwrap().as_i64(), Some(1));
        assert_eq!(arr.get_index(2).unwrap().as_i64(), Some(3));
    }
}