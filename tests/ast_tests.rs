use bulu::ast::*;
use bulu::lexer::token::Position;

fn dummy_pos() -> Position {
    Position::new(1, 1, 0)
}

#[test]
fn test_ast_builder_basic() {
    // Test basic AST construction using the builder
    let program = AstBuilder::program(vec![
        AstBuilder::variable_decl(
            "x",
            Some(AstBuilder::int32_type()),
            Some(AstBuilder::literal_int(42)),
        ),
        AstBuilder::function_decl(
            "main",
            vec![],
            None,
            AstBuilder::block_stmt(vec![AstBuilder::expression_stmt(AstBuilder::call_expr(
                AstBuilder::identifier("println"),
                vec![AstBuilder::literal_string("Hello, World!")],
            ))]),
        ),
    ]);

    assert_eq!(program.statements.len(), 2);

    // Check variable declaration
    if let Statement::VariableDecl(var_decl) = &program.statements[0] {
        assert_eq!(var_decl.name, "x");
        assert!(!var_decl.is_const);
        assert!(var_decl.type_annotation.is_some());
        assert!(var_decl.initializer.is_some());
    } else {
        panic!("Expected variable declaration");
    }

    // Check function declaration
    if let Statement::FunctionDecl(func_decl) = &program.statements[1] {
        assert_eq!(func_decl.name, "main");
        assert_eq!(func_decl.params.len(), 0);
        assert!(func_decl.return_type.is_none());
        assert!(!func_decl.is_async);
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_binary_expressions() {
    let expr = AstBuilder::binary_expr(
        AstBuilder::literal_int(2),
        BinaryOperator::Add,
        AstBuilder::binary_expr(
            AstBuilder::literal_int(3),
            BinaryOperator::Multiply,
            AstBuilder::literal_int(4),
        ),
    );

    if let Expression::Binary(binary_expr) = expr {
        assert_eq!(binary_expr.operator, BinaryOperator::Add);

        // Check left operand
        if let Expression::Literal(lit) = &*binary_expr.left {
            assert_eq!(lit.value, LiteralValue::Integer(2));
        } else {
            panic!("Expected literal on left side");
        }

        // Check right operand (nested binary expression)
        if let Expression::Binary(right_binary) = &*binary_expr.right {
            assert_eq!(right_binary.operator, BinaryOperator::Multiply);
        } else {
            panic!("Expected binary expression on right side");
        }
    } else {
        panic!("Expected binary expression");
    }
}

#[test]
fn test_function_call_with_type_args() {
    let call = CallExpr {
        callee: Box::new(AstBuilder::identifier("make")),
        type_args: vec![AstBuilder::slice_type(AstBuilder::int32_type())],
        args: vec![AstBuilder::literal_int(10)],
        position: dummy_pos(),
    };

    assert_eq!(call.type_args.len(), 1);
    assert_eq!(call.args.len(), 1);

    if let Type::Slice(slice_type) = &call.type_args[0] {
        assert_eq!(*slice_type.element_type, Type::Int32);
    } else {
        panic!("Expected slice type");
    }
}

#[test]
fn test_struct_declaration() {
    let struct_decl = StructDecl {
        name: "Point".to_string(),
        type_params: vec![],
        fields: vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::Float64,
                position: dummy_pos(),
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::Float64,
                position: dummy_pos(),
            },
        ],
        methods: vec![],
        doc_comment: None,
        is_exported: false,
        position: dummy_pos(),
    };

    assert_eq!(struct_decl.name, "Point");
    assert_eq!(struct_decl.fields.len(), 2);
    assert_eq!(struct_decl.fields[0].name, "x");
    assert_eq!(struct_decl.fields[1].name, "y");
    assert_eq!(struct_decl.fields[0].field_type, Type::Float64);
}

#[test]
fn test_interface_declaration() {
    let interface_decl = InterfaceDecl {
        name: "Shape".to_string(),
        type_params: vec![],
        methods: vec![
            InterfaceMethod {
                name: "area".to_string(),
                params: vec![],
                return_type: Some(Type::Float64),
                position: dummy_pos(),
            },
            InterfaceMethod {
                name: "perimeter".to_string(),
                params: vec![],
                return_type: Some(Type::Float64),
                position: dummy_pos(),
            },
        ],
        doc_comment: None,
        is_exported: false,
        position: dummy_pos(),
    };

    assert_eq!(interface_decl.name, "Shape");
    assert_eq!(interface_decl.methods.len(), 2);
    assert_eq!(interface_decl.methods[0].name, "area");
    assert_eq!(interface_decl.methods[1].name, "perimeter");
}

