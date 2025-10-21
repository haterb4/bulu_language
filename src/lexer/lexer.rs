//! Lexer implementation for the Bulu language

use crate::error::{BuluError, Result};
use super::token::{Token, TokenType, Literal, Position};
use std::collections::HashMap;

/// Lexer for tokenizing Bulu source code
pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    keywords: HashMap<String, TokenType>,
    file_path: Option<String>,
}

impl Lexer {
    /// Create a new lexer for the given input
    pub fn new(input: &str) -> Self {
        let mut keywords = HashMap::new();
        
        // Control flow keywords
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("in".to_string(), TokenType::In);
        keywords.insert("break".to_string(), TokenType::Break);
        keywords.insert("continue".to_string(), TokenType::Continue);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("match".to_string(), TokenType::Match);
        
        // Declaration keywords
        keywords.insert("let".to_string(), TokenType::Let);
        keywords.insert("const".to_string(), TokenType::Const);
        keywords.insert("func".to_string(), TokenType::Func);
        keywords.insert("struct".to_string(), TokenType::Struct);
        keywords.insert("interface".to_string(), TokenType::Interface);
        keywords.insert("type".to_string(), TokenType::Type);
        keywords.insert("as".to_string(), TokenType::As);
        
        // Literal keywords
        keywords.insert("true".to_string(), TokenType::True);
        keywords.insert("false".to_string(), TokenType::False);
        keywords.insert("null".to_string(), TokenType::Null);
        
        // Logical operator keywords
        keywords.insert("and".to_string(), TokenType::And);
        keywords.insert("or".to_string(), TokenType::Or);
        keywords.insert("not".to_string(), TokenType::Not);
        
        // Module system keywords
        keywords.insert("import".to_string(), TokenType::Import);
        keywords.insert("export".to_string(), TokenType::Export);
        keywords.insert("pub".to_string(), TokenType::Pub);
        keywords.insert("priv".to_string(), TokenType::Priv);
        
        // Error handling keywords
        keywords.insert("try".to_string(), TokenType::Try);
        keywords.insert("fail".to_string(), TokenType::Fail);
        keywords.insert("defer".to_string(), TokenType::Defer);
        
        // Concurrency keywords
        keywords.insert("async".to_string(), TokenType::Async);
        keywords.insert("await".to_string(), TokenType::Await);
        keywords.insert("run".to_string(), TokenType::Run);
        keywords.insert("chan".to_string(), TokenType::Chan);
        keywords.insert("lock".to_string(), TokenType::Lock);
        keywords.insert("select".to_string(), TokenType::Select);
        
        // Generator keyword
        keywords.insert("yield".to_string(), TokenType::Yield);
        
        // Generics keyword
        keywords.insert("where".to_string(), TokenType::Where);
        
        // Loop control keyword
        keywords.insert("step".to_string(), TokenType::Step);

        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            keywords,
            file_path: None,
        }
    }

    /// Create a new lexer with file path information
    pub fn with_file(input: &str, file_path: String) -> Self {
        let mut lexer = Self::new(input);
        lexer.file_path = Some(file_path);
        lexer
    }

    /// Tokenize the entire input and return a vector of tokens
    pub fn tokenize(&mut self) -> Result<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            if let Some(token) = self.next_token()? {
                tokens.push(token);
            }
        }
        
        tokens.push(Token::new(
            TokenType::Eof,
            "".to_string(),
            None,
            self.current_position(),
        ));
        
        Ok(tokens)
    }

    /// Get the next token from the input
    pub fn next_token(&mut self) -> Result<Option<Token>> {
        self.skip_whitespace();
        
        if self.is_at_end() {
            return Ok(None);
        }
        
        let start_pos = self.current_position();
        let ch = self.advance();
        
        let token = match ch {
            // Single character tokens
            '(' => self.make_token(TokenType::LeftParen, start_pos),
            ')' => self.make_token(TokenType::RightParen, start_pos),
            '{' => self.make_token(TokenType::LeftBrace, start_pos),
            '}' => self.make_token(TokenType::RightBrace, start_pos),
            '[' => self.make_token(TokenType::LeftBracket, start_pos),
            ']' => self.make_token(TokenType::RightBracket, start_pos),
            ',' => self.make_token(TokenType::Comma, start_pos),
            ';' => self.make_token(TokenType::Semicolon, start_pos),
            ':' => self.make_token(TokenType::Colon, start_pos),
            '?' => self.make_token(TokenType::Question, start_pos),
            '~' => self.make_token(TokenType::Tilde, start_pos),
            '^' => self.make_token(TokenType::Caret, start_pos),
            '&' => {
                if self.match_char('&') {
                    self.make_token(TokenType::LogicalAnd, start_pos)
                } else {
                    self.make_token(TokenType::Ampersand, start_pos)
                }
            }
            '|' => {
                if self.match_char('|') {
                    self.make_token(TokenType::LogicalOr, start_pos)
                } else {
                    self.make_token(TokenType::Pipe, start_pos)
                }
            }
            '%' => {
                if self.match_char('=') {
                    self.make_token(TokenType::PercentAssign, start_pos)
                } else {
                    self.make_token(TokenType::Percent, start_pos)
                }
            }
            '+' => {
                if self.match_char('=') {
                    self.make_token(TokenType::PlusAssign, start_pos)
                } else {
                    self.make_token(TokenType::Plus, start_pos)
                }
            }
            '-' => {
                if self.match_char('=') {
                    self.make_token(TokenType::MinusAssign, start_pos)
                } else if self.match_char('>') {
                    self.make_token(TokenType::RightArrow, start_pos)
                } else {
                    self.make_token(TokenType::Minus, start_pos)
                }
            }
            '*' => {
                if self.match_char('=') {
                    self.make_token(TokenType::StarAssign, start_pos)
                } else if self.match_char('*') {
                    self.make_token(TokenType::Power, start_pos)
                } else {
                    self.make_token(TokenType::Star, start_pos)
                }
            }
            '/' => {
                if self.match_char('=') {
                    self.make_token(TokenType::SlashAssign, start_pos)
                } else if self.match_char('/') {
                    self.line_comment()?;
                    return self.next_token();
                } else if self.match_char('*') {
                    // Check if it's a documentation comment (/**)
                    if self.peek() == '*' && self.peek_next() != '/' {
                        return Ok(Some(self.doc_comment(start_pos)?));
                    } else {
                        self.block_comment()?;
                        return self.next_token();
                    }
                } else {
                    self.make_token(TokenType::Slash, start_pos)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.make_token(TokenType::Equal, start_pos)
                } else if self.match_char('>') {
                    self.make_token(TokenType::FatArrow, start_pos)
                } else {
                    self.make_token(TokenType::Assign, start_pos)
                }
            }
            '!' => {
                if self.match_char('=') {
                    self.make_token(TokenType::NotEqual, start_pos)
                } else {
                    self.make_token(TokenType::Bang, start_pos)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.make_token(TokenType::LessEqual, start_pos)
                } else if self.match_char('<') {
                    self.make_token(TokenType::LeftShift, start_pos)
                } else if self.match_char('-') {
                    self.make_token(TokenType::LeftArrow, start_pos)
                } else {
                    self.make_token(TokenType::Less, start_pos)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.make_token(TokenType::GreaterEqual, start_pos)
                } else if self.match_char('>') {
                    self.make_token(TokenType::RightShift, start_pos)
                } else {
                    self.make_token(TokenType::Greater, start_pos)
                }
            }
            '.' => {
                if self.match_char('.') {
                    if self.match_char('.') {
                        self.make_token(TokenType::DotDotDot, start_pos)
                    } else if self.match_char('<') {
                        self.make_token(TokenType::DotDotLess, start_pos)
                    } else {
                        self.make_token(TokenType::DotDot, start_pos)
                    }
                } else {
                    self.make_token(TokenType::Dot, start_pos)
                }
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
                self.make_token(TokenType::Newline, start_pos)
            }
            '"' => self.string_literal(start_pos)?,
            '\'' => self.char_literal(start_pos)?,
            _ => {
                if ch.is_ascii_digit() {
                    self.number_literal(start_pos)?
                } else if ch.is_alphabetic() || ch == '_' {
                    self.identifier_or_keyword(start_pos)
                } else {
                    return Err(self.lex_error(
                        format!("Unexpected character '{}'", ch),
                        start_pos.line,
                        start_pos.column,
                    ));
                }
            }
        };
        
        Ok(Some(token))
    }

    fn current_position(&self) -> Position {
        Position::new(self.line, self.column, self.position)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn advance(&mut self) -> char {
        let ch = self.input[self.position];
        self.position += 1;
        if ch != '\n' {
            self.column += 1;
        }
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_next(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.input[self.position] != expected {
            false
        } else {
            self.advance();
            true
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() {
            match self.peek() {
                ' ' | '\r' | '\t' => {
                    self.advance();
                }
                _ => break,
            }
        }
    }

    fn make_token(&self, token_type: TokenType, position: Position) -> Token {
        let lexeme = match token_type {
            TokenType::Newline => "\n".to_string(),
            _ => token_type.to_string(),
        };
        
        Token::new(token_type, lexeme, None, position)
    }

    fn line_comment(&mut self) -> Result<()> {
        while self.peek() != '\n' && !self.is_at_end() {
            self.advance();
        }
        Ok(())
    }

    fn block_comment(&mut self) -> Result<()> {
        let mut depth = 1;
        
        while depth > 0 && !self.is_at_end() {
            if self.peek() == '/' && self.peek_next() == '*' {
                self.advance();
                self.advance();
                depth += 1;
            } else if self.peek() == '*' && self.peek_next() == '/' {
                self.advance();
                self.advance();
                depth -= 1;
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                self.advance();
            }
        }
        
        if depth > 0 {
            return Err(BuluError::LexError {
                message: "Unterminated block comment".to_string(),
                line: self.line,
                column: self.column,
                file: self.file_path.clone(),
            });
        }
        
        Ok(())
    }

    fn doc_comment(&mut self, start_pos: Position) -> Result<Token> {
        let mut content = String::new();
        
        // Skip the initial /**
        self.advance(); // skip the extra *
        
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_next() == '/' {
                self.advance(); // consume *
                self.advance(); // consume /
                break;
            } else {
                if self.peek() == '\n' {
                    self.line += 1;
                    self.column = 1;
                }
                content.push(self.advance());
            }
        }
        
        Ok(Token::new(
            TokenType::DocComment,
            content,
            None,
            start_pos,
        ))
    }

    fn string_literal(&mut self, start_pos: Position) -> Result<Token> {
        let mut value = String::new();
        
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            
            if self.peek() == '\\' {
                self.advance(); // consume backslash
                match self.peek() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    '0' => value.push('\0'),
                    _ => {
                        return Err(BuluError::LexError {
                            message: format!("Invalid escape sequence '\\{}'", self.peek()),
                            file: None,
                            line: self.line,
                            column: self.column,
                        });
                    }
                }
                self.advance();
            } else {
                value.push(self.advance());
            }
        }
        
        if self.is_at_end() {
            return Err(BuluError::LexError {
                message: "Unterminated string".to_string(),
                file: None,
                line: start_pos.line,
                column: start_pos.column,
            });
        }
        
        self.advance(); // consume closing quote
        
        Ok(Token::new(
            TokenType::StringLiteral,
            format!("\"{}\"", value),
            Some(Literal::String(value)),
            start_pos,
        ))
    }

    fn char_literal(&mut self, start_pos: Position) -> Result<Token> {
        if self.is_at_end() {
            return Err(BuluError::LexError {
                message: "Unterminated character literal".to_string(),
                file: None,
                line: start_pos.line,
                column: start_pos.column,
            });
        }
        
        let ch = if self.peek() == '\\' {
            self.advance(); // consume backslash
            match self.peek() {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                '\\' => '\\',
                '"' => '"',
                '\'' => '\'',
                '0' => '\0',
                _ => {
                    return Err(BuluError::LexError {
                        message: format!("Invalid escape sequence '\\{}'", self.peek()),
                        file: None,
                        line: self.line,
                        column: self.column,
                    });
                }
            }
        } else {
            self.peek()
        };
        
        self.advance(); // consume character
        
        if self.peek() != '\'' {
            return Err(BuluError::LexError {
                message: "Unterminated character literal".to_string(),
                file: None,
                line: start_pos.line,
                column: start_pos.column,
            });
        }
        
        self.advance(); // consume closing quote
        
        Ok(Token::new(
            TokenType::CharLiteral,
            format!("'{}'", ch),
            Some(Literal::Char(ch)),
            start_pos,
        ))
    }

    fn number_literal(&mut self, start_pos: Position) -> Result<Token> {
        // Go back one position since we already consumed the first digit
        self.position -= 1;
        self.column -= 1;
        
        let mut value = String::new();
        let mut is_float = false;
        
        // Handle different number bases
        if self.peek() == '0' && !self.is_at_end() {
            match self.peek_next() {
                'x' | 'X' => return self.hex_number(start_pos),
                'o' | 'O' => return self.octal_number(start_pos),
                'b' | 'B' => return self.binary_number(start_pos),
                _ => {}
            }
        }
        
        // Decimal number
        while self.peek().is_ascii_digit() {
            value.push(self.advance());
        }
        
        // Check for decimal point
        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            is_float = true;
            value.push(self.advance()); // consume '.'
            
            while self.peek().is_ascii_digit() {
                value.push(self.advance());
            }
        }
        
        // Check for scientific notation
        if self.peek() == 'e' || self.peek() == 'E' {
            is_float = true;
            value.push(self.advance());
            
            if self.peek() == '+' || self.peek() == '-' {
                value.push(self.advance());
            }
            
            if !self.peek().is_ascii_digit() {
                return Err(BuluError::LexError {
                    message: "Invalid number format".to_string(),
                    file: None,
                    line: start_pos.line,
                    column: start_pos.column,
                });
            }
            
            while self.peek().is_ascii_digit() {
                value.push(self.advance());
            }
        }
        
        if is_float {
            let float_val: f64 = value.parse().map_err(|_| BuluError::LexError {
                file: None,
                message: "Invalid float literal".to_string(),
                line: start_pos.line,
                column: start_pos.column,
            })?;
            
            Ok(Token::new(
                TokenType::FloatLiteral,
                value,
                Some(Literal::Float(float_val)),
                start_pos,
            ))
        } else {
            let int_val: i64 = value.parse().map_err(|_| BuluError::LexError {
                file: None,
                message: "Invalid integer literal".to_string(),
                line: start_pos.line,
                column: start_pos.column,
            })?;
            
            Ok(Token::new(
                TokenType::IntegerLiteral,
                value,
                Some(Literal::Integer(int_val)),
                start_pos,
            ))
        }
    }

    fn hex_number(&mut self, start_pos: Position) -> Result<Token> {
        let mut value = String::new();
        self.advance(); // consume '0'
        self.advance(); // consume 'x' or 'X'
        
        while self.peek().is_ascii_hexdigit() {
            value.push(self.advance());
        }
        
        if value.is_empty() {
            return Err(BuluError::LexError {
                message: "Invalid hexadecimal number".to_string(),
                file: None,
                line: start_pos.line,
                column: start_pos.column,
            });
        }
        
        let int_val = i64::from_str_radix(&value, 16).map_err(|_| BuluError::LexError {
            message: "Invalid hexadecimal literal".to_string(),
            file: None,
            line: start_pos.line,
            column: start_pos.column,
        })?;
        
        Ok(Token::new(
            TokenType::IntegerLiteral,
            format!("0x{}", value),
            Some(Literal::Integer(int_val)),
            start_pos,
        ))
    }

    fn octal_number(&mut self, start_pos: Position) -> Result<Token> {
        let mut value = String::new();
        self.advance(); // consume '0'
        self.advance(); // consume 'o' or 'O'
        
        while self.peek().is_digit(8) {
            value.push(self.advance());
        }
        
        if value.is_empty() {
            return Err(BuluError::LexError {
                message: "Invalid octal number".to_string(),
                file: None,
                line: start_pos.line,
                column: start_pos.column,
            });
        }
        
        let int_val = i64::from_str_radix(&value, 8).map_err(|_| BuluError::LexError {
            message: "Invalid octal literal".to_string(),
            file: None,
            line: start_pos.line,
            column: start_pos.column,
        })?;
        
        Ok(Token::new(
            TokenType::IntegerLiteral,
            format!("0o{}", value),
            Some(Literal::Integer(int_val)),
            start_pos,
        ))
    }

    fn binary_number(&mut self, start_pos: Position) -> Result<Token> {
        let mut value = String::new();
        self.advance(); // consume '0'
        self.advance(); // consume 'b' or 'B'
        
        while self.peek() == '0' || self.peek() == '1' {
            value.push(self.advance());
        }
        
        if value.is_empty() {
            return Err(BuluError::LexError {
                message: "Invalid binary number".to_string(),
                file: None,
                line: start_pos.line,
                column: start_pos.column,
            });
        }
        
        let int_val = i64::from_str_radix(&value, 2).map_err(|_| BuluError::LexError {
            message: "Invalid binary literal".to_string(),
            file: None,
            line: start_pos.line,
            column: start_pos.column,
        })?;
        
        Ok(Token::new(
            TokenType::IntegerLiteral,
            format!("0b{}", value),
            Some(Literal::Integer(int_val)),
            start_pos,
        ))
    }

    /// Create a lexer error with file information
    fn lex_error(&self, message: String, line: usize, column: usize) -> BuluError {
        BuluError::lex_error(message, line, column, self.file_path.clone())
    }

    fn identifier_or_keyword(&mut self, start_pos: Position) -> Token {
        // Go back one position since we already consumed the first character
        self.position -= 1;
        self.column -= 1;
        
        let mut value = String::new();
        
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_') {
            value.push(self.advance());
        }
        
        let token_type = self.keywords.get(&value)
            .copied()
            .unwrap_or(TokenType::Identifier);
        
        let literal = match token_type {
            TokenType::True => Some(Literal::Boolean(true)),
            TokenType::False => Some(Literal::Boolean(false)),
            _ => None,
        };
        
        Token::new(token_type, value, literal, start_pos)
    }
}