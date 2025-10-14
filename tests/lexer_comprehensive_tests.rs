use bulu::error::BuluError;
use bulu::lexer::{Lexer, Literal, TokenType};

#[test]
fn test_all_keywords() {
    let source = "if else while for break continue return match let const func struct interface as true false null and or not import export try fail defer async await run chan lock select yield step";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    let expected_types = vec![
        TokenType::If,
        TokenType::Else,
        TokenType::While,
        TokenType::For,
        TokenType::Break,
        TokenType::Continue,
        TokenType::Return,
        TokenType::Match,
        TokenType::Let,
        TokenType::Const,
        TokenType::Func,
        TokenType::Struct,
        TokenType::Interface,
        TokenType::As,
        TokenType::True,
        TokenType::False,
        TokenType::Null,
        TokenType::And,
        TokenType::Or,
        TokenType::Not,
        TokenType::Import,
        TokenType::Export,
        TokenType::Try,
        TokenType::Fail,
        TokenType::Defer,
        TokenType::Async,
        TokenType::Await,
        TokenType::Run,
        TokenType::Chan,
        TokenType::Lock,
        TokenType::Select,
        TokenType::Yield,
        TokenType::Step,
        TokenType::Eof,
    ];

    assert_eq!(tokens.len(), expected_types.len());
    for (i, expected) in expected_types.iter().enumerate() {
        assert_eq!(tokens[i].token_type, *expected, "Keyword {} mismatch", i);
    }
}

#[test]
fn test_all_operators() {
    let source = "+ - * / % ** == != < > <= >= = += -= *= /= %= & | ^ ~ << >> <- -> => ( ) { } [ ] , ; : . .. ..< ... ?";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    let expected_types = vec![
        TokenType::Plus,
        TokenType::Minus,
        TokenType::Star,
        TokenType::Slash,
        TokenType::Percent,
        TokenType::Power,
        TokenType::Equal,
        TokenType::NotEqual,
        TokenType::Less,
        TokenType::Greater,
        TokenType::LessEqual,
        TokenType::GreaterEqual,
        TokenType::Assign,
        TokenType::PlusAssign,
        TokenType::MinusAssign,
        TokenType::StarAssign,
        TokenType::SlashAssign,
        TokenType::PercentAssign,
        TokenType::Ampersand,
        TokenType::Pipe,
        TokenType::Caret,
        TokenType::Tilde,
        TokenType::LeftShift,
        TokenType::RightShift,
        TokenType::LeftArrow,
        TokenType::RightArrow,
        TokenType::FatArrow,
        TokenType::LeftParen,
        TokenType::RightParen,
        TokenType::LeftBrace,
        TokenType::RightBrace,
        TokenType::LeftBracket,
        TokenType::RightBracket,
        TokenType::Comma,
        TokenType::Semicolon,
        TokenType::Colon,
        TokenType::Dot,
        TokenType::DotDot,
        TokenType::DotDotLess,
        TokenType::DotDotDot,
        TokenType::Question,
        TokenType::Eof,
    ];

    assert_eq!(tokens.len(), expected_types.len());
    for (i, expected) in expected_types.iter().enumerate() {
        assert_eq!(tokens[i].token_type, *expected, "Operator {} mismatch", i);
    }
}

#[test]
fn test_string_literals_with_escapes() {
    let source = r#""hello" "world\n" "tab\there" "quote\"inside" "backslash\\" "null\0""#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(
        tokens[0].literal,
        Some(Literal::String("hello".to_string()))
    );
    assert_eq!(
        tokens[1].literal,
        Some(Literal::String("world\n".to_string()))
    );
    assert_eq!(
        tokens[2].literal,
        Some(Literal::String("tab\there".to_string()))
    );
    assert_eq!(
        tokens[3].literal,
        Some(Literal::String("quote\"inside".to_string()))
    );
    assert_eq!(
        tokens[4].literal,
        Some(Literal::String("backslash\\".to_string()))
    );
    assert_eq!(
        tokens[5].literal,
        Some(Literal::String("null\0".to_string()))
    );
}

#[test]
fn test_char_literals_with_escapes() {
    let source = r#"'a' '\n' '\t' '\r' '\\' '\'' '\"' '\0'"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].literal, Some(Literal::Char('a')));
    assert_eq!(tokens[1].literal, Some(Literal::Char('\n')));
    assert_eq!(tokens[2].literal, Some(Literal::Char('\t')));
    assert_eq!(tokens[3].literal, Some(Literal::Char('\r')));
    assert_eq!(tokens[4].literal, Some(Literal::Char('\\')));
    assert_eq!(tokens[5].literal, Some(Literal::Char('\'')));
    assert_eq!(tokens[6].literal, Some(Literal::Char('\"')));
    assert_eq!(tokens[7].literal, Some(Literal::Char('\0')));
}