#[test]
fn test_match_expression() {
    let match_expr = MatchExpr {
        expr: Box::new(AstBuilder::identifier("x")),
        arms: vec![
            MatchExprArm {
                pattern: Pattern::Literal(LiteralValue::Integer(1), dummy_pos()),
                guard: None,
                expr: AstBuilder::literal_string("one"),
                position: dummy_pos(),
            },
            MatchExprArm {
                pattern: Pattern::Literal(LiteralValue::Integer(2), dummy_pos()),
                guard: None,
                expr: AstBuilder::literal_string("two"),
                position: dummy_pos(),
            },
            MatchExprArm {
                pattern: Pattern::Wildcard(dummy_pos()),
                guard: None,
                expr: AstBuilder::literal_string("other"),
                position: dummy_pos(),
            },
        ],
        position: dummy_pos(),
    };

    assert_eq!(match_expr.arms.len(), 3);

    // Check first arm
    if let Pattern::Literal(LiteralValue::Integer(1), _) = match_expr.arms[0].pattern {
        // OK
    } else {
        panic!("Expected literal pattern with value 1");
    }

    // Check wildcard arm
    if let Pattern::Wildcard(_) = match_expr.arms[2].pattern {
        // OK
    } else {
        panic!("Expected wildcard pattern");
    }
}

#[test]
fn test_async_await_expressions() {
    let async_expr = AsyncExpr {
        expr: Box::new(AstBuilder::call_expr(
            AstBuilder::identifier("fetch_data"),
            vec![AstBuilder::literal_string("https://api.example.com")],
        )),
        position: dummy_pos(),
    };

    let await_expr = AwaitExpr {
        expr: Box::new(Expression::Async(async_expr)),
        position: dummy_pos(),
    };

    if let Expression::Async(async_inner) = &*await_expr.expr {
        if let Expression::Call(call) = &*async_inner.expr {
            if let Expression::Identifier(ident) = &*call.callee {
                assert_eq!(ident.name, "fetch_data");
            } else {
                panic!("Expected identifier in async call");
            }
        } else {
            panic!("Expected call expression in async");
        }
    } else {
        panic!("Expected async expression");
    }
}

#[test]
fn test_channel_expressions() {
    // Send operation: ch <- value
    let send_expr = ChannelExpr {
        direction: ChannelDirection::Send,
        channel: Box::new(AstBuilder::identifier("ch")),
        value: Some(Box::new(AstBuilder::literal_int(42))),
        position: dummy_pos(),
    };

    assert_eq!(send_expr.direction, ChannelDirection::Send);
    assert!(send_expr.value.is_some());

    // Receive operation: <-ch
    let recv_expr = ChannelExpr {
        direction: ChannelDirection::Receive,
        channel: Box::new(AstBuilder::identifier("ch")),
        value: None,
        position: dummy_pos(),
    };

    assert_eq!(recv_expr.direction, ChannelDirection::Receive);
    assert!(recv_expr.value.is_none());
}

#[test]
fn test_lambda_expression() {
    let lambda = LambdaExpr {
        params: vec![
            Parameter {
                name: "x".to_string(),
                param_type: Type::Int32,
                default_value: None,
                is_variadic: false,
                position: dummy_pos(),
            },
            Parameter {
                name: "y".to_string(),
                param_type: Type::Int32,
                default_value: None,
                is_variadic: false,
                position: dummy_pos(),
            },
        ],
        return_type: Some(Type::Int32),
        body: Box::new(AstBuilder::binary_expr(
            AstBuilder::identifier("x"),
            BinaryOperator::Add,
            AstBuilder::identifier("y"),
        )),
        captures: Vec::new(),
        position: dummy_pos(),
    };

    assert_eq!(lambda.params.len(), 2);
    assert_eq!(lambda.params[0].name, "x");
    assert_eq!(lambda.params[1].name, "y");
    assert_eq!(lambda.return_type, Some(Type::Int32));
}

