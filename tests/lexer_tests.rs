use bulu::lexer::{Lexer, TokenType, Literal};

#[test]
fn test_keywords() {
    let source = "if else while for func let const step";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    
    let expected_types = vec![
        TokenType::If,
        TokenType::Else,
        TokenType::While,
        TokenType::For,
        TokenType::Func,
        TokenType::Let,
        TokenType::Const,
        TokenType::Step,
        TokenType::Eof,
    ];
    
    for (i, expected) in expected_types.iter().enumerate() {
        assert_eq!(tokens[i].token_type, *expected);
    }
}

#[test]
fn test_literals() {
    let source = r#"42 3.14 "hello" 'c' true false null"#;
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].token_type, TokenType::IntegerLiteral);
    assert_eq!(tokens[0].literal, Some(Literal::Integer(42)));
    
    assert_eq!(tokens[1].token_type, TokenType::FloatLiteral);
    assert_eq!(tokens[1].literal, Some(Literal::Float(3.14)));
    
    assert_eq!(tokens[2].token_type, TokenType::StringLiteral);
    assert_eq!(tokens[2].literal, Some(Literal::String("hello".to_string())));
    
    assert_eq!(tokens[3].token_type, TokenType::CharLiteral);
    assert_eq!(tokens[3].literal, Some(Literal::Char('c')));
    
    assert_eq!(tokens[4].token_type, TokenType::True);
    assert_eq!(tokens[4].literal, Some(Literal::Boolean(true)));
    
    assert_eq!(tokens[5].token_type, TokenType::False);
    assert_eq!(tokens[5].literal, Some(Literal::Boolean(false)));
    
    assert_eq!(tokens[6].token_type, TokenType::Null);
}

#[test]
fn test_operators() {
    let source = "+ - * / % ** == != < > <= >= = += -= *= /= %= & | ^ ~ << >> <- -> =>";
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
        TokenType::Eof,
    ];
    
    for (i, expected) in expected_types.iter().enumerate() {
        assert_eq!(tokens[i].token_type, *expected, "Token {} mismatch", i);
    }
}

#[test]
fn test_number_formats() {
    let source = "42 0x2A 0o52 0b101010";
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    
    // All should parse to the same value (42)
    assert_eq!(tokens[0].literal, Some(Literal::Integer(42)));
    assert_eq!(tokens[1].literal, Some(Literal::Integer(42)));
    assert_eq!(tokens[2].literal, Some(Literal::Integer(42)));
    assert_eq!(tokens[3].literal, Some(Literal::Integer(42)));
}

#[test]
fn test_comments() {
    let source = r#"
    // Single line comment
    let x = 42
    /* Multi-line
       comment */
    let y = 24
    "#;
    
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();
    
    // Comments should be filtered out, only tokens should remain
    let non_newline_tokens: Vec<_> = tokens.iter()
        .filter(|t| t.token_type != TokenType::Newline && t.token_type != TokenType::Eof)
        .collect();
    
    assert_eq!(non_newline_tokens.len(), 8); // let, x, =, 42, let, y, =, 24
    assert_eq!(non_newline_tokens[0].token_type, TokenType::Let);
    assert_eq!(non_newline_tokens[1].token_type, TokenType::Identifier);
    assert_eq!(non_newline_tokens[2].token_type, TokenType::Assign);
    assert_eq!(non_newline_tokens[3].token_type, TokenType::IntegerLiteral);
}