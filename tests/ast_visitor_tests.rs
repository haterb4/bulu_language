use bulu::ast::*;

// Test visitor that counts different types of nodes
struct NodeCounter {
    variable_decls: usize,
    function_decls: usize,
    binary_exprs: usize,
    identifiers: usize,
    literals: usize,
}

impl NodeCounter {
    fn new() -> Self {
        Self {
            variable_decls: 0,
            function_decls: 0,
            binary_exprs: 0,
            identifiers: 0,
            literals: 0,
        }
    }
}

impl Visitor<()> for NodeCounter {
    fn visit_program(&mut self, program: &Program) {
        for stmt in &program.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_variable_decl(&mut self, decl: &VariableDecl) {
        self.variable_decls += 1;
        if let Some(ref init) = decl.initializer {
            self.visit_expression(init);
        }
    }

    fn visit_function_decl(&mut self, decl: &FunctionDecl) {
        self.function_decls += 1;
        self.visit_block_stmt(&decl.body);
    }

    fn visit_struct_decl(&mut self, _decl: &StructDecl) {}
    fn visit_interface_decl(&mut self, _decl: &InterfaceDecl) {}
    fn visit_type_alias_decl(&mut self, _decl: &TypeAliasDecl) {}
    fn visit_if_stmt(&mut self, stmt: &IfStmt) {
        self.visit_expression(&stmt.condition);
        self.visit_block_stmt(&stmt.then_branch);
        if let Some(ref else_branch) = stmt.else_branch {
            self.visit_statement(else_branch);
        }
    }
    fn visit_while_stmt(&mut self, stmt: &WhileStmt) {
        self.visit_expression(&stmt.condition);
        self.visit_block_stmt(&stmt.body);
    }
    fn visit_for_stmt(&mut self, stmt: &ForStmt) {
        self.visit_expression(&stmt.iterable);
        self.visit_block_stmt(&stmt.body);
    }
    fn visit_match_stmt(&mut self, stmt: &MatchStmt) {
        self.visit_expression(&stmt.expr);
        for arm in &stmt.arms {
            if let Some(ref guard) = arm.guard {
                self.visit_expression(guard);
            }
            self.visit_statement(&arm.body);
        }
    }

    fn visit_select_stmt(&mut self, stmt: &SelectStmt) {
        for arm in &stmt.arms {
            if let Some(ref channel_op) = arm.channel_op {
                self.visit_expression(&channel_op.channel);
                if let Some(ref value) = channel_op.value {
                    self.visit_expression(value);
                }
            }
            self.visit_statement(&arm.body);
        }
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) {
        if let Some(ref value) = stmt.value {
            self.visit_expression(value);
        }
    }
    fn visit_break_stmt(&mut self, _stmt: &BreakStmt) {}
    fn visit_continue_stmt(&mut self, _stmt: &ContinueStmt) {}
    fn visit_defer_stmt(&mut self, stmt: &DeferStmt) {
        self.visit_statement(&stmt.stmt);
    }
    fn visit_try_stmt(&mut self, stmt: &TryStmt) {
        self.visit_block_stmt(&stmt.body);
        if let Some(ref catch) = stmt.catch_clause {
            self.visit_block_stmt(&catch.body);
        }
    }
    fn visit_fail_stmt(&mut self, stmt: &FailStmt) {
        self.visit_expression(&stmt.message);
    }
    fn visit_import_stmt(&mut self, _stmt: &ImportStmt) {}
    fn visit_export_stmt(&mut self, stmt: &ExportStmt) {
        self.visit_statement(&stmt.item);
    }
    fn visit_expression_stmt(&mut self, stmt: &ExpressionStmt) {
        self.visit_expression(&stmt.expr);
    }
    fn visit_block_stmt(&mut self, stmt: &BlockStmt) {
        for statement in &stmt.statements {
            self.visit_statement(statement);
        }
    }

    fn visit_literal_expr(&mut self, _expr: &LiteralExpr) {
        self.literals += 1;
    }

    fn visit_identifier_expr(&mut self, _expr: &IdentifierExpr) {
        self.identifiers += 1;
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) {
        self.binary_exprs += 1;
        self.visit_expression(&expr.left);
        self.visit_expression(&expr.right);
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) {
        self.visit_expression(&expr.operand);
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) {
        self.visit_expression(&expr.callee);
        for arg in &expr.args {
            self.visit_expression(arg);
        }
    }

    fn visit_member_access_expr(&mut self, expr: &MemberAccessExpr) {
        self.visit_expression(&expr.object);
    }

    fn visit_index_expr(&mut self, expr: &IndexExpr) {
        self.visit_expression(&expr.object);
        self.visit_expression(&expr.index);
    }

    fn visit_assignment_expr(&mut self, expr: &AssignmentExpr) {
        self.visit_expression(&expr.target);
        self.visit_expression(&expr.value);
    }

    fn visit_if_expr(&mut self, expr: &IfExpr) {
        self.visit_expression(&expr.condition);
        self.visit_expression(&expr.then_expr);
        self.visit_expression(&expr.else_expr);
    }

    fn visit_match_expr(&mut self, expr: &MatchExpr) {
        self.visit_expression(&expr.expr);
        for arm in &expr.arms {
            if let Some(ref guard) = arm.guard {
                self.visit_expression(guard);
            }
            self.visit_expression(&arm.expr);
        }
    }

    fn visit_array_expr(&mut self, expr: &ArrayExpr) {
        for element in &expr.elements {
            self.visit_expression(element);
        }
    }

    fn visit_map_expr(&mut self, expr: &MapExpr) {
        for entry in &expr.entries {
            self.visit_expression(&entry.key);
            self.visit_expression(&entry.value);
        }
    }

    fn visit_lambda_expr(&mut self, expr: &LambdaExpr) {
        self.visit_expression(&expr.body);
    }

    fn visit_async_expr(&mut self, expr: &AsyncExpr) {
        self.visit_expression(&expr.expr);
    }

    fn visit_await_expr(&mut self, expr: &AwaitExpr) {
        self.visit_expression(&expr.expr);
    }

    fn visit_run_expr(&mut self, expr: &RunExpr) {
        self.visit_expression(&expr.expr);
    }

    fn visit_channel_expr(&mut self, expr: &ChannelExpr) {
        self.visit_expression(&expr.channel);
        if let Some(ref value) = expr.value {
            self.visit_expression(value);
        }
    }

    fn visit_select_expr(&mut self, expr: &SelectExpr) {
        for arm in &expr.arms {
            if let Some(ref channel_op) = arm.channel_op {
                self.visit_expression(&channel_op.channel);
                if let Some(ref value) = channel_op.value {
                    self.visit_expression(value);
                }
            }
            self.visit_expression(&arm.expr);
        }
    }

    fn visit_cast_expr(&mut self, expr: &CastExpr) {
        self.visit_expression(&expr.expr);
    }

    fn visit_typeof_expr(&mut self, expr: &TypeOfExpr) {
        self.visit_expression(&expr.expr);
    }

    fn visit_range_expr(&mut self, expr: &RangeExpr) {
        self.visit_expression(&expr.start);
        self.visit_expression(&expr.end);
        if let Some(ref step) = expr.step {
            self.visit_expression(step);
        }
    }

    fn visit_yield_expr(&mut self, expr: &YieldExpr) {
        if let Some(ref value) = expr.value {
            self.visit_expression(value);
        }
    }

    fn visit_parenthesized_expr(&mut self, expr: &ParenthesizedExpr) {
        self.visit_expression(&expr.expr);
    }

    fn visit_block_expr(&mut self, expr: &BlockExpr) {
        for stmt in &expr.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_tuple_expr(&mut self, expr: &TupleExpr) {
        for element in &expr.elements {
            self.visit_expression(element);
        }
    }
    
    fn visit_struct_literal_expr(&mut self, expr: &StructLiteralExpr) {
        for field in &expr.fields {
            self.visit_expression(&field.value);
        }
    }

    fn visit_pattern(&mut self, _pattern: &Pattern) {}
    fn visit_type(&mut self, _type_node: &Type) {}
}

#[test]
fn test_visitor_pattern() {
    // Create a program with various node types
    let program = AstBuilder::program(vec![
        AstBuilder::variable_decl(
            "x",
            Some(AstBuilder::int32_type()),
            Some(AstBuilder::literal_int(42)),
        ),
        AstBuilder::variable_decl(
            "y",
            Some(AstBuilder::string_type()),
            Some(AstBuilder::literal_string("hello")),
        ),
        AstBuilder::function_decl(
            "add",
            vec![
                AstBuilder::parameter("a", AstBuilder::int32_type()),
                AstBuilder::parameter("b", AstBuilder::int32_type()),
            ],
            Some(AstBuilder::int32_type()),
            AstBuilder::block_stmt(vec![AstBuilder::return_stmt(Some(
                AstBuilder::binary_expr(
                    AstBuilder::identifier("a"),
                    BinaryOperator::Add,
                    AstBuilder::identifier("b"),
                ),
            ))]),
        ),
        AstBuilder::expression_stmt(AstBuilder::call_expr(
            AstBuilder::identifier("add"),
            vec![
                AstBuilder::binary_expr(
                    AstBuilder::literal_int(1),
                    BinaryOperator::Multiply,
                    AstBuilder::literal_int(2),
                ),
                AstBuilder::literal_int(3),
            ],
        )),
    ]);

    let mut counter = NodeCounter::new();
    counter.visit_program(&program);

    // Check counts
    assert_eq!(counter.variable_decls, 2); // x and y
    assert_eq!(counter.function_decls, 1); // add function
    assert_eq!(counter.binary_exprs, 2); // a + b and 1 * 2
    assert_eq!(counter.identifiers, 3); // a, b, add (a appears twice but we count each occurrence)
    assert_eq!(counter.literals, 5); // 42, "hello", 1, 2, 3
}

// Test mutable visitor that renames identifiers
struct IdentifierRenamer {
    old_name: String,
    new_name: String,
}

impl IdentifierRenamer {
    fn new(old_name: &str, new_name: &str) -> Self {
        Self {
            old_name: old_name.to_string(),
            new_name: new_name.to_string(),
        }
    }
}

impl MutVisitor for IdentifierRenamer {
    fn visit_program(&mut self, program: &mut Program) {
        for stmt in &mut program.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_variable_decl(&mut self, decl: &mut VariableDecl) {
        if decl.name == self.old_name {
            decl.name = self.new_name.clone();
        }
        if let Some(ref mut init) = decl.initializer {
            self.visit_expression(init);
        }
    }

    fn visit_function_decl(&mut self, decl: &mut FunctionDecl) {
        if decl.name == self.old_name {
            decl.name = self.new_name.clone();
        }
        for param in &mut decl.params {
            if param.name == self.old_name {
                param.name = self.new_name.clone();
            }
        }
        self.visit_block_stmt(&mut decl.body);
    }

    fn visit_struct_decl(&mut self, _decl: &mut StructDecl) {}
    fn visit_interface_decl(&mut self, _decl: &mut InterfaceDecl) {}
    fn visit_type_alias_decl(&mut self, _decl: &mut TypeAliasDecl) {}
    fn visit_if_stmt(&mut self, stmt: &mut IfStmt) {
        self.visit_expression(&mut stmt.condition);
        self.visit_block_stmt(&mut stmt.then_branch);
        if let Some(ref mut else_branch) = stmt.else_branch {
            self.visit_statement(else_branch);
        }
    }
    fn visit_while_stmt(&mut self, stmt: &mut WhileStmt) {
        self.visit_expression(&mut stmt.condition);
        self.visit_block_stmt(&mut stmt.body);
    }
    fn visit_for_stmt(&mut self, stmt: &mut ForStmt) {
        if stmt.variable == self.old_name {
            stmt.variable = self.new_name.clone();
        }
        self.visit_expression(&mut stmt.iterable);
        self.visit_block_stmt(&mut stmt.body);
    }
    fn visit_match_stmt(&mut self, stmt: &mut MatchStmt) {
        self.visit_expression(&mut stmt.expr);
        for arm in &mut stmt.arms {
            if let Some(ref mut guard) = arm.guard {
                self.visit_expression(guard);
            }
            self.visit_statement(&mut arm.body);
        }
    }

    fn visit_select_stmt(&mut self, stmt: &mut SelectStmt) {
        for arm in &mut stmt.arms {
            if let Some(ref mut channel_op) = arm.channel_op {
                if let Some(ref mut var) = channel_op.variable {
                    if *var == self.old_name {
                        *var = self.new_name.clone();
                    }
                }
                self.visit_expression(&mut channel_op.channel);
                if let Some(ref mut value) = channel_op.value {
                    self.visit_expression(value);
                }
            }
            self.visit_statement(&mut arm.body);
        }
    }

    fn visit_return_stmt(&mut self, stmt: &mut ReturnStmt) {
        if let Some(ref mut value) = stmt.value {
            self.visit_expression(value);
        }
    }
    fn visit_break_stmt(&mut self, _stmt: &mut BreakStmt) {}
    fn visit_continue_stmt(&mut self, _stmt: &mut ContinueStmt) {}
    fn visit_defer_stmt(&mut self, stmt: &mut DeferStmt) {
        self.visit_statement(&mut stmt.stmt);
    }
    fn visit_try_stmt(&mut self, stmt: &mut TryStmt) {
        self.visit_block_stmt(&mut stmt.body);
        if let Some(ref mut catch) = stmt.catch_clause {
            self.visit_block_stmt(&mut catch.body);
        }
    }
    fn visit_fail_stmt(&mut self, stmt: &mut FailStmt) {
        self.visit_expression(&mut stmt.message);
    }
    fn visit_import_stmt(&mut self, _stmt: &mut ImportStmt) {}
    fn visit_export_stmt(&mut self, stmt: &mut ExportStmt) {
        self.visit_statement(&mut stmt.item);
    }
    fn visit_expression_stmt(&mut self, stmt: &mut ExpressionStmt) {
        self.visit_expression(&mut stmt.expr);
    }
    fn visit_block_stmt(&mut self, stmt: &mut BlockStmt) {
        for statement in &mut stmt.statements {
            self.visit_statement(statement);
        }
    }

    fn visit_literal_expr(&mut self, _expr: &mut LiteralExpr) {}

    fn visit_identifier_expr(&mut self, expr: &mut IdentifierExpr) {
        if expr.name == self.old_name {
            expr.name = self.new_name.clone();
        }
    }

    fn visit_binary_expr(&mut self, expr: &mut BinaryExpr) {
        self.visit_expression(&mut expr.left);
        self.visit_expression(&mut expr.right);
    }

    fn visit_unary_expr(&mut self, expr: &mut UnaryExpr) {
        self.visit_expression(&mut expr.operand);
    }

    fn visit_call_expr(&mut self, expr: &mut CallExpr) {
        self.visit_expression(&mut expr.callee);
        for arg in &mut expr.args {
            self.visit_expression(arg);
        }
    }

    fn visit_member_access_expr(&mut self, expr: &mut MemberAccessExpr) {
        self.visit_expression(&mut expr.object);
    }

    fn visit_index_expr(&mut self, expr: &mut IndexExpr) {
        self.visit_expression(&mut expr.object);
        self.visit_expression(&mut expr.index);
    }

    fn visit_assignment_expr(&mut self, expr: &mut AssignmentExpr) {
        self.visit_expression(&mut expr.target);
        self.visit_expression(&mut expr.value);
    }

    fn visit_if_expr(&mut self, expr: &mut IfExpr) {
        self.visit_expression(&mut expr.condition);
        self.visit_expression(&mut expr.then_expr);
        self.visit_expression(&mut expr.else_expr);
    }

    fn visit_match_expr(&mut self, expr: &mut MatchExpr) {
        self.visit_expression(&mut expr.expr);
        for arm in &mut expr.arms {
            if let Some(ref mut guard) = arm.guard {
                self.visit_expression(guard);
            }
            self.visit_expression(&mut arm.expr);
        }
    }

    fn visit_array_expr(&mut self, expr: &mut ArrayExpr) {
        for element in &mut expr.elements {
            self.visit_expression(element);
        }
    }

    fn visit_map_expr(&mut self, expr: &mut MapExpr) {
        for entry in &mut expr.entries {
            self.visit_expression(&mut entry.key);
            self.visit_expression(&mut entry.value);
        }
    }

    fn visit_lambda_expr(&mut self, expr: &mut LambdaExpr) {
        for param in &mut expr.params {
            if param.name == self.old_name {
                param.name = self.new_name.clone();
            }
        }
        self.visit_expression(&mut expr.body);
    }

    fn visit_async_expr(&mut self, expr: &mut AsyncExpr) {
        self.visit_expression(&mut expr.expr);
    }

    fn visit_await_expr(&mut self, expr: &mut AwaitExpr) {
        self.visit_expression(&mut expr.expr);
    }

    fn visit_run_expr(&mut self, expr: &mut RunExpr) {
        self.visit_expression(&mut expr.expr);
    }

    fn visit_channel_expr(&mut self, expr: &mut ChannelExpr) {
        self.visit_expression(&mut expr.channel);
        if let Some(ref mut value) = expr.value {
            self.visit_expression(value);
        }
    }

    fn visit_select_expr(&mut self, expr: &mut SelectExpr) {
        for arm in &mut expr.arms {
            if let Some(ref mut channel_op) = arm.channel_op {
                if let Some(ref mut var) = channel_op.variable {
                    if *var == self.old_name {
                        *var = self.new_name.clone();
                    }
                }
                self.visit_expression(&mut channel_op.channel);
                if let Some(ref mut value) = channel_op.value {
                    self.visit_expression(value);
                }
            }
            self.visit_expression(&mut arm.expr);
        }
    }

    fn visit_cast_expr(&mut self, expr: &mut CastExpr) {
        self.visit_expression(&mut expr.expr);
    }

    fn visit_typeof_expr(&mut self, expr: &mut TypeOfExpr) {
        self.visit_expression(&mut expr.expr);
    }

    fn visit_range_expr(&mut self, expr: &mut RangeExpr) {
        self.visit_expression(&mut expr.start);
        self.visit_expression(&mut expr.end);
        if let Some(ref mut step) = expr.step {
            self.visit_expression(step);
        }
    }

    fn visit_yield_expr(&mut self, expr: &mut YieldExpr) {
        if let Some(ref mut value) = expr.value {
            self.visit_expression(value);
        }
    }

    fn visit_parenthesized_expr(&mut self, expr: &mut ParenthesizedExpr) {
        self.visit_expression(&mut expr.expr);
    }

    fn visit_block_expr(&mut self, expr: &mut BlockExpr) {
        for stmt in &mut expr.statements {
            self.visit_statement(stmt);
        }
    }

    fn visit_tuple_expr(&mut self, expr: &mut TupleExpr) {
        for element in &mut expr.elements {
            self.visit_expression(element);
        }
    }
    
    fn visit_struct_literal_expr(&mut self, expr: &mut StructLiteralExpr) {
        for field in &mut expr.fields {
            self.visit_expression(&mut field.value);
        }
    }

    fn visit_pattern(&mut self, _pattern: &mut Pattern) {}
    fn visit_type(&mut self, _type_node: &mut Type) {}
}

#[test]
fn test_mutable_visitor() {
    // Create a program with identifiers to rename
    let mut program = AstBuilder::program(vec![
        AstBuilder::variable_decl(
            "old_var",
            Some(AstBuilder::int32_type()),
            Some(AstBuilder::literal_int(42)),
        ),
        AstBuilder::expression_stmt(AstBuilder::binary_expr(
            AstBuilder::identifier("old_var"),
            BinaryOperator::Add,
            AstBuilder::literal_int(1),
        )),
    ]);

    // Rename "old_var" to "new_var"
    let mut renamer = IdentifierRenamer::new("old_var", "new_var");
    renamer.visit_program(&mut program);

    // Check that the variable declaration was renamed
    if let Statement::VariableDecl(var_decl) = &program.statements[0] {
        assert_eq!(var_decl.name, "new_var");
    } else {
        panic!("Expected variable declaration");
    }

    // Check that the identifier in the expression was renamed
    if let Statement::Expression(expr_stmt) = &program.statements[1] {
        if let Expression::Binary(binary_expr) = &expr_stmt.expr {
            if let Expression::Identifier(ident) = &*binary_expr.left {
                assert_eq!(ident.name, "new_var");
            } else {
                panic!("Expected identifier");
            }
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_walker_functions() {
    let stmt = AstBuilder::expression_stmt(AstBuilder::binary_expr(
        AstBuilder::identifier("x"),
        BinaryOperator::Add,
        AstBuilder::literal_int(1),
    ));

    let mut counter = NodeCounter::new();
    walk_statement(&mut counter, &stmt);

    assert_eq!(counter.binary_exprs, 1);
    assert_eq!(counter.identifiers, 1);
    assert_eq!(counter.literals, 1);
}

#[test]
fn test_complex_visitor_traversal() {
    // Create a complex program with nested structures
    let program = AstBuilder::program(vec![AstBuilder::function_decl(
        "complex_func",
        vec![AstBuilder::parameter("param", AstBuilder::int32_type())],
        Some(AstBuilder::int32_type()),
        AstBuilder::block_stmt(vec![AstBuilder::if_stmt(
            AstBuilder::binary_expr(
                AstBuilder::identifier("param"),
                BinaryOperator::Greater,
                AstBuilder::literal_int(0),
            ),
            AstBuilder::block_stmt(vec![AstBuilder::return_stmt(Some(
                AstBuilder::binary_expr(
                    AstBuilder::identifier("param"),
                    BinaryOperator::Multiply,
                    AstBuilder::literal_int(2),
                ),
            ))]),
            Some(AstBuilder::return_stmt(Some(AstBuilder::literal_int(0)))),
        )]),
    )]);

    let mut counter = NodeCounter::new();
    counter.visit_program(&program);

    // Should count all nested nodes
    assert_eq!(counter.function_decls, 1);
    assert_eq!(counter.binary_exprs, 2); // param > 0 and param * 2
    assert_eq!(counter.identifiers, 2); // param appears twice
    assert_eq!(counter.literals, 3); // 0, 2, and 0 again
}
