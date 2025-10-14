// CSV processing functionality for the Bulu programming language
// Requirements: 7.3.3

use std::collections::HashMap;
use std::fmt;
use std::io::{BufRead, BufReader, Write};

/// CSV parsing and processing errors
#[derive(Debug, Clone, PartialEq)]
pub enum CsvError {
    ParseError(String),
    IoError(String),
    ValidationError(String),
}

impl fmt::Display for CsvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CsvError::ParseError(msg) => write!(f, "CSV Parse Error: {}", msg),
            CsvError::IoError(msg) => write!(f, "CSV IO Error: {}", msg),
            CsvError::ValidationError(msg) => write!(f, "CSV Validation Error: {}", msg),
        }
    }
}

impl std::error::Error for CsvError {}

/// CSV record - represents a single row of data
#[derive(Debug, Clone, PartialEq)]
pub struct CsvRecord {
    fields: Vec<String>,
}

impl CsvRecord {
    /// Create a new empty record
    pub fn new() -> Self {
        CsvRecord {
            fields: Vec::new(),
        }
    }

    /// Create a record from a vector of fields
    pub fn from_fields(fields: Vec<String>) -> Self {
        CsvRecord { fields }
    }

    /// Get the number of fields in this record
    pub fn len(&self) -> usize {
        self.fields.len()
    }

    /// Check if the record is empty
    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// Get a field by index
    pub fn get(&self, index: usize) -> Option<&String> {
        self.fields.get(index)
    }

    /// Get a mutable reference to a field by index
    pub fn get_mut(&mut self, index: usize) -> Option<&mut String> {
        self.fields.get_mut(index)
    }

    /// Set a field value by index
    pub fn set(&mut self, index: usize, value: String) -> Result<(), CsvError> {
        if index >= self.fields.len() {
            return Err(CsvError::ValidationError(format!("Index {} out of bounds", index)));
        }
        self.fields[index] = value;
        Ok(())
    }

    /// Add a field to the end of the record
    pub fn push(&mut self, field: String) {
        self.fields.push(field);
    }

    /// Remove and return the last field
    pub fn pop(&mut self) -> Option<String> {
        self.fields.pop()
    }

    /// Get all fields as a slice
    pub fn fields(&self) -> &[String] {
        &self.fields
    }

    /// Get all fields as a mutable slice
    pub fn fields_mut(&mut self) -> &mut Vec<String> {
        &mut self.fields
    }

    /// Convert the record to a vector of strings
    pub fn into_fields(self) -> Vec<String> {
        self.fields
    }

    /// Iterate over fields
    pub fn iter(&self) -> std::slice::Iter<String> {
        self.fields.iter()
    }

    /// Parse field as integer
    pub fn get_i64(&self, index: usize) -> Result<i64, CsvError> {
        match self.get(index) {
            Some(field) => field.trim().parse::<i64>()
                .map_err(|_| CsvError::ParseError(format!("Cannot parse '{}' as integer", field))),
            None => Err(CsvError::ValidationError(format!("Index {} out of bounds", index))),
        }
    }

    /// Parse field as float
    pub fn get_f64(&self, index: usize) -> Result<f64, CsvError> {
        match self.get(index) {
            Some(field) => field.trim().parse::<f64>()
                .map_err(|_| CsvError::ParseError(format!("Cannot parse '{}' as float", field))),
            None => Err(CsvError::ValidationError(format!("Index {} out of bounds", index))),
        }
    }

    /// Parse field as boolean
    pub fn get_bool(&self, index: usize) -> Result<bool, CsvError> {
        match self.get(index) {
            Some(field) => {
                let trimmed = field.trim().to_lowercase();
                match trimmed.as_str() {
                    "true" | "1" | "yes" | "y" | "on" => Ok(true),
                    "false" | "0" | "no" | "n" | "off" => Ok(false),
                    _ => Err(CsvError::ParseError(format!("Cannot parse '{}' as boolean", field))),
                }
            }
            None => Err(CsvError::ValidationError(format!("Index {} out of bounds", index))),
        }
    }
}

impl std::ops::Index<usize> for CsvRecord {
    type Output = String;

    fn index(&self, index: usize) -> &Self::Output {
        &self.fields[index]
    }
}

impl std::ops::IndexMut<usize> for CsvRecord {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.fields[index]
    }
}

