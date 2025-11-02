//! AST node definitions for the Bulu language
//!
//! This module defines all Abstract Syntax Tree node types for representing
//! Bulu language constructs in memory after parsing.

use crate::lexer::token::Position;


/// Root node of the AST representing a complete Bulu program
#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub position: Position,
}

/// All possible statement types in Bulu
#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    // Declarations
    VariableDecl(VariableDecl),
    DestructuringDecl(DestructuringDecl),
    MultipleVariableDecl(MultipleVariableDecl),
    MultipleAssignment(MultipleAssignmentStmt),
    FunctionDecl(FunctionDecl),
    StructDecl(StructDecl),
    InterfaceDecl(InterfaceDecl),
    TypeAlias(TypeAliasDecl),
    
    // Control flow
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Match(MatchStmt),
    Select(SelectStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Defer(DeferStmt),
    
    // Error handling
    Try(TryStmt),
    Fail(FailStmt),
    
    // Module system
    Import(ImportStmt),
    Export(ExportStmt),
    
    // Expression statement
    Expression(ExpressionStmt),
    
    // Block statement
    Block(BlockStmt),
}

/// All possible expression types in Bulu
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    // Literals
    Literal(LiteralExpr),
    Identifier(IdentifierExpr),
    
    // Arithmetic operations
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    
    // Function calls and member access
    Call(CallExpr),
    MemberAccess(MemberAccessExpr),
    Index(IndexExpr),
    
    // Assignment
    Assignment(AssignmentExpr),
    
    // Control flow expressions
    If(IfExpr),
    Match(MatchExpr),
    
    // Collection literals
    Array(ArrayExpr),
    Map(MapExpr),
    StructLiteral(StructLiteralExpr),
    
    // Function expressions
    Lambda(LambdaExpr),
    
    // Concurrency
    Async(AsyncExpr),
    Await(AwaitExpr),
    Run(RunExpr),
    Channel(ChannelExpr),
    Select(SelectExpr),
    
    // Type operations
    Cast(CastExpr),
    TypeOf(TypeOfExpr),
    
    // Range expressions
    Range(RangeExpr),
    
    // Yield expression for generators
    Yield(YieldExpr),
    
    // Parenthesized expression
    Parenthesized(ParenthesizedExpr),
    
    // Block expression
    Block(BlockExpr),
    
    // Tuple expression
    Tuple(TupleExpr),
}

// ============================================================================
// DECLARATIONS
// ============================================================================

/// Variable declaration: let x = 5, const PI = 3.14
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub is_const: bool,
    pub name: String,
    pub type_annotation: Option<Type>,
    pub initializer: Option<Expression>,
    pub doc_comment: Option<Vec<crate::lexer::token::Token>>,
    pub is_exported: bool,
    pub position: Position,
}

/// Destructuring variable declaration: let {a, b} = obj
#[derive(Debug, Clone, PartialEq)]
pub struct DestructuringDecl {
    pub is_const: bool,
    pub pattern: Pattern,
    pub initializer: Expression,
    pub doc_comment: Option<Vec<crate::lexer::token::Token>>,
    pub is_exported: bool,
    pub position: Position,
}

/// Multiple variable declaration: let a, b: int64
#[derive(Debug, Clone, PartialEq)]
pub struct MultipleVariableDecl {
    pub is_const: bool,
    pub declarations: Vec<SingleVariableDecl>,
    pub doc_comment: Option<Vec<crate::lexer::token::Token>>,
    pub is_exported: bool,
    pub position: Position,
}

/// Single variable in a multiple declaration
#[derive(Debug, Clone, PartialEq)]
pub struct SingleVariableDecl {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub initializer: Option<Expression>,
}

/// Multiple assignment statement: a, b = b, a
#[derive(Debug, Clone, PartialEq)]
pub struct MultipleAssignmentStmt {
    pub targets: Vec<Expression>,
    pub values: Vec<Expression>,
    pub position: Position,
}

/// Function declaration
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: BlockStmt,
    pub is_async: bool,
    pub doc_comment: Option<Vec<crate::lexer::token::Token>>,
    pub is_exported: bool,
    pub is_private: bool,
    pub position: Position,
}

/// Function parameter
#[derive(Debug, Clone, PartialEq)]
pub struct Parameter {
    pub name: String,
    pub param_type: Type,
    pub default_value: Option<Expression>,
    pub is_variadic: bool,
    pub position: Position,
}

