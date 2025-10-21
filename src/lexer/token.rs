//! Token definitions for the Bulu language

use std::fmt;

/// Position information for tokens
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize,
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Self {
            line,
            column,
            offset,
        }
    }
}

/// Token with position information
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub position: Position,
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: String,
        literal: Option<Literal>,
        position: Position,
    ) -> Self {
        Self {
            token_type,
            lexeme,
            literal,
            position,
        }
    }
}

/// Literal values that can be represented in tokens
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
}

/// All token types in the Bulu language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    // Keywords (33 total)
    // Control flow
    If,
    Else,
    While,
    For,
    In,
    Break,
    Continue,
    Return,
    Match,
    // Declarations
    Let,
    Const,
    Func,
    Struct,
    Interface,
    Type,
    As,
    // Literals
    True,
    False,
    Null,
    // Logical operators
    And,
    Or,
    Not,
    // Symbolic logical operators
    LogicalAnd,    // &&
    LogicalOr,     // ||
    Bang,          // !
    // Module system
    Import,
    Export,
    Pub,
    Priv,
    // Error handling
    Try,
    Fail,
    Defer,
    // Concurrency
    Async,
    Await,
    Run,
    Chan,
    Lock,
    Select,
    // Generators
    Yield,
    // Generics
    Where,
    // Loop control
    Step,

    // Identifiers and literals
    Identifier,
    IntegerLiteral,
    FloatLiteral,
    StringLiteral,
    CharLiteral,

    // Operators
    Plus,          // +
    Minus,         // -
    Star,          // *
    Slash,         // /
    Percent,       // %
    Power,         // **
    Equal,         // ==
    NotEqual,      // !=
    Less,          // <
    Greater,       // >
    LessEqual,     // <=
    GreaterEqual,  // >=
    Assign,        // =
    PlusAssign,    // +=
    MinusAssign,   // -=
    StarAssign,    // *=
    SlashAssign,   // /=
    PercentAssign, // %=
    Ampersand,     // &
    Pipe,          // |
    Caret,         // ^
    Tilde,         // ~
    LeftShift,     // <<
    RightShift,    // >>
    LeftArrow,     // <-
    RightArrow,    // ->
    FatArrow,      // =>

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Comma,        // ,
    Semicolon,    // ;
    Colon,        // :
    Dot,          // .
    DotDot,       // ..
    DotDotLess,   // ..<
    DotDotDot,    // ...
    Question,     // ?

    // Special
    Newline,
    Eof,
    Comment,
    DocComment,
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            TokenType::If => "if",
            TokenType::Else => "else",
            TokenType::While => "while",
            TokenType::For => "for",
            TokenType::In => "in",
            TokenType::Break => "break",
            TokenType::Continue => "continue",
            TokenType::Return => "return",
            TokenType::Match => "match",
            TokenType::Let => "let",
            TokenType::Const => "const",
            TokenType::Func => "func",
            TokenType::Struct => "struct",
            TokenType::Interface => "interface",
            TokenType::Type => "type",
            TokenType::As => "as",
            TokenType::True => "true",
            TokenType::False => "false",
            TokenType::Null => "null",
            TokenType::And => "and",
            TokenType::Or => "or",
            TokenType::Not => "not",
            TokenType::LogicalAnd => "&&",
            TokenType::LogicalOr => "||",
            TokenType::Bang => "!",
            TokenType::Import => "import",
            TokenType::Export => "export",
            TokenType::Pub => "pub",
            TokenType::Priv => "priv",
            TokenType::Try => "try",
            TokenType::Fail => "fail",
            TokenType::Defer => "defer",
            TokenType::Async => "async",
            TokenType::Await => "await",
            TokenType::Run => "run",
            TokenType::Chan => "chan",
            TokenType::Lock => "lock",
            TokenType::Select => "select",
            TokenType::Yield => "yield",
            TokenType::Where => "where",
            TokenType::Step => "step",
            TokenType::Identifier => "identifier",
            TokenType::IntegerLiteral => "integer",
            TokenType::FloatLiteral => "float",
            TokenType::StringLiteral => "string",
            TokenType::CharLiteral => "char",
            TokenType::Plus => "+",
            TokenType::Minus => "-",
            TokenType::Star => "*",
            TokenType::Slash => "/",
            TokenType::Percent => "%",
            TokenType::Power => "**",
            TokenType::Equal => "==",
            TokenType::NotEqual => "!=",
            TokenType::Less => "<",
            TokenType::Greater => ">",
            TokenType::LessEqual => "<=",
            TokenType::GreaterEqual => ">=",
            TokenType::Assign => "=",
            TokenType::PlusAssign => "+=",
            TokenType::MinusAssign => "-=",
            TokenType::StarAssign => "*=",
            TokenType::SlashAssign => "/=",
            TokenType::PercentAssign => "%=",
            TokenType::Ampersand => "&",
            TokenType::Pipe => "|",
            TokenType::Caret => "^",
            TokenType::Tilde => "~",
            TokenType::LeftShift => "<<",
            TokenType::RightShift => ">>",
            TokenType::LeftArrow => "<-",
            TokenType::RightArrow => "->",
            TokenType::FatArrow => "=>",
            TokenType::LeftParen => "(",
            TokenType::RightParen => ")",
            TokenType::LeftBrace => "{",
            TokenType::RightBrace => "}",
            TokenType::LeftBracket => "[",
            TokenType::RightBracket => "]",
            TokenType::Comma => ",",
            TokenType::Semicolon => ";",
            TokenType::Colon => ":",
            TokenType::Dot => ".",
            TokenType::DotDot => "..",
            TokenType::DotDotLess => "..<",
            TokenType::DotDotDot => "...",
            TokenType::Question => "?",
            TokenType::Newline => "newline",
            TokenType::Eof => "EOF",
            TokenType::Comment => "comment",
            TokenType::DocComment => "doc comment",
        };
        write!(f, "{}", s)
    }
}
