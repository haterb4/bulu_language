//! Documentation extraction from source code

use crate::Result;
use crate::lexer::{Lexer, Token, TokenType};
use crate::parser::Parser;
use crate::ast::nodes::*;
use super::{DocumentedItem, DocComment, ItemKind, Visibility};
use std::path::PathBuf;

/// Extracts documentation from source code
pub struct DocExtractor;

impl DocExtractor {
    pub fn new() -> Self {
        Self
    }

    /// Extract documentation from a source file
    pub fn extract_from_file(&self, content: &str, file_path: &PathBuf) -> Result<Vec<DocumentedItem>> {
        let mut lexer = Lexer::new(content);
        let tokens = lexer.tokenize()?;
        
        let mut parser = Parser::new(tokens);
        let ast = parser.parse()?;
        
        let mut items = Vec::new();
        self.extract_from_ast(&ast, file_path, &mut items);
        
        Ok(items)
    }

    fn extract_from_ast(&self, ast: &Program, file_path: &PathBuf, items: &mut Vec<DocumentedItem>) {
        for stmt in &ast.statements {
            self.extract_from_statement(stmt, file_path, items);
        }
    }

    fn extract_from_statement(&self, stmt: &Statement, file_path: &PathBuf, items: &mut Vec<DocumentedItem>) {
        match stmt {
            Statement::FunctionDecl(func) => {
                let doc_comment = self.extract_doc_comment_from_tokens(&func.doc_comment);
                let signature = self.generate_function_signature(func);
                
                items.push(DocumentedItem {
                    name: func.name.clone(),
                    kind: ItemKind::Function,
                    signature,
                    doc_comment,
                    visibility: if func.is_exported { Visibility::Public } else { Visibility::Private },
                    file_path: file_path.clone(),
                    line_number: func.position.line,
                });
            }
            Statement::StructDecl(struct_def) => {
                let doc_comment = self.extract_doc_comment_from_tokens(&struct_def.doc_comment);
                let signature = self.generate_struct_signature(struct_def);
                
                items.push(DocumentedItem {
                    name: struct_def.name.clone(),
                    kind: ItemKind::Struct,
                    signature,
                    doc_comment,
                    visibility: if struct_def.is_exported { Visibility::Public } else { Visibility::Private },
                    file_path: file_path.clone(),
                    line_number: struct_def.position.line,
                });

                // Extract methods
                for method in &struct_def.methods {
                    let method_doc = self.extract_doc_comment_from_tokens(&method.doc_comment);
                    let method_signature = self.generate_function_signature(method);
                    
                    items.push(DocumentedItem {
                        name: format!("{}.{}", struct_def.name, method.name),
                        kind: ItemKind::Function,
                        signature: method_signature,
                        doc_comment: method_doc,
                        visibility: if method.is_exported { Visibility::Public } else { Visibility::Private },
                        file_path: file_path.clone(),
                        line_number: method.position.line,
                    });
                }
            }
            Statement::InterfaceDecl(interface_def) => {
                let doc_comment = self.extract_doc_comment_from_tokens(&interface_def.doc_comment);
                let signature = self.generate_interface_signature(interface_def);
                
                items.push(DocumentedItem {
                    name: interface_def.name.clone(),
                    kind: ItemKind::Interface,
                    signature,
                    doc_comment,
                    visibility: if interface_def.is_exported { Visibility::Public } else { Visibility::Private },
                    file_path: file_path.clone(),
                    line_number: interface_def.position.line,
                });
            }
            Statement::VariableDecl(var_decl) => {
                if var_decl.is_const {
                    let doc_comment = self.extract_doc_comment_from_tokens(&var_decl.doc_comment);
                    let signature = self.generate_variable_signature(var_decl);
                    
                    items.push(DocumentedItem {
                        name: var_decl.name.clone(),
                        kind: ItemKind::Constant,
                        signature,
                        doc_comment,
                        visibility: if var_decl.is_exported { Visibility::Public } else { Visibility::Private },
                        file_path: file_path.clone(),
                        line_number: var_decl.position.line,
                    });
                }
            }
            _ => {}
        }
    }

    fn extract_doc_comment_from_tokens(&self, tokens: &Option<Vec<Token>>) -> Option<DocComment> {
        if let Some(tokens) = tokens {
            for token in tokens {
                if token.token_type == TokenType::DocComment {
                    return Some(DocComment::parse(&token.lexeme));
                }
            }
        }
        None
    }

    fn generate_function_signature(&self, func: &FunctionDecl) -> String {
        let mut signature = String::new();
        
        if func.is_async {
            signature.push_str("async ");
        }
        
        signature.push_str("func ");
        signature.push_str(&func.name);
        
        if !func.type_params.is_empty() {
            signature.push('<');
            for (i, type_param) in func.type_params.iter().enumerate() {
                if i > 0 {
                    signature.push_str(", ");
                }
                signature.push_str(&type_param.name);
            }
            signature.push('>');
        }
        
        signature.push('(');
        for (i, param) in func.params.iter().enumerate() {
            if i > 0 {
                signature.push_str(", ");
            }
            signature.push_str(&param.name);
            signature.push_str(": ");
            signature.push_str(&self.type_to_string(&param.param_type));
        }
        signature.push(')');
        
        if let Some(return_type) = &func.return_type {
            signature.push_str(": ");
            signature.push_str(&self.type_to_string(return_type));
        }
        
        signature
    }