/// Struct declaration
#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub fields: Vec<StructField>,
    pub methods: Vec<FunctionDecl>,
    pub doc_comment: Option<Vec<crate::lexer::token::Token>>,
    pub is_exported: bool,
    pub position: Position,
}

/// Struct field
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub is_private: bool,
    pub position: Position,
}

/// Interface declaration
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub methods: Vec<InterfaceMethod>,
    pub doc_comment: Option<Vec<crate::lexer::token::Token>>,
    pub is_exported: bool,
    pub position: Position,
}

/// Interface method signature
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceMethod {
    pub name: String,
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub is_private: bool,
    pub position: Position,
}


/// Type alias declaration
#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDecl {
    pub name: String,
    pub type_params: Vec<TypeParam>,
    pub target_type: Type,
    pub position: Position,
}

// ============================================================================
// STATEMENTS
// ============================================================================

/// If statement
#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expression,
    pub then_branch: BlockStmt,
    pub else_branch: Option<Box<Statement>>,
    pub position: Position,
}

/// While loop
#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Expression,
    pub body: BlockStmt,
    pub position: Position,
}

/// For loop
#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub variable: String,
    pub index_variable: Option<String>, // For index, value iteration
    pub iterable: Expression,
    pub body: BlockStmt,
    pub position: Position,
}

/// Match statement
#[derive(Debug, Clone, PartialEq)]
pub struct MatchStmt {
    pub expr: Expression,
    pub arms: Vec<MatchArm>,
    pub position: Position,
}

/// Match arm
#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Statement,
    pub position: Position,
}

/// Select statement
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStmt {
    pub arms: Vec<SelectStmtArm>,
    pub position: Position,
}

/// Select statement arm
#[derive(Debug, Clone, PartialEq)]
pub struct SelectStmtArm {
    pub channel_op: Option<ChannelOperation>,
    pub body: Statement,
    pub position: Position,
}

/// Channel operation for select statement
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelOperation {
    pub is_send: bool,
    pub channel: Expression,
    pub value: Option<Expression>, // For send operations
    pub variable: Option<String>,  // For receive operations with assignment
    pub position: Position,
}

/// Return statement
#[derive(Debug, Clone, PartialEq)]
pub struct ReturnStmt {
    pub value: Option<Expression>,
    pub position: Position,
}

/// Break statement
#[derive(Debug, Clone, PartialEq)]
pub struct BreakStmt {
    pub position: Position,
}

/// Continue statement
#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStmt {
    pub position: Position,
}

/// Defer statement
#[derive(Debug, Clone, PartialEq)]
pub struct DeferStmt {
    pub stmt: Box<Statement>,
    pub position: Position,
}

/// Try statement
#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    pub body: BlockStmt,
    pub catch_clause: Option<CatchClause>,
    pub position: Position,
}

/// Catch clause for try statement
#[derive(Debug, Clone, PartialEq)]
pub struct CatchClause {
    pub error_var: Option<String>,
    pub body: BlockStmt,
    pub position: Position,
}

/// Fail statement
#[derive(Debug, Clone, PartialEq)]
pub struct FailStmt {
    pub message: Expression,
    pub position: Position,
}

/// Import statement
#[derive(Debug, Clone, PartialEq)]
pub struct ImportStmt {
    pub path: String,
    pub alias: Option<String>,
    pub items: Option<Vec<ImportItem>>,
    pub position: Position,
}

/// Import item
#[derive(Debug, Clone, PartialEq)]
pub struct ImportItem {
    pub name: String,
    pub alias: Option<String>,
    pub position: Position,
}

/// Export statement
#[derive(Debug, Clone, PartialEq)]
pub struct ExportStmt {
    pub item: Box<Statement>,
    pub position: Position,
}

/// Expression statement
#[derive(Debug, Clone, PartialEq)]
pub struct ExpressionStmt {
    pub expr: Expression,
    pub position: Position,
}

/// Block statement
#[derive(Debug, Clone, PartialEq)]
pub struct BlockStmt {
    pub statements: Vec<Statement>,
    pub position: Position,
}

// ============================================================================
// EXPRESSIONS
// ============================================================================

/// Literal expression
#[derive(Debug, Clone, PartialEq)]
pub struct LiteralExpr {
    pub value: LiteralValue,
    pub position: Position,
}

