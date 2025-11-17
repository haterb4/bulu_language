use dashmap::DashMap;
use std::collections::HashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::nodes::*;

use super::backend::DocumentState;

/// Provides refactoring support (rename, extract function, quick fixes)
pub struct RefactorProvider {
    documents: Arc<DashMap<String, DocumentState>>,
}

impl RefactorProvider {
    pub fn new(documents: Arc<DashMap<String, DocumentState>>) -> Self {
        Self { documents }
    }

    pub async fn rename(&self, params: RenameParams) -> Result<Option<WorkspaceEdit>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let position = params.text_document_position.position;
        let new_name = params.new_name;

        // Parse the document
        let mut lexer = Lexer::new(&doc.text);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(_) => return Ok(None),
        };

        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };

        // Find symbol at position
        if let Some(old_name) = self.get_symbol_at_position(&doc.text, position) {
            let edits = self.find_rename_locations(&ast, &old_name, &new_name, &doc.uri);
            
            if !edits.is_empty() {
                let mut changes = HashMap::new();
                changes.insert(doc.uri.clone(), edits);
                
                return Ok(Some(WorkspaceEdit {
                    changes: Some(changes),
                    document_changes: None,
                    change_annotations: None,
                }));
            }
        }

        Ok(None)
    }

    pub async fn code_actions(
        &self,
        params: CodeActionParams,
    ) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let mut actions = Vec::new();

        // Add quick fixes based on diagnostics
        for diagnostic in &params.context.diagnostics {
            if let Some(action) = self.create_quick_fix(&doc, diagnostic) {
                actions.push(CodeActionOrCommand::CodeAction(action));
            }
        }

        // Add refactoring actions
        actions.extend(self.create_refactoring_actions(&doc, &params.range));

        if !actions.is_empty() {
            Ok(Some(actions))
        } else {
            Ok(None)
        }
    }

    fn get_symbol_at_position(&self, text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let char_pos = position.character as usize;
        
        if char_pos > line.len() {
            return None;
        }

        // Find word boundaries
        let mut start = char_pos;
        let mut end = char_pos;

        let chars: Vec<char> = line.chars().collect();
        
        // Move start backward
        while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
            start -= 1;
        }

        // Move end forward
        while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
            end += 1;
        }

        if start < end {
            Some(chars[start..end].iter().collect())
        } else {
            None
        }
    }

    fn find_rename_locations(
        &self,
        ast: &Program,
        old_name: &str,
        new_name: &str,
        _uri: &Url,
    ) -> Vec<TextEdit> {
        let mut edits = Vec::new();

        // Find all occurrences of the symbol
        for stmt in &ast.statements {
            match stmt {
                Statement::FunctionDecl(func) if func.name == old_name => {
                    let line = func.position.line;
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 5, // After "func "
                            },
                            end: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: (5 + old_name.len()) as u32,
                            },
                        },
                        new_text: new_name.to_string(),
                    });
                }
                Statement::StructDecl(struct_decl) if struct_decl.name == old_name => {
                    let line = struct_decl.position.line;
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 7, // After "struct "
                            },
                            end: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: (7 + old_name.len()) as u32,
                            },
                        },
                        new_text: new_name.to_string(),
                    });
                }
                Statement::VariableDecl(var_decl) if var_decl.name == old_name => {
                    let line = var_decl.position.line;
                    edits.push(TextEdit {
                        range: Range {
                            start: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 4, // After "let "
                            },
                            end: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: (4 + old_name.len()) as u32,
                            },
                        },
                        new_text: new_name.to_string(),
                    });
                }
                _ => {}
            }
        }

        edits
    }

    fn create_quick_fix(&self, doc: &DocumentState, diagnostic: &Diagnostic) -> Option<CodeAction> {
        // Example: Add missing import
        if diagnostic.message.contains("undefined") || diagnostic.message.contains("not found") {
            return Some(CodeAction {
                title: "Add import statement".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some({
                        let mut changes = HashMap::new();
                        changes.insert(
                            doc.uri.clone(),
                            vec![TextEdit {
                                range: Range {
                                    start: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                    end: Position {
                                        line: 0,
                                        character: 0,
                                    },
                                },
                                new_text: "import std.io\n".to_string(),
                            }],
                        );
                        changes
                    }),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(true),
                disabled: None,
                data: None,
            });
        }

        // Example: Fix unused variable
        if diagnostic.message.contains("unused") {
            return Some(CodeAction {
                title: "Remove unused variable".to_string(),
                kind: Some(CodeActionKind::QUICKFIX),
                diagnostics: Some(vec![diagnostic.clone()]),
                edit: Some(WorkspaceEdit {
                    changes: Some({
                        let mut changes = HashMap::new();
                        changes.insert(
                            doc.uri.clone(),
                            vec![TextEdit {
                                range: diagnostic.range,
                                new_text: String::new(),
                            }],
                        );
                        changes
                    }),
                    document_changes: None,
                    change_annotations: None,
                }),
                command: None,
                is_preferred: Some(false),
                disabled: None,
                data: None,
            });
        }

        None
    }

    fn create_refactoring_actions(&self, doc: &DocumentState, range: &Range) -> Vec<CodeActionOrCommand> {
        let mut actions = Vec::new();

        // Extract function refactoring
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Extract function".to_string(),
            kind: Some(CodeActionKind::REFACTOR_EXTRACT),
            diagnostics: None,
            edit: None, // Would need to implement extraction logic
            command: Some(Command {
                title: "Extract function".to_string(),
                command: "bulu.extractFunction".to_string(),
                arguments: None,
            }),
            is_preferred: None,
            disabled: None,
            data: None,
        }));

        // Inline variable refactoring
        actions.push(CodeActionOrCommand::CodeAction(CodeAction {
            title: "Inline variable".to_string(),
            kind: Some(CodeActionKind::REFACTOR_INLINE),
            diagnostics: None,
            edit: None,
            command: Some(Command {
                title: "Inline variable".to_string(),
                command: "bulu.inlineVariable".to_string(),
                arguments: None,
            }),
            is_preferred: None,
            disabled: None,
            data: None,
        }));

        actions
    }
}
