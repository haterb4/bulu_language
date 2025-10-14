//! AST visitor pattern for traversing and transforming AST nodes
//!
//! This module provides visitor traits for walking through AST nodes,
//! enabling operations like semantic analysis, code generation, and optimization.

use super::nodes::*;

/// Generic visitor trait for AST traversal
pub trait Visitor<T> {
    // Program
    fn visit_program(&mut self, program: &Program) -> T;
    
    // Statements
    fn visit_statement(&mut self, statement: &Statement) -> T where Self: Sized {
        walk_statement(self, statement)
    }
    
    fn visit_variable_decl(&mut self, decl: &VariableDecl) -> T;
    fn visit_function_decl(&mut self, decl: &FunctionDecl) -> T;
    fn visit_struct_decl(&mut self, decl: &StructDecl) -> T;
    fn visit_interface_decl(&mut self, decl: &InterfaceDecl) -> T;
    fn visit_type_alias_decl(&mut self, decl: &TypeAliasDecl) -> T;
    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> T;
    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> T;
    fn visit_for_stmt(&mut self, stmt: &ForStmt) -> T;
    fn visit_match_stmt(&mut self, stmt: &MatchStmt) -> T;
    fn visit_select_stmt(&mut self, stmt: &SelectStmt) -> T;
    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> T;
    fn visit_break_stmt(&mut self, stmt: &BreakStmt) -> T;
    fn visit_continue_stmt(&mut self, stmt: &ContinueStmt) -> T;
    fn visit_defer_stmt(&mut self, stmt: &DeferStmt) -> T;
    fn visit_try_stmt(&mut self, stmt: &TryStmt) -> T;
    fn visit_fail_stmt(&mut self, stmt: &FailStmt) -> T;
    fn visit_import_stmt(&mut self, stmt: &ImportStmt) -> T;
    fn visit_export_stmt(&mut self, stmt: &ExportStmt) -> T;
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) -> T;
    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> T;
    
    // Expressions
    fn visit_expression(&mut self, expression: &Expression) -> T where Self: Sized {
        walk_expression(self, expression)
    }
    
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> T;
    fn visit_identifier_expr(&mut self, expr: &IdentifierExpr) -> T;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> T;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> T;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> T;
    fn visit_member_access_expr(&mut self, expr: &MemberAccessExpr) -> T;
    fn visit_index_expr(&mut self, expr: &IndexExpr) -> T;
    fn visit_assignment_expr(&mut self, expr: &AssignmentExpr) -> T;
    fn visit_if_expr(&mut self, expr: &IfExpr) -> T;
    fn visit_match_expr(&mut self, expr: &MatchExpr) -> T;
    fn visit_array_expr(&mut self, expr: &ArrayExpr) -> T;
    fn visit_map_expr(&mut self, expr: &MapExpr) -> T;
    fn visit_lambda_expr(&mut self, expr: &LambdaExpr) -> T;
    fn visit_async_expr(&mut self, expr: &AsyncExpr) -> T;
    fn visit_await_expr(&mut self, expr: &AwaitExpr) -> T;
    fn visit_run_expr(&mut self, expr: &RunExpr) -> T;
    fn visit_channel_expr(&mut self, expr: &ChannelExpr) -> T;
    fn visit_select_expr(&mut self, expr: &SelectExpr) -> T;
    fn visit_cast_expr(&mut self, expr: &CastExpr) -> T;
    fn visit_typeof_expr(&mut self, expr: &TypeOfExpr) -> T;
    fn visit_range_expr(&mut self, expr: &RangeExpr) -> T;
    fn visit_yield_expr(&mut self, expr: &YieldExpr) -> T;
    fn visit_parenthesized_expr(&mut self, expr: &ParenthesizedExpr) -> T;
    fn visit_block_expr(&mut self, expr: &BlockExpr) -> T;
    fn visit_tuple_expr(&mut self, expr: &TupleExpr) -> T;
    fn visit_struct_literal_expr(&mut self, expr: &StructLiteralExpr) -> T;
    
    // Patterns
    fn visit_pattern(&mut self, pattern: &Pattern) -> T;
    
    // Types
    fn visit_type(&mut self, type_node: &Type) -> T;
}