/// Identifier expression
#[derive(Debug, Clone, PartialEq)]
pub struct IdentifierExpr {
    pub name: String,
    pub position: Position,
}

/// Binary operation expression
#[derive(Debug, Clone, PartialEq)]
pub struct BinaryExpr {
    pub left: Box<Expression>,
    pub operator: BinaryOperator,
    pub right: Box<Expression>,
    pub position: Position,
}

/// Unary operation expression
#[derive(Debug, Clone, PartialEq)]
pub struct UnaryExpr {
    pub operator: UnaryOperator,
    pub operand: Box<Expression>,
    pub position: Position,
}

/// Function call expression
#[derive(Debug, Clone, PartialEq)]
pub struct CallExpr {
    pub callee: Box<Expression>,
    pub type_args: Vec<Type>,
    pub args: Vec<Expression>,
    pub position: Position,
}

/// Member access expression (obj.field)
#[derive(Debug, Clone, PartialEq)]
pub struct MemberAccessExpr {
    pub object: Box<Expression>,
    pub member: String,
    pub position: Position,
}

/// Index expression (arr[index])
#[derive(Debug, Clone, PartialEq)]
pub struct IndexExpr {
    pub object: Box<Expression>,
    pub index: Box<Expression>,
    pub position: Position,
}

/// Assignment expression
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentExpr {
    pub target: Box<Expression>,
    pub operator: AssignmentOperator,
    pub value: Box<Expression>,
    pub position: Position,
}

/// If expression (ternary-like)
#[derive(Debug, Clone, PartialEq)]
pub struct IfExpr {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Box<Expression>,
    pub position: Position,
}

/// Match expression
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExpr {
    pub expr: Box<Expression>,
    pub arms: Vec<MatchExprArm>,
    pub position: Position,
}

/// Match expression arm
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExprArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub expr: Expression,
    pub position: Position,
}

/// Array literal expression
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayExpr {
    pub elements: Vec<Expression>,
    pub position: Position,
}

/// Map literal expression
#[derive(Debug, Clone, PartialEq)]
pub struct MapExpr {
    pub entries: Vec<MapEntry>,
    pub position: Position,
}

/// Map entry
#[derive(Debug, Clone, PartialEq)]
pub struct MapEntry {
    pub key: Expression,
    pub value: Expression,
    pub position: Position,
}

/// Struct literal expression
#[derive(Debug, Clone, PartialEq)]
pub struct StructLiteralExpr {
    pub type_name: String,
    pub fields: Vec<StructFieldInit>,
    pub position: Position,
}

/// Struct field initialization
#[derive(Debug, Clone, PartialEq)]
pub struct StructFieldInit {
    pub name: String,
    pub value: Expression,
    pub position: Position,
}

/// Lambda expression
#[derive(Debug, Clone, PartialEq)]
pub struct LambdaExpr {
    pub params: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Box<Expression>,
    pub captures: Vec<Capture>, // Variables captured from outer scopes
    pub position: Position,
}

/// Captured variable information for closures
#[derive(Debug, Clone, PartialEq)]
pub struct Capture {
    pub name: String,
    pub capture_type: CaptureType,
    pub position: Position,
}

/// Type of variable capture
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureType {
    ByValue,    // Capture by value (immutable)
    ByReference, // Capture by reference (mutable)
}

/// Async expression
#[derive(Debug, Clone, PartialEq)]
pub struct AsyncExpr {
    pub expr: Box<Expression>,
    pub position: Position,
}

/// Await expression
#[derive(Debug, Clone, PartialEq)]
pub struct AwaitExpr {
    pub expr: Box<Expression>,
    pub position: Position,
}

/// Run expression (spawn goroutine)
#[derive(Debug, Clone, PartialEq)]
pub struct RunExpr {
    pub expr: Box<Expression>,
    pub position: Position,
}

/// Channel expression
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelExpr {
    pub direction: ChannelDirection,
    pub channel: Box<Expression>,
    pub value: Option<Box<Expression>>,
    pub position: Position,
}

/// Select expression
#[derive(Debug, Clone, PartialEq)]
pub struct SelectExpr {
    pub arms: Vec<SelectExprArm>,
    pub position: Position,
}

/// Select arm
#[derive(Debug, Clone, PartialEq)]
pub struct SelectArm {
    pub channel_op: Option<ChannelExpr>,
    pub body: Expression,
    pub position: Position,
}

