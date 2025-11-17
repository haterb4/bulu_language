use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use super::backend::DocumentState;

/// Provides code completion for Bulu
pub struct CompletionProvider {
    documents: Arc<DashMap<String, DocumentState>>,
}

impl CompletionProvider {
    pub fn new(documents: Arc<DashMap<String, DocumentState>>) -> Self {
        Self { documents }
    }

    pub async fn provide_completion(
        &self,
        params: CompletionParams,
    ) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let position = params.text_document_position.position;
        let mut items = Vec::new();

        // Add keyword completions
        items.extend(self.keyword_completions());

        // Add built-in function completions
        items.extend(self.builtin_completions());

        // Add type completions
        items.extend(self.type_completions());

        // Context-aware completions based on cursor position
        if let Some(context_items) = self.context_completions(&doc.text, position) {
            items.extend(context_items);
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    fn keyword_completions(&self) -> Vec<CompletionItem> {
        let keywords = vec![
            ("if", "Conditional statement"),
            ("else", "Alternative branch"),
            ("while", "Loop with condition"),
            ("for", "Iteration loop"),
            ("break", "Exit loop"),
            ("continue", "Skip to next iteration"),
            ("return", "Return from function"),
            ("match", "Pattern matching"),
            ("let", "Mutable variable"),
            ("const", "Immutable constant"),
            ("func", "Function definition"),
            ("struct", "Struct definition"),
            ("interface", "Interface definition"),
            ("as", "Type casting"),
            ("true", "Boolean true"),
            ("false", "Boolean false"),
            ("null", "Null value"),
            ("and", "Logical AND"),
            ("or", "Logical OR"),
            ("not", "Logical NOT"),
            ("import", "Import module"),
            ("export", "Export symbol"),
            ("try", "Error handling"),
            ("fail", "Throw error"),
            ("defer", "Deferred execution"),
            ("async", "Async function"),
            ("await", "Wait for async"),
            ("run", "Concurrent task"),
            ("chan", "Channel type"),
            ("lock", "Mutex lock"),
            ("select", "Channel multiplexing"),
            ("yield", "Generator yield"),
        ];

        keywords
            .into_iter()
            .map(|(keyword, detail)| CompletionItem {
                label: keyword.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(detail.to_string()),
                insert_text: Some(keyword.to_string()),
                ..Default::default()
            })
            .collect()
    }

    fn builtin_completions(&self) -> Vec<CompletionItem> {
        let builtins = vec![
            ("print", "func(args: ...any)", "Print to stdout"),
            ("println", "func(args: ...any)", "Print with newline"),
            ("printf", "func(format: string, args: ...any)", "Formatted print"),
            ("input", "func(prompt: string): string", "Read from stdin"),
            ("len", "func(x: any): int32", "Get length"),
            ("cap", "func(x: any): int32", "Get capacity"),
            ("append", "func(slice: []T, item: T): []T", "Append to slice"),
            ("make", "func(type: Type, args: ...any): Type", "Create collection"),
            ("delete", "func(map: map[K]V, key: K)", "Delete from map"),
            ("close", "func(ch: chan T)", "Close channel"),
            ("copy", "func(dst: []T, src: []T): int32", "Copy slice"),
            ("clone", "func(x: T): T", "Deep copy"),
            ("panic", "func(message: string)", "Panic with message"),
            ("recover", "func(): any", "Recover from panic"),
            ("assert", "func(condition: bool, message: string)", "Assert condition"),
            ("typeof", "func(x: any): string", "Get type name"),
            ("instanceof", "func(x: any, T: Type): bool", "Check type"),
            ("sizeof", "func(T: Type): int32", "Get type size"),
        ];

        builtins
            .into_iter()
            .map(|(name, signature, detail)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(signature.to_string()),
                documentation: Some(Documentation::String(detail.to_string())),
                insert_text: Some(format!("{}($0)", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            })
            .collect()
    }

    fn type_completions(&self) -> Vec<CompletionItem> {
        let types = vec![
            ("int8", "8-bit signed integer"),
            ("int16", "16-bit signed integer"),
            ("int32", "32-bit signed integer"),
            ("int64", "64-bit signed integer"),
            ("uint8", "8-bit unsigned integer"),
            ("uint16", "16-bit unsigned integer"),
            ("uint32", "32-bit unsigned integer"),
            ("uint64", "64-bit unsigned integer"),
            ("int", "Platform-dependent integer"),
            ("uint", "Platform-dependent unsigned integer"),
            ("float32", "32-bit floating point"),
            ("float64", "64-bit floating point"),
            ("bool", "Boolean type"),
            ("char", "UTF-8 character"),
            ("string", "UTF-8 string"),
            ("byte", "Alias for uint8"),
            ("rune", "Alias for int32"),
            ("any", "Any type"),
        ];

        types
            .into_iter()
            .map(|(name, detail)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::TYPE_PARAMETER),
                detail: Some(detail.to_string()),
                ..Default::default()
            })
            .collect()
    }

    fn context_completions(&self, text: &str, position: Position) -> Option<Vec<CompletionItem>> {
        // Get the line at cursor position
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;
        
        if char_pos > line.len() {
            return None;
        }

        let before_cursor = &line[..char_pos];

        // Check for member access (dot notation)
        if before_cursor.ends_with('.') {
            return Some(self.member_completions(before_cursor));
        }

        // Check for import statement
        if before_cursor.trim_start().starts_with("import") {
            return Some(self.import_completions());
        }

        None
    }

    fn member_completions(&self, before_cursor: &str) -> Vec<CompletionItem> {
        // Provide common method completions
        vec![
            CompletionItem {
                label: "len".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Get length".to_string()),
                ..Default::default()
            },
            CompletionItem {
                label: "toString".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("Convert to string".to_string()),
                ..Default::default()
            },
        ]
    }

    fn import_completions(&self) -> Vec<CompletionItem> {
        let modules = vec![
            ("std.io", "Input/output operations"),
            ("std.fmt", "String formatting"),
            ("std.strings", "String manipulation"),
            ("std.arrays", "Array operations"),
            ("std.math", "Mathematical functions"),
            ("std.time", "Time and duration"),
            ("std.sync", "Synchronization primitives"),
            ("std.os", "Operating system interface"),
            ("std.http", "HTTP client and server"),
            ("std.net", "TCP/UDP networking"),
            ("std.json", "JSON encoding/decoding"),
            ("std.crypto", "Cryptographic operations"),
        ];

        modules
            .into_iter()
            .map(|(name, detail)| CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some(detail.to_string()),
                ..Default::default()
            })
            .collect()
    }
}