/// Mutable visitor trait for AST transformation
pub trait MutVisitor {
    // Program
    fn visit_program(&mut self, program: &mut Program);
    
    // Statements
    fn visit_statement(&mut self, statement: &mut Statement) where Self: Sized {
        walk_statement_mut(self, statement)
    }
    
    fn visit_variable_decl(&mut self, decl: &mut VariableDecl);
    fn visit_function_decl(&mut self, decl: &mut FunctionDecl);
    fn visit_struct_decl(&mut self, decl: &mut StructDecl);
    fn visit_interface_decl(&mut self, decl: &mut InterfaceDecl);
    fn visit_type_alias_decl(&mut self, decl: &mut TypeAliasDecl);
    fn visit_if_stmt(&mut self, stmt: &mut IfStmt);
    fn visit_while_stmt(&mut self, stmt: &mut WhileStmt);
    fn visit_for_stmt(&mut self, stmt: &mut ForStmt);
    fn visit_match_stmt(&mut self, stmt: &mut MatchStmt);
    fn visit_select_stmt(&mut self, stmt: &mut SelectStmt);
    fn visit_return_stmt(&mut self, stmt: &mut ReturnStmt);
    fn visit_break_stmt(&mut self, stmt: &mut BreakStmt);
    fn visit_continue_stmt(&mut self, stmt: &mut ContinueStmt);
    fn visit_defer_stmt(&mut self, stmt: &mut DeferStmt);
    fn visit_try_stmt(&mut self, stmt: &mut TryStmt);
    fn visit_fail_stmt(&mut self, stmt: &mut FailStmt);
    fn visit_import_stmt(&mut self, stmt: &mut ImportStmt);
    fn visit_export_stmt(&mut self, stmt: &mut ExportStmt);
    fn visit_expression_stmt(&mut self, stmt: &mut ExpressionStmt);
    fn visit_block_stmt(&mut self, stmt: &mut BlockStmt);
    
    // Expressions
    fn visit_expression(&mut self, expression: &mut Expression) where Self: Sized {
        walk_expression_mut(self, expression)
    }
    
    fn visit_literal_expr(&mut self, expr: &mut LiteralExpr);
    fn visit_identifier_expr(&mut self, expr: &mut IdentifierExpr);
    fn visit_binary_expr(&mut self, expr: &mut BinaryExpr);
    fn visit_unary_expr(&mut self, expr: &mut UnaryExpr);
    fn visit_call_expr(&mut self, expr: &mut CallExpr);
    fn visit_member_access_expr(&mut self, expr: &mut MemberAccessExpr);
    fn visit_index_expr(&mut self, expr: &mut IndexExpr);
    fn visit_assignment_expr(&mut self, expr: &mut AssignmentExpr);
    fn visit_if_expr(&mut self, expr: &mut IfExpr);
    fn visit_match_expr(&mut self, expr: &mut MatchExpr);
    fn visit_select_expr(&mut self, expr: &mut SelectExpr);
    fn visit_array_expr(&mut self, expr: &mut ArrayExpr);
    fn visit_map_expr(&mut self, expr: &mut MapExpr);
    fn visit_lambda_expr(&mut self, expr: &mut LambdaExpr);
    fn visit_async_expr(&mut self, expr: &mut AsyncExpr);
    fn visit_await_expr(&mut self, expr: &mut AwaitExpr);
    fn visit_run_expr(&mut self, expr: &mut RunExpr);
    fn visit_channel_expr(&mut self, expr: &mut ChannelExpr);
    fn visit_cast_expr(&mut self, expr: &mut CastExpr);
    fn visit_typeof_expr(&mut self, expr: &mut TypeOfExpr);
    fn visit_range_expr(&mut self, expr: &mut RangeExpr);
    fn visit_yield_expr(&mut self, expr: &mut YieldExpr);
    fn visit_parenthesized_expr(&mut self, expr: &mut ParenthesizedExpr);
    fn visit_block_expr(&mut self, expr: &mut BlockExpr);
    fn visit_tuple_expr(&mut self, expr: &mut TupleExpr);
    fn visit_struct_literal_expr(&mut self, expr: &mut StructLiteralExpr);
    