/// Select expression arm
#[derive(Debug, Clone, PartialEq)]
pub struct SelectExprArm {
    pub channel_op: Option<ChannelOperation>,
    pub expr: Expression,
    pub position: Position,
}

/// Type cast expression
#[derive(Debug, Clone, PartialEq)]
pub struct CastExpr {
    pub expr: Box<Expression>,
    pub target_type: Type,
    pub position: Position,
}

/// TypeOf expression
#[derive(Debug, Clone, PartialEq)]
pub struct TypeOfExpr {
    pub expr: Box<Expression>,
    pub position: Position,
}

/// Range expression
#[derive(Debug, Clone, PartialEq)]
pub struct RangeExpr {
    pub start: Box<Expression>,
    pub end: Box<Expression>,
    pub step: Option<Box<Expression>>,
    pub inclusive: bool,
    pub position: Position,
}

/// Yield expression
#[derive(Debug, Clone, PartialEq)]
pub struct YieldExpr {
    pub value: Option<Box<Expression>>,
    pub position: Position,
}

/// Parenthesized expression
#[derive(Debug, Clone, PartialEq)]
pub struct ParenthesizedExpr {
    pub expr: Box<Expression>,
    pub position: Position,
}

/// Block expression
#[derive(Debug, Clone, PartialEq)]
pub struct BlockExpr {
    pub statements: Vec<Statement>,
    pub position: Position,
}

/// Tuple expression
#[derive(Debug, Clone, PartialEq)]
pub struct TupleExpr {
    pub elements: Vec<Expression>,
    pub position: Position,
}

// ============================================================================
// TYPES
// ============================================================================

/// Type representations
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    // Primitive types
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Bool,
    Char,
    String,
    Any,
    Void, // For functions that don't return a value
    
    // Composite types
    Array(ArrayType),
    Slice(SliceType),
    Map(MapType),
    Tuple(TupleType),
    Function(FunctionType),
    
    // User-defined types
    Struct(StructType),
    Interface(InterfaceType),
    
    // Generic types
    Generic(GenericType),
    
    // Channel types
    Channel(ChannelType),
    
    // Async types
    Promise(PromiseType),
    
    // Named type (identifier)
    Named(String),
}

/// Array type
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayType {
    pub element_type: Box<Type>,
    pub size: Option<usize>,
}

/// Slice type
#[derive(Debug, Clone, PartialEq)]
pub struct SliceType {
    pub element_type: Box<Type>,
}

/// Map type
#[derive(Debug, Clone, PartialEq)]
pub struct MapType {
    pub key_type: Box<Type>,
    pub value_type: Box<Type>,
}

/// Tuple type
#[derive(Debug, Clone, PartialEq)]
pub struct TupleType {
    pub element_types: Vec<Type>,
}

/// Function type
#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub param_types: Vec<Type>,
    pub return_type: Option<Box<Type>>,
    pub is_async: bool,
}

/// Struct type
#[derive(Debug, Clone, PartialEq)]
pub struct StructType {
    pub name: String,
    pub type_args: Vec<Type>,
}

/// Interface type
#[derive(Debug, Clone, PartialEq)]
pub struct InterfaceType {
    pub name: String,
    pub type_args: Vec<Type>,
}

/// Generic type
#[derive(Debug, Clone, PartialEq)]
pub struct GenericType {
    pub name: String,
    pub constraints: Vec<Type>,
}

/// Channel type
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelType {
    pub element_type: Box<Type>,
    pub direction: ChannelDirection,
}

/// Promise type for async operations
#[derive(Debug, Clone, PartialEq)]
pub struct PromiseType {
    pub result_type: Box<Type>,
    pub position: Position,
}

/// Type parameter for generics
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    pub constraints: Vec<Type>,
    pub position: Position,
}

// ============================================================================
// PATTERNS
// ============================================================================

/// Pattern for match expressions
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard(Position),
    Literal(LiteralValue, Position),
    Identifier(String, Position),
    Struct(StructPattern),
    Array(ArrayPattern),
    Range(RangePattern),
    Or(OrPattern),
}

/// Struct pattern
#[derive(Debug, Clone, PartialEq)]
pub struct StructPattern {
    pub name: String,
    pub fields: Vec<FieldPattern>,
    pub position: Position,
}