#[test]
fn test_array_and_map_expressions() {
    // Array expression
    let array_expr = ArrayExpr {
        elements: vec![
            AstBuilder::literal_int(1),
            AstBuilder::literal_int(2),
            AstBuilder::literal_int(3),
        ],
        position: dummy_pos(),
    };

    assert_eq!(array_expr.elements.len(), 3);

    // Map expression
    let map_expr = MapExpr {
        entries: vec![
            MapEntry {
                key: AstBuilder::literal_string("name"),
                value: AstBuilder::literal_string("Alice"),
                position: dummy_pos(),
            },
            MapEntry {
                key: AstBuilder::literal_string("age"),
                value: AstBuilder::literal_int(30),
                position: dummy_pos(),
            },
        ],
        position: dummy_pos(),
    };

    assert_eq!(map_expr.entries.len(), 2);
}

#[test]
fn test_type_system() {
    // Test primitive types
    assert_eq!(Type::Int32, Type::Int32);
    assert_ne!(Type::Int32, Type::Int64);

    // Test array type
    let array_type = Type::Array(ArrayType {
        element_type: Box::new(Type::String),
        size: Some(10),
    });

    if let Type::Array(arr) = array_type {
        assert_eq!(*arr.element_type, Type::String);
        assert_eq!(arr.size, Some(10));
    } else {
        panic!("Expected array type");
    }

    // Test map type
    let map_type = Type::Map(MapType {
        key_type: Box::new(Type::String),
        value_type: Box::new(Type::Int32),
    });

    if let Type::Map(map) = map_type {
        assert_eq!(*map.key_type, Type::String);
        assert_eq!(*map.value_type, Type::Int32);
    } else {
        panic!("Expected map type");
    }

    // Test function type
    let func_type = Type::Function(FunctionType {
        param_types: vec![Type::Int32, Type::String],
        return_type: Some(Box::new(Type::Bool)),
        is_async: false,
    });

    if let Type::Function(func) = func_type {
        assert_eq!(func.param_types.len(), 2);
        assert_eq!(func.param_types[0], Type::Int32);
        assert_eq!(func.param_types[1], Type::String);
        assert_eq!(func.return_type, Some(Box::new(Type::Bool)));
        assert!(!func.is_async);
    } else {
        panic!("Expected function type");
    }
}

#[test]
fn test_patterns() {
    // Literal pattern
    let lit_pattern = Pattern::Literal(LiteralValue::Integer(42), dummy_pos());
    assert_eq!(lit_pattern.position(), dummy_pos());

    // Identifier pattern
    let id_pattern = Pattern::Identifier("x".to_string(), dummy_pos());
    if let Pattern::Identifier(name, _) = id_pattern {
        assert_eq!(name, "x");
    } else {
        panic!("Expected identifier pattern");
    }

    // Struct pattern
    let struct_pattern = Pattern::Struct(StructPattern {
        name: "Point".to_string(),
        fields: vec![
            FieldPattern {
                name: "x".to_string(),
                pattern: Box::new(Pattern::Identifier("px".to_string(), dummy_pos())),
                position: dummy_pos(),
            },
            FieldPattern {
                name: "y".to_string(),
                pattern: Box::new(Pattern::Identifier("py".to_string(), dummy_pos())),
                position: dummy_pos(),
            },
        ],
        position: dummy_pos(),
    });

    if let Pattern::Struct(s) = struct_pattern {
        assert_eq!(s.name, "Point");
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[0].name, "x");
        assert_eq!(s.fields[1].name, "y");
    } else {
        panic!("Expected struct pattern");
    }

    // Array pattern
    let array_pattern = Pattern::Array(ArrayPattern {
        elements: vec![
            Pattern::Identifier("first".to_string(), dummy_pos()),
            Pattern::Wildcard(dummy_pos()),
            Pattern::Identifier("last".to_string(), dummy_pos()),
        ],
        position: dummy_pos(),
    });

    if let Pattern::Array(arr) = array_pattern {
        assert_eq!(arr.elements.len(), 3);
    } else {
        panic!("Expected array pattern");
    }
}

#[test]
fn test_has_position_trait() {
    let stmt = AstBuilder::variable_decl("x", None, Some(AstBuilder::literal_int(42)));
    let pos = stmt.position();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);

    let expr = AstBuilder::binary_expr(
        AstBuilder::literal_int(1),
        BinaryOperator::Add,
        AstBuilder::literal_int(2),
    );
    let pos = expr.position();
    assert_eq!(pos.line, 1);
    assert_eq!(pos.column, 1);
}

