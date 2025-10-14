// XML processing functionality for the Bulu programming language
// Requirements: 7.3.2

use std::collections::HashMap;
use std::fmt;

/// XML node types
#[derive(Debug, Clone, PartialEq)]
pub enum XmlNode {
    Element {
        name: String,
        attributes: HashMap<String, String>,
        children: Vec<XmlNode>,
    },
    Text(String),
    Comment(String),
    ProcessingInstruction {
        target: String,
        data: String,
    },
    Declaration {
        version: String,
        encoding: Option<String>,
        standalone: Option<bool>,
    },
}

impl XmlNode {
    /// Create a new element node
    pub fn element(name: String) -> Self {
        XmlNode::Element {
            name,
            attributes: HashMap::new(),
            children: Vec::new(),
        }
    }

    /// Create a new text node
    pub fn text(content: String) -> Self {
        XmlNode::Text(content)
    }

    /// Create a new comment node
    pub fn comment(content: String) -> Self {
        XmlNode::Comment(content)
    }

    /// Create a new processing instruction node
    pub fn processing_instruction(target: String, data: String) -> Self {
        XmlNode::ProcessingInstruction { target, data }
    }

    /// Create a new XML declaration node
    pub fn declaration(version: String, encoding: Option<String>, standalone: Option<bool>) -> Self {
        XmlNode::Declaration {
            version,
            encoding,
            standalone,
        }
    }

    /// Check if this is an element node
    pub fn is_element(&self) -> bool {
        matches!(self, XmlNode::Element { .. })
    }

    /// Check if this is a text node
    pub fn is_text(&self) -> bool {
        matches!(self, XmlNode::Text(_))
    }

    /// Check if this is a comment node
    pub fn is_comment(&self) -> bool {
        matches!(self, XmlNode::Comment(_))
    }

    /// Get the element name (if this is an element)
    pub fn name(&self) -> Option<&str> {
        match self {
            XmlNode::Element { name, .. } => Some(name),
            _ => None,
        }
    }

    /// Get the text content (if this is a text node)
    pub fn text_content(&self) -> Option<&str> {
        match self {
            XmlNode::Text(content) => Some(content),
            _ => None,
        }
    }