/// CSV document - represents the entire CSV data
#[derive(Debug, Clone)]
pub struct CsvDocument {
    headers: Option<Vec<String>>,
    records: Vec<CsvRecord>,
}

impl CsvDocument {
    /// Create a new empty CSV document
    pub fn new() -> Self {
        CsvDocument {
            headers: None,
            records: Vec::new(),
        }
    }

    /// Create a CSV document with headers
    pub fn with_headers(headers: Vec<String>) -> Self {
        CsvDocument {
            headers: Some(headers),
            records: Vec::new(),
        }
    }

    /// Set the headers
    pub fn set_headers(&mut self, headers: Vec<String>) {
        self.headers = Some(headers);
    }

    /// Get the headers
    pub fn headers(&self) -> Option<&Vec<String>> {
        self.headers.as_ref()
    }

    /// Check if the document has headers
    pub fn has_headers(&self) -> bool {
        self.headers.is_some()
    }

    /// Get the number of records
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if the document is empty
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Add a record to the document
    pub fn add_record(&mut self, record: CsvRecord) {
        self.records.push(record);
    }

    /// Get a record by index
    pub fn get_record(&self, index: usize) -> Option<&CsvRecord> {
        self.records.get(index)
    }

    /// Get a mutable reference to a record by index
    pub fn get_record_mut(&mut self, index: usize) -> Option<&mut CsvRecord> {
        self.records.get_mut(index)
    }

    /// Get all records
    pub fn records(&self) -> &[CsvRecord] {
        &self.records
    }

    /// Get all records as mutable slice
    pub fn records_mut(&mut self) -> &mut Vec<CsvRecord> {
        &mut self.records
    }

    /// Iterate over records
    pub fn iter(&self) -> std::slice::Iter<CsvRecord> {
        self.records.iter()
    }

    /// Get a field value by record index and column name
    pub fn get_field_by_name(&self, record_index: usize, column_name: &str) -> Result<Option<&String>, CsvError> {
        if let Some(headers) = &self.headers {
            if let Some(column_index) = headers.iter().position(|h| h == column_name) {
                if let Some(record) = self.get_record(record_index) {
                    Ok(record.get(column_index))
                } else {
                    Err(CsvError::ValidationError(format!("Record index {} out of bounds", record_index)))
                }
            } else {
                Err(CsvError::ValidationError(format!("Column '{}' not found", column_name)))
            }
        } else {
            Err(CsvError::ValidationError("Document has no headers".to_string()))
        }
    }

    /// Convert to a vector of hash maps (each record as a map of column name to value)
    pub fn to_maps(&self) -> Result<Vec<HashMap<String, String>>, CsvError> {
        if let Some(headers) = &self.headers {
            let mut result = Vec::new();
            
            for record in &self.records {
                let mut map = HashMap::new();
                
                for (i, header) in headers.iter().enumerate() {
                    let value = record.get(i).cloned().unwrap_or_default();
                    map.insert(header.clone(), value);
                }
                
                result.push(map);
            }
            
            Ok(result)
        } else {
            Err(CsvError::ValidationError("Document has no headers".to_string()))
        }
    }

    /// Create a CSV document from a vector of hash maps
    pub fn from_maps(maps: Vec<HashMap<String, String>>) -> Result<Self, CsvError> {
        if maps.is_empty() {
            return Ok(CsvDocument::new());
        }

        // Extract headers from the first map
        let mut headers: Vec<String> = maps[0].keys().cloned().collect();
        headers.sort(); // Sort for consistent ordering

        let mut document = CsvDocument::with_headers(headers.clone());

        for map in maps {
            let mut record = CsvRecord::new();
            
            for header in &headers {
                let value = map.get(header).cloned().unwrap_or_default();
                record.push(value);
            }
            
            document.add_record(record);
        }

        Ok(document)
    }
}

/// CSV parser configuration
#[derive(Debug, Clone)]
pub struct CsvConfig {
    pub delimiter: char,
    pub quote_char: char,
    pub escape_char: Option<char>,
    pub has_headers: bool,
    pub skip_empty_lines: bool,
    pub trim_whitespace: bool,
}

impl Default for CsvConfig {
    fn default() -> Self {
        CsvConfig {
            delimiter: ',',
            quote_char: '"',
            escape_char: None,
            has_headers: false,
            skip_empty_lines: true,
            trim_whitespace: false,
        }
    }
}

