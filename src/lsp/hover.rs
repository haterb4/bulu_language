use dashmap::DashMap;
use std::sync::Arc;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;

use super::backend::DocumentState;

/// Provides hover information and signature help
pub struct HoverProvider {
    documents: Arc<DashMap<String, DocumentState>>,
}

impl HoverProvider {
    pub fn new(documents: Arc<DashMap<String, DocumentState>>) -> Self {
        Self { documents }
    }

    pub async fn provide_hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let position = params.text_document_position_params.position;
        
        // Get word at cursor position
        if let Some(word) = self.get_word_at_position(&doc.text, position) {
            if let Some(hover_info) = self.get_hover_info(&word) {
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_info,
                    }),
                    range: None,
                }));
            }
        }

        Ok(None)
    }

    pub async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        
        let doc = match self.documents.get(&uri) {
            Some(doc) => doc.clone(),
            None => return Ok(None),
        };

        let position = params.text_document_position_params.position;
        
        // Find function call at cursor
        if let Some(func_name) = self.get_function_at_position(&doc.text, position) {
            if let Some(signature) = self.get_function_signature(&func_name) {
                return Ok(Some(signature));
            }
        }

        Ok(None)
    }

    fn get_word_at_position(&self, text: &str, position: Position) -> Option<String> {
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

    fn get_function_at_position(&self, text: &str, position: Position) -> Option<String> {
        let lines: Vec<&str> = text.lines().collect();
        if position.line as usize >= lines.len() {
            return None;
        }

        let line = lines[position.line as usize];
        let before_cursor = &line[..position.character.min(line.len() as u32) as usize];
        
        // Find last function call before cursor
        if let Some(paren_pos) = before_cursor.rfind('(') {
            let func_part = &before_cursor[..paren_pos];
            if let Some(word_start) = func_part.rfind(|c: char| !c.is_alphanumeric() && c != '_') {
                return Some(func_part[word_start + 1..].to_string());
            } else {
                return Some(func_part.to_string());
            }
        }

        None
    }

    fn get_hover_info(&self, word: &str) -> Option<String> {
        match word {
            // Keywords
            "if" => Some("```bulu\nif condition { ... }\n```\nConditional execution".to_string()),
            "else" => Some("```bulu\nelse { ... }\n```\nAlternative branch".to_string()),
            "while" => Some("```bulu\nwhile condition { ... }\n```\nLoop with condition".to_string()),
            "for" => Some("```bulu\nfor item in collection { ... }\n```\nIteration loop".to_string()),
            "func" => Some("```bulu\nfunc name(params): returnType { ... }\n```\nFunction definition".to_string()),
            "struct" => Some("```bulu\nstruct Name { fields }\n```\nStruct definition".to_string()),
            "interface" => Some("```bulu\ninterface Name { methods }\n```\nInterface definition".to_string()),
            "let" => Some("```bulu\nlet variable = value\n```\nMutable variable declaration".to_string()),
            "const" => Some("```bulu\nconst CONSTANT = value\n```\nImmutable constant declaration".to_string()),
            "async" => Some("```bulu\nasync func name() { ... }\n```\nAsynchronous function".to_string()),
            "await" => Some("```bulu\nawait asyncFunction()\n```\nWait for async operation".to_string()),
            "run" => Some("```bulu\nrun function()\n```\nCreate concurrent task (goroutine)".to_string()),
            "chan" => Some("```bulu\nchan T\n```\nChannel type for concurrent communication".to_string()),
            "select" => Some("```bulu\nselect { case <- ch: ... }\n```\nChannel multiplexing".to_string()),
            "defer" => Some("```bulu\ndefer cleanup()\n```\nDeferred execution (runs before return)".to_string()),
            "match" => Some("```bulu\nmatch value { pattern -> ... }\n```\nPattern matching".to_string()),
            
            // Built-in functions
            "print" => Some("```bulu\nfunc print(args: ...any)\n```\nPrint values to stdout".to_string()),
            "println" => Some("```bulu\nfunc println(args: ...any)\n```\nPrint values with newline".to_string()),
            "len" => Some("```bulu\nfunc len(x: any): int32\n```\nGet length of array, slice, string, or map".to_string()),
            "make" => Some("```bulu\nfunc make(type: Type, args: ...any): Type\n```\nCreate slice, map, or channel".to_string()),
            "append" => Some("```bulu\nfunc append(slice: []T, item: T): []T\n```\nAppend item to slice".to_string()),
            "close" => Some("```bulu\nfunc close(ch: chan T)\n```\nClose a channel".to_string()),
            "panic" => Some("```bulu\nfunc panic(message: string)\n```\nTrigger a panic with message".to_string()),
            "typeof" => Some("```bulu\nfunc typeof(x: any): string\n```\nGet type name as string".to_string()),
            
            // Types
            "int32" => Some("```bulu\nint32\n```\n32-bit signed integer (-2,147,483,648 to 2,147,483,647)".to_string()),
            "int64" => Some("```bulu\nint64\n```\n64-bit signed integer".to_string()),
            "float64" => Some("```bulu\nfloat64\n```\n64-bit floating point number".to_string()),
            "string" => Some("```bulu\nstring\n```\nUTF-8 encoded string".to_string()),
            "bool" => Some("```bulu\nbool\n```\nBoolean type (true or false)".to_string()),
            "any" => Some("```bulu\nany\n```\nType that can hold any value".to_string()),
            
            _ => None,
        }
    }

    fn get_function_signature(&self, func_name: &str) -> Option<SignatureHelp> {
        let signature_info = match func_name {
            "print" | "println" => SignatureInformation {
                label: "func print(args: ...any)".to_string(),
                documentation: Some(Documentation::String("Print values to stdout".to_string())),
                parameters: Some(vec![ParameterInformation {
                    label: ParameterLabel::Simple("args: ...any".to_string()),
                    documentation: Some(Documentation::String("Values to print".to_string())),
                }]),
                active_parameter: None,
            },
            "printf" => SignatureInformation {
                label: "func printf(format: string, args: ...any)".to_string(),
                documentation: Some(Documentation::String("Print formatted output".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("format: string".to_string()),
                        documentation: Some(Documentation::String("Format string".to_string())),
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("args: ...any".to_string()),
                        documentation: Some(Documentation::String("Values to format".to_string())),
                    },
                ]),
                active_parameter: None,
            },
            "make" => SignatureInformation {
                label: "func make(type: Type, args: ...any): Type".to_string(),
                documentation: Some(Documentation::String("Create slice, map, or channel".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("type: Type".to_string()),
                        documentation: Some(Documentation::String("Type to create".to_string())),
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("args: ...any".to_string()),
                        documentation: Some(Documentation::String("Size or capacity arguments".to_string())),
                    },
                ]),
                active_parameter: None,
            },
            "append" => SignatureInformation {
                label: "func append(slice: []T, item: T): []T".to_string(),
                documentation: Some(Documentation::String("Append item to slice".to_string())),
                parameters: Some(vec![
                    ParameterInformation {
                        label: ParameterLabel::Simple("slice: []T".to_string()),
                        documentation: Some(Documentation::String("Slice to append to".to_string())),
                    },
                    ParameterInformation {
                        label: ParameterLabel::Simple("item: T".to_string()),
                        documentation: Some(Documentation::String("Item to append".to_string())),
                    },
                ]),
                active_parameter: None,
            },
            _ => return None,
        };

        Some(SignatureHelp {
            signatures: vec![signature_info],
            active_signature: Some(0),
            active_parameter: None,
        })
    }
}