    /// Get attributes (if this is an element)
    pub fn attributes(&self) -> Option<&HashMap<String, String>> {
        match self {
            XmlNode::Element { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    /// Get mutable attributes (if this is an element)
    pub fn attributes_mut(&mut self) -> Option<&mut HashMap<String, String>> {
        match self {
            XmlNode::Element { attributes, .. } => Some(attributes),
            _ => None,
        }
    }

    /// Get children (if this is an element)
    pub fn children(&self) -> Option<&Vec<XmlNode>> {
        match self {
            XmlNode::Element { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Get mutable children (if this is an element)
    pub fn children_mut(&mut self) -> Option<&mut Vec<XmlNode>> {
        match self {
            XmlNode::Element { children, .. } => Some(children),
            _ => None,
        }
    }

    /// Add an attribute to an element
    pub fn set_attribute(&mut self, name: String, value: String) -> Result<(), XmlError> {
        match self {
            XmlNode::Element { attributes, .. } => {
                attributes.insert(name, value);
                Ok(())
            }
            _ => Err(XmlError::TypeError("Cannot set attribute on non-element node".to_string())),
        }
    }

    /// Get an attribute value
    pub fn get_attribute(&self, name: &str) -> Option<&String> {
        match self {
            XmlNode::Element { attributes, .. } => attributes.get(name),
            _ => None,
        }
    }

    /// Add a child node to an element
    pub fn add_child(&mut self, child: XmlNode) -> Result<(), XmlError> {
        match self {
            XmlNode::Element { children, .. } => {
                children.push(child);
                Ok(())
            }
            _ => Err(XmlError::TypeError("Cannot add child to non-element node".to_string())),
        }
    }

    /// Find the first child element with the given name
    pub fn find_child(&self, name: &str) -> Option<&XmlNode> {
        match self {
            XmlNode::Element { children, .. } => {
                children.iter().find(|child| {
                    matches!(child, XmlNode::Element { name: child_name, .. } if child_name == name)
                })
            }
            _ => None,
        }
    }

    /// Find all child elements with the given name
    pub fn find_children(&self, name: &str) -> Vec<&XmlNode> {
        match self {
            XmlNode::Element { children, .. } => {
                children.iter().filter(|child| {
                    matches!(child, XmlNode::Element { name: child_name, .. } if child_name == name)
                }).collect()
            }
            _ => Vec::new(),
        }
    }

    /// Get the text content of this node and all its descendants
    pub fn inner_text(&self) -> String {
        match self {
            XmlNode::Element { children, .. } => {
                children.iter().map(|child| child.inner_text()).collect::<Vec<_>>().join("")
            }
            XmlNode::Text(content) => content.clone(),
            _ => String::new(),
        }
    }
}

/// XML parsing and processing errors
#[derive(Debug, Clone, PartialEq)]
pub enum XmlError {
    ParseError(String),
    TypeError(String),
    ValidationError(String),
}

impl fmt::Display for XmlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XmlError::ParseError(msg) => write!(f, "XML Parse Error: {}", msg),
            XmlError::TypeError(msg) => write!(f, "XML Type Error: {}", msg),
            XmlError::ValidationError(msg) => write!(f, "XML Validation Error: {}", msg),
        }
    }
}

impl std::error::Error for XmlError {}

/// XML document structure
#[derive(Debug, Clone)]
pub struct XmlDocument {
    pub declaration: Option<XmlNode>,
    pub root: Option<XmlNode>,
    pub processing_instructions: Vec<XmlNode>,
}

impl XmlDocument {
    pub fn new() -> Self {
        XmlDocument {
            declaration: None,
            root: None,
            processing_instructions: Vec::new(),
        }
    }

    pub fn with_root(root: XmlNode) -> Self {
        XmlDocument {
            declaration: None,
            root: Some(root),
            processing_instructions: Vec::new(),
        }
    }

    pub fn set_declaration(&mut self, declaration: XmlNode) {
        self.declaration = Some(declaration);
    }

    pub fn set_root(&mut self, root: XmlNode) {
        self.root = Some(root);
    }

    pub fn add_processing_instruction(&mut self, pi: XmlNode) {
        self.processing_instructions.push(pi);
    }
}

/// XML parser
pub struct XmlParser {
    input: Vec<char>,
    position: usize,
}

impl XmlParser {
    pub fn new(input: &str) -> Self {
        XmlParser {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn parse(&mut self) -> Result<XmlDocument, XmlError> {
        let mut document = XmlDocument::new();
        
        self.skip_whitespace();
        
        // Check for empty input
        if self.position >= self.input.len() {
            return Err(XmlError::ParseError("Empty document".to_string()));
        }
        
        // Parse XML declaration if present
        if self.peek_string("<?xml") {
            document.declaration = Some(self.parse_declaration()?);
            self.skip_whitespace();
        }
        
        // Parse processing instructions and comments before root element
        while self.position < self.input.len() {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                break;
            }
            
            if self.peek_string("<?") && !self.peek_string("<?xml ") && !self.peek_string("<?xml\t") && !self.peek_string("<?xml\n") && !self.peek_string("<?xml\r") {
                document.processing_instructions.push(self.parse_processing_instruction()?);
            } else if self.peek_string("<!--") {
                let _comment = self.parse_comment()?; // Skip comments at document level
            } else if self.current_char() == '<' {
                // Found root element
                document.root = Some(self.parse_element()?);
                break;
            } else {
                return Err(XmlError::ParseError("Unexpected content before root element".to_string()));
            }
        }
        
        // Check if we have a root element
        if document.root.is_none() {
            return Err(XmlError::ParseError("No root element found".to_string()));
        }
        
        // Skip any trailing whitespace and comments
        self.skip_whitespace();
        while self.position < self.input.len() && self.peek_string("<!--") {
            let _comment = self.parse_comment()?;
            self.skip_whitespace();
        }
        
        if self.position < self.input.len() {
            return Err(XmlError::ParseError("Unexpected content after root element".to_string()));
        }
        
        Ok(document)
    }

    fn parse_declaration(&mut self) -> Result<XmlNode, XmlError> {
        if !self.consume_string("<?xml") {
            return Err(XmlError::ParseError("Expected '<?xml' at start of declaration".to_string()));
        }
        
        self.skip_whitespace();
        
        let mut version = None;
        let mut encoding = None;
        let mut standalone = None;
        
        // Parse attributes
        while self.position < self.input.len() && self.current_char() != '?' {
            let (name, value) = self.parse_attribute()?;
            
            match name.as_str() {
                "version" => version = Some(value),
                "encoding" => encoding = Some(value),
                "standalone" => {
                    standalone = Some(value == "yes");
                }
                _ => return Err(XmlError::ParseError(format!("Unknown declaration attribute: {}", name))),
            }
            
            self.skip_whitespace();
        }
        
        if !self.consume_string("?>") {
            return Err(XmlError::ParseError("Expected '?>' at end of declaration".to_string()));
        }
        
        let version = version.ok_or_else(|| XmlError::ParseError("Missing version in XML declaration".to_string()))?;
        
        Ok(XmlNode::declaration(version, encoding, standalone))
    }

    fn parse_processing_instruction(&mut self) -> Result<XmlNode, XmlError> {
        if !self.consume_string("<?") {
            return Err(XmlError::ParseError("Expected '<?' at start of processing instruction".to_string()));
        }
        
        let target = self.parse_name()?;
        self.skip_whitespace();
        
        let mut data = String::new();
        while self.position < self.input.len() && !self.peek_string("?>") {
            data.push(self.current_char());
            self.advance();
        }
        
        if !self.consume_string("?>") {
            return Err(XmlError::ParseError("Expected '?>' at end of processing instruction".to_string()));
        }
        
        Ok(XmlNode::processing_instruction(target, data.trim().to_string()))
    }

    fn parse_element(&mut self) -> Result<XmlNode, XmlError> {
        if self.current_char() != '<' {
            return Err(XmlError::ParseError("Expected '<' at start of element".to_string()));
        }
        
        self.advance(); // Skip '<'
        
        let name = self.parse_name()?;
        let mut attributes = HashMap::new();
        
        // Parse attributes
        loop {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                return Err(XmlError::ParseError("Unexpected end of input in element".to_string()));
            }
            
            if self.current_char() == '>' {
                // End of opening tag
                self.advance();
                break;
            } else if self.peek_string("/>") {
                // Self-closing tag
                self.advance();
                self.advance();
                return Ok(XmlNode::Element {
                    name,
                    attributes,
                    children: Vec::new(),
                });
            } else {
                // Parse attribute
                let (attr_name, attr_value) = self.parse_attribute()?;
                attributes.insert(attr_name, attr_value);
            }
        }
        
        // Parse children
        let mut children = Vec::new();
        
        loop {
            self.skip_whitespace();
            
            if self.position >= self.input.len() {
                return Err(XmlError::ParseError("Unexpected end of input in element content".to_string()));
            }
            
            if self.peek_string("</") {
                // End tag
                self.advance();
                self.advance();
                let end_name = self.parse_name()?;
                
                if end_name != name {
                    return Err(XmlError::ParseError(format!("Mismatched end tag: expected '{}', found '{}'", name, end_name)));
                }
                
                self.skip_whitespace();
                if self.current_char() != '>' {
                    return Err(XmlError::ParseError("Expected '>' at end of closing tag".to_string()));
                }
                self.advance();
                break;
            } else if self.peek_string("<!--") {
                children.push(self.parse_comment()?);
            } else if self.peek_string("<?") {
                children.push(self.parse_processing_instruction()?);
            } else if self.current_char() == '<' {
                children.push(self.parse_element()?);
            } else {
                // Text content
                let text = self.parse_text()?;
                if !text.trim().is_empty() {
                    children.push(XmlNode::Text(text));
                }
            }
        }
        
        Ok(XmlNode::Element {
            name,
            attributes,
            children,
        })
    }

    fn parse_comment(&mut self) -> Result<XmlNode, XmlError> {
        if !self.consume_string("<!--") {
            return Err(XmlError::ParseError("Expected '<!--' at start of comment".to_string()));
        }
        
        let mut content = String::new();
        
        while self.position < self.input.len() && !self.peek_string("-->") {
            content.push(self.current_char());
            self.advance();
        }
        
        if !self.consume_string("-->") {
            return Err(XmlError::ParseError("Expected '-->' at end of comment".to_string()));
        }
        
        Ok(XmlNode::Comment(content))
    }

    fn parse_text(&mut self) -> Result<String, XmlError> {
        let mut text = String::new();
        
        while self.position < self.input.len() && self.current_char() != '<' {
            if self.current_char() == '&' {
                text.push_str(&self.parse_entity()?);
            } else {
                text.push(self.current_char());
                self.advance();
            }
        }
        
        Ok(text)
    }

    fn parse_entity(&mut self) -> Result<String, XmlError> {
        if self.current_char() != '&' {
            return Err(XmlError::ParseError("Expected '&' at start of entity".to_string()));
        }
        
        self.advance(); // Skip '&'
        
        let mut entity_name = String::new();
        while self.position < self.input.len() && self.current_char() != ';' {
            entity_name.push(self.current_char());
            self.advance();
        }
        
        if self.position >= self.input.len() || self.current_char() != ';' {
            return Err(XmlError::ParseError("Expected ';' at end of entity".to_string()));
        }
        
        self.advance(); // Skip ';'
        
        // Handle common entities
        match entity_name.as_str() {
            "lt" => Ok("<".to_string()),
            "gt" => Ok(">".to_string()),
            "amp" => Ok("&".to_string()),
            "quot" => Ok("\"".to_string()),
            "apos" => Ok("'".to_string()),
            _ => {
                // Handle numeric entities
                if entity_name.starts_with('#') {
                    let num_str = &entity_name[1..];
                    if let Ok(code_point) = if num_str.starts_with('x') {
                        u32::from_str_radix(&num_str[1..], 16)
                    } else {
                        num_str.parse::<u32>()
                    } {
                        if let Some(ch) = char::from_u32(code_point) {
                            Ok(ch.to_string())
                        } else {
                            Err(XmlError::ParseError(format!("Invalid character code: {}", code_point)))
                        }
                    } else {
                        Err(XmlError::ParseError(format!("Invalid numeric entity: {}", entity_name)))
                    }
                } else {
                    // Unknown entity - return as-is for now
                    Ok(format!("&{};", entity_name))
                }
            }
        }
    }

    fn parse_attribute(&mut self) -> Result<(String, String), XmlError> {
        let name = self.parse_name()?;
        
        self.skip_whitespace();
        
        if self.position >= self.input.len() || self.current_char() != '=' {
            return Err(XmlError::ParseError("Expected '=' after attribute name".to_string()));
        }
        
        self.advance(); // Skip '='
        self.skip_whitespace();
        
        let value = self.parse_attribute_value()?;
        
        Ok((name, value))
    }

    fn parse_attribute_value(&mut self) -> Result<String, XmlError> {
        if self.position >= self.input.len() {
            return Err(XmlError::ParseError("Expected attribute value".to_string()));
        }
        
        let quote_char = self.current_char();
        if quote_char != '"' && quote_char != '\'' {
            return Err(XmlError::ParseError("Attribute value must be quoted".to_string()));
        }
        
        self.advance(); // Skip opening quote
        
        let mut value = String::new();
        
        while self.position < self.input.len() && self.current_char() != quote_char {
            if self.current_char() == '&' {
                value.push_str(&self.parse_entity()?);
            } else {
                value.push(self.current_char());
                self.advance();
            }
        }
        
        if self.position >= self.input.len() {
            return Err(XmlError::ParseError("Unterminated attribute value".to_string()));
        }
        
        self.advance(); // Skip closing quote
        
        Ok(value)
    }

    fn parse_name(&mut self) -> Result<String, XmlError> {
        if self.position >= self.input.len() {
            return Err(XmlError::ParseError("Unexpected end of input while parsing name".to_string()));
        }
        
        let ch = self.input[self.position];
        if !is_name_start_char(ch) {
            return Err(XmlError::ParseError(format!("Invalid name start character: '{}'", ch)));
        }
        
        let mut name = String::new();
        
        while self.position < self.input.len() {
            let ch = self.input[self.position];
            if !is_name_char(ch) {
                break;
            }
            name.push(ch);
            self.advance();
        }
        
        Ok(name)
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

    fn peek_string(&self, s: &str) -> bool {
        let chars: Vec<char> = s.chars().collect();
        
        if self.position + chars.len() > self.input.len() {
            return false;
        }
        
        for (i, &ch) in chars.iter().enumerate() {
            if self.input[self.position + i] != ch {
                return false;
            }
        }
        
        true
    }

    fn consume_string(&mut self, s: &str) -> bool {
        if self.peek_string(s) {
            self.position += s.chars().count();
            true
        } else {
            false
        }
    }
}

/// XML serializer
pub struct XmlSerializer {
    pretty: bool,
    indent: String,
}

impl XmlSerializer {
    pub fn new() -> Self {
        XmlSerializer {
            pretty: false,
            indent: "  ".to_string(),
        }
    }

    pub fn pretty() -> Self {
        XmlSerializer {
            pretty: true,
            indent: "  ".to_string(),
        }
    }

    pub fn with_indent(indent: String) -> Self {
        XmlSerializer {
            pretty: true,
            indent,
        }
    }

    pub fn serialize_document(&self, document: &XmlDocument) -> String {
        let mut result = String::new();
        
        // Serialize declaration
        if let Some(ref declaration) = document.declaration {
            result.push_str(&self.serialize_node(declaration, 0));
            if self.pretty {
                result.push('\n');
            }
        }
        
        // Serialize processing instructions
        for pi in &document.processing_instructions {
            result.push_str(&self.serialize_node(pi, 0));
            if self.pretty {
                result.push('\n');
            }
        }
        
        // Serialize root element
        if let Some(ref root) = document.root {
            result.push_str(&self.serialize_node(root, 0));
        }
        
        result
    }

    pub fn serialize_node(&self, node: &XmlNode, depth: usize) -> String {
        match node {
            XmlNode::Element { name, attributes, children } => {
                self.serialize_element(name, attributes, children, depth)
            }
            XmlNode::Text(content) => escape_text(content),
            XmlNode::Comment(content) => format!("<!--{}-->", content),
            XmlNode::ProcessingInstruction { target, data } => {
                if data.is_empty() {
                    format!("<?{} ?>", target)
                } else {
                    format!("<?{} {}?>", target, data)
                }
            }
            XmlNode::Declaration { version, encoding, standalone } => {
                let mut decl = format!("<?xml version=\"{}\"", version);
                if let Some(ref enc) = encoding {
                    decl.push_str(&format!(" encoding=\"{}\"", enc));
                }
                if let Some(sa) = standalone {
                    decl.push_str(&format!(" standalone=\"{}\"", if *sa { "yes" } else { "no" }));
                }
                decl.push_str("?>");
                decl
            }
        }
    }

    fn serialize_element(&self, name: &str, attributes: &HashMap<String, String>, children: &[XmlNode], depth: usize) -> String {
        let mut result = String::new();
        
        if self.pretty && depth > 0 {
            result.push_str(&self.indent.repeat(depth));
        }
        
        result.push('<');
        result.push_str(name);
        
        // Sort attributes for consistent output
        let mut attr_keys: Vec<&String> = attributes.keys().collect();
        attr_keys.sort();
        
        for key in attr_keys {
            if let Some(value) = attributes.get(key) {
                result.push_str(&format!(" {}=\"{}\"", key, escape_attribute_value(value)));
            }
        }
        
        if children.is_empty() {
            result.push_str("/>");
        } else {
            result.push('>');
            
            let has_element_children = children.iter().any(|child| child.is_element());
            
            if self.pretty && has_element_children {
                result.push('\n');
            }
            
            for child in children {
                if self.pretty && child.is_element() {
                    result.push_str(&self.serialize_node(child, depth + 1));
                    result.push('\n');
                } else {
                    result.push_str(&self.serialize_node(child, depth + 1));
                }
            }
            
            if self.pretty && has_element_children {
                result.push_str(&self.indent.repeat(depth));
            }
            
            result.push_str(&format!("</{}>", name));
        }
        
        result
    }
}

/// XML utility functions
pub struct Xml;

impl Xml {
    /// Parse an XML string into a document
    pub fn parse(input: &str) -> Result<XmlDocument, XmlError> {
        let mut parser = XmlParser::new(input);
        parser.parse()
    }

    /// Serialize a document to XML string
    pub fn stringify(document: &XmlDocument) -> String {
        let serializer = XmlSerializer::new();
        serializer.serialize_document(document)
    }

    /// Serialize a document to pretty-printed XML string
    pub fn stringify_pretty(document: &XmlDocument) -> String {
        let serializer = XmlSerializer::pretty();
        serializer.serialize_document(document)
    }

    /// Serialize a single node to XML string
    pub fn stringify_node(node: &XmlNode) -> String {
        let serializer = XmlSerializer::new();
        serializer.serialize_node(node, 0)
    }
}

// Helper functions
fn is_name_start_char(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == ':'
}

fn is_name_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == ':' || ch == '-' || ch == '.'
}

fn escape_text(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_attribute_value(value: &str) -> String {
    value.replace('&', "&amp;")
         .replace('<', "&lt;")
         .replace('>', "&gt;")
         .replace('"', "&quot;")
         .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xml_node_creation() {
        let element = XmlNode::element("root".to_string());
        assert!(element.is_element());
        assert_eq!(element.name(), Some("root"));

        let text = XmlNode::text("Hello, World!".to_string());
        assert!(text.is_text());
        assert_eq!(text.text_content(), Some("Hello, World!"));

        let comment = XmlNode::comment("This is a comment".to_string());
        assert!(comment.is_comment());
    }

    #[test]
    fn test_xml_parse_simple_element() {
        let xml = "<root>Hello, World!</root>";
        let document = Xml::parse(xml).unwrap();
        
        let root = document.root.unwrap();
        assert_eq!(root.name(), Some("root"));
        
        let children = root.children().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].text_content(), Some("Hello, World!"));
    }

    #[test]
    fn test_xml_parse_with_attributes() {
        let xml = r#"<person name="Alice" age="30">Developer</person>"#;
        let document = Xml::parse(xml).unwrap();
        
        let root = document.root.unwrap();
        assert_eq!(root.name(), Some("person"));
        assert_eq!(root.get_attribute("name"), Some(&"Alice".to_string()));
        assert_eq!(root.get_attribute("age"), Some(&"30".to_string()));
        
        let children = root.children().unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].text_content(), Some("Developer"));
    }