#[test]
fn test_control_flow_statements() {
    // If statement
    let if_stmt = IfStmt {
        condition: AstBuilder::binary_expr(
            AstBuilder::identifier("x"),
            BinaryOperator::Greater,
            AstBuilder::literal_int(0),
        ),
        then_branch: AstBuilder::block_stmt(vec![AstBuilder::expression_stmt(
            AstBuilder::call_expr(
                AstBuilder::identifier("println"),
                vec![AstBuilder::literal_string("positive")],
            ),
        )]),
        else_branch: Some(Box::new(Statement::Block(AstBuilder::block_stmt(vec![
            AstBuilder::expression_stmt(AstBuilder::call_expr(
                AstBuilder::identifier("println"),
                vec![AstBuilder::literal_string("not positive")],
            )),
        ])))),
        position: dummy_pos(),
    };

    assert!(if_stmt.else_branch.is_some());

    // While statement
    let while_stmt = WhileStmt {
        condition: AstBuilder::binary_expr(
            AstBuilder::identifier("i"),
            BinaryOperator::Less,
            AstBuilder::literal_int(10),
        ),
        body: AstBuilder::block_stmt(vec![AstBuilder::expression_stmt(AstBuilder::assignment(
            AstBuilder::identifier("i"),
            AstBuilder::binary_expr(
                AstBuilder::identifier("i"),
                BinaryOperator::Add,
                AstBuilder::literal_int(1),
            ),
        ))]),
        position: dummy_pos(),
    };

    if let Expression::Binary(cond) = while_stmt.condition {
        assert_eq!(cond.operator, BinaryOperator::Less);
    } else {
        panic!("Expected binary condition");
    }

    // For statement
    let for_stmt = ForStmt {
        variable: "item".to_string(),
        index_variable: None,
        iterable: AstBuilder::identifier("items"),
        body: AstBuilder::block_stmt(vec![AstBuilder::expression_stmt(AstBuilder::call_expr(
            AstBuilder::identifier("process"),
            vec![AstBuilder::identifier("item")],
        ))]),
        position: dummy_pos(),
    };

    assert_eq!(for_stmt.variable, "item");
}

#[test]
fn test_error_handling_constructs() {
    // Try statement
    let try_stmt = TryStmt {
        body: AstBuilder::block_stmt(vec![AstBuilder::expression_stmt(AstBuilder::call_expr(
            AstBuilder::identifier("risky_operation"),
            vec![],
        ))]),
        catch_clause: Some(CatchClause {
            error_var: Some("e".to_string()),
            body: AstBuilder::block_stmt(vec![AstBuilder::expression_stmt(AstBuilder::call_expr(
                AstBuilder::identifier("handle_error"),
                vec![AstBuilder::identifier("e")],
            ))]),
            position: dummy_pos(),
        }),
        position: dummy_pos(),
    };

    assert!(try_stmt.catch_clause.is_some());
    if let Some(catch) = try_stmt.catch_clause {
        assert_eq!(catch.error_var, Some("e".to_string()));
    }

    // Fail statement
    let fail_stmt = FailStmt {
        message: AstBuilder::literal_string("Something went wrong"),
        position: dummy_pos(),
    };

    if let Expression::Literal(lit) = fail_stmt.message {
        if let LiteralValue::String(msg) = lit.value {
            assert_eq!(msg, "Something went wrong");
        } else {
            panic!("Expected string literal");
        }
    } else {
        panic!("Expected literal expression");
    }

    // Defer statement
    let defer_stmt = DeferStmt {
        stmt: Box::new(AstBuilder::expression_stmt(AstBuilder::call_expr(
            AstBuilder::identifier("cleanup"),
            vec![],
        ))),
        position: dummy_pos(),
    };

    if let Statement::Expression(expr_stmt) = &*defer_stmt.stmt {
        if let Expression::Call(call) = &expr_stmt.expr {
            if let Expression::Identifier(ident) = &*call.callee {
                assert_eq!(ident.name, "cleanup");
            } else {
                panic!("Expected identifier in defer call");
            }
        } else {
            panic!("Expected call expression in defer");
        }
    } else {
        panic!("Expected expression statement in defer");
    }
}