    fn generate_struct_signature(&self, struct_def: &StructDecl) -> String {
        let mut signature = String::new();
        
        signature.push_str("struct ");
        signature.push_str(&struct_def.name);
        
        if !struct_def.type_params.is_empty() {
            signature.push('<');
            for (i, type_param) in struct_def.type_params.iter().enumerate() {
                if i > 0 {
                    signature.push_str(", ");
                }
                signature.push_str(&type_param.name);
            }
            signature.push('>');
        }
        
        signature.push_str(" {\n");
        for field in &struct_def.fields {
            signature.push_str("    ");
            signature.push_str(&field.name);
            signature.push_str(": ");
            signature.push_str(&self.type_to_string(&field.field_type));
            signature.push('\n');
        }
        signature.push('}');
        
        signature
    }

    fn generate_interface_signature(&self, interface_def: &InterfaceDecl) -> String {
        let mut signature = String::new();
        
        signature.push_str("interface ");
        signature.push_str(&interface_def.name);
        
        if !interface_def.type_params.is_empty() {
            signature.push('<');
            for (i, type_param) in interface_def.type_params.iter().enumerate() {
                if i > 0 {
                    signature.push_str(", ");
                }
                signature.push_str(&type_param.name);
            }
            signature.push('>');
        }
        
        signature.push_str(" {\n");
        for method in &interface_def.methods {
            signature.push_str("    func ");
            signature.push_str(&method.name);
            signature.push('(');
            for (i, param) in method.params.iter().enumerate() {
                if i > 0 {
                    signature.push_str(", ");
                }
                signature.push_str(&param.name);
                signature.push_str(": ");
                signature.push_str(&self.type_to_string(&param.param_type));
            }
            signature.push(')');
            if let Some(return_type) = &method.return_type {
                signature.push_str(": ");
                signature.push_str(&self.type_to_string(return_type));
            }
            signature.push('\n');
        }
        signature.push('}');
        
        signature
    }

    fn generate_variable_signature(&self, var_decl: &VariableDecl) -> String {
        let mut signature = String::new();
        
        if var_decl.is_const {
            signature.push_str("const ");
        } else {
            signature.push_str("let ");
        }
        
        signature.push_str(&var_decl.name);
        
        if let Some(var_type) = &var_decl.type_annotation {
            signature.push_str(": ");
            signature.push_str(&self.type_to_string(var_type));
        }
        
        signature
    }

    fn type_to_string(&self, type_def: &Type) -> String {
        match type_def {
            Type::Int8 => "int8".to_string(),
            Type::Int16 => "int16".to_string(),
            Type::Int32 => "int32".to_string(),
            Type::Int64 => "int64".to_string(),
            Type::UInt8 => "uint8".to_string(),
            Type::UInt16 => "uint16".to_string(),
            Type::UInt32 => "uint32".to_string(),
            Type::UInt64 => "uint64".to_string(),
            Type::Float32 => "float32".to_string(),
            Type::Float64 => "float64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Char => "char".to_string(),
            Type::String => "string".to_string(),
            Type::Any => "any".to_string(),
            Type::Void => "void".to_string(),
            Type::Array(array_type) => {
                if let Some(size) = &array_type.size {
                    format!("[{}]{}", size, self.type_to_string(&array_type.element_type))
                } else {
                    format!("[]{}", self.type_to_string(&array_type.element_type))
                }
            }
            Type::Slice(slice_type) => {
                format!("[]{}", self.type_to_string(&slice_type.element_type))
            }
            Type::Map(map_type) => {
                format!("map[{}]{}", self.type_to_string(&map_type.key_type), self.type_to_string(&map_type.value_type))
            }
            Type::Function(func_type) => {
                let mut sig = String::from("func(");
                for (i, param) in func_type.param_types.iter().enumerate() {
                    if i > 0 {
                        sig.push_str(", ");
                    }
                    sig.push_str(&self.type_to_string(param));
                }
                sig.push(')');
                if let Some(ret) = &func_type.return_type {
                    sig.push_str(": ");
                    sig.push_str(&self.type_to_string(ret));
                }
                sig
            }
            Type::Generic(generic_type) => {
                let mut sig = generic_type.name.clone();
                if !generic_type.constraints.is_empty() {
                    sig.push('<');
                    for (i, constraint) in generic_type.constraints.iter().enumerate() {
                        if i > 0 {
                            sig.push_str(", ");
                        }
                        sig.push_str(&self.type_to_string(constraint));
                    }
                    sig.push('>');
                }
                sig
            }
            Type::Channel(channel_type) => {
                match channel_type.direction {
                    crate::ast::nodes::ChannelDirection::Send => {
                        format!("chan<- {}", self.type_to_string(&channel_type.element_type))
                    }
                    crate::ast::nodes::ChannelDirection::Receive => {
                        format!("<-chan {}", self.type_to_string(&channel_type.element_type))
                    }
                    crate::ast::nodes::ChannelDirection::Bidirectional => {
                        format!("chan {}", self.type_to_string(&channel_type.element_type))
                    }
                }
            }
            Type::Struct(struct_type) => struct_type.name.clone(),
            Type::Interface(interface_type) => interface_type.name.clone(),
            Type::Tuple(tuple_type) => {
                let mut sig = String::from("(");
                for (i, elem) in tuple_type.element_types.iter().enumerate() {
                    if i > 0 {
                        sig.push_str(", ");
                    }
                    sig.push_str(&self.type_to_string(elem));
                }
                sig.push(')');
                sig
            }
            Type::Promise(promise_type) => {
                format!("Promise<{}>", self.type_to_string(&promise_type.result_type))
            }
            Type::Named(name) => name.clone(),
        }
    }
}