    #[test]
    fn test_xml_parse_nested_elements() {
        let xml = r#"
            <users>
                <user id="1">
                    <name>Alice</name>
                    <email>alice@example.com</email>
                </user>
                <user id="2">
                    <name>Bob</name>
                    <email>bob@example.com</email>
                </user>
            </users>
        "#;
        
        let document = Xml::parse(xml).unwrap();
        let root = document.root.unwrap();
        
        assert_eq!(root.name(), Some("users"));
        
        let users = root.find_children("user");
        assert_eq!(users.len(), 2);
        
        let alice = users[0];
        assert_eq!(alice.get_attribute("id"), Some(&"1".to_string()));
        assert_eq!(alice.find_child("name").unwrap().inner_text(), "Alice");
        assert_eq!(alice.find_child("email").unwrap().inner_text(), "alice@example.com");
    }

    #[test]
    fn test_xml_parse_self_closing_tag() {
        let xml = r#"<config><setting name="debug" value="true"/></config>"#;
        let document = Xml::parse(xml).unwrap();
        
        let root = document.root.unwrap();
        let setting = root.find_child("setting").unwrap();
        
        assert_eq!(setting.get_attribute("name"), Some(&"debug".to_string()));
        assert_eq!(setting.get_attribute("value"), Some(&"true".to_string()));
        assert_eq!(setting.children().unwrap().len(), 0);
    }

