//! AST builder utilities for constructing AST nodes programmatically
//!
//! This module provides convenient builder functions for creating AST nodes,
//! useful for testing, code generation, and AST transformations.

use super::nodes::*;
use crate::lexer::token::Position;

/// Builder for creating AST nodes with default positions
pub struct AstBuilder;

impl AstBuilder {
    /// Create a dummy position for testing
    pub fn dummy_pos() -> Position {
        Position::new(1, 1, 0)
    }
    
    // ============================================================================
    // PROGRAM AND STATEMENTS
    // ============================================================================
    
    pub fn program(statements: Vec<Statement>) -> Program {
        Program {
            statements,
            position: Self::dummy_pos(),
        }
    }
    
    pub fn variable_decl(name: &str, type_annotation: Option<Type>, initializer: Option<Expression>) -> Statement {
        Statement::VariableDecl(VariableDecl {
            is_const: false,
            name: name.to_string(),
            type_annotation,
            initializer,
            doc_comment: None,
            is_exported: false,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn const_decl(name: &str, type_annotation: Option<Type>, initializer: Expression) -> Statement {
        Statement::VariableDecl(VariableDecl {
            is_const: true,
            name: name.to_string(),
            type_annotation,
            initializer: Some(initializer),
            doc_comment: None,
            is_exported: false,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn function_decl(
        name: &str,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: BlockStmt,
    ) -> Statement {
        Statement::FunctionDecl(FunctionDecl {
            name: name.to_string(),
            type_params: vec![],
            params,
            return_type,
            body,
            is_async: false,
            doc_comment: None,
            is_exported: false,
            is_private: false,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn async_function_decl(
        name: &str,
        params: Vec<Parameter>,
        return_type: Option<Type>,
        body: BlockStmt,
    ) -> Statement {
        Statement::FunctionDecl(FunctionDecl {
            name: name.to_string(),
            type_params: vec![],
            params,
            return_type,
            body,
            is_async: true,
            doc_comment: None,
            is_exported: false,
            is_private: false,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn parameter(name: &str, param_type: Type) -> Parameter {
        Parameter {
            name: name.to_string(),
            param_type,
            default_value: None,
            is_variadic: false,
            position: Self::dummy_pos(),
        }
    }
    
    pub fn block_stmt(statements: Vec<Statement>) -> BlockStmt {
        BlockStmt {
            statements,
            position: Self::dummy_pos(),
        }
    }
    
    pub fn if_stmt(condition: Expression, then_branch: BlockStmt, else_branch: Option<Statement>) -> Statement {
        Statement::If(IfStmt {
            condition,
            then_branch,
            else_branch: else_branch.map(Box::new),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn while_stmt(condition: Expression, body: BlockStmt) -> Statement {
        Statement::While(WhileStmt {
            condition,
            body,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn return_stmt(value: Option<Expression>) -> Statement {
        Statement::Return(ReturnStmt {
            value,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn expression_stmt(expr: Expression) -> Statement {
        Statement::Expression(ExpressionStmt {
            expr,
            position: Self::dummy_pos(),
        })
    }
    
    // ============================================================================
    // EXPRESSIONS
    // ============================================================================
    
    pub fn literal_int(value: i64) -> Expression {
        Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(value),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn literal_float(value: f64) -> Expression {
        Expression::Literal(LiteralExpr {
            value: LiteralValue::Float(value),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn literal_string(value: &str) -> Expression {
        Expression::Literal(LiteralExpr {
            value: LiteralValue::String(value.to_string()),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn literal_bool(value: bool) -> Expression {
        Expression::Literal(LiteralExpr {
            value: LiteralValue::Boolean(value),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn literal_null() -> Expression {
        Expression::Literal(LiteralExpr {
            value: LiteralValue::Null,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn identifier(name: &str) -> Expression {
        Expression::Identifier(IdentifierExpr {
            name: name.to_string(),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn binary_expr(left: Expression, operator: BinaryOperator, right: Expression) -> Expression {
        Expression::Binary(BinaryExpr {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn unary_expr(operator: UnaryOperator, operand: Expression) -> Expression {
        Expression::Unary(UnaryExpr {
            operator,
            operand: Box::new(operand),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn call_expr(callee: Expression, args: Vec<Expression>) -> Expression {
        Expression::Call(CallExpr {
            callee: Box::new(callee),
            type_args: vec![],
            args,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn member_access(object: Expression, member: &str) -> Expression {
        Expression::MemberAccess(MemberAccessExpr {
            object: Box::new(object),
            member: member.to_string(),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn index_expr(object: Expression, index: Expression) -> Expression {
        Expression::Index(IndexExpr {
            object: Box::new(object),
            index: Box::new(index),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn assignment(target: Expression, value: Expression) -> Expression {
        Expression::Assignment(AssignmentExpr {
            target: Box::new(target),
            operator: AssignmentOperator::Assign,
            value: Box::new(value),
            position: Self::dummy_pos(),
        })
    }
    
    pub fn array_expr(elements: Vec<Expression>) -> Expression {
        Expression::Array(ArrayExpr {
            elements,
            position: Self::dummy_pos(),
        })
    }
    
    pub fn lambda_expr(params: Vec<Parameter>, body: Expression) -> Expression {
        Expression::Lambda(LambdaExpr {
            params,
            return_type: None,
            body: Box::new(body),
            captures: Vec::new(),
            position: Self::dummy_pos(),
        })
    }
    
    // ============================================================================
    // TYPES
    // ============================================================================
    
    pub fn int32_type() -> Type {
        Type::Int32
    }
    
    pub fn int64_type() -> Type {
        Type::Int64
    }
    
    pub fn float64_type() -> Type {
        Type::Float64
    }
    
    pub fn bool_type() -> Type {
        Type::Bool
    }
    
    pub fn string_type() -> Type {
        Type::String
    }
    
    pub fn array_type(element_type: Type, size: Option<usize>) -> Type {
        Type::Array(ArrayType {
            element_type: Box::new(element_type),
            size,
        })
    }
    
    pub fn slice_type(element_type: Type) -> Type {
        Type::Slice(SliceType {
            element_type: Box::new(element_type),
        })
    }
    
    pub fn map_type(key_type: Type, value_type: Type) -> Type {
        Type::Map(MapType {
            key_type: Box::new(key_type),
            value_type: Box::new(value_type),
        })
    }
    
    pub fn function_type(param_types: Vec<Type>, return_type: Option<Type>) -> Type {
        Type::Function(FunctionType {
            param_types,
            return_type: return_type.map(Box::new),
            is_async: false,
        })
    }
    
    pub fn named_type(name: &str) -> Type {
        Type::Named(name.to_string())
    }
}

// ============================================================================
// CONVENIENCE MACROS
// ============================================================================

/// Macro for creating binary expressions more easily
#[macro_export]
macro_rules! binary {
    ($left:expr, +, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::Add, $right)
    };
    ($left:expr, -, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::Subtract, $right)
    };
    ($left:expr, *, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::Multiply, $right)
    };
    ($left:expr, /, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::Divide, $right)
    };
    ($left:expr, ==, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::Equal, $right)
    };
    ($left:expr, !=, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::NotEqual, $right)
    };
    ($left:expr, <, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::Less, $right)
    };
    ($left:expr, >, $right:expr) => {
        AstBuilder::binary_expr($left, BinaryOperator::Greater, $right)
    };
}

/// Macro for creating identifiers more easily
#[macro_export]
macro_rules! ident {
    ($name:expr) => {
        AstBuilder::identifier($name)
    };
}

/// Macro for creating integer literals more easily
#[macro_export]
macro_rules! int {
    ($value:expr) => {
        AstBuilder::literal_int($value)
    };
}