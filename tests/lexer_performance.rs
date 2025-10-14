use bulu::lexer::Lexer;
use std::time::Instant;

#[test]
fn test_lexer_performance_large_file() {
    // Create a large source file to test performance
    let mut source = String::new();
    
    // Add a large function with many statements
    source.push_str("func large_function() {\n");
    for i in 0..1000 {
        source.push_str(&format!("    let var{} = {} + {} * {} / {}\n", i, i, i+1, i+2, i+3));
    }
    source.push_str("}\n");
    
    // Add many small functions
    for i in 0..100 {
        source.push_str(&format!("func func{}() {{ return {} }}\n", i, i));
    }
    
    let start = Instant::now();
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().unwrap();
    let duration = start.elapsed();
    
    // Should tokenize quickly (less than 100ms for this size)
    assert!(duration.as_millis() < 100, "Lexing took too long: {:?}", duration);
    
    // Should have many tokens
    assert!(tokens.len() > 10000);
    
    println!("Tokenized {} tokens in {:?}", tokens.len(), duration);
}

#[test]
fn test_lexer_performance_many_strings() {
    let mut source = String::new();
    
    // Create many string literals
    for i in 0..1000 {
        source.push_str(&format!("let str{} = \"This is string number {}\"\n", i, i));
    }
    
    let start = Instant::now();
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().unwrap();
    let duration = start.elapsed();
    
    // Should tokenize quickly
    assert!(duration.as_millis() < 50, "String lexing took too long: {:?}", duration);
    
    println!("Tokenized {} string tokens in {:?}", tokens.len(), duration);
}

#[test]
fn test_lexer_performance_deep_nesting() {
    let mut source = String::new();
    
    // Create deeply nested structure
    for _ in 0..100 {
        source.push_str("if true {\n");
    }
    source.push_str("let x = 42\n");
    for _ in 0..100 {
        source.push_str("}\n");
    }
    
    let start = Instant::now();
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().unwrap();
    let duration = start.elapsed();
    
    // Should handle nesting efficiently
    assert!(duration.as_millis() < 10, "Nested structure lexing took too long: {:?}", duration);
    
    println!("Tokenized nested structure with {} tokens in {:?}", tokens.len(), duration);
}

#[test]
fn test_lexer_memory_usage() {
    // Test that lexer doesn't use excessive memory
    let source = "x".repeat(100000); // 100KB identifier
    
    let mut lexer = Lexer::new(&source);
    let tokens = lexer.tokenize().unwrap();
    
    // Should have just one identifier token and EOF
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].lexeme.len(), 100000);
}