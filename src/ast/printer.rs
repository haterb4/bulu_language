//! AST pretty printer for debugging and visualization
//!
//! This module provides functionality to print AST nodes in a human-readable format,
//! useful for debugging, testing, and development tools.

use super::nodes::*;
use std::fmt::Write;

/// Pretty printer for AST nodes
pub struct AstPrinter {
    indent_level: usize,
    indent_size: usize,
}

impl AstPrinter {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            indent_size: 2,
        }
    }

    pub fn with_indent_size(indent_size: usize) -> Self {
        Self {
            indent_level: 0,
            indent_size,
        }
    }

    fn indent(&self) -> String {
        " ".repeat(self.indent_level * self.indent_size)
    }

    fn with_increased_indent<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut Self) -> R,
    {
        self.indent_level += 1;
        let result = f(self);
        self.indent_level -= 1;
        result
    }

    pub fn print_program(&mut self, program: &Program) -> String {
        let mut output = String::new();
        writeln!(output, "Program {{").unwrap();

        self.with_increased_indent(|printer| {
            for stmt in &program.statements {
                let stmt_str = printer.print_statement(stmt);
                for line in stmt_str.lines() {
                    writeln!(output, "{}{}", printer.indent(), line).unwrap();
                }
            }
        });

        writeln!(output, "}}").unwrap();
        output
    }

    pub fn print_statement(&mut self, stmt: &Statement) -> String {
        match stmt {
            Statement::VariableDecl(decl) => self.print_variable_decl(decl),
            Statement::FunctionDecl(decl) => self.print_function_decl(decl),
            Statement::StructDecl(decl) => self.print_struct_decl(decl),
            Statement::InterfaceDecl(decl) => self.print_interface_decl(decl),
            Statement::TypeAlias(decl) => self.print_type_alias_decl(decl),
            Statement::If(stmt) => self.print_if_stmt(stmt),
            Statement::While(stmt) => self.print_while_stmt(stmt),
            Statement::For(stmt) => self.print_for_stmt(stmt),
            Statement::Match(stmt) => self.print_match_stmt(stmt),
            Statement::Select(stmt) => self.print_select_stmt(stmt),
            Statement::Return(stmt) => self.print_return_stmt(stmt),
            Statement::Break(_) => "Break".to_string(),
            Statement::Continue(_) => "Continue".to_string(),
            Statement::Defer(stmt) => format!("Defer({})", self.print_statement(&stmt.stmt)),
            Statement::Try(stmt) => self.print_try_stmt(stmt),
            Statement::Fail(stmt) => format!("Fail({})", self.print_expression(&stmt.message)),
            Statement::Import(stmt) => self.print_import_stmt(stmt),
            Statement::Export(stmt) => format!("Export({})", self.print_statement(&stmt.item)),
            Statement::Expression(stmt) => {
                format!("ExprStmt({})", self.print_expression(&stmt.expr))
            }
            Statement::Block(stmt) => self.print_block_stmt(stmt),
        }
    }

    pub fn print_expression(&mut self, expr: &Expression) -> String {
        match expr {
            Expression::Literal(expr) => self.print_literal_expr(expr),
            Expression::Identifier(expr) => format!("Ident({})", expr.name),
            Expression::Binary(expr) => self.print_binary_expr(expr),
            Expression::Unary(expr) => self.print_unary_expr(expr),
            Expression::Call(expr) => self.print_call_expr(expr),
            Expression::MemberAccess(expr) => {
                format!(
                    "MemberAccess({}.{})",
                    self.print_expression(&expr.object),
                    expr.member
                )
            }
            Expression::Index(expr) => {
                format!(
                    "Index({}[{}])",
                    self.print_expression(&expr.object),
                    self.print_expression(&expr.index)
                )
            }
            Expression::Assignment(expr) => self.print_assignment_expr(expr),
            Expression::If(expr) => self.print_if_expr(expr),
            Expression::Match(expr) => self.print_match_expr(expr),
            Expression::Select(expr) => self.print_select_expr(expr),
            Expression::Array(expr) => self.print_array_expr(expr),
            Expression::Map(expr) => self.print_map_expr(expr),
            Expression::Lambda(expr) => self.print_lambda_expr(expr),
            Expression::Async(expr) => format!("Async({})", self.print_expression(&expr.expr)),
            Expression::Await(expr) => format!("Await({})", self.print_expression(&expr.expr)),
            Expression::Run(expr) => format!("Run({})", self.print_expression(&expr.expr)),
            Expression::Channel(expr) => self.print_channel_expr(expr),
            Expression::Cast(expr) => {
                format!(
                    "Cast({} as {})",
                    self.print_expression(&expr.expr),
                    self.print_type(&expr.target_type)
                )
            }
            Expression::TypeOf(expr) => format!("TypeOf({})", self.print_expression(&expr.expr)),
            Expression::Range(expr) => self.print_range_expr(expr),
            Expression::Yield(expr) => match &expr.value {
                Some(val) => format!("Yield({})", self.print_expression(val)),
                None => "Yield".to_string(),
            },
            Expression::Parenthesized(expr) => {
                format!("({})", self.print_expression(&expr.expr))
            }
            Expression::Block(expr) => self.print_block_expr(expr),
            Expression::Tuple(expr) => self.print_tuple_expr(expr),
            Expression::StructLiteral(expr) => self.print_struct_literal_expr(expr),
        }
    }

    fn print_variable_decl(&mut self, decl: &VariableDecl) -> String {
        let mut result = if decl.is_const {
            "Const ".to_string()
        } else {
            "Let ".to_string()
        };
        result.push_str(&decl.name);

        if let Some(ref type_ann) = decl.type_annotation {
            result.push_str(&format!(": {}", self.print_type(type_ann)));
        }

        if let Some(ref init) = decl.initializer {
            result.push_str(&format!(" = {}", self.print_expression(init)));
        }

        result
    }

    fn print_function_decl(&mut self, decl: &FunctionDecl) -> String {
        let mut result = String::new();

        if decl.is_async {
            result.push_str("Async ");
        }
        result.push_str("Func ");
        result.push_str(&decl.name);

        if !decl.type_params.is_empty() {
            result.push('<');
            for (i, param) in decl.type_params.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                result.push_str(&param.name);
            }
            result.push('>');
        }

        result.push('(');
        for (i, param) in decl.params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!(
                "{}: {}",
                param.name,
                self.print_type(&param.param_type)
            ));
        }
        result.push(')');

        if let Some(ref return_type) = decl.return_type {
            result.push_str(&format!(": {}", self.print_type(return_type)));
        }

        result.push_str(" ");
        result.push_str(&self.print_block_stmt(&decl.body));

        result
    }

    fn print_struct_decl(&mut self, decl: &StructDecl) -> String {
        let mut result = format!("Struct {} {{", decl.name);

        self.with_increased_indent(|printer| {
            for field in &decl.fields {
                result.push('\n');
                result.push_str(&printer.indent());
                result.push_str(&format!(
                    "{}: {}",
                    field.name,
                    printer.print_type(&field.field_type)
                ));
            }
        });

        result.push_str("\n}");
        result
    }

    fn print_interface_decl(&mut self, decl: &InterfaceDecl) -> String {
        let mut result = format!("Interface {} {{", decl.name);

        self.with_increased_indent(|printer| {
            for method in &decl.methods {
                result.push('\n');
                result.push_str(&printer.indent());
                result.push_str(&format!("{}(", method.name));

                for (i, param) in method.params.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&format!(
                        "{}: {}",
                        param.name,
                        printer.print_type(&param.param_type)
                    ));
                }

                result.push(')');
                if let Some(ref return_type) = method.return_type {
                    result.push_str(&format!(": {}", printer.print_type(return_type)));
                }
            }
        });

        result.push_str("\n}");
        result
    }

    fn print_type_alias_decl(&mut self, decl: &TypeAliasDecl) -> String {
        let mut result = format!("type {}", decl.name);

        if !decl.type_params.is_empty() {
            result.push('<');
            for (i, param) in decl.type_params.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                result.push_str(&param.name);
                if !param.constraints.is_empty() {
                    result.push_str(": ");
                    for (j, constraint) in param.constraints.iter().enumerate() {
                        if j > 0 {
                            result.push_str(" + ");
                        }
                        result.push_str(&self.print_type(constraint));
                    }
                }
            }
            result.push('>');
        }

        result.push_str(" = ");
        result.push_str(&self.print_type(&decl.target_type));
        result
    }

    fn print_if_stmt(&mut self, stmt: &IfStmt) -> String {
        let mut result = format!(
            "If ({}) {}",
            self.print_expression(&stmt.condition),
            self.print_block_stmt(&stmt.then_branch)
        );

        if let Some(ref else_branch) = stmt.else_branch {
            result.push_str(&format!(" Else {}", self.print_statement(else_branch)));
        }

        result
    }

    fn print_while_stmt(&mut self, stmt: &WhileStmt) -> String {
        format!(
            "While ({}) {}",
            self.print_expression(&stmt.condition),
            self.print_block_stmt(&stmt.body)
        )
    }

    fn print_for_stmt(&mut self, stmt: &ForStmt) -> String {
        let variables = if let Some(ref index_var) = stmt.index_variable {
            format!("{}, {}", index_var, stmt.variable)
        } else {
            stmt.variable.clone()
        };

        format!(
            "For {} in {} {}",
            variables,
            self.print_expression(&stmt.iterable),
            self.print_block_stmt(&stmt.body)
        )
    }

    fn print_match_stmt(&mut self, stmt: &MatchStmt) -> String {
        let mut result = format!("Match ({}) {{", self.print_expression(&stmt.expr));

        self.with_increased_indent(|printer| {
            for arm in &stmt.arms {
                result.push('\n');
                result.push_str(&printer.indent());
                result.push_str(&printer.print_pattern(&arm.pattern));
                if let Some(ref guard) = arm.guard {
                    result.push_str(&format!(" if {}", printer.print_expression(guard)));
                }
                result.push_str(" => ");
                result.push_str(&printer.print_statement(&arm.body));
            }
        });

        result.push_str("\n}");
        result
    }

    fn print_select_stmt(&mut self, stmt: &SelectStmt) -> String {
        let mut result = String::from("Select {");

        self.with_increased_indent(|printer| {
            for arm in &stmt.arms {
                result.push('\n');
                result.push_str(&printer.indent());

                if let Some(ref channel_op) = arm.channel_op {
                    if channel_op.is_send {
                        // Send operation: channel <- value
                        result.push_str(&printer.print_expression(&channel_op.channel));
                        result.push_str(" <- ");
                        if let Some(ref value) = channel_op.value {
                            result.push_str(&printer.print_expression(value));
                        }
                    } else {
                        // Receive operation: [variable :=] <-channel
                        if let Some(ref var) = channel_op.variable {
                            result.push_str(var);
                            result.push_str(" := ");
                        }
                        result.push_str("<-");
                        result.push_str(&printer.print_expression(&channel_op.channel));
                    }
                } else {
                    // Default case
                    result.push('_');
                }

                result.push_str(" => ");
                result.push_str(&printer.print_statement(&arm.body));
            }
        });

        result.push_str("\n}");
        result
    }

    fn print_return_stmt(&mut self, stmt: &ReturnStmt) -> String {
        match &stmt.value {
            Some(expr) => format!("Return({})", self.print_expression(expr)),
            None => "Return".to_string(),
        }
    }

    fn print_try_stmt(&mut self, stmt: &TryStmt) -> String {
        let mut result = format!("Try {}", self.print_block_stmt(&stmt.body));

        if let Some(ref catch) = stmt.catch_clause {
            result.push_str(" Catch");
            if let Some(ref var) = catch.error_var {
                result.push_str(&format!(" {}", var));
            }
            result.push_str(&format!(" {}", self.print_block_stmt(&catch.body)));
        }

        result
    }

    fn print_import_stmt(&mut self, stmt: &ImportStmt) -> String {
        let mut result = format!("Import \"{}\"", stmt.path);

        if let Some(ref alias) = stmt.alias {
            result.push_str(&format!(" as {}", alias));
        }

        if let Some(ref items) = stmt.items {
            result.push_str(" {");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                result.push_str(&item.name);
                if let Some(ref alias) = item.alias {
                    result.push_str(&format!(" as {}", alias));
                }
            }
            result.push('}');
        }

        result
    }

    fn print_block_stmt(&mut self, stmt: &BlockStmt) -> String {
        if stmt.statements.is_empty() {
            return "{}".to_string();
        }

        let mut result = String::from("{\n");

        self.with_increased_indent(|printer| {
            for statement in &stmt.statements {
                let stmt_str = printer.print_statement(statement);
                for line in stmt_str.lines() {
                    result.push_str(&printer.indent());
                    result.push_str(line);
                    result.push('\n');
                }
            }
        });

        result.push_str(&self.indent());
        result.push('}');
        result
    }

    fn print_literal_expr(&mut self, expr: &LiteralExpr) -> String {
        match &expr.value {
            LiteralValue::Integer(n) => n.to_string(),
            LiteralValue::Float(f) => f.to_string(),
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Char(c) => format!("'{}'", c),
            LiteralValue::Boolean(b) => b.to_string(),
            LiteralValue::Null => "null".to_string(),
        }
    }

    fn print_binary_expr(&mut self, expr: &BinaryExpr) -> String {
        format!(
            "({} {} {})",
            self.print_expression(&expr.left),
            self.print_binary_operator(expr.operator),
            self.print_expression(&expr.right)
        )
    }

    fn print_unary_expr(&mut self, expr: &UnaryExpr) -> String {
        format!(
            "({}{})",
            self.print_unary_operator(expr.operator),
            self.print_expression(&expr.operand)
        )
    }

    fn print_call_expr(&mut self, expr: &CallExpr) -> String {
        let mut result = self.print_expression(&expr.callee);

        if !expr.type_args.is_empty() {
            result.push('<');
            for (i, type_arg) in expr.type_args.iter().enumerate() {
                if i > 0 {
                    result.push_str(", ");
                }
                result.push_str(&self.print_type(type_arg));
            }
            result.push('>');
        }

        result.push('(');
        for (i, arg) in expr.args.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&self.print_expression(arg));
        }
        result.push(')');

        result
    }

    fn print_assignment_expr(&mut self, expr: &AssignmentExpr) -> String {
        format!(
            "{} {} {}",
            self.print_expression(&expr.target),
            self.print_assignment_operator(expr.operator),
            self.print_expression(&expr.value)
        )
    }

    fn print_if_expr(&mut self, expr: &IfExpr) -> String {
        format!(
            "if {} then {} else {}",
            self.print_expression(&expr.condition),
            self.print_expression(&expr.then_expr),
            self.print_expression(&expr.else_expr)
        )
    }

    fn print_match_expr(&mut self, expr: &MatchExpr) -> String {
        let mut result = format!("match {} {{", self.print_expression(&expr.expr));

        for arm in &expr.arms {
            result.push_str(&format!(
                " {} => {}",
                self.print_pattern(&arm.pattern),
                self.print_expression(&arm.expr)
            ));
        }

        result.push_str(" }");
        result
    }

    fn print_select_expr(&mut self, expr: &SelectExpr) -> String {
        let mut result = String::from("select {");

        for arm in &expr.arms {
            result.push(' ');

            if let Some(ref channel_op) = arm.channel_op {
                if channel_op.is_send {
                    // Send operation: channel <- value
                    result.push_str(&self.print_expression(&channel_op.channel));
                    result.push_str(" <- ");
                    if let Some(ref value) = channel_op.value {
                        result.push_str(&self.print_expression(value));
                    }
                } else {
                    // Receive operation: [variable :=] <-channel
                    if let Some(ref var) = channel_op.variable {
                        result.push_str(var);
                        result.push_str(" := ");
                    }
                    result.push_str("<-");
                    result.push_str(&self.print_expression(&channel_op.channel));
                }
            } else {
                // Default case
                result.push('_');
            }

            result.push_str(" => ");
            result.push_str(&self.print_expression(&arm.expr));
        }

        result.push_str(" }");
        result
    }

    fn print_array_expr(&mut self, expr: &ArrayExpr) -> String {
        let mut result = String::from("[");
        for (i, element) in expr.elements.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&self.print_expression(element));
        }
        result.push(']');
        result
    }

    fn print_map_expr(&mut self, expr: &MapExpr) -> String {
        let mut result = String::from("{");
        for (i, entry) in expr.entries.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!(
                "{}: {}",
                self.print_expression(&entry.key),
                self.print_expression(&entry.value)
            ));
        }
        result.push('}');
        result
    }

    fn print_lambda_expr(&mut self, expr: &LambdaExpr) -> String {
        let mut result = String::from("(");
        for (i, param) in expr.params.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!(
                "{}: {}",
                param.name,
                self.print_type(&param.param_type)
            ));
        }
        result.push_str(") => ");
        result.push_str(&self.print_expression(&expr.body));
        result
    }

    fn print_channel_expr(&mut self, expr: &ChannelExpr) -> String {
        match expr.direction {
            ChannelDirection::Send => {
                format!(
                    "{} <- {}",
                    self.print_expression(&expr.channel),
                    self.print_expression(expr.value.as_ref().unwrap())
                )
            }
            ChannelDirection::Receive => {
                format!("<-{}", self.print_expression(&expr.channel))
            }
            ChannelDirection::Bidirectional => {
                format!("chan({})", self.print_expression(&expr.channel))
            }
        }
    }

    fn print_range_expr(&mut self, expr: &RangeExpr) -> String {
        let op = if expr.inclusive { "..." } else { "..<" };
        let base_range = format!(
            "{}{}{}",
            self.print_expression(&expr.start),
            op,
            self.print_expression(&expr.end)
        );

        if let Some(ref step) = expr.step {
            format!("{} step {}", base_range, self.print_expression(step))
        } else {
            base_range
        }
    }

    pub fn print_pattern(&mut self, pattern: &Pattern) -> String {
        match pattern {
            Pattern::Wildcard(_) => "_".to_string(),
            Pattern::Literal(lit, _) => self.print_literal_value(lit),
            Pattern::Identifier(name, _) => name.clone(),
            Pattern::Struct(pat) => self.print_struct_pattern(pat),
            Pattern::Array(pat) => self.print_array_pattern(pat),
            Pattern::Range(pat) => self.print_range_pattern(pat),
            Pattern::Or(pat) => self.print_or_pattern(pat),
        }
    }

    fn print_struct_pattern(&mut self, pat: &StructPattern) -> String {
        let mut result = format!("{} {{", pat.name);
        for (i, field) in pat.fields.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!(
                "{}: {}",
                field.name,
                self.print_pattern(&field.pattern)
            ));
        }
        result.push('}');
        result
    }

    fn print_array_pattern(&mut self, pat: &ArrayPattern) -> String {
        let mut result = String::from("[");
        for (i, element) in pat.elements.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&self.print_pattern(element));
        }
        result.push(']');
        result
    }

    fn print_range_pattern(&mut self, pat: &RangePattern) -> String {
        let op = if pat.inclusive { "..." } else { "..<" };
        format!(
            "{}{}{}",
            self.print_literal_value(&pat.start),
            op,
            self.print_literal_value(&pat.end)
        )
    }

    fn print_or_pattern(&mut self, pat: &OrPattern) -> String {
        let mut result = String::new();
        for (i, pattern) in pat.patterns.iter().enumerate() {
            if i > 0 {
                result.push_str(" | ");
            }
            result.push_str(&self.print_pattern(pattern));
        }
        result
    }

    fn print_literal_value(&mut self, lit: &LiteralValue) -> String {
        match lit {
            LiteralValue::Integer(n) => n.to_string(),
            LiteralValue::Float(f) => f.to_string(),
            LiteralValue::String(s) => format!("\"{}\"", s),
            LiteralValue::Char(c) => format!("'{}'", c),
            LiteralValue::Boolean(b) => b.to_string(),
            LiteralValue::Null => "null".to_string(),
        }
    }

    pub fn print_type(&mut self, type_node: &Type) -> String {
        match type_node {
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
            Type::Array(arr) => match arr.size {
                Some(size) => format!("[{}; {}]", self.print_type(&arr.element_type), size),
                None => format!("[{}]", self.print_type(&arr.element_type)),
            },
            Type::Slice(slice) => format!("[{}]", self.print_type(&slice.element_type)),
            Type::Map(map) => format!(
                "map[{}, {}]",
                self.print_type(&map.key_type),
                self.print_type(&map.value_type)
            ),
            Type::Function(func) => {
                let mut result = String::from("func(");
                for (i, param_type) in func.param_types.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.print_type(param_type));
                }
                result.push(')');
                if let Some(ref return_type) = func.return_type {
                    result.push_str(&format!(": {}", self.print_type(return_type)));
                }
                result
            }
            Type::Struct(s) => {
                let mut result = s.name.clone();
                if !s.type_args.is_empty() {
                    result.push('<');
                    for (i, arg) in s.type_args.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(&self.print_type(arg));
                    }
                    result.push('>');
                }
                result
            }
            Type::Interface(i) => {
                let mut result = i.name.clone();
                if !i.type_args.is_empty() {
                    result.push('<');
                    for (i, arg) in i.type_args.iter().enumerate() {
                        if i > 0 {
                            result.push_str(", ");
                        }
                        result.push_str(&self.print_type(arg));
                    }
                    result.push('>');
                }
                result
            }
            Type::Generic(g) => g.name.clone(),
            Type::Channel(c) => match c.direction {
                ChannelDirection::Send => format!("chan<- {}", self.print_type(&c.element_type)),
                ChannelDirection::Receive => format!("<-chan {}", self.print_type(&c.element_type)),
                ChannelDirection::Bidirectional => {
                    format!("chan {}", self.print_type(&c.element_type))
                }
            },
            Type::Named(name) => name.clone(),
            Type::Tuple(tuple) => {
                let mut result = String::from("(");
                for (i, element_type) in tuple.element_types.iter().enumerate() {
                    if i > 0 {
                        result.push_str(", ");
                    }
                    result.push_str(&self.print_type(element_type));
                }
                result.push(')');
                result
            }
            Type::Void => "void".to_string(),
            Type::Promise(promise) => format!("Promise<{}>", self.print_type(&promise.result_type)),
        }
    }

    fn print_block_expr(&mut self, expr: &BlockExpr) -> String {
        let mut result = String::from("{\n");
        self.indent_level += 1;

        for stmt in &expr.statements {
            result.push_str(&format!(
                "{}{}\n",
                "  ".repeat(self.indent_level),
                self.print_statement(stmt)
            ));
        }

        self.indent_level -= 1;
        result.push_str(&format!("{}}}", "  ".repeat(self.indent_level)));
        result
    }

    fn print_tuple_expr(&mut self, expr: &TupleExpr) -> String {
        let mut result = String::from("(");
        for (i, element) in expr.elements.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&self.print_expression(element));
        }
        result.push(')');
        result
    }

    fn print_struct_literal_expr(&mut self, expr: &StructLiteralExpr) -> String {
        let mut result = format!("{} {{", expr.type_name);
        for (i, field) in expr.fields.iter().enumerate() {
            if i > 0 {
                result.push_str(", ");
            }
            result.push_str(&format!("{}: {}", field.name, self.print_expression(&field.value)));
        }
        result.push('}');
        result
    }

    fn print_binary_operator(&self, op: BinaryOperator) -> &'static str {
        match op {
            BinaryOperator::Add => "+",
            BinaryOperator::Subtract => "-",
            BinaryOperator::Multiply => "*",
            BinaryOperator::Divide => "/",
            BinaryOperator::Modulo => "%",
            BinaryOperator::Power => "**",
            BinaryOperator::Equal => "==",
            BinaryOperator::NotEqual => "!=",
            BinaryOperator::Less => "<",
            BinaryOperator::Greater => ">",
            BinaryOperator::LessEqual => "<=",
            BinaryOperator::GreaterEqual => ">=",
            BinaryOperator::And => "and",
            BinaryOperator::Or => "or",
            BinaryOperator::BitwiseAnd => "&",
            BinaryOperator::BitwiseOr => "|",
            BinaryOperator::BitwiseXor => "^",
            BinaryOperator::LeftShift => "<<",
            BinaryOperator::RightShift => ">>",
        }
    }

    fn print_unary_operator(&self, op: UnaryOperator) -> &'static str {
        match op {
            UnaryOperator::Plus => "+",
            UnaryOperator::Minus => "-",
            UnaryOperator::Not => "not",
            UnaryOperator::BitwiseNot => "~",
        }
    }

    fn print_assignment_operator(&self, op: AssignmentOperator) -> &'static str {
        match op {
            AssignmentOperator::Assign => "=",
            AssignmentOperator::AddAssign => "+=",
            AssignmentOperator::SubtractAssign => "-=",
            AssignmentOperator::MultiplyAssign => "*=",
            AssignmentOperator::DivideAssign => "/=",
            AssignmentOperator::ModuloAssign => "%=",
        }
    }
}

impl Default for AstPrinter {
    fn default() -> Self {
        Self::new()
    }
}
