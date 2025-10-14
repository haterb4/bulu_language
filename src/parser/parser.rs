//! Recursive descent parser implementation for the Bulu language

use crate::ast::*;
use crate::error::{BuluError, Result};
use crate::lexer::token::Position;
use crate::lexer::{Literal, Token, TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    file_path: Option<String>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
            file_path: None,
        }
    }

    pub fn with_file(tokens: Vec<Token>, file_path: String) -> Self {
        Self {
            tokens,
            current: 0,
            file_path: Some(file_path),
        }
    }

    /// Parse the entire program
    pub fn parse(&mut self) -> Result<Program> {
        let start_pos = self.current_position();
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // Skip newlines at the top level
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(e) => {
                    // Error recovery: skip to next statement boundary
                    self.synchronize();
                    return Err(e);
                }
            }
        }

        Ok(Program {
            statements,
            position: start_pos,
        })
    }

    // ============================================================================
    // STATEMENT PARSING
    // ============================================================================

    /// Parse a statement
    fn parse_statement(&mut self) -> Result<Statement> {
        // Collect any preceding documentation comments
        let doc_comments = self.collect_doc_comments();

        // Check for export modifier or export statement
        let is_exported = if self.check(&TokenType::Export) {
            // Look ahead to see if this is a re-export statement
            if self
                .tokens
                .get(self.current + 1)
                .map_or(false, |t| t.token_type == TokenType::LeftBrace)
            {
                // This is a re-export statement, not a modifier
                return self.parse_export_statement();
            } else {
                self.advance(); // consume 'export'
                true
            }
        } else {
            false
        };

        match self.peek().token_type {
            TokenType::Let | TokenType::Const => {
                self.parse_variable_declaration_with_docs_and_export(doc_comments, is_exported)
            }
            TokenType::Func | TokenType::Async => {
                self.parse_function_declaration_with_docs_and_export(doc_comments, is_exported)
            }
            TokenType::Struct => {
                self.parse_struct_declaration_with_docs_and_export(doc_comments, is_exported)
            }
            TokenType::Interface => {
                self.parse_interface_declaration_with_docs_and_export(doc_comments, is_exported)
            }
            TokenType::Type => self.parse_type_alias_declaration(),
            TokenType::If => self.parse_if_statement(),
            TokenType::While => self.parse_while_statement(),
            TokenType::For => self.parse_for_statement(),
            TokenType::Match => self.parse_match_statement(),
            TokenType::Select => self.parse_select_statement(),
            TokenType::Return => self.parse_return_statement(),
            TokenType::Break => self.parse_break_statement(),
            TokenType::Continue => self.parse_continue_statement(),
            TokenType::Defer => self.parse_defer_statement(),
            TokenType::Try => self.parse_try_statement(),
            TokenType::Fail => self.parse_fail_statement(),
            TokenType::Import => self.parse_import_statement(),
            TokenType::LeftBrace => self.parse_block_statement(),
            TokenType::DocComment => {
                // Skip standalone doc comments that aren't followed by declarations
                self.advance();
                self.parse_statement()
            }
            _ => {
                if is_exported {
                    return Err(self.error("Export can only be used with declarations"));
                }
                self.parse_expression_statement()
            }
        }
    }

    /// Collect documentation comments that precede a declaration
    fn collect_doc_comments(&mut self) -> Option<Vec<Token>> {
        let mut doc_comments = Vec::new();

        while self.check(&TokenType::DocComment) {
            doc_comments.push(self.advance().clone());
        }

        if doc_comments.is_empty() {
            None
        } else {
            Some(doc_comments)
        }
    }

    /// Parse variable declaration with documentation comments (backward compatibility)
    fn parse_variable_declaration_with_docs(
        &mut self,
        doc_comments: Option<Vec<Token>>,
    ) -> Result<Statement> {
        self.parse_variable_declaration_with_docs_and_export(doc_comments, false)
    }

    /// Parse function declaration with documentation comments (backward compatibility)
    fn parse_function_declaration_with_docs(
        &mut self,
        doc_comments: Option<Vec<Token>>,
    ) -> Result<Statement> {
        self.parse_function_declaration_with_docs_and_export(doc_comments, false)
    }

    /// Parse struct declaration with documentation comments (backward compatibility)
    fn parse_struct_declaration_with_docs(
        &mut self,
        doc_comments: Option<Vec<Token>>,
    ) -> Result<Statement> {
        self.parse_struct_declaration_with_docs_and_export(doc_comments, false)
    }

    /// Parse interface declaration with documentation comments (backward compatibility)
    fn parse_interface_declaration_with_docs(
        &mut self,
        doc_comments: Option<Vec<Token>>,
    ) -> Result<Statement> {
        self.parse_interface_declaration_with_docs_and_export(doc_comments, false)
    }

    /// Parse variable declaration: let x = 5, const PI = 3.14
    fn parse_variable_declaration(&mut self) -> Result<Statement> {
        let start_pos = self.current_position();
        let is_const = self.match_token(&TokenType::Const);

        if !is_const {
            self.consume(&TokenType::Let, "Expected 'let' or 'const'")?;
        }

        let name = self.consume_identifier("Expected variable name")?;

        // Optional type annotation
        let type_annotation = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional initializer
        let initializer = if self.match_token(&TokenType::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        // For const declarations, initializer is required
        if is_const && initializer.is_none() {
            return Err(self.error("Constant declarations must have an initializer"));
        }

        self.consume_statement_terminator()?;

        Ok(Statement::VariableDecl(VariableDecl {
            is_const,
            name,
            type_annotation,
            initializer,
            doc_comment: None,  // TODO: Extract doc comments from preceding tokens
            is_exported: false, // TODO: Handle export keyword
            position: start_pos,
        }))
    }

    /// Parse variable declaration with documentation comments and export flag
    fn parse_variable_declaration_with_docs_and_export(
        &mut self,
        doc_comments: Option<Vec<Token>>,
        is_exported: bool,
    ) -> Result<Statement> {
        let start_pos = self.current_position();
        let is_const = self.match_token(&TokenType::Const);

        if !is_const {
            self.consume(&TokenType::Let, "Expected 'let' or 'const'")?;
        }

        let name = self.consume_identifier("Expected variable name")?;

        // Optional type annotation
        let type_annotation = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional initializer
        let initializer = if self.match_token(&TokenType::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume_statement_terminator()?;

        Ok(Statement::VariableDecl(VariableDecl {
            is_const,
            name,
            type_annotation,
            initializer,
            doc_comment: doc_comments,
            is_exported,
            position: start_pos,
        }))
    }

    /// Parse function declaration
    fn parse_function_declaration(&mut self) -> Result<Statement> {
        let start_pos = self.current_position();

        // Check for async keyword first
        let is_async = if self.check(&TokenType::Async) {
            self.advance();
            if !self.check(&TokenType::Func) {
                return Err(self.error("Expected 'func' after 'async'"));
            }
            true
        } else {
            false
        };

        self.consume(&TokenType::Func, "Expected 'func'")?;
        let name = self.consume_identifier("Expected function name")?;

        // Type parameters (generics)
        let mut type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parameters
        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;
        let params = self.parse_parameter_list()?;
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        // Return type (can be tuple for multiple return values)
        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_return_type()?)
        } else {
            None
        };

        // Parse where clause
        if let Some(where_constraints) = self.parse_where_clause()? {
            // Merge where clause constraints with type parameters
            type_params.extend(where_constraints);
        }

        // Function body
        let body = self.parse_block_statement_body()?;

        // Function declarations don't need statement terminators

        Ok(Statement::FunctionDecl(FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body,
            is_async,
            doc_comment: None,  // TODO: Extract doc comments from preceding tokens
            is_exported: false, // TODO: Handle export keyword
            is_private: false,  // Functions are public by default
            position: start_pos,
        }))
    }

    /// Parse function declaration with documentation comments and export flag
    fn parse_function_declaration_with_docs_and_export(
        &mut self,
        doc_comments: Option<Vec<Token>>,
        is_exported: bool,
    ) -> Result<Statement> {
        let start_pos = self.current_position();

        // Check for async keyword first
        let is_async = if self.check(&TokenType::Async) {
            self.advance();
            if !self.check(&TokenType::Func) {
                return Err(self.error("Expected 'func' after 'async'"));
            }
            true
        } else {
            false
        };

        self.consume(&TokenType::Func, "Expected 'func'")?;
        let name = self.consume_identifier("Expected function name")?;

        // Type parameters (generics)
        let mut type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parameters
        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;
        let params = self.parse_parameter_list()?;
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        // Return type (can be tuple for multiple return values)
        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_return_type()?)
        } else {
            None
        };

        // Parse where clause
        if let Some(where_constraints) = self.parse_where_clause()? {
            // Merge where clause constraints with type parameters
            type_params.extend(where_constraints);
        }

        // Function body
        let body = self.parse_block_statement_body()?;

        Ok(Statement::FunctionDecl(FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body,
            is_async,
            doc_comment: doc_comments,
            is_exported,
            is_private: false,  // Functions are public by default
            position: start_pos,
        }))
    }

    /// Parse struct declaration
    fn parse_struct_declaration(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.consume(&TokenType::Struct, "Expected 'struct'")?;
        let name = self.consume_identifier("Expected struct name")?;

        // Type parameters (generics)
        let mut type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parse where clause
        if let Some(where_constraints) = self.parse_where_clause()? {
            // Merge where clause constraints with type parameters
            type_params.extend(where_constraints);
        }

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            // Check if this is a method (starts with 'func')
            if self.check(&TokenType::Func) {
                methods.push(self.parse_method_declaration()?);
            } else {
                // Parse field
                fields.push(self.parse_struct_field()?);
            }
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Statement::StructDecl(StructDecl {
            name,
            type_params,
            fields,
            methods,
            doc_comment: None,  // TODO: Extract doc comments from preceding tokens
            is_exported: false, // TODO: Handle export keyword
            position: pos,
        }))
    }

    /// Parse struct declaration with documentation comments and export flag
    fn parse_struct_declaration_with_docs_and_export(
        &mut self,
        doc_comments: Option<Vec<Token>>,
        is_exported: bool,
    ) -> Result<Statement> {
        let pos = self.current_position();
        self.consume(&TokenType::Struct, "Expected 'struct'")?;
        let name = self.consume_identifier("Expected struct name")?;

        // Type parameters (generics)
        let mut type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parse where clause
        if let Some(where_constraints) = self.parse_where_clause()? {
            // Merge where clause constraints with type parameters
            type_params.extend(where_constraints);
        }

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut fields = Vec::new();
        let mut methods = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            // Check if this is a method (starts with visibility modifier or 'func')
            if self.check(&TokenType::Pub) {
                self.advance(); // consume 'pub'
                methods.push(self.parse_method_declaration_with_visibility(false)?);
            } else if self.check(&TokenType::Priv) {
                self.advance(); // consume 'priv'
                methods.push(self.parse_method_declaration_with_visibility(true)?);
            } else if self.check(&TokenType::Func) {
                methods.push(self.parse_method_declaration()?);
            } else {
                // Parse field
                fields.push(self.parse_struct_field()?);
            }
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Statement::StructDecl(StructDecl {
            name,
            type_params,
            fields,
            methods,
            doc_comment: doc_comments,
            is_exported,
            position: pos,
        }))
    }

    /// Parse struct field
    fn parse_struct_field(&mut self) -> Result<StructField> {
        self.parse_struct_field_with_visibility(false)
    }

    /// Parse struct field with visibility
    fn parse_struct_field_with_visibility(&mut self, is_private: bool) -> Result<StructField> {
        let pos = self.current_position();
        let name = self.consume_identifier("Expected field name")?;
        self.consume(&TokenType::Colon, "Expected ':' after field name")?;
        let field_type = self.parse_type()?;

        // Optional newline or comma
        if self.check(&TokenType::Newline) || self.check(&TokenType::Comma) {
            self.advance();
        }

        Ok(StructField {
            name,
            field_type,
            is_private,
            position: pos,
        })
    }

    /// Parse method declaration (similar to function but inside struct)
    fn parse_method_declaration(&mut self) -> Result<FunctionDecl> {
        self.parse_method_declaration_with_visibility(false)
    }

    /// Parse method declaration with visibility
    fn parse_method_declaration_with_visibility(&mut self, is_private: bool) -> Result<FunctionDecl> {
        let start_pos = self.current_position();

        self.consume(&TokenType::Func, "Expected 'func'")?;
        let name = self.consume_identifier("Expected method name")?;

        // Type parameters (generics)
        let type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parameters
        self.consume(&TokenType::LeftParen, "Expected '(' after method name")?;
        let params = self.parse_parameter_list()?;
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        // Return type (can be tuple for multiple return values)
        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_return_type()?)
        } else {
            None
        };

        // Method body
        let body = self.parse_block_statement_body()?;

        Ok(FunctionDecl {
            name,
            type_params,
            params,
            return_type,
            body,
            is_async: false,
            doc_comment: None,  // TODO: Extract doc comments from preceding tokens
            is_exported: false, // TODO: Handle export keyword
            is_private,
            position: start_pos,
        })
    }

    /// Parse interface declaration
    fn parse_interface_declaration(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.consume(&TokenType::Interface, "Expected 'interface'")?;
        let name = self.consume_identifier("Expected interface name")?;

        // Type parameters (generics)
        let mut type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parse where clause
        if let Some(where_constraints) = self.parse_where_clause()? {
            // Merge where clause constraints with type parameters
            type_params.extend(where_constraints);
        }

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut methods = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            // Parse method signature
            methods.push(self.parse_interface_method()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Statement::InterfaceDecl(InterfaceDecl {
            name,
            type_params,
            methods,
            doc_comment: None,  // TODO: Extract doc comments from preceding tokens
            is_exported: false, // TODO: Handle export keyword
            position: pos,
        }))
    }

    /// Parse interface declaration with documentation comments and export flag
    fn parse_interface_declaration_with_docs_and_export(
        &mut self,
        doc_comments: Option<Vec<Token>>,
        is_exported: bool,
    ) -> Result<Statement> {
        let pos = self.current_position();
        self.consume(&TokenType::Interface, "Expected 'interface'")?;
        let name = self.consume_identifier("Expected interface name")?;

        // Type parameters (generics)
        let mut type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        // Parse where clause
        if let Some(where_constraints) = self.parse_where_clause()? {
            // Merge where clause constraints with type parameters
            type_params.extend(where_constraints);
        }

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut methods = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            // Parse method signature
            methods.push(self.parse_interface_method()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Statement::InterfaceDecl(InterfaceDecl {
            name,
            type_params,
            methods,
            doc_comment: doc_comments,
            is_exported,
            position: pos,
        }))
    }

    /// Parse type alias declaration
    fn parse_type_alias_declaration(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.consume(&TokenType::Type, "Expected 'type'")?;
        let name = self.consume_identifier("Expected type alias name")?;

        // Type parameters (generics)
        let mut type_params = if self.match_token(&TokenType::Less) {
            self.parse_type_parameters()?
        } else {
            Vec::new()
        };

        self.consume(&TokenType::Assign, "Expected '=' after type alias name")?;
        let target_type = self.parse_type()?;

        // Parse where clause
        if let Some(where_constraints) = self.parse_where_clause()? {
            // Merge where clause constraints with type parameters
            type_params.extend(where_constraints);
        }

        self.consume_statement_terminator()?;

        Ok(Statement::TypeAlias(TypeAliasDecl {
            name,
            type_params,
            target_type,
            position: pos,
        }))
    }

    /// Parse interface method signature
    fn parse_interface_method(&mut self) -> Result<InterfaceMethod> {
        let pos = self.current_position();
        self.consume(&TokenType::Func, "Expected 'func'")?;
        let name = self.consume_identifier("Expected method name")?;

        // Parameters
        self.consume(&TokenType::LeftParen, "Expected '(' after method name")?;
        let params = self.parse_parameter_list()?;
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        // Return type
        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        // Optional newline or semicolon
        if self.check(&TokenType::Newline) || self.check(&TokenType::Semicolon) {
            self.advance();
        }

        Ok(InterfaceMethod {
            name,
            params,
            return_type,
            position: pos,
        })
    }

    /// Parse if statement (stub for now)
    fn parse_if_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'if'
        let condition = self.parse_expression()?;
        let then_branch = self.parse_block_statement_body()?;

        let else_branch = if self.match_token(&TokenType::Else) {
            if self.check(&TokenType::If) {
                Some(Box::new(self.parse_if_statement()?))
            } else {
                Some(Box::new(Statement::Block(
                    self.parse_block_statement_body()?,
                )))
            }
        } else {
            None
        };

        // If statements don't need statement terminators

        Ok(Statement::If(IfStmt {
            condition,
            then_branch,
            else_branch,
            position: pos,
        }))
    }

    /// Parse while statement (stub for now)
    fn parse_while_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'while'
        let condition = self.parse_expression()?;
        let body = self.parse_block_statement_body()?;

        // While statements don't need statement terminators

        Ok(Statement::While(WhileStmt {
            condition,
            body,
            position: pos,
        }))
    }

    /// Parse for statement with support for range syntax and for-in loops
    fn parse_for_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'for'

        // Parse variable name (could be single variable or tuple for index, value)
        let variable = self.consume_identifier("Expected variable name")?;

        // Check for optional second variable (for index, value iteration)
        let (index_variable, value_variable) = if self.match_token(&TokenType::Comma) {
            let value_var = self.consume_identifier("Expected value variable name after comma")?;
            (Some(variable), value_var)
        } else {
            (None, variable)
        };

        self.consume(&TokenType::In, "Expected 'in'")?;
        let iterable = self.parse_expression()?;
        let body = self.parse_block_statement_body()?;

        // For statements don't need statement terminators

        Ok(Statement::For(ForStmt {
            variable: value_variable,
            index_variable,
            iterable,
            body,
            position: pos,
        }))
    }

    /// Parse match statement
    fn parse_match_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'match'
        let expr = self.parse_expression()?;

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut arms = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            arms.push(self.parse_match_arm()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        // Match statements don't need statement terminators

        Ok(Statement::Match(MatchStmt {
            expr,
            arms,
            position: pos,
        }))
    }

    /// Parse select statement
    fn parse_select_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'select'

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut arms = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            arms.push(self.parse_select_arm()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        // Select statements don't need statement terminators

        Ok(Statement::Select(SelectStmt {
            arms,
            position: pos,
        }))
    }

    /// Parse a select arm
    fn parse_select_arm(&mut self) -> Result<SelectStmtArm> {
        let pos = self.current_position();

        // Check for default case
        if self.check(&TokenType::Identifier) && self.peek().lexeme == "_" {
            self.advance(); // consume '_'
            self.consume(&TokenType::RightArrow, "Expected '->' after default case")?;

            let body = if self.check(&TokenType::LeftBrace) {
                self.parse_block_statement()?
            } else {
                // Parse as expression statement to handle match expressions correctly
                let expr_pos = self.current_position();
                let expr = self.parse_expression()?;
                Statement::Expression(ExpressionStmt {
                    expr,
                    position: expr_pos,
                })
            };

            // Optional comma or newline
            if self.check(&TokenType::Comma) || self.check(&TokenType::Newline) {
                self.advance();
            }

            return Ok(SelectStmtArm {
                channel_op: None,
                body,
                position: pos,
            });
        }

        // Parse channel operation
        let channel_op = self.parse_channel_operation()?;

        self.consume(
            &TokenType::RightArrow,
            "Expected '->' after channel operation",
        )?;

        let body = if self.check(&TokenType::LeftBrace) {
            self.parse_block_statement()?
        } else {
            // Parse as expression statement to handle match expressions correctly
            let expr_pos = self.current_position();
            let expr = self.parse_expression()?;
            Statement::Expression(ExpressionStmt {
                expr,
                position: expr_pos,
            })
        };

        // Optional comma or newline
        if self.check(&TokenType::Comma) || self.check(&TokenType::Newline) {
            self.advance();
        }

        Ok(SelectStmtArm {
            channel_op: Some(channel_op),
            body,
            position: pos,
        })
    }

    /// Parse channel operation for select statement
    fn parse_channel_operation(&mut self) -> Result<ChannelOperation> {
        let pos = self.current_position();

        // Check if this is a receive operation: value := <-channel
        if self.check(&TokenType::Identifier) {
            let var_name = self.consume_identifier("Expected variable name")?;

            if self.match_token(&TokenType::Colon) && self.match_token(&TokenType::Assign) {
                // This is a receive with assignment: value := <-channel
                self.consume(&TokenType::LeftArrow, "Expected '<-' for channel receive")?;
                let channel = self.parse_expression()?;

                return Ok(ChannelOperation {
                    is_send: false,
                    channel,
                    value: None,
                    variable: Some(var_name),
                    position: pos,
                });
            } else {
                // This might be a send operation, backtrack
                // Put the identifier back by creating an identifier expression
                let channel_expr = Expression::Identifier(IdentifierExpr {
                    name: var_name,
                    position: pos,
                });

                // Check for send operation: channel <- value
                if self.match_token(&TokenType::LeftArrow) {
                    let value = self.parse_expression()?;
                    return Ok(ChannelOperation {
                        is_send: true,
                        channel: channel_expr,
                        value: Some(value),
                        variable: None,
                        position: pos,
                    });
                } else {
                    return Err(self.error("Expected channel operation"));
                }
            }
        }

        // Check for receive operation without assignment: <-channel
        if self.match_token(&TokenType::LeftArrow) {
            let channel = self.parse_expression()?;
            return Ok(ChannelOperation {
                is_send: false,
                channel,
                value: None,
                variable: None,
                position: pos,
            });
        }

        // Parse as expression and check for send operation
        let expr = self.parse_expression()?;

        if self.match_token(&TokenType::LeftArrow) {
            let value = self.parse_expression()?;
            Ok(ChannelOperation {
                is_send: true,
                channel: expr,
                value: Some(value),
                variable: None,
                position: pos,
            })
        } else {
            Err(self.error("Expected channel operation"))
        }
    }

    /// Parse a match arm: pattern -> statement
    fn parse_match_arm(&mut self) -> Result<MatchArm> {
        let pos = self.current_position();
        let pattern = self.parse_pattern()?;

        // Optional guard clause
        let guard = if self.match_token(&TokenType::If) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenType::RightArrow, "Expected '->' after match pattern")?;

        // Parse the body - can be a single statement or a block
        let body = if self.check(&TokenType::LeftBrace) {
            self.parse_block_statement()?
        } else {
            self.parse_statement()?
        };

        // Optional comma or newline
        if self.check(&TokenType::Comma) || self.check(&TokenType::Newline) {
            self.advance();
        }

        Ok(MatchArm {
            pattern,
            guard,
            body,
            position: pos,
        })
    }

    /// Parse a pattern for match expressions
    fn parse_pattern(&mut self) -> Result<Pattern> {
        self.parse_or_pattern()
    }

    /// Parse OR pattern (pattern1 | pattern2)
    fn parse_or_pattern(&mut self) -> Result<Pattern> {
        let mut patterns = vec![self.parse_primary_pattern()?];

        while self.match_token(&TokenType::Pipe) {
            patterns.push(self.parse_primary_pattern()?);
        }

        if patterns.len() == 1 {
            Ok(patterns.into_iter().next().unwrap())
        } else {
            let pos = patterns[0].position();
            Ok(Pattern::Or(OrPattern {
                patterns,
                position: pos,
            }))
        }
    }

    /// Parse primary pattern
    fn parse_primary_pattern(&mut self) -> Result<Pattern> {
        let pos = self.current_position();

        match &self.peek().token_type {
            // Wildcard pattern
            TokenType::Identifier if self.peek().lexeme == "_" => {
                self.advance();
                Ok(Pattern::Wildcard(pos))
            }

            // Literal patterns (check for range patterns first)
            TokenType::IntegerLiteral => {
                // Check if this is a range pattern by looking ahead
                if self.peek_ahead(1).map(|t| &t.token_type) == Some(&TokenType::DotDotDot)
                    || self.peek_ahead(1).map(|t| &t.token_type) == Some(&TokenType::DotDotLess)
                {
                    self.parse_range_pattern()
                } else {
                    // Regular literal pattern
                    if let Some(crate::lexer::Literal::Integer(value)) = &self.peek().literal {
                        let value = *value;
                        self.advance();
                        Ok(Pattern::Literal(LiteralValue::Integer(value), pos))
                    } else {
                        Err(self.error("Invalid integer literal"))
                    }
                }
            }

            TokenType::FloatLiteral => {
                // Check if this is a range pattern by looking ahead
                if self.peek_ahead(1).map(|t| &t.token_type) == Some(&TokenType::DotDotDot)
                    || self.peek_ahead(1).map(|t| &t.token_type) == Some(&TokenType::DotDotLess)
                {
                    self.parse_range_pattern()
                } else {
                    // Regular literal pattern
                    if let Some(crate::lexer::Literal::Float(value)) = &self.peek().literal {
                        let value = *value;
                        self.advance();
                        Ok(Pattern::Literal(LiteralValue::Float(value), pos))
                    } else {
                        Err(self.error("Invalid float literal"))
                    }
                }
            }

            TokenType::StringLiteral => {
                if let Some(crate::lexer::Literal::String(value)) = &self.peek().literal {
                    let value = value.clone();
                    self.advance();
                    Ok(Pattern::Literal(LiteralValue::String(value), pos))
                } else {
                    Err(self.error("Invalid string literal"))
                }
            }

            TokenType::CharLiteral => {
                if let Some(crate::lexer::Literal::Char(value)) = &self.peek().literal {
                    let value = *value;
                    self.advance();
                    Ok(Pattern::Literal(LiteralValue::Char(value), pos))
                } else {
                    Err(self.error("Invalid char literal"))
                }
            }

            TokenType::True => {
                self.advance();
                Ok(Pattern::Literal(LiteralValue::Boolean(true), pos))
            }

            TokenType::False => {
                self.advance();
                Ok(Pattern::Literal(LiteralValue::Boolean(false), pos))
            }

            TokenType::Null => {
                self.advance();
                Ok(Pattern::Literal(LiteralValue::Null, pos))
            }

            // Identifier patterns (variables or struct patterns)
            TokenType::Identifier => {
                let name = self.consume_identifier("Expected identifier")?;

                // Check if this is a struct pattern
                if self.check(&TokenType::LeftBrace) {
                    self.parse_struct_pattern(name)
                } else {
                    // Variable binding pattern
                    Ok(Pattern::Identifier(name, pos))
                }
            }

            // Array patterns
            TokenType::LeftBracket => self.parse_array_pattern(),

            // Parenthesized patterns
            TokenType::LeftParen => {
                self.advance(); // consume '('
                let pattern = self.parse_pattern()?;
                self.consume(&TokenType::RightParen, "Expected ')' after pattern")?;
                Ok(pattern)
            }

            _ => Err(self.error("Expected pattern")),
        }
    }

    /// Parse range pattern (e.g., 1...10, 0..<100)
    fn parse_range_pattern(&mut self) -> Result<Pattern> {
        let pos = self.current_position();

        // Parse start value
        let start = match &self.peek().token_type {
            TokenType::IntegerLiteral => {
                if let Some(crate::lexer::Literal::Integer(value)) = &self.peek().literal {
                    let value = *value;
                    self.advance();
                    LiteralValue::Integer(value)
                } else {
                    return Err(self.error("Invalid integer literal"));
                }
            }
            TokenType::FloatLiteral => {
                if let Some(crate::lexer::Literal::Float(value)) = &self.peek().literal {
                    let value = *value;
                    self.advance();
                    LiteralValue::Float(value)
                } else {
                    return Err(self.error("Invalid float literal"));
                }
            }
            _ => return Err(self.error("Expected number in range pattern")),
        };

        // Parse range operator
        let inclusive = if self.match_token(&TokenType::DotDotDot) {
            true
        } else if self.match_token(&TokenType::DotDotLess) {
            false
        } else {
            return Err(self.error("Expected range operator (...) or (..<)"));
        };

        // Parse end value
        let end = match &self.peek().token_type {
            TokenType::IntegerLiteral => {
                if let Some(crate::lexer::Literal::Integer(value)) = &self.peek().literal {
                    let value = *value;
                    self.advance();
                    LiteralValue::Integer(value)
                } else {
                    return Err(self.error("Invalid integer literal"));
                }
            }
            TokenType::FloatLiteral => {
                if let Some(crate::lexer::Literal::Float(value)) = &self.peek().literal {
                    let value = *value;
                    self.advance();
                    LiteralValue::Float(value)
                } else {
                    return Err(self.error("Invalid float literal"));
                }
            }
            _ => return Err(self.error("Expected number in range pattern")),
        };

        Ok(Pattern::Range(RangePattern {
            start,
            end,
            inclusive,
            position: pos,
        }))
    }

    /// Parse struct pattern (e.g., Point{x: 1, y: 2})
    fn parse_struct_pattern(&mut self, name: String) -> Result<Pattern> {
        let pos = self.current_position();

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut fields = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            let field_pos = self.current_position();
            let field_name = self.consume_identifier("Expected field name")?;
            self.consume(&TokenType::Colon, "Expected ':' after field name")?;
            let pattern = self.parse_pattern()?;

            fields.push(FieldPattern {
                name: field_name,
                pattern: Box::new(pattern),
                position: field_pos,
            });

            // Optional comma
            if self.check(&TokenType::Comma) {
                self.advance();
            } else if !self.check(&TokenType::RightBrace) && !self.check(&TokenType::Newline) {
                return Err(self.error("Expected ',' or '}' after field pattern"));
            }
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Pattern::Struct(StructPattern {
            name,
            fields,
            position: pos,
        }))
    }

    /// Parse array pattern (e.g., [1, 2, x])
    fn parse_array_pattern(&mut self) -> Result<Pattern> {
        let pos = self.current_position();

        self.consume(&TokenType::LeftBracket, "Expected '['")?;

        let mut elements = Vec::new();

        while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
            elements.push(self.parse_pattern()?);

            if !self.check(&TokenType::RightBracket) {
                self.consume(
                    &TokenType::Comma,
                    "Expected ',' between array pattern elements",
                )?;
            }
        }

        self.consume(&TokenType::RightBracket, "Expected ']'")?;

        Ok(Pattern::Array(ArrayPattern {
            elements,
            position: pos,
        }))
    }

    /// Parse match expression
    fn parse_match_expression(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.advance(); // consume 'match'
        let expr = Box::new(self.parse_expression()?);

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut arms = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            arms.push(self.parse_match_expr_arm()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Expression::Match(MatchExpr {
            expr,
            arms,
            position: pos,
        }))
    }

    /// Parse a match expression arm: pattern -> expression
    fn parse_match_expr_arm(&mut self) -> Result<MatchExprArm> {
        let pos = self.current_position();
        let pattern = self.parse_pattern()?;

        // Optional guard clause
        let guard = if self.match_token(&TokenType::If) {
            Some(self.parse_expression()?)
        } else {
            None
        };

        self.consume(&TokenType::RightArrow, "Expected '->' after match pattern")?;

        // Parse the expression
        let expr = self.parse_expression()?;

        // Optional comma or newline
        if self.check(&TokenType::Comma) || self.check(&TokenType::Newline) {
            self.advance();
        }

        Ok(MatchExprArm {
            pattern,
            guard,
            expr,
            position: pos,
        })
    }

    /// Parse select expression
    fn parse_select_expression(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.advance(); // consume 'select'

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut arms = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            arms.push(self.parse_select_expr_arm()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Expression::Select(SelectExpr {
            arms,
            position: pos,
        }))
    }

    /// Parse a select expression arm
    fn parse_select_expr_arm(&mut self) -> Result<SelectExprArm> {
        let pos = self.current_position();

        // Check for default case
        if self.check(&TokenType::Identifier) && self.peek().lexeme == "_" {
            self.advance(); // consume '_'
            self.consume(&TokenType::RightArrow, "Expected '->' after default case")?;

            let expr = self.parse_expression()?;

            // Optional comma or newline
            if self.check(&TokenType::Comma) || self.check(&TokenType::Newline) {
                self.advance();
            }

            return Ok(SelectExprArm {
                channel_op: None,
                expr,
                position: pos,
            });
        }

        // Parse channel operation
        let channel_op = self.parse_channel_operation()?;

        self.consume(
            &TokenType::RightArrow,
            "Expected '->' after channel operation",
        )?;

        let expr = self.parse_expression()?;

        // Optional comma or newline
        if self.check(&TokenType::Comma) || self.check(&TokenType::Newline) {
            self.advance();
        }

        Ok(SelectExprArm {
            channel_op: Some(channel_op),
            expr,
            position: pos,
        })
    }

    /// Parse return statement
    fn parse_return_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'return'

        let value = if self.check(&TokenType::Newline)
            || self.check(&TokenType::Semicolon)
            || self.is_at_end()
        {
            None
        } else {
            // Parse multiple return values as tuple expression
            let first_expr = self.parse_expression()?;

            // Check if there are more expressions (comma-separated)
            if self.match_token(&TokenType::Comma) {
                let mut expressions = vec![first_expr];

                loop {
                    expressions.push(self.parse_expression()?);

                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                }

                // Create a tuple expression for multiple return values
                Some(Expression::Tuple(TupleExpr {
                    elements: expressions,
                    position: pos,
                }))
            } else {
                Some(first_expr)
            }
        };

        self.consume_statement_terminator()?;

        Ok(Statement::Return(ReturnStmt {
            value,
            position: pos,
        }))
    }

    /// Parse break statement
    fn parse_break_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'break'
        self.consume_statement_terminator()?;

        Ok(Statement::Break(BreakStmt { position: pos }))
    }

    /// Parse continue statement
    fn parse_continue_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'continue'
        self.consume_statement_terminator()?;

        Ok(Statement::Continue(ContinueStmt { position: pos }))
    }

    /// Parse defer statement (stub for now)
    fn parse_defer_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'defer'
        let stmt = Box::new(self.parse_statement()?);

        Ok(Statement::Defer(DeferStmt {
            stmt,
            position: pos,
        }))
    }

    /// Parse try statement with optional fail on clause
    fn parse_try_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'try'
        let body = self.parse_block_statement_body()?;

        // Check for optional 'fail on' clause
        let catch_clause = if self.match_token(&TokenType::Fail) {
            if self
                .consume(&TokenType::Identifier, "Expected 'on' after 'fail'")
                .is_ok()
                && self.previous().lexeme == "on"
            {
                // Optional error variable binding
                let error_var = if self.check(&TokenType::Identifier) {
                    Some(self.consume_identifier("Expected error variable name")?)
                } else {
                    None
                };

                let catch_body = self.parse_block_statement_body()?;

                Some(CatchClause {
                    error_var,
                    body: catch_body,
                    position: self.current_position(),
                })
            } else {
                return Err(self.error("Expected 'on' after 'fail' in try statement"));
            }
        } else {
            None
        };

        Ok(Statement::Try(TryStmt {
            body,
            catch_clause,
            position: pos,
        }))
    }

    /// Parse fail statement - can be used to throw errors or handle them
    fn parse_fail_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'fail'
        let message = self.parse_expression()?;
        self.consume_statement_terminator()?;

        Ok(Statement::Fail(FailStmt {
            message,
            position: pos,
        }))
    }

    /// Parse import statement
    fn parse_import_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'import'

        // Check for destructuring import: import { item1, item2 } from "path"
        if self.check(&TokenType::LeftBrace) {
            self.advance(); // consume '{'

            let mut items = Vec::new();

            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                // Skip newlines inside import braces
                while self.check(&TokenType::Newline) {
                    self.advance();
                }

                // Check if we've reached the end after skipping newlines
                if self.check(&TokenType::RightBrace) {
                    break;
                }

                let name = self.consume_identifier("Expected import item name")?;

                // Check for alias: import { item as alias }
                let alias = if self.match_token(&TokenType::As) {
                    Some(self.consume_identifier("Expected alias name")?)
                } else {
                    None
                };

                items.push(ImportItem {
                    name,
                    alias,
                    position: self.current_position(),
                });

                // Skip newlines after import item
                while self.check(&TokenType::Newline) {
                    self.advance();
                }

                if !self.check(&TokenType::RightBrace) {
                    if self.check(&TokenType::Comma) {
                        self.advance(); // consume comma
                                        // Skip newlines after comma
                        while self.check(&TokenType::Newline) {
                            self.advance();
                        }
                    } else if !self.check(&TokenType::Newline) {
                        // Only require comma if not followed by newline or closing brace
                        return Err(self.error("Expected ',' between import items"));
                    }
                }
            }

            self.consume(&TokenType::RightBrace, "Expected '}'")?;

            // Expect 'from' keyword (using identifier for now)
            if self.check(&TokenType::Identifier) && self.peek().lexeme == "from" {
                self.advance();
            } else {
                return Err(self.error("Expected 'from' after import items"));
            }

            let path = if let Some(Literal::String(s)) = &self.peek().literal {
                let path = s.clone();
                self.advance();
                path
            } else {
                return Err(self.error("Expected import path string"));
            };

            self.consume_statement_terminator()?;

            return Ok(Statement::Import(ImportStmt {
                path,
                alias: None,
                items: Some(items),
                position: pos,
            }));
        }

        // Regular import: import "path" or import "path" as alias
        let path = if let Some(Literal::String(s)) = &self.peek().literal {
            let path = s.clone();
            self.advance();
            path
        } else if self.check(&TokenType::Identifier) {
            // Handle bare identifier imports like: import std
            let path = self.consume_identifier("Expected import path")?;
            path
        } else {
            return Err(self.error("Expected import path"));
        };

        // Check for alias: import "path" as alias
        let alias = if self.match_token(&TokenType::As) {
            Some(self.consume_identifier("Expected alias name")?)
        } else {
            None
        };

        self.consume_statement_terminator()?;

        Ok(Statement::Import(ImportStmt {
            path,
            alias,
            items: None,
            position: pos,
        }))
    }

    /// Parse export statement
    fn parse_export_statement(&mut self) -> Result<Statement> {
        let pos = self.current_position();
        self.advance(); // consume 'export'

        // Check for re-export: export { item1, item2 } from "path"
        if self.check(&TokenType::LeftBrace) {
            self.advance(); // consume '{'

            let mut items = Vec::new();

            while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
                // Skip newlines inside export braces
                while self.check(&TokenType::Newline) {
                    self.advance();
                }

                // Check if we've reached the end after skipping newlines
                if self.check(&TokenType::RightBrace) {
                    break;
                }

                let name = self.consume_identifier("Expected export item name")?;

                // Check for alias: export { item as alias }
                let alias = if self.match_token(&TokenType::As) {
                    Some(self.consume_identifier("Expected alias name")?)
                } else {
                    None
                };

                items.push(ImportItem {
                    name,
                    alias,
                    position: self.current_position(),
                });

                // Skip newlines after export item
                while self.check(&TokenType::Newline) {
                    self.advance();
                }

                if !self.check(&TokenType::RightBrace) {
                    if self.check(&TokenType::Comma) {
                        self.advance(); // consume comma
                                        // Skip newlines after comma
                        while self.check(&TokenType::Newline) {
                            self.advance();
                        }
                    } else if !self.check(&TokenType::Newline) {
                        // Only require comma if not followed by newline or closing brace
                        return Err(self.error("Expected ',' between export items"));
                    }
                }
            }

            self.consume(&TokenType::RightBrace, "Expected '}'")?;

            // Check for 'from' keyword for re-export
            if self.check(&TokenType::Identifier) && self.peek().lexeme == "from" {
                self.advance();

                let path = if let Some(Literal::String(s)) = &self.peek().literal {
                    let path = s.clone();
                    self.advance();
                    path
                } else {
                    return Err(self.error("Expected re-export path string"));
                };

                // Create a re-export statement (import + export)
                let import_stmt = ImportStmt {
                    path,
                    alias: None,
                    items: Some(items),
                    position: pos,
                };

                return Ok(Statement::Export(ExportStmt {
                    item: Box::new(Statement::Import(import_stmt)),
                    position: pos,
                }));
            } else {
                // Export specific items from current module (not implemented yet)
                return Err(
                    self.error("Export of specific items from current module not yet implemented")
                );
            }
        }

        // Regular export: export declaration
        // But first check if we already handled this case above
        if self.check(&TokenType::LeftBrace) {
            return Err(
                self.error("Unexpected '{' - re-export syntax should have been handled above")
            );
        }

        let item = Box::new(self.parse_statement()?);

        Ok(Statement::Export(ExportStmt {
            item,
            position: pos,
        }))
    }

    /// Parse block statement
    fn parse_block_statement(&mut self) -> Result<Statement> {
        let body = self.parse_block_statement_body()?;
        Ok(Statement::Block(body))
    }

    /// Parse block statement body (the actual block)
    fn parse_block_statement_body(&mut self) -> Result<BlockStmt> {
        let start_pos = self.current_position();
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            // Skip newlines inside blocks
            if self.check(&TokenType::Newline) {
                self.advance();
                continue;
            }

            statements.push(self.parse_statement()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(BlockStmt {
            statements,
            position: start_pos,
        })
    }

    /// Parse expression statement
    fn parse_expression_statement(&mut self) -> Result<Statement> {
        let start_pos = self.current_position();
        let expr = self.parse_expression()?;
        self.consume_statement_terminator()?;

        Ok(Statement::Expression(ExpressionStmt {
            expr,
            position: start_pos,
        }))
    }

    // ============================================================================
    // EXPRESSION PARSING
    // ============================================================================

    /// Parse expression with precedence climbing
    pub fn parse_expression(&mut self) -> Result<Expression> {
        self.parse_assignment()
    }

    /// Parse assignment expression
    fn parse_assignment(&mut self) -> Result<Expression> {
        let expr = self.parse_or()?;

        // Check for channel send operation first
        if self.match_token(&TokenType::LeftArrow) {
            let value = self.parse_assignment()?;
            let pos = expr.position();
            return Ok(Expression::Channel(ChannelExpr {
                direction: ChannelDirection::Send,
                channel: Box::new(expr),
                value: Some(Box::new(value)),
                position: pos,
            }));
        }

        if self.match_tokens(&[
            TokenType::Assign,
            TokenType::PlusAssign,
            TokenType::MinusAssign,
            TokenType::StarAssign,
            TokenType::SlashAssign,
            TokenType::PercentAssign,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Assign => AssignmentOperator::Assign,
                TokenType::PlusAssign => AssignmentOperator::AddAssign,
                TokenType::MinusAssign => AssignmentOperator::SubtractAssign,
                TokenType::StarAssign => AssignmentOperator::MultiplyAssign,
                TokenType::SlashAssign => AssignmentOperator::DivideAssign,
                TokenType::PercentAssign => AssignmentOperator::ModuloAssign,
                _ => unreachable!(),
            };

            let value = self.parse_assignment()?;
            let pos = expr.position();

            return Ok(Expression::Assignment(AssignmentExpr {
                target: Box::new(expr),
                operator,
                value: Box::new(value),
                position: pos,
            }));
        }

        Ok(expr)
    }

    /// Parse logical OR expression
    fn parse_or(&mut self) -> Result<Expression> {
        let mut expr = self.parse_and()?;

        while self.match_token(&TokenType::Or) {
            let pos = expr.position();
            let right = self.parse_and()?;
            expr = Expression::Binary(BinaryExpr {
                left: Box::new(expr),
                operator: BinaryOperator::Or,
                right: Box::new(right),
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse logical AND expression
    fn parse_and(&mut self) -> Result<Expression> {
        let mut expr = self.parse_equality()?;

        while self.match_token(&TokenType::And) {
            let pos = expr.position();
            let right = self.parse_equality()?;
            expr = Expression::Binary(BinaryExpr {
                left: Box::new(expr),
                operator: BinaryOperator::And,
                right: Box::new(right),
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse equality expression
    fn parse_equality(&mut self) -> Result<Expression> {
        let mut expr = self.parse_comparison()?;

        while self.match_tokens(&[TokenType::Equal, TokenType::NotEqual]) {
            let operator = match self.previous().token_type {
                TokenType::Equal => BinaryOperator::Equal,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                _ => unreachable!(),
            };
            let pos = expr.position();
            let right = self.parse_comparison()?;
            expr = Expression::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse comparison expression
    fn parse_comparison(&mut self) -> Result<Expression> {
        let mut expr = self.parse_range()?;

        while self.match_tokens(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                _ => unreachable!(),
            };
            let pos = expr.position();
            let right = self.parse_range()?;
            expr = Expression::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse range expression (0..<10, 0...10, 0..<100 step 5)
    fn parse_range(&mut self) -> Result<Expression> {
        let mut expr = self.parse_term()?;

        if self.match_tokens(&[TokenType::DotDotLess, TokenType::DotDotDot]) {
            let inclusive = match self.previous().token_type {
                TokenType::DotDotLess => false, // 0..<10 (exclusive)
                TokenType::DotDotDot => true,   // 0...10 (inclusive)
                _ => unreachable!(),
            };
            let pos = expr.position();
            let end = self.parse_term()?;

            // Check for optional step
            let step = if self.match_token(&TokenType::Step) {
                Some(Box::new(self.parse_term()?))
            } else {
                None
            };

            expr = Expression::Range(RangeExpr {
                start: Box::new(expr),
                end: Box::new(end),
                step,
                inclusive,
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse term expression (+ -)
    fn parse_term(&mut self) -> Result<Expression> {
        let mut expr = self.parse_factor()?;

        while self.match_tokens(&[TokenType::Minus, TokenType::Plus]) {
            let operator = match self.previous().token_type {
                TokenType::Minus => BinaryOperator::Subtract,
                TokenType::Plus => BinaryOperator::Add,
                _ => unreachable!(),
            };
            let pos = expr.position();
            let right = self.parse_factor()?;
            expr = Expression::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse factor expression (* / %)
    fn parse_factor(&mut self) -> Result<Expression> {
        let mut expr = self.parse_power()?;

        while self.match_tokens(&[TokenType::Slash, TokenType::Star, TokenType::Percent]) {
            let operator = match self.previous().token_type {
                TokenType::Slash => BinaryOperator::Divide,
                TokenType::Star => BinaryOperator::Multiply,
                TokenType::Percent => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let pos = expr.position();
            let right = self.parse_power()?;
            expr = Expression::Binary(BinaryExpr {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse power expression (**)
    fn parse_power(&mut self) -> Result<Expression> {
        let mut expr = self.parse_unary()?;

        // Power is right-associative, so we use recursion instead of a loop
        if self.match_token(&TokenType::Power) {
            let pos = expr.position();
            let right = self.parse_power()?; // Right-associative recursion
            expr = Expression::Binary(BinaryExpr {
                left: Box::new(expr),
                operator: BinaryOperator::Power,
                right: Box::new(right),
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Parse unary expression
    fn parse_unary(&mut self) -> Result<Expression> {
        // Check for channel receive operation
        if self.match_token(&TokenType::LeftArrow) {
            let pos = self.previous().position;
            let channel = self.parse_unary()?;
            return Ok(Expression::Channel(ChannelExpr {
                direction: ChannelDirection::Receive,
                channel: Box::new(channel),
                value: None,
                position: pos,
            }));
        }

        if self.match_tokens(&[TokenType::Not, TokenType::Minus, TokenType::Plus]) {
            let operator = match self.previous().token_type {
                TokenType::Not => UnaryOperator::Not,
                TokenType::Minus => UnaryOperator::Minus,
                TokenType::Plus => UnaryOperator::Plus,
                _ => unreachable!(),
            };
            let pos = self.previous().position;
            let right = self.parse_unary()?;
            return Ok(Expression::Unary(UnaryExpr {
                operator,
                operand: Box::new(right),
                position: pos,
            }));
        }

        self.parse_call()
    }

    /// Parse call expression
    fn parse_call(&mut self) -> Result<Expression> {
        let mut expr = self.parse_cast()?;

        loop {
            if self.match_token(&TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenType::Dot) {
                let name = self.consume_identifier("Expected property name after '.'")?;
                let pos = expr.position();
                expr = Expression::MemberAccess(MemberAccessExpr {
                    object: Box::new(expr),
                    member: name,
                    position: pos,
                });
            } else if self.match_token(&TokenType::LeftBracket) {
                let index = self.parse_expression()?;
                self.consume(&TokenType::RightBracket, "Expected ']' after index")?;
                let pos = expr.position();
                expr = Expression::Index(IndexExpr {
                    object: Box::new(expr),
                    index: Box::new(index),
                    position: pos,
                });
            } else if self.check(&TokenType::LeftBrace) {
                // Check if this is a struct literal (TypeName{...})
                if let Expression::Identifier(_) = expr {
                    // Look ahead to see if this looks like a struct literal
                    // A struct literal should have: { identifier : ... } or be empty { }
                    if self.looks_like_struct_literal() {
                        expr = self.finish_struct_literal(expr)?;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse cast expression (expr as Type)
    fn parse_cast(&mut self) -> Result<Expression> {
        let mut expr = self.parse_primary()?;

        while self.match_token(&TokenType::As) {
            let target_type = self.parse_type()?;
            let pos = expr.position();
            expr = Expression::Cast(CastExpr {
                expr: Box::new(expr),
                target_type,
                position: pos,
            });
        }

        Ok(expr)
    }

    /// Check if the upcoming tokens look like a struct literal
    fn looks_like_struct_literal(&self) -> bool {
        // Look ahead to see what's after the '{'
        if let Some(token1) = self.peek_ahead(1) {
            match &token1.token_type {
                // Empty struct literal: {}
                TokenType::RightBrace => true,
                // Field in struct literal: { identifier : ... }
                TokenType::Identifier => {
                    if let Some(token2) = self.peek_ahead(2) {
                        // Skip potential newlines
                        if token2.token_type == TokenType::Newline {
                            if let Some(token3) = self.peek_ahead(3) {
                                token3.token_type == TokenType::Colon
                            } else {
                                false
                            }
                        } else {
                            token2.token_type == TokenType::Colon
                        }
                    } else {
                        false
                    }
                }
                // Newlines followed by identifier:colon
                TokenType::Newline => {
                    if let Some(token2) = self.peek_ahead(2) {
                        if token2.token_type == TokenType::Identifier {
                            if let Some(token3) = self.peek_ahead(3) {
                                token3.token_type == TokenType::Colon
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                }
                // Anything else is probably not a struct literal
                _ => false,
            }
        } else {
            false
        }
    }

    /// Finish parsing a struct literal (TypeName{field: value, ...})
    fn finish_struct_literal(&mut self, type_expr: Expression) -> Result<Expression> {
        let pos = type_expr.position();

        // Extract the type name
        let type_name = if let Expression::Identifier(ident) = type_expr {
            ident.name
        } else {
            return Err(self.error("Expected type name before struct literal"));
        };

        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut fields = Vec::new();

        if !self.check(&TokenType::RightBrace) {
            loop {
                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Check if we've reached the end
                if self.check(&TokenType::RightBrace) {
                    break;
                }

                // Parse field name
                let field_name = self.consume_identifier("Expected field name")?;

                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Expect colon
                self.consume(&TokenType::Colon, "Expected ':' after field name")?;

                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Parse field value
                let field_value = self.parse_expression()?;

                fields.push(StructFieldInit {
                    name: field_name,
                    value: field_value,
                    position: self.current_position(),
                });

                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Check for comma or end
                if self.check(&TokenType::Comma) {
                    self.advance();
                    // Skip newlines after comma
                    while self.match_token(&TokenType::Newline) {}
                } else if !self.check(&TokenType::RightBrace) {
                    return Err(self.error("Expected ',' or '}' after struct field"));
                }
            }
        }

        self.consume(&TokenType::RightBrace, "Expected '}'")?;

        Ok(Expression::StructLiteral(StructLiteralExpr {
            type_name,
            fields,
            position: pos,
        }))
    }

    /// Finish parsing a function call
    fn finish_call(&mut self, callee: Expression) -> Result<Expression> {
        // Check if this is a special make() call
        if let Expression::Identifier(ref ident) = callee {
            if ident.name == "make" {
                return self.parse_make_call(callee);
            }
        }

        let mut args = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;
        let pos = callee.position();

        Ok(Expression::Call(CallExpr {
            callee: Box::new(callee),
            type_args: Vec::new(),
            args,
            position: pos,
        }))
    }

    /// Parse make() call with special type syntax
    fn parse_make_call(&mut self, callee: Expression) -> Result<Expression> {
        let mut args = Vec::new();
        let pos = callee.position();

        if !self.check(&TokenType::RightParen) {
            // First argument should be a type expression
            // For now, we'll parse it as a regular expression and handle it in the interpreter
            // In a full implementation, we'd have special type expression parsing here

            loop {
                args.push(self.parse_expression()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after make arguments")?;

        // Create a special make call expression
        Ok(Expression::Call(CallExpr {
            callee: Box::new(callee),
            type_args: Vec::new(),
            args,
            position: pos,
        }))
    }

    /// Parse primary expression
    fn parse_primary(&mut self) -> Result<Expression> {
        let token = self.peek();
        let pos = token.position;

        match &token.token_type {
            TokenType::True => {
                self.advance();
                Ok(Expression::Literal(LiteralExpr {
                    value: LiteralValue::Boolean(true),
                    position: pos,
                }))
            }
            TokenType::False => {
                self.advance();
                Ok(Expression::Literal(LiteralExpr {
                    value: LiteralValue::Boolean(false),
                    position: pos,
                }))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expression::Literal(LiteralExpr {
                    value: LiteralValue::Null,
                    position: pos,
                }))
            }
            TokenType::IntegerLiteral => {
                if let Some(Literal::Integer(value)) = &token.literal {
                    let value = *value;
                    self.advance();
                    Ok(Expression::Literal(LiteralExpr {
                        value: LiteralValue::Integer(value),
                        position: pos,
                    }))
                } else {
                    Err(self.error("Invalid integer literal"))
                }
            }
            TokenType::FloatLiteral => {
                if let Some(Literal::Float(value)) = &token.literal {
                    let value = *value;
                    self.advance();
                    Ok(Expression::Literal(LiteralExpr {
                        value: LiteralValue::Float(value),
                        position: pos,
                    }))
                } else {
                    Err(self.error("Invalid float literal"))
                }
            }
            TokenType::StringLiteral => {
                if let Some(Literal::String(value)) = &token.literal {
                    let value = value.clone();
                    self.advance();
                    Ok(Expression::Literal(LiteralExpr {
                        value: LiteralValue::String(value),
                        position: pos,
                    }))
                } else {
                    Err(self.error("Invalid string literal"))
                }
            }
            TokenType::CharLiteral => {
                if let Some(Literal::Char(value)) = &token.literal {
                    let value = *value;
                    self.advance();
                    Ok(Expression::Literal(LiteralExpr {
                        value: LiteralValue::Char(value),
                        position: pos,
                    }))
                } else {
                    Err(self.error("Invalid char literal"))
                }
            }
            TokenType::Identifier => {
                let name = token.lexeme.clone();

                // Check if this is a single-parameter arrow function: param => expr
                if self.peek_ahead(1).map(|t| &t.token_type) == Some(&TokenType::FatArrow) {
                    self.advance(); // consume identifier
                    self.advance(); // consume '=>'

                    // Create parameter from identifier
                    let param = Parameter {
                        name: name.clone(),
                        param_type: Type::Any, // Inferred type
                        default_value: None,
                        is_variadic: false,
                        position: pos,
                    };

                    // Parse body
                    let body = if self.check(&TokenType::LeftBrace) {
                        let block = self.parse_block_statement_body()?;
                        Expression::Block(BlockExpr {
                            statements: block.statements,
                            position: block.position,
                        })
                    } else {
                        self.parse_expression()?
                    };

                    Ok(Expression::Lambda(LambdaExpr {
                        params: vec![param],
                        return_type: None,
                        body: Box::new(body),
                        captures: Vec::new(), // Will be filled by semantic analysis
                        position: pos,
                    }))
                } else {
                    self.advance();
                    Ok(Expression::Identifier(IdentifierExpr {
                        name,
                        position: pos,
                    }))
                }
            }

            TokenType::LeftBracket => self.parse_array_literal(),
            TokenType::LeftBrace => self.parse_map_or_struct_literal(),
            TokenType::Match => self.parse_match_expression(),
            TokenType::Select => self.parse_select_expression(),
            TokenType::Func => self.parse_lambda_expression(),
            TokenType::Run => self.parse_run_expression(),
            TokenType::Async => self.parse_async_expression(),
            TokenType::Await => self.parse_await_expression(),
            _ => {
                // Check for arrow function syntax (param) => expr or (param1, param2) => expr
                if self.check(&TokenType::LeftParen) {
                    // Look ahead to see if this might be an arrow function
                    if self.is_arrow_function() {
                        self.parse_arrow_function()
                    } else {
                        // Regular parenthesized expression
                        self.advance();
                        let expr = self.parse_expression()?;
                        self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
                        Ok(Expression::Parenthesized(ParenthesizedExpr {
                            expr: Box::new(expr),
                            position: pos,
                        }))
                    }
                } else {
                    Err(self.error(&format!("Unexpected token: {}", token.token_type)))
                }
            }
        }
    }

    /// Parse array literal
    fn parse_array_literal(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.consume(&TokenType::LeftBracket, "Expected '['")?;

        let mut elements = Vec::new();

        if !self.check(&TokenType::RightBracket) {
            loop {
                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Check if we've reached the end
                if self.check(&TokenType::RightBracket) {
                    break;
                }

                elements.push(self.parse_expression()?);

                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                if !self.match_token(&TokenType::Comma) {
                    break;
                }

                // Skip newlines after comma
                while self.match_token(&TokenType::Newline) {}
            }
        }

        // Skip newlines before closing bracket
        while self.match_token(&TokenType::Newline) {}

        self.consume(
            &TokenType::RightBracket,
            "Expected ']' after array elements",
        )?;

        Ok(Expression::Array(ArrayExpr {
            elements,
            position: pos,
        }))
    }

    /// Parse map or struct literal (determined by content)
    fn parse_map_or_struct_literal(&mut self) -> Result<Expression> {
        // For now, we'll assume it's a map literal
        // In a full implementation, we'd need to look ahead to determine
        // if it's a struct literal (Type{field: value}) or map literal ({key: value})
        self.parse_map_literal()
    }

    /// Parse map literal
    fn parse_map_literal(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.consume(&TokenType::LeftBrace, "Expected '{'")?;

        let mut entries = Vec::new();

        if !self.check(&TokenType::RightBrace) {
            loop {
                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Check if we've reached the end
                if self.check(&TokenType::RightBrace) {
                    break;
                }

                // Parse key
                let key = self.parse_expression()?;

                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Expect colon
                self.consume(&TokenType::Colon, "Expected ':' after map key")?;

                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                // Parse value
                let value = self.parse_expression()?;

                entries.push(MapEntry {
                    key,
                    value,
                    position: pos,
                });

                // Skip newlines
                while self.match_token(&TokenType::Newline) {}

                if !self.match_token(&TokenType::Comma) {
                    break;
                }

                // Skip newlines after comma
                while self.match_token(&TokenType::Newline) {}
            }
        }

        // Skip newlines before closing brace
        while self.match_token(&TokenType::Newline) {}

        self.consume(&TokenType::RightBrace, "Expected '}' after map entries")?;

        Ok(Expression::Map(MapExpr {
            entries,
            position: pos,
        }))
    }

    // ============================================================================
    // TYPE PARSING
    // ============================================================================

    /// Parse return type (can be tuple for multiple return values)
    fn parse_return_type(&mut self) -> Result<Type> {
        // Check if it's a tuple type (multiple return values)
        if self.check(&TokenType::LeftParen) {
            self.advance(); // consume '('

            let mut types = Vec::new();

            if !self.check(&TokenType::RightParen) {
                loop {
                    types.push(self.parse_type()?);

                    if !self.match_token(&TokenType::Comma) {
                        break;
                    }
                }
            }

            self.consume(&TokenType::RightParen, "Expected ')' after tuple types")?;

            // If only one type, return it directly (not a tuple)
            if types.len() == 1 {
                Ok(types.into_iter().next().unwrap())
            } else {
                Ok(Type::Tuple(TupleType {
                    element_types: types,
                }))
            }
        } else {
            // Single return type
            self.parse_type()
        }
    }

    /// Parse type annotation
    fn parse_type(&mut self) -> Result<Type> {
        match self.peek().token_type {
            TokenType::Func => {
                // Function type: func(T1, T2): R
                self.advance(); // consume 'func'

                if !self.check(&TokenType::LeftParen) {
                    return Err(self.error("Expected '(' after 'func' in function type"));
                }
                self.advance(); // consume '('

                let mut param_types = Vec::new();
                if !self.check(&TokenType::RightParen) {
                    loop {
                        param_types.push(self.parse_type()?);
                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(
                    &TokenType::RightParen,
                    "Expected ')' after function parameters",
                )?;

                let return_type = if self.match_token(&TokenType::Colon) {
                    Some(Box::new(self.parse_return_type()?))
                } else {
                    None
                };

                Ok(Type::Function(FunctionType {
                    param_types,
                    return_type,
                    is_async: false,
                }))
            }
            TokenType::Identifier => {
                let name = self.advance().lexeme.clone();
                if name == "map" {
                    // Map type: map[K]V
                    self.consume(&TokenType::LeftBracket, "Expected '[' after 'map'")?;
                    let key_type = Box::new(self.parse_type()?);
                    self.consume(&TokenType::RightBracket, "Expected ']' after map key type")?;
                    let value_type = Box::new(self.parse_type()?);
                    Ok(Type::Map(MapType {
                        key_type,
                        value_type,
                    }))
                } else {
                    // Handle other identifier types
                    match name.as_str() {
                        "int8" => Ok(Type::Int8),
                        "int16" => Ok(Type::Int16),
                        "int32" => Ok(Type::Int32),
                        "int64" => Ok(Type::Int64),
                        "uint8" => Ok(Type::UInt8),
                        "uint16" => Ok(Type::UInt16),
                        "uint32" => Ok(Type::UInt32),
                        "uint64" => Ok(Type::UInt64),
                        "float32" => Ok(Type::Float32),
                        "float64" => Ok(Type::Float64),
                        "bool" => Ok(Type::Bool),
                        "char" => Ok(Type::Char),
                        "string" => Ok(Type::String),
                        "any" => Ok(Type::Any),
                        _ => Ok(Type::Named(name)),
                    }
                }
            }
            TokenType::LeftBracket => {
                self.advance();
                if self.match_token(&TokenType::RightBracket) {
                    // Slice type []T
                    let element_type = Box::new(self.parse_type()?);
                    Ok(Type::Slice(SliceType { element_type }))
                } else {
                    // Array type [N]T - for now, just parse as slice
                    while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
                        self.advance();
                    }
                    self.consume(&TokenType::RightBracket, "Expected ']'")?;
                    let element_type = Box::new(self.parse_type()?);
                    Ok(Type::Array(ArrayType {
                        element_type,
                        size: None,
                    }))
                }
            }
            TokenType::LeftParen => {
                // Tuple type (T1, T2, T3) or parenthesized type (T)
                self.advance();

                let mut types = Vec::new();

                if !self.check(&TokenType::RightParen) {
                    loop {
                        types.push(self.parse_type()?);

                        if !self.match_token(&TokenType::Comma) {
                            break;
                        }
                    }
                }

                self.consume(&TokenType::RightParen, "Expected ')' after tuple types")?;

                // If only one type, return it directly (parenthesized type)
                if types.len() == 1 {
                    Ok(types.into_iter().next().unwrap())
                } else {
                    Ok(Type::Tuple(TupleType {
                        element_types: types,
                    }))
                }
            }
            _ => Err(self.error("Expected type")),
        }
    }

    /// Parse type parameters for generics with enhanced constraint support
    fn parse_type_parameters(&mut self) -> Result<Vec<TypeParam>> {
        let mut params = Vec::new();

        if !self.check(&TokenType::Greater) {
            loop {
                let pos = self.current_position();
                let name = self.consume_identifier("Expected type parameter name")?;

                // Parse constraints (T: Interface + OtherInterface)
                let mut constraints = Vec::new();
                if self.match_token(&TokenType::Colon) {
                    loop {
                        constraints.push(self.parse_type()?);
                        if !self.match_token(&TokenType::Plus) {
                            break;
                        }
                    }
                }

                // Parse default type (T = DefaultType)
                let _default_type = if self.match_token(&TokenType::Assign) {
                    Some(self.parse_type()?)
                } else {
                    None
                };

                params.push(TypeParam {
                    name,
                    constraints,
                    position: pos,
                });

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::Greater, "Expected '>' after type parameters")?;
        Ok(params)
    }

    /// Parse where clause for complex generic constraints
    fn parse_where_clause(&mut self) -> Result<Option<Vec<TypeParam>>> {
        if !self.match_token(&TokenType::Where) {
            return Ok(None);
        }

        let mut constraints = Vec::new();

        loop {
            let pos = self.current_position();
            let type_param =
                self.consume_identifier("Expected type parameter name in where clause")?;
            self.consume(
                &TokenType::Colon,
                "Expected ':' after type parameter in where clause",
            )?;

            let mut type_constraints = Vec::new();
            loop {
                type_constraints.push(self.parse_type()?);
                if !self.match_token(&TokenType::Plus) {
                    break;
                }
            }

            constraints.push(TypeParam {
                name: type_param,
                constraints: type_constraints,
                position: pos,
            });

            if !self.match_token(&TokenType::Comma) {
                break;
            }
        }

        Ok(Some(constraints))
    }

    /// Check if the current position starts an arrow function
    fn is_arrow_function(&self) -> bool {
        // Look ahead to find '=>' token
        let mut lookahead = 1;
        let mut paren_count = 1; // We already saw the opening paren

        while let Some(token) = self.peek_ahead(lookahead) {
            match token.token_type {
                TokenType::LeftParen => paren_count += 1,
                TokenType::RightParen => {
                    paren_count -= 1;
                    if paren_count == 0 {
                        // Check if next token is '=>'
                        if let Some(next_token) = self.peek_ahead(lookahead + 1) {
                            return next_token.token_type == TokenType::FatArrow;
                        }
                        return false;
                    }
                }
                TokenType::Eof => return false,
                _ => {}
            }
            lookahead += 1;
        }
        false
    }

    /// Parse lambda expression: func(params) { body }
    fn parse_lambda_expression(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.advance(); // consume 'func'

        // Parameters
        self.consume(&TokenType::LeftParen, "Expected '(' after 'func'")?;
        let params = self.parse_parameter_list()?;
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;

        // Optional return type
        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_return_type()?)
        } else {
            None
        };

        // Body - can be expression or block
        let body = if self.check(&TokenType::LeftBrace) {
            // Block body - convert to expression
            let block = self.parse_block_statement_body()?;
            Expression::Block(BlockExpr {
                statements: block.statements,
                position: block.position,
            })
        } else {
            // Expression body
            self.parse_expression()?
        };

        Ok(Expression::Lambda(LambdaExpr {
            params,
            return_type,
            body: Box::new(body),
            captures: Vec::new(), // Will be filled by semantic analysis
            position: pos,
        }))
    }

    /// Parse arrow function: (params) => expr
    fn parse_arrow_function(&mut self) -> Result<Expression> {
        let pos = self.current_position();

        // Parse parameters
        self.consume(&TokenType::LeftParen, "Expected '('")?;
        let params = self.parse_parameter_list()?;
        self.consume(&TokenType::RightParen, "Expected ')'")?;

        // Optional return type
        let return_type = if self.match_token(&TokenType::Colon) {
            Some(self.parse_return_type()?)
        } else {
            None
        };

        self.consume(&TokenType::FatArrow, "Expected '=>'")?;

        // Body - can be expression or block
        let body = if self.check(&TokenType::LeftBrace) {
            // Block body
            let block = self.parse_block_statement_body()?;
            Expression::Block(BlockExpr {
                statements: block.statements,
                position: block.position,
            })
        } else {
            // Expression body
            self.parse_expression()?
        };

        Ok(Expression::Lambda(LambdaExpr {
            params,
            return_type,
            body: Box::new(body),
            captures: Vec::new(), // Will be filled by semantic analysis
            position: pos,
        }))
    }

    /// Parse parameter list for functions
    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>> {
        let mut params = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                let pos = self.current_position();

                // Check for variadic parameter (...)
                let is_variadic = if self.match_token(&TokenType::DotDotDot) {
                    true
                } else {
                    false
                };

                let name = self.consume_identifier("Expected parameter name")?;
                self.consume(&TokenType::Colon, "Expected ':' after parameter name")?;
                let param_type = self.parse_type()?;

                // Parse default value if present
                let default_value = if self.match_token(&TokenType::Assign) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };

                params.push(Parameter {
                    name,
                    param_type,
                    default_value,
                    is_variadic,
                    position: pos,
                });

                // Variadic parameter must be the last parameter
                if is_variadic {
                    break;
                }

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        Ok(params)
    }

    // ============================================================================
    // UTILITY METHODS
    // ============================================================================

    /// Check if we're at the end of tokens
    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::Eof
    }

    /// Get current token without consuming it
    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    /// Get previous token
    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    /// Look ahead at a token at the given offset
    fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        let index = self.current + offset;
        if index < self.tokens.len() {
            Some(&self.tokens[index])
        } else {
            None
        }
    }

    /// Advance to next token and return current
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    /// Check if current token matches given type
    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    /// Match and consume token if it matches
    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Match and consume token if it matches any of the given types
    fn match_tokens(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    /// Consume token of expected type or return error
    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(self.error(message))
        }
    }

    /// Consume identifier token
    fn consume_identifier(&mut self, message: &str) -> Result<String> {
        if self.check(&TokenType::Identifier) {
            Ok(self.advance().lexeme.clone())
        } else {
            Err(self.error(message))
        }
    }

    /// Consume statement terminator (newline or semicolon)
    fn consume_statement_terminator(&mut self) -> Result<()> {
        if self.match_token(&TokenType::Semicolon)
            || self.match_token(&TokenType::Newline)
            || self.is_at_end()
            || self.check(&TokenType::RightBrace)
        {
            // Allow statements to end at closing brace
            Ok(())
        } else {
            Err(self.error("Expected newline or semicolon"))
        }
    }

    /// Get current position
    fn current_position(&self) -> Position {
        self.peek().position
    }

    /// Create error at current position
    fn error(&self, message: &str) -> BuluError {
        let pos = self.current_position();
        BuluError::parse_error(
            message.to_string(),
            pos.line,
            pos.column,
            self.file_path.clone(),
        )
    }

    /// Synchronize after error for error recovery
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon
                || self.previous().token_type == TokenType::Newline
            {
                return;
            }

            match self.peek().token_type {
                TokenType::Func
                | TokenType::Let
                | TokenType::Const
                | TokenType::If
                | TokenType::While
                | TokenType::For
                | TokenType::Return => return,
                _ => {}
            }

            self.advance();
        }
    }

    /// Parse run expression: run expr
    fn parse_run_expression(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.advance(); // consume 'run'

        let expr = self.parse_expression()?;

        Ok(Expression::Run(RunExpr {
            expr: Box::new(expr),
            position: pos,
        }))
    }

    /// Parse async expression: async expr
    fn parse_async_expression(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.advance(); // consume 'async'

        let expr = self.parse_expression()?;

        Ok(Expression::Async(AsyncExpr {
            expr: Box::new(expr),
            position: pos,
        }))
    }

    /// Parse await expression: await expr
    fn parse_await_expression(&mut self) -> Result<Expression> {
        let pos = self.current_position();
        self.advance(); // consume 'await'

        let expr = self.parse_expression()?;

        Ok(Expression::Await(AwaitExpr {
            expr: Box::new(expr),
            position: pos,
        }))
    }
}