#[test]
fn test_number_literals_comprehensive() {
    let source = "0 42 123456789 0x0 0xFF 0xDEADBEEF 0o0 0o777 0b0 0b11111111 3.14 0.5 123.456 1e10 1.5e-3 2E+5";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    // Decimal integers
    assert_eq!(tokens[0].literal, Some(Literal::Integer(0)));
    assert_eq!(tokens[1].literal, Some(Literal::Integer(42)));
    assert_eq!(tokens[2].literal, Some(Literal::Integer(123456789)));

    // Hexadecimal integers
    assert_eq!(tokens[3].literal, Some(Literal::Integer(0)));
    assert_eq!(tokens[4].literal, Some(Literal::Integer(255)));
    assert_eq!(tokens[5].literal, Some(Literal::Integer(0xDEADBEEF)));

    // Octal integers
    assert_eq!(tokens[6].literal, Some(Literal::Integer(0)));
    assert_eq!(tokens[7].literal, Some(Literal::Integer(511)));

    // Binary integers
    assert_eq!(tokens[8].literal, Some(Literal::Integer(0)));
    assert_eq!(tokens[9].literal, Some(Literal::Integer(255)));

    // Float literals
    assert_eq!(tokens[10].literal, Some(Literal::Float(3.14)));
    assert_eq!(tokens[11].literal, Some(Literal::Float(0.5)));
    assert_eq!(tokens[12].literal, Some(Literal::Float(123.456)));
    assert_eq!(tokens[13].literal, Some(Literal::Float(1e10)));
    assert_eq!(tokens[14].literal, Some(Literal::Float(1.5e-3)));
    assert_eq!(tokens[15].literal, Some(Literal::Float(2E+5)));
}

#[test]
fn test_identifiers() {
    let source =
        "x variable_name _private __dunder__ CamelCase CONSTANT snake_case mixedCase123 _123abc";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    let expected_names = vec![
        "x",
        "variable_name",
        "_private",
        "__dunder__",
        "CamelCase",
        "CONSTANT",
        "snake_case",
        "mixedCase123",
        "_123abc",
    ];

    for (i, expected_name) in expected_names.iter().enumerate() {
        assert_eq!(tokens[i].token_type, TokenType::Identifier);
        assert_eq!(tokens[i].lexeme, *expected_name);
    }
}

#[test]
fn test_nested_block_comments() {
    let source = r#"
    let x = 42
    /* outer comment
       /* nested comment */
       still in outer
    */
    let y = 24
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    let non_newline_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type != TokenType::Newline && t.token_type != TokenType::Eof)
        .collect();

    // Should have: let, x, =, 42, let, y, =, 24
    assert_eq!(non_newline_tokens.len(), 8);
    assert_eq!(non_newline_tokens[0].token_type, TokenType::Let);
    assert_eq!(non_newline_tokens[1].lexeme, "x");
    assert_eq!(non_newline_tokens[4].token_type, TokenType::Let);
    assert_eq!(non_newline_tokens[5].lexeme, "y");
}

#[test]
fn test_position_tracking() {
    let source = "let x = 42\nlet y = 24";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    // First line tokens
    assert_eq!(tokens[0].position.line, 1);
    assert_eq!(tokens[0].position.column, 1); // "let"
    assert_eq!(tokens[1].position.line, 1);
    assert_eq!(tokens[1].position.column, 5); // "x"
    assert_eq!(tokens[2].position.line, 1);
    assert_eq!(tokens[2].position.column, 7); // "="
    assert_eq!(tokens[3].position.line, 1);
    assert_eq!(tokens[3].position.column, 9); // "42"

    // Newline
    assert_eq!(tokens[4].position.line, 1);
    assert_eq!(tokens[4].position.column, 11); // "\n"

    // Second line tokens
    assert_eq!(tokens[5].position.line, 2);
    assert_eq!(tokens[5].position.column, 1); // "let"
    assert_eq!(tokens[6].position.line, 2);
    assert_eq!(tokens[6].position.column, 5); // "y"
}

#[test]
fn test_error_handling_unterminated_string() {
    let source = r#"let x = "unterminated string"#;
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();

    assert!(result.is_err());
    match result.unwrap_err() {
        BuluError::LexError {
            message,
            line,
            column,
            file,
        } => {
            assert!(message.contains("Unterminated string"));
            assert_eq!(line, 1);
            assert_eq!(column, 9);
        }
        _ => panic!("Expected LexError"),
    }
}

#[test]
fn test_error_handling_unterminated_char() {
    let source = "let x = 'a";
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();

    assert!(result.is_err());
    match result.unwrap_err() {
        BuluError::LexError { message, .. } => {
            assert!(message.contains("Unterminated character literal"));
        }
        _ => panic!("Expected LexError"),
    }
}

#[test]
fn test_error_handling_invalid_escape() {
    let source = r#"let x = "invalid\x""#;
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();

    assert!(result.is_err());
    match result.unwrap_err() {
        BuluError::LexError { message, .. } => {
            assert!(message.contains("Invalid escape sequence"));
        }
        _ => panic!("Expected LexError"),
    }
}