    // Patterns
    fn visit_pattern(&mut self, pattern: &mut Pattern);
    
    // Types
    fn visit_type(&mut self, type_node: &mut Type);
}

// ============================================================================
// WALKER FUNCTIONS
// ============================================================================

/// Walk through a statement and visit its children
pub fn walk_statement<T, V: Visitor<T>>(visitor: &mut V, statement: &Statement) -> T {
    match statement {
        Statement::VariableDecl(decl) => visitor.visit_variable_decl(decl),
        Statement::FunctionDecl(decl) => visitor.visit_function_decl(decl),
        Statement::StructDecl(decl) => visitor.visit_struct_decl(decl),
        Statement::InterfaceDecl(decl) => visitor.visit_interface_decl(decl),
        Statement::TypeAlias(decl) => visitor.visit_type_alias_decl(decl),
        Statement::If(stmt) => visitor.visit_if_stmt(stmt),
        Statement::While(stmt) => visitor.visit_while_stmt(stmt),
        Statement::For(stmt) => visitor.visit_for_stmt(stmt),
        Statement::Match(stmt) => visitor.visit_match_stmt(stmt),
        Statement::Select(stmt) => visitor.visit_select_stmt(stmt),
        Statement::Return(stmt) => visitor.visit_return_stmt(stmt),
        Statement::Break(stmt) => visitor.visit_break_stmt(stmt),
        Statement::Continue(stmt) => visitor.visit_continue_stmt(stmt),
        Statement::Defer(stmt) => visitor.visit_defer_stmt(stmt),
        Statement::Try(stmt) => visitor.visit_try_stmt(stmt),
        Statement::Fail(stmt) => visitor.visit_fail_stmt(stmt),
        Statement::Import(stmt) => visitor.visit_import_stmt(stmt),
        Statement::Export(stmt) => visitor.visit_export_stmt(stmt),
        Statement::Expression(stmt) => visitor.visit_expression_stmt(stmt),
        Statement::Block(stmt) => visitor.visit_block_stmt(stmt),
    }
}

/// Walk through an expression and visit its children
pub fn walk_expression<T, V: Visitor<T>>(visitor: &mut V, expression: &Expression) -> T {
    match expression {
        Expression::Literal(expr) => visitor.visit_literal_expr(expr),
        Expression::Identifier(expr) => visitor.visit_identifier_expr(expr),
        Expression::Binary(expr) => visitor.visit_binary_expr(expr),
        Expression::Unary(expr) => visitor.visit_unary_expr(expr),
        Expression::Call(expr) => visitor.visit_call_expr(expr),
        Expression::MemberAccess(expr) => visitor.visit_member_access_expr(expr),
        Expression::Index(expr) => visitor.visit_index_expr(expr),
        Expression::Assignment(expr) => visitor.visit_assignment_expr(expr),
        Expression::If(expr) => visitor.visit_if_expr(expr),
        Expression::Match(expr) => visitor.visit_match_expr(expr),
        Expression::Array(expr) => visitor.visit_array_expr(expr),
        Expression::Map(expr) => visitor.visit_map_expr(expr),
        Expression::Lambda(expr) => visitor.visit_lambda_expr(expr),
        Expression::Async(expr) => visitor.visit_async_expr(expr),
        Expression::Await(expr) => visitor.visit_await_expr(expr),
        Expression::Run(expr) => visitor.visit_run_expr(expr),
        Expression::Channel(expr) => visitor.visit_channel_expr(expr),
        Expression::Select(expr) => visitor.visit_select_expr(expr),
        Expression::Cast(expr) => visitor.visit_cast_expr(expr),
        Expression::TypeOf(expr) => visitor.visit_typeof_expr(expr),
        Expression::Range(expr) => visitor.visit_range_expr(expr),
        Expression::Yield(expr) => visitor.visit_yield_expr(expr),
        Expression::Parenthesized(expr) => visitor.visit_parenthesized_expr(expr),
        Expression::Block(expr) => visitor.visit_block_expr(expr),
        Expression::Tuple(expr) => visitor.visit_tuple_expr(expr),
        Expression::StructLiteral(expr) => visitor.visit_struct_literal_expr(expr),
    }
}

