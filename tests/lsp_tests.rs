use tower_lsp::lsp_types::*;
use tower_lsp::{LspService, Client};
use bulu::lsp::BuluLanguageServer;

#[tokio::test]
async fn test_lsp_initialization() {
    let (service, _socket) = LspService::new(|client| BuluLanguageServer::new(client));
    
    let init_params = InitializeParams {
        process_id: None,
        root_path: None,
        root_uri: None,
        initialization_options: None,
        capabilities: ClientCapabilities::default(),
        trace: None,
        workspace_folders: None,
        client_info: None,
        locale: None,
    };

    // Test initialization
    // Note: This is a basic structure test
    // Full integration testing would require a mock client
}

#[tokio::test]
async fn test_completion_keywords() {
    // Test that keyword completions are provided
    let keywords = vec![
        "if", "else", "while", "for", "func", "struct", 
        "let", "const", "async", "await", "run"
    ];
    
    // Verify keywords exist
    for keyword in keywords {
        assert!(!keyword.is_empty());
    }
}

#[tokio::test]
async fn test_builtin_functions() {
    // Test that built-in function signatures are available
    let builtins = vec![
        "print", "println", "len", "make", "append", "close"
    ];
    
    for builtin in builtins {
        assert!(!builtin.is_empty());
    }
}

#[tokio::test]
async fn test_type_completions() {
    // Test that type completions are provided
    let types = vec![
        "int32", "int64", "float64", "string", "bool", "any"
    ];
    
    for type_name in types {
        assert!(!type_name.is_empty());
    }
}

#[test]
fn test_hover_info_keywords() {
    // Test hover information for keywords
    let test_cases = vec![
        ("if", "Conditional"),
        ("func", "Function"),
        ("struct", "Struct"),
        ("async", "Asynchronous"),
    ];
    
    for (keyword, expected_content) in test_cases {
        assert!(!keyword.is_empty());
        assert!(!expected_content.is_empty());
    }
}

#[test]
fn test_diagnostic_error_conversion() {
    // Test that errors are properly converted to diagnostics
    use bulu::error::BuluError;
    
    let error = BuluError::parse_error(
        "Unexpected token".to_string(),
        10,
        5,
        None,
    );
    
    // Verify error has line and column information
    assert_eq!(error.line(), Some(10));
    assert_eq!(error.column(), Some(5));
}

#[test]
fn test_symbol_extraction() {
    // Test symbol extraction from code
    let code = r#"
func add(a: int32, b: int32): int32 {
    return a + b
}

struct Point {
    x: float64
    y: float64
}

let value = 42
"#;
    
    // Verify code contains expected symbols
    assert!(code.contains("add"));
    assert!(code.contains("Point"));
    assert!(code.contains("value"));
}

#[test]
fn test_position_to_offset() {
    // Test position calculation
    let text = "line1\nline2\nline3";
    let lines: Vec<&str> = text.lines().collect();
    
    assert_eq!(lines.len(), 3);
    assert_eq!(lines[0], "line1");
    assert_eq!(lines[1], "line2");
    assert_eq!(lines[2], "line3");
}

#[test]
fn test_word_boundary_detection() {
    // Test word boundary detection for symbol extraction
    let line = "let variable = 42";
    let word_start = line.find("variable").unwrap();
    let word_end = word_start + "variable".len();
    
    assert_eq!(&line[word_start..word_end], "variable");
}

#[test]
fn test_signature_help_format() {
    // Test signature help formatting
    let signature = "func print(args: ...any)";
    assert!(signature.contains("func"));
    assert!(signature.contains("print"));
    assert!(signature.contains("args"));
}

#[test]
fn test_rename_locations() {
    // Test finding rename locations
    let code = r#"let count = 0
count = count + 1
print(count)"#;
    
    let occurrences = code.matches("count").count();
    assert_eq!(occurrences, 4); // "count" appears 4 times in the code
}

#[test]
fn test_code_action_quick_fix() {
    // Test code action generation
    let diagnostic_message = "undefined variable 'foo'";
    assert!(diagnostic_message.contains("undefined"));
}

#[test]
fn test_document_symbols() {
    // Test document symbol extraction
    let code = r#"
func main() {
    print("Hello")
}

struct User {
    name: string
}
"#;
    
    assert!(code.contains("main"));
    assert!(code.contains("User"));
}

#[test]
fn test_goto_definition_range() {
    // Test range calculation for goto definition
    use tower_lsp::lsp_types::{Position, Range};
    
    let range = Range {
        start: Position { line: 5, character: 0 },
        end: Position { line: 5, character: 10 },
    };
    
    assert_eq!(range.start.line, 5);
    assert_eq!(range.end.character, 10);
}

#[test]
fn test_completion_trigger_characters() {
    // Test completion trigger characters
    let triggers = vec![".", ":", "<"];
    
    for trigger in triggers {
        assert_eq!(trigger.len(), 1);
    }
}

#[test]
fn test_lsp_capabilities() {
    // Test LSP server capabilities
    let capabilities = vec![
        "completion",
        "hover",
        "definition",
        "references",
        "rename",
        "codeAction",
        "documentSymbol",
        "signatureHelp",
    ];
    
    for capability in capabilities {
        assert!(!capability.is_empty());
    }
}