/// Field pattern in struct pattern
#[derive(Debug, Clone, PartialEq)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: Box<Pattern>,
    pub position: Position,
}

/// Array pattern
#[derive(Debug, Clone, PartialEq)]
pub struct ArrayPattern {
    pub elements: Vec<Pattern>,
    pub position: Position,
}

/// Range pattern
#[derive(Debug, Clone, PartialEq)]
pub struct RangePattern {
    pub start: LiteralValue,
    pub end: LiteralValue,
    pub inclusive: bool,
    pub position: Position,
}

/// Or pattern (pattern1 | pattern2)
#[derive(Debug, Clone, PartialEq)]
pub struct OrPattern {
    pub patterns: Vec<Pattern>,
    pub position: Position,
}

// ============================================================================
// OPERATORS AND ENUMS
// ============================================================================

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    
    // Comparison
    Equal,
    NotEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    
    // Logical
    And,
    Or,
    
    // Bitwise
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    LeftShift,
    RightShift,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperator {
    Plus,
    Minus,
    Not,
    BitwiseNot,
}

/// Assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
    ModuloAssign,
}

/// Channel direction
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChannelDirection {
    Send,
    Receive,
    Bidirectional,
}

/// Literal values
#[derive(Debug, Clone, PartialEq)]
pub enum LiteralValue {
    Integer(i64),
    Float(f64),
    String(String),
    Char(char),
    Boolean(bool),
    Null,
}

// ============================================================================
// UTILITY TRAITS AND IMPLEMENTATIONS
// ============================================================================

/// Trait for getting position information from AST nodes
pub trait HasPosition {
    fn position(&self) -> Position;
}

impl HasPosition for Statement {
    fn position(&self) -> Position {
        match self {
            Statement::VariableDecl(node) => node.position,
            Statement::FunctionDecl(node) => node.position,
            Statement::StructDecl(node) => node.position,
            Statement::InterfaceDecl(node) => node.position,

            Statement::TypeAlias(node) => node.position,
            Statement::If(node) => node.position,
            Statement::While(node) => node.position,
            Statement::For(node) => node.position,
            Statement::Match(node) => node.position,
            Statement::Select(node) => node.position,
            Statement::Return(node) => node.position,
            Statement::Break(node) => node.position,
            Statement::Continue(node) => node.position,
            Statement::Defer(node) => node.position,
            Statement::Try(node) => node.position,
            Statement::Fail(node) => node.position,
            Statement::Import(node) => node.position,
            Statement::Export(node) => node.position,
            Statement::Expression(node) => node.position,
            Statement::Block(node) => node.position,
            Statement::DestructuringDecl(node) => node.position,
            Statement::MultipleVariableDecl(node) => node.position,
            Statement::MultipleAssignment(node) => node.position,
        }
    }
}

impl HasPosition for Expression {
    fn position(&self) -> Position {
        match self {
            Expression::Literal(node) => node.position,
            Expression::Identifier(node) => node.position,
            Expression::Binary(node) => node.position,
            Expression::Unary(node) => node.position,
            Expression::Call(node) => node.position,
            Expression::MemberAccess(node) => node.position,
            Expression::Index(node) => node.position,
            Expression::Assignment(node) => node.position,
            Expression::If(node) => node.position,
            Expression::Match(node) => node.position,
            Expression::Array(node) => node.position,
            Expression::Map(node) => node.position,
            Expression::Lambda(node) => node.position,
            Expression::Async(node) => node.position,
            Expression::Await(node) => node.position,
            Expression::Run(node) => node.position,
            Expression::Channel(node) => node.position,
            Expression::Select(node) => node.position,
            Expression::Cast(node) => node.position,
            Expression::TypeOf(node) => node.position,
            Expression::Range(node) => node.position,
            Expression::Yield(node) => node.position,
            Expression::Parenthesized(node) => node.position,
            Expression::Block(node) => node.position,
            Expression::Tuple(node) => node.position,
            Expression::StructLiteral(node) => node.position,
        }
    }
}

impl HasPosition for Pattern {
    fn position(&self) -> Position {
        match self {
            Pattern::Wildcard(pos) => *pos,
            Pattern::Literal(_, pos) => *pos,
            Pattern::Identifier(_, pos) => *pos,
            Pattern::Struct(node) => node.position,
            Pattern::Array(node) => node.position,
            Pattern::Range(node) => node.position,
            Pattern::Or(node) => node.position,
        }
    }
}