    #[test]
    fn test_xml_parse_with_declaration() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?><root>Content</root>"#;
        let document = Xml::parse(xml).unwrap();
        
        assert!(document.declaration.is_some());
        let decl = document.declaration.unwrap();
        
        match decl {
            XmlNode::Declaration { version, encoding, standalone } => {
                assert_eq!(version, "1.0");
                assert_eq!(encoding, Some("UTF-8".to_string()));
                assert_eq!(standalone, None);
            }
            _ => panic!("Expected declaration node"),
        }
    }

    #[test]
    fn test_xml_parse_with_comments() {
        let xml = r#"
            <!-- This is a comment -->
            <root>
                <!-- Another comment -->
                <child>Content</child>
            </root>
        "#;
        
        let document = Xml::parse(xml).unwrap();
        let root = document.root.unwrap();
        
        let children = root.children().unwrap();
        assert!(children.iter().any(|child| child.is_comment()));
    }

    #[test]
    fn test_xml_serialize() {
        let mut root = XmlNode::element("person".to_string());
        root.set_attribute("name".to_string(), "Alice".to_string()).unwrap();
        root.add_child(XmlNode::text("Developer".to_string())).unwrap();
        
        let document = XmlDocument::with_root(root);
        let xml = Xml::stringify(&document);
        
        assert!(xml.contains("<person"));
        assert!(xml.contains("name=\"Alice\""));
        assert!(xml.contains("Developer"));
        assert!(xml.contains("</person>"));
    }

    #[test]
    fn test_xml_serialize_pretty() {
        let mut root = XmlNode::element("users".to_string());
        
        let mut user1 = XmlNode::element("user".to_string());
        user1.set_attribute("id".to_string(), "1".to_string()).unwrap();
        user1.add_child(XmlNode::text("Alice".to_string())).unwrap();
        
        let mut user2 = XmlNode::element("user".to_string());
        user2.set_attribute("id".to_string(), "2".to_string()).unwrap();
        user2.add_child(XmlNode::text("Bob".to_string())).unwrap();
        
        root.add_child(user1).unwrap();
        root.add_child(user2).unwrap();
        
        let document = XmlDocument::with_root(root);
        let xml = Xml::stringify_pretty(&document);
        
        assert!(xml.contains("  <user"));
        assert!(xml.contains("\n"));
    }

    #[test]
    fn test_xml_entity_parsing() {
        let xml = "<root>Hello &lt;world&gt; &amp; &quot;friends&quot;</root>";
        let document = Xml::parse(xml).unwrap();
        
        let root = document.root.unwrap();
        let text_content = root.inner_text();
        assert_eq!(text_content, "Hello <world> & \"friends\"");
    }

    #[test]
    fn test_xml_parse_errors() {
        assert!(Xml::parse("<root>").is_err()); // Unclosed tag
        assert!(Xml::parse("<root></other>").is_err()); // Mismatched tags
        assert!(Xml::parse("<root attr=value>").is_err()); // Unquoted attribute
        assert!(Xml::parse("").is_err()); // Empty document
    }

    #[test]
    fn test_xml_node_manipulation() {
        let mut root = XmlNode::element("root".to_string());
        
        root.set_attribute("version".to_string(), "1.0".to_string()).unwrap();
        assert_eq!(root.get_attribute("version"), Some(&"1.0".to_string()));
        
        let child = XmlNode::element("child".to_string());
        root.add_child(child).unwrap();
        
        assert_eq!(root.children().unwrap().len(), 1);
        assert_eq!(root.find_child("child").unwrap().name(), Some("child"));
    }
}