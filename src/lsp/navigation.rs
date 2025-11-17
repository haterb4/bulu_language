use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::ast::nodes::*;

use super::backend::DocumentState;

/// Provides navigation features (go-to-definition, find-references, symbols)
pub struct NavigationProvider {
    documents: Arc<DashMap<String, DocumentState>>,
}

impl NavigationProvider {
    pub fn new(documents: Arc<DashMap<String, DocumentState>>) -> Self {
        Self { documents }
    }

    pub async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let position = params.text_document_position_params.position;
        
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
        if let Some(symbol_name) = self.get_symbol_at_position(&doc.text, position) {
            if let Some(location) = self.find_definition(&ast, &symbol_name, &doc.uri) {
                return Ok(Some(GotoDefinitionResponse::Scalar(location)));
            }
        }

        Ok(None)
    }

    pub async fn find_references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let position = params.text_document_position.position;
        
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
        if let Some(symbol_name) = self.get_symbol_at_position(&doc.text, position) {
            let locations = self.find_all_references(&ast, &symbol_name, &doc.uri);
            if !locations.is_empty() {
                return Ok(Some(locations));
            }
        }

        Ok(None)
    }

    pub async fn document_symbols(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

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

        let symbols = self.extract_symbols(&ast, &doc.uri);
        
        if !symbols.is_empty() {
            Ok(Some(DocumentSymbolResponse::Flat(symbols)))
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

    fn find_definition(&self, ast: &Program, symbol_name: &str, uri: &Url) -> Option<Location> {
        // Search for function definitions
        for stmt in &ast.statements {
            if let Statement::FunctionDecl(func) = stmt {
                if func.name == symbol_name {
                    let line = func.position.line;
                    return Some(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 0,
                            },
                            end: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 100,
                            },
                        },
                    });
                }
            }
            
            // Search for struct definitions
            if let Statement::StructDecl(struct_decl) = stmt {
                if struct_decl.name == symbol_name {
                    let line = struct_decl.position.line;
                    return Some(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 0,
                            },
                            end: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 100,
                            },
                        },
                    });
                }
            }

            // Search for variable declarations
            if let Statement::VariableDecl(var_decl) = stmt {
                if var_decl.name == symbol_name {
                    let line = var_decl.position.line;
                    return Some(Location {
                        uri: uri.clone(),
                        range: Range {
                            start: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 0,
                            },
                            end: Position {
                                line: (line.saturating_sub(1)) as u32,
                                character: 100,
                            },
                        },
                    });
                }
            }
        }

        None
    }

    fn find_all_references(&self, ast: &Program, symbol_name: &str, uri: &Url) -> Vec<Location> {
        let mut locations = Vec::new();

        // This is a simplified implementation
        // In a real implementation, we would traverse the entire AST
        // and find all identifier references
        
        for stmt in &ast.statements {
            if let Some(line) = self.statement_references_symbol(stmt, symbol_name) {
                locations.push(Location {
                    uri: uri.clone(),
                    range: Range {
                        start: Position {
                            line: (line.saturating_sub(1)) as u32,
                            character: 0,
                        },
                        end: Position {
                            line: (line.saturating_sub(1)) as u32,
                            character: 100,
                        },
                    },
                });
            }
        }

        locations
    }

    fn statement_references_symbol(&self, stmt: &Statement, symbol_name: &str) -> Option<usize> {
        match stmt {
            Statement::FunctionDecl(func) if func.name == symbol_name => Some(func.position.line),
            Statement::StructDecl(struct_decl) if struct_decl.name == symbol_name => Some(struct_decl.position.line),
            Statement::VariableDecl(var_decl) if var_decl.name == symbol_name => Some(var_decl.position.line),
            _ => None,
        }
    }

    fn extract_symbols(&self, ast: &Program, uri: &Url) -> Vec<SymbolInformation> {
        let mut symbols = Vec::new();

        for stmt in &ast.statements {
            match stmt {
                Statement::FunctionDecl(func) => {
                    let line = func.position.line;
                    symbols.push(SymbolInformation {
                        name: func.name.clone(),
                        kind: SymbolKind::FUNCTION,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: (line.saturating_sub(1)) as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: (line.saturating_sub(1)) as u32,
                                    character: 100,
                                },
                            },
                        },
                        container_name: None,
                    });
                }
                Statement::StructDecl(struct_decl) => {
                    let line = struct_decl.position.line;
                    symbols.push(SymbolInformation {
                        name: struct_decl.name.clone(),
                        kind: SymbolKind::STRUCT,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: (line.saturating_sub(1)) as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: (line.saturating_sub(1)) as u32,
                                    character: 100,
                                },
                            },
                        },
                        container_name: None,
                    });
                }
                Statement::VariableDecl(var_decl) => {
                    let line = var_decl.position.line;
                    symbols.push(SymbolInformation {
                        name: var_decl.name.clone(),
                        kind: SymbolKind::VARIABLE,
                        tags: None,
                        deprecated: None,
                        location: Location {
                            uri: uri.clone(),
                            range: Range {
                                start: Position {
                                    line: (line.saturating_sub(1)) as u32,
                                    character: 0,
                                },
                                end: Position {
                                    line: (line.saturating_sub(1)) as u32,
                                    character: 100,
                                },
                            },
                        },
                        container_name: None,
                    });
                }
                _ => {}
            }
        }

        symbols
    }
}