#[test]
fn test_error_handling_unterminated_block_comment() {
    let source = "let x = 42 /* unterminated comment";
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();

    assert!(result.is_err());
    match result.unwrap_err() {
        BuluError::LexError { message, .. } => {
            assert!(message.contains("Unterminated block comment"));
        }
        _ => panic!("Expected LexError"),
    }
}

#[test]
fn test_error_handling_invalid_number_format() {
    let source = "let x = 0x"; // Invalid hex number
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();

    assert!(result.is_err());
    match result.unwrap_err() {
        BuluError::LexError { message, .. } => {
            assert!(message.contains("Invalid hexadecimal number"));
        }
        _ => panic!("Expected LexError"),
    }
}

#[test]
fn test_error_handling_invalid_character() {
    let source = "let x = @"; // @ is not a valid character in Bulu
    let mut lexer = Lexer::new(source);
    let result = lexer.tokenize();

    assert!(result.is_err());
    match result.unwrap_err() {
        BuluError::LexError { message, .. } => {
            assert!(message.contains("Unexpected character"));
        }
        _ => panic!("Expected LexError"),
    }
}

#[test]
fn test_multiline_strings() {
    let source = r#""line1
line2
line3""#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].token_type, TokenType::StringLiteral);
    assert_eq!(
        tokens[0].literal,
        Some(Literal::String("line1\nline2\nline3".to_string()))
    );
}

#[test]
fn test_whitespace_handling() {
    let source = "  \t  let   \t x \t = \t 42  \t  ";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    let non_eof_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type != TokenType::Eof)
        .collect();

    assert_eq!(non_eof_tokens.len(), 4);
    assert_eq!(non_eof_tokens[0].token_type, TokenType::Let);
    assert_eq!(non_eof_tokens[1].token_type, TokenType::Identifier);
    assert_eq!(non_eof_tokens[2].token_type, TokenType::Assign);
    assert_eq!(non_eof_tokens[3].token_type, TokenType::IntegerLiteral);
}

#[test]
fn test_complex_expression() {
    let source = "result = (a + b) * c / d - e % f ** g";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    let expected_types = vec![
        TokenType::Identifier, // result
        TokenType::Assign,     // =
        TokenType::LeftParen,  // (
        TokenType::Identifier, // a
        TokenType::Plus,       // +
        TokenType::Identifier, // b
        TokenType::RightParen, // )
        TokenType::Star,       // *
        TokenType::Identifier, // c
        TokenType::Slash,      // /
        TokenType::Identifier, // d
        TokenType::Minus,      // -
        TokenType::Identifier, // e
        TokenType::Percent,    // %
        TokenType::Identifier, // f
        TokenType::Power,      // **
        TokenType::Identifier, // g
        TokenType::Eof,
    ];

    assert_eq!(tokens.len(), expected_types.len());
    for (i, expected) in expected_types.iter().enumerate() {
        assert_eq!(tokens[i].token_type, *expected, "Token {} mismatch", i);
    }
}

#[test]
fn test_function_definition() {
    let source = r#"func fibonacci(n: int32): int32 {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    // Just verify we can tokenize a complete function without errors
    assert!(tokens.len() > 20);
    assert_eq!(tokens[0].token_type, TokenType::Func);
    assert_eq!(tokens[1].lexeme, "fibonacci");

    // Find the return statements
    let return_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type == TokenType::Return)
        .collect();
    assert_eq!(return_tokens.len(), 2);
}

#[test]
fn test_empty_input() {
    let source = "";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token_type, TokenType::Eof);
}

#[test]
fn test_only_whitespace() {
    let source = "   \t\n  \r\n  \t  ";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    // Should only have newlines and EOF
    let non_newline_tokens: Vec<_> = tokens
        .iter()
        .filter(|t| t.token_type != TokenType::Newline)
        .collect();

    assert_eq!(non_newline_tokens.len(), 1);
    assert_eq!(non_newline_tokens[0].token_type, TokenType::Eof);
}

#[test]
fn test_scientific_notation_edge_cases() {
    let source = "1e0 1E0 1e+0 1e-0 1.0e10 1.0E-10 123.456e+789";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    for i in 0..7 {
        assert_eq!(tokens[i].token_type, TokenType::FloatLiteral);
        assert!(matches!(tokens[i].literal, Some(Literal::Float(_))));
    }
}

#[test]
fn test_keyword_vs_identifier() {
    let source = "if ifx xif letx xlet";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    assert_eq!(tokens[0].token_type, TokenType::If);
    assert_eq!(tokens[1].token_type, TokenType::Identifier);
    assert_eq!(tokens[1].lexeme, "ifx");
    assert_eq!(tokens[2].token_type, TokenType::Identifier);
    assert_eq!(tokens[2].lexeme, "xif");
    assert_eq!(tokens[3].token_type, TokenType::Identifier);
    assert_eq!(tokens[3].lexeme, "letx");
    assert_eq!(tokens[4].token_type, TokenType::Identifier);
    assert_eq!(tokens[4].lexeme, "xlet");
}