impl CsvConfig {
    /// Create a new CSV configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the field delimiter
    pub fn delimiter(mut self, delimiter: char) -> Self {
        self.delimiter = delimiter;
        self
    }

    /// Set the quote character
    pub fn quote_char(mut self, quote_char: char) -> Self {
        self.quote_char = quote_char;
        self
    }

    /// Set the escape character
    pub fn escape_char(mut self, escape_char: Option<char>) -> Self {
        self.escape_char = escape_char;
        self
    }

    /// Set whether the first row contains headers
    pub fn has_headers(mut self, has_headers: bool) -> Self {
        self.has_headers = has_headers;
        self
    }

    /// Set whether to skip empty lines
    pub fn skip_empty_lines(mut self, skip_empty_lines: bool) -> Self {
        self.skip_empty_lines = skip_empty_lines;
        self
    }

    /// Set whether to trim whitespace from fields
    pub fn trim_whitespace(mut self, trim_whitespace: bool) -> Self {
        self.trim_whitespace = trim_whitespace;
        self
    }
}

/// CSV parser
pub struct CsvParser {
    config: CsvConfig,
}

impl CsvParser {
    /// Create a new CSV parser with default configuration
    pub fn new() -> Self {
        CsvParser {
            config: CsvConfig::default(),
        }
    }

    /// Create a CSV parser with custom configuration
    pub fn with_config(config: CsvConfig) -> Self {
        CsvParser { config }
    }

    /// Parse CSV from a string
    pub fn parse_string(&self, input: &str) -> Result<CsvDocument, CsvError> {
        let lines: Vec<&str> = input.lines().collect();
        self.parse_lines(&lines)
    }

    /// Parse CSV from a reader
    pub fn parse_reader<R: std::io::Read>(&self, reader: R) -> Result<CsvDocument, CsvError> {
        let buf_reader = BufReader::new(reader);
        let lines: Result<Vec<String>, _> = buf_reader.lines().collect();
        let lines = lines.map_err(|e| CsvError::IoError(e.to_string()))?;
        let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
        self.parse_lines(&line_refs)
    }

    fn parse_lines(&self, lines: &[&str]) -> Result<CsvDocument, CsvError> {
        let mut document = CsvDocument::new();
        let mut line_number = 0;

        for line in lines {
            line_number += 1;

            if self.config.skip_empty_lines && line.trim().is_empty() {
                continue;
            }

            let record = self.parse_line(line, line_number)?;

            if self.config.has_headers && document.records.is_empty() && document.headers.is_none() {
                // First record becomes headers
                document.set_headers(record.into_fields());
            } else {
                document.add_record(record);
            }
        }

        Ok(document)
    }

    fn parse_line(&self, line: &str, line_number: usize) -> Result<CsvRecord, CsvError> {
        let mut fields = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = line.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == self.config.quote_char {
                if in_quotes {
                    // Check for escaped quote (double quote)
                    if chars.peek() == Some(&self.config.quote_char) {
                        current_field.push(self.config.quote_char);
                        chars.next(); // Skip the second quote
                    } else {
                        in_quotes = false;
                    }
                } else {
                    in_quotes = true;
                }
            } else if ch == self.config.delimiter && !in_quotes {
                // End of field
                let field = if self.config.trim_whitespace {
                    current_field.trim().to_string()
                } else {
                    current_field
                };
                fields.push(field);
                current_field = String::new();
            } else if let Some(escape_char) = self.config.escape_char {
                if ch == escape_char && in_quotes {
                    // Handle escape character
                    if let Some(next_ch) = chars.next() {
                        match next_ch {
                            'n' => current_field.push('\n'),
                            'r' => current_field.push('\r'),
                            't' => current_field.push('\t'),
                            '\\' => current_field.push('\\'),
                            c if c == self.config.quote_char => current_field.push(c),
                            c if c == self.config.delimiter => current_field.push(c),
                            c => {
                                current_field.push(escape_char);
                                current_field.push(c);
                            }
                        }
                    } else {
                        return Err(CsvError::ParseError(format!("Unexpected end of line after escape character at line {}", line_number)));
                    }
                } else {
                    current_field.push(ch);
                }
            } else {
                current_field.push(ch);
            }
        }