/// Walk through a statement and visit its children (mutable version)
pub fn walk_statement_mut<V: MutVisitor>(visitor: &mut V, statement: &mut Statement) {
    match statement {
        Statement::VariableDecl(decl) => visitor.visit_variable_decl(decl),
        Statement::FunctionDecl(decl) => visitor.visit_function_decl(decl),
        Statement::StructDecl(decl) => visitor.visit_struct_decl(decl),
        Statement::InterfaceDecl(decl) => visitor.visit_interface_decl(decl),
        Statement::TypeAlias(decl) => visitor.visit_type_alias_decl(decl),
        Statement::If(stmt) => visitor.visit_if_stmt(stmt),
        Statement::While(stmt) => visitor.visit_while_stmt(stmt),
        Statement::For(stmt) => visitor.visit_for_stmt(stmt),
        Statement::Match(stmt) => visitor.visit_match_stmt(stmt),
        Statement::Select(stmt) => visitor.visit_select_stmt(stmt),
        Statement::Return(stmt) => visitor.visit_return_stmt(stmt),
        Statement::Break(stmt) => visitor.visit_break_stmt(stmt),
        Statement::Continue(stmt) => visitor.visit_continue_stmt(stmt),
        Statement::Defer(stmt) => visitor.visit_defer_stmt(stmt),
        Statement::Try(stmt) => visitor.visit_try_stmt(stmt),
        Statement::Fail(stmt) => visitor.visit_fail_stmt(stmt),
        Statement::Import(stmt) => visitor.visit_import_stmt(stmt),
        Statement::Export(stmt) => visitor.visit_export_stmt(stmt),
        Statement::Expression(stmt) => visitor.visit_expression_stmt(stmt),
        Statement::Block(stmt) => visitor.visit_block_stmt(stmt),
    }
}

/// Walk through an expression and visit its children (mutable version)
pub fn walk_expression_mut<V: MutVisitor>(visitor: &mut V, expression: &mut Expression) {
    match expression {
        Expression::Literal(expr) => visitor.visit_literal_expr(expr),
        Expression::Identifier(expr) => visitor.visit_identifier_expr(expr),
        Expression::Binary(expr) => visitor.visit_binary_expr(expr),
        Expression::Unary(expr) => visitor.visit_unary_expr(expr),
        Expression::Call(expr) => visitor.visit_call_expr(expr),
        Expression::MemberAccess(expr) => visitor.visit_member_access_expr(expr),
        Expression::Index(expr) => visitor.visit_index_expr(expr),
        Expression::Assignment(expr) => visitor.visit_assignment_expr(expr),
        Expression::If(expr) => visitor.visit_if_expr(expr),
        Expression::Match(expr) => visitor.visit_match_expr(expr),
        Expression::Array(expr) => visitor.visit_array_expr(expr),
        Expression::Map(expr) => visitor.visit_map_expr(expr),
        Expression::Lambda(expr) => visitor.visit_lambda_expr(expr),
        Expression::Async(expr) => visitor.visit_async_expr(expr),
        Expression::Await(expr) => visitor.visit_await_expr(expr),
        Expression::Run(expr) => visitor.visit_run_expr(expr),
        Expression::Channel(expr) => visitor.visit_channel_expr(expr),
        Expression::Select(expr) => visitor.visit_select_expr(expr),
        Expression::Cast(expr) => visitor.visit_cast_expr(expr),
        Expression::TypeOf(expr) => visitor.visit_typeof_expr(expr),
        Expression::Range(expr) => visitor.visit_range_expr(expr),
        Expression::Yield(expr) => visitor.visit_yield_expr(expr),
        Expression::Parenthesized(expr) => visitor.visit_parenthesized_expr(expr),
        Expression::Block(expr) => visitor.visit_block_expr(expr),
        Expression::Tuple(expr) => visitor.visit_tuple_expr(expr),
        Expression::StructLiteral(expr) => visitor.visit_struct_literal_expr(expr),
    }
}