        if in_quotes {
            return Err(CsvError::ParseError(format!("Unterminated quoted field at line {}", line_number)));
        }

        // Add the last field
        let field = if self.config.trim_whitespace {
            current_field.trim().to_string()
        } else {
            current_field
        };
        fields.push(field);

        Ok(CsvRecord::from_fields(fields))
    }
}

/// CSV writer/serializer
pub struct CsvWriter {
    config: CsvConfig,
}

impl CsvWriter {
    /// Create a new CSV writer with default configuration
    pub fn new() -> Self {
        CsvWriter {
            config: CsvConfig::default(),
        }
    }

    /// Create a CSV writer with custom configuration
    pub fn with_config(config: CsvConfig) -> Self {
        CsvWriter { config }
    }

    /// Write CSV document to a string
    pub fn write_string(&self, document: &CsvDocument) -> String {
        let mut result = String::new();

        // Write headers if present
        if let Some(headers) = document.headers() {
            result.push_str(&self.format_record(&CsvRecord::from_fields(headers.clone())));
            result.push('\n');
        }

        // Write records
        for record in document.records() {
            result.push_str(&self.format_record(record));
            result.push('\n');
        }

        result
    }

    /// Write CSV document to a writer
    pub fn write_writer<W: Write>(&self, document: &CsvDocument, mut writer: W) -> Result<(), CsvError> {
        // Write headers if present
        if let Some(headers) = document.headers() {
            let header_line = self.format_record(&CsvRecord::from_fields(headers.clone()));
            writer.write_all(header_line.as_bytes())
                .map_err(|e| CsvError::IoError(e.to_string()))?;
            writer.write_all(b"\n")
                .map_err(|e| CsvError::IoError(e.to_string()))?;
        }

        // Write records
        for record in document.records() {
            let record_line = self.format_record(record);
            writer.write_all(record_line.as_bytes())
                .map_err(|e| CsvError::IoError(e.to_string()))?;
            writer.write_all(b"\n")
                .map_err(|e| CsvError::IoError(e.to_string()))?;
        }

        writer.flush().map_err(|e| CsvError::IoError(e.to_string()))?;
        Ok(())
    }

    fn format_record(&self, record: &CsvRecord) -> String {
        let formatted_fields: Vec<String> = record.fields()
            .iter()
            .map(|field| self.format_field(field))
            .collect();

        formatted_fields.join(&self.config.delimiter.to_string())
    }

    fn format_field(&self, field: &str) -> String {
        let needs_quoting = field.contains(self.config.delimiter)
            || field.contains(self.config.quote_char)
            || field.contains('\n')
            || field.contains('\r')
            || (field.starts_with(' ') || field.ends_with(' '));

        if needs_quoting {
            let escaped = field.replace(
                &self.config.quote_char.to_string(),
                &format!("{}{}", self.config.quote_char, self.config.quote_char)
            );
            format!("{}{}{}", self.config.quote_char, escaped, self.config.quote_char)
        } else {
            field.to_string()
        }
    }
}

/// CSV utility functions
pub struct Csv;

impl Csv {
    /// Parse CSV from a string with default configuration
    pub fn parse(input: &str) -> Result<CsvDocument, CsvError> {
        let parser = CsvParser::new();
        parser.parse_string(input)
    }

    /// Parse CSV from a string with headers
    pub fn parse_with_headers(input: &str) -> Result<CsvDocument, CsvError> {
        let config = CsvConfig::new().has_headers(true);
        let parser = CsvParser::with_config(config);
        parser.parse_string(input)
    }

    /// Write CSV document to string
    pub fn write(document: &CsvDocument) -> String {
        let writer = CsvWriter::new();
        writer.write_string(document)
    }

    /// Create a CSV document from a 2D vector of strings
    pub fn from_vec(data: Vec<Vec<String>>) -> CsvDocument {
        let mut document = CsvDocument::new();
        
        for row in data {
            document.add_record(CsvRecord::from_fields(row));
        }
        
        document
    }

    /// Convert CSV document to a 2D vector of strings
    pub fn to_vec(document: &CsvDocument) -> Vec<Vec<String>> {
        let mut result = Vec::new();
        
        // Add headers if present
        if let Some(headers) = document.headers() {
            result.push(headers.clone());
        }
        
        // Add records
        for record in document.records() {
            result.push(record.fields().to_vec());
        }
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_record_creation() {
        let mut record = CsvRecord::new();
        assert!(record.is_empty());
        
        record.push("Alice".to_string());
        record.push("30".to_string());
        record.push("Developer".to_string());
        
        assert_eq!(record.len(), 3);
        assert_eq!(record.get(0), Some(&"Alice".to_string()));
        assert_eq!(record.get(1), Some(&"30".to_string()));
        assert_eq!(record.get(2), Some(&"Developer".to_string()));
    }

    #[test]
    fn test_csv_record_type_conversion() {
        let mut record = CsvRecord::new();
        record.push("42".to_string());
        record.push("3.14".to_string());
        record.push("true".to_string());
        record.push("false".to_string());
        
        assert_eq!(record.get_i64(0).unwrap(), 42);
        assert_eq!(record.get_f64(1).unwrap(), 3.14);
        assert_eq!(record.get_bool(2).unwrap(), true);
        assert_eq!(record.get_bool(3).unwrap(), false);
    }

    #[test]
    fn test_csv_parse_simple() {
        let csv = "Alice,30,Developer\nBob,25,Designer";
        let document = Csv::parse(csv).unwrap();
        
        assert_eq!(document.len(), 2);
        
        let first_record = document.get_record(0).unwrap();
        assert_eq!(first_record.get(0), Some(&"Alice".to_string()));
        assert_eq!(first_record.get(1), Some(&"30".to_string()));
        assert_eq!(first_record.get(2), Some(&"Developer".to_string()));
    }

    #[test]
    fn test_csv_parse_with_headers() {
        let csv = "Name,Age,Job\nAlice,30,Developer\nBob,25,Designer";
        let document = Csv::parse_with_headers(csv).unwrap();
        
        assert!(document.has_headers());
        assert_eq!(document.headers().unwrap(), &vec!["Name".to_string(), "Age".to_string(), "Job".to_string()]);
        assert_eq!(document.len(), 2);
        
        let alice_name = document.get_field_by_name(0, "Name").unwrap();
        assert_eq!(alice_name, Some(&"Alice".to_string()));
    }

    #[test]
    fn test_csv_parse_quoted_fields() {
        let csv = r#""Alice Smith","Software Developer","San Francisco, CA""#;
        let document = Csv::parse(csv).unwrap();
        
        let record = document.get_record(0).unwrap();
        assert_eq!(record.get(0), Some(&"Alice Smith".to_string()));
        assert_eq!(record.get(1), Some(&"Software Developer".to_string()));
        assert_eq!(record.get(2), Some(&"San Francisco, CA".to_string()));
    }

    #[test]
    fn test_csv_parse_escaped_quotes() {
        let csv = r#""She said ""Hello""","Normal field""#;
        let document = Csv::parse(csv).unwrap();
        
        let record = document.get_record(0).unwrap();
        assert_eq!(record.get(0), Some(&"She said \"Hello\"".to_string()));
        assert_eq!(record.get(1), Some(&"Normal field".to_string()));
    }

    #[test]
    fn test_csv_write() {
        let mut document = CsvDocument::with_headers(vec![
            "Name".to_string(),
            "Age".to_string(),
            "Job".to_string(),
        ]);
        
        let mut record1 = CsvRecord::new();
        record1.push("Alice".to_string());
        record1.push("30".to_string());
        record1.push("Developer".to_string());
        document.add_record(record1);
        
        let mut record2 = CsvRecord::new();
        record2.push("Bob".to_string());
        record2.push("25".to_string());
        record2.push("Designer".to_string());
        document.add_record(record2);
        
        let csv = Csv::write(&document);
        
        assert!(csv.contains("Name,Age,Job"));
        assert!(csv.contains("Alice,30,Developer"));
        assert!(csv.contains("Bob,25,Designer"));
    }

    #[test]
    fn test_csv_write_quoted_fields() {
        let mut document = CsvDocument::new();
        
        let mut record = CsvRecord::new();
        record.push("Alice Smith".to_string());
        record.push("Software Developer".to_string());
        record.push("San Francisco, CA".to_string());
        document.add_record(record);
        
        let csv = Csv::write(&document);
        
        // Only "San Francisco, CA" should be quoted because it contains a comma
        assert!(csv.contains("\"San Francisco, CA\""));
        // The other fields don't need quoting
        assert!(csv.contains("Alice Smith"));
        assert!(csv.contains("Software Developer"));
    }

    #[test]
    fn test_csv_custom_delimiter() {
        let csv = "Alice;30;Developer\nBob;25;Designer";
        let config = CsvConfig::new().delimiter(';');
        let parser = CsvParser::with_config(config);
        let document = parser.parse_string(csv).unwrap();
        
        let record = document.get_record(0).unwrap();
        assert_eq!(record.get(0), Some(&"Alice".to_string()));
        assert_eq!(record.get(1), Some(&"30".to_string()));
        assert_eq!(record.get(2), Some(&"Developer".to_string()));
    }

    #[test]
    fn test_csv_trim_whitespace() {
        let csv = " Alice , 30 , Developer ";
        let config = CsvConfig::new().trim_whitespace(true);
        let parser = CsvParser::with_config(config);
        let document = parser.parse_string(csv).unwrap();
        
        let record = document.get_record(0).unwrap();
        assert_eq!(record.get(0), Some(&"Alice".to_string()));
        assert_eq!(record.get(1), Some(&"30".to_string()));
        assert_eq!(record.get(2), Some(&"Developer".to_string()));
    }

    #[test]
    fn test_csv_to_maps() {
        let csv = "Name,Age,Job\nAlice,30,Developer\nBob,25,Designer";
        let document = Csv::parse_with_headers(csv).unwrap();
        let maps = document.to_maps().unwrap();
        
        assert_eq!(maps.len(), 2);
        
        let alice = &maps[0];
        assert_eq!(alice.get("Name"), Some(&"Alice".to_string()));
        assert_eq!(alice.get("Age"), Some(&"30".to_string()));
        assert_eq!(alice.get("Job"), Some(&"Developer".to_string()));
    }

    #[test]
    fn test_csv_from_maps() {
        let mut map1 = HashMap::new();
        map1.insert("Name".to_string(), "Alice".to_string());
        map1.insert("Age".to_string(), "30".to_string());
        map1.insert("Job".to_string(), "Developer".to_string());
        
        let mut map2 = HashMap::new();
        map2.insert("Name".to_string(), "Bob".to_string());
        map2.insert("Age".to_string(), "25".to_string());
        map2.insert("Job".to_string(), "Designer".to_string());
        
        let document = CsvDocument::from_maps(vec![map1, map2]).unwrap();
        
        assert!(document.has_headers());
        assert_eq!(document.len(), 2);
        
        let alice_name = document.get_field_by_name(0, "Name").unwrap();
        assert_eq!(alice_name, Some(&"Alice".to_string()));
    }

    #[test]
    fn test_csv_parse_errors() {
        // Unterminated quote
        let csv = r#""Alice,30,Developer"#;
        assert!(Csv::parse(csv).is_err());
        
        // Invalid boolean
        let mut record = CsvRecord::new();
        record.push("maybe".to_string());
        assert!(record.get_bool(0).is_err());
        
        // Invalid number
        record.push("not_a_number".to_string());
        assert!(record.get_i64(1).is_err());
    }

    #[test]
    fn test_csv_from_vec() {
        let data = vec![
            vec!["Name".to_string(), "Age".to_string(), "Job".to_string()],
            vec!["Alice".to_string(), "30".to_string(), "Developer".to_string()],
            vec!["Bob".to_string(), "25".to_string(), "Designer".to_string()],
        ];
        
        let document = Csv::from_vec(data);
        assert_eq!(document.len(), 3);
        
        let first_record = document.get_record(0).unwrap();
        assert_eq!(first_record.get(0), Some(&"Name".to_string()));
    }

    #[test]
    fn test_csv_to_vec() {
        let mut document = CsvDocument::with_headers(vec![
            "Name".to_string(),
            "Age".to_string(),
        ]);
        
        let mut record = CsvRecord::new();
        record.push("Alice".to_string());
        record.push("30".to_string());
        document.add_record(record);
        
        let vec_data = Csv::to_vec(&document);
        assert_eq!(vec_data.len(), 2); // Headers + 1 record
        assert_eq!(vec_data[0], vec!["Name".to_string(), "Age".to_string()]);
        assert_eq!(vec_data[1], vec!["Alice".to_string(), "30".to_string()]);
    }
}