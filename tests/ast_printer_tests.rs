use bulu::ast::*;

#[test]
fn test_ast_printer_simple_program() {
    let program = AstBuilder::program(vec![
        AstBuilder::variable_decl(
            "x",
            Some(AstBuilder::int32_type()),
            Some(AstBuilder::literal_int(42)),
        ),
        AstBuilder::expression_stmt(AstBuilder::call_expr(
            AstBuilder::identifier("println"),
            vec![AstBuilder::literal_string("Hello, World!")],
        )),
    ]);

    let mut printer = AstPrinter::new();
    let output = printer.print_program(&program);

    assert!(output.contains("Program {"));
    assert!(output.contains("Let x: int32 = 42"));
    assert!(output.contains("Ident(println)(\"Hello, World!\")"));
}

#[test]
fn test_ast_printer_function_declaration() {
    let func_decl = AstBuilder::function_decl(
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
    );

    let mut printer = AstPrinter::new();
    let output = printer.print_statement(&func_decl);

    assert!(output.contains("Func add"));
    assert!(output.contains("a: int32"));
    assert!(output.contains("b: int32"));
    assert!(output.contains(": int32"));
    assert!(output.contains("Return"));
}

#[test]
fn test_ast_printer_binary_expressions() {
    let expr = AstBuilder::binary_expr(
        AstBuilder::binary_expr(
            AstBuilder::literal_int(2),
            BinaryOperator::Add,
            AstBuilder::literal_int(3),
        ),
        BinaryOperator::Multiply,
        AstBuilder::literal_int(4),
    );

    let mut printer = AstPrinter::new();
    let output = printer.print_expression(&expr);

    assert!(output.contains("(2 + 3) * 4"));
}

#[test]
fn test_ast_printer_struct_declaration() {
    let struct_decl = Statement::StructDecl(StructDecl {
        name: "Point".to_string(),
        type_params: vec![],
        fields: vec![
            StructField {
                name: "x".to_string(),
                field_type: Type::Float64,
                position: AstBuilder::dummy_pos(),
                is_private: false
            },
            StructField {
                name: "y".to_string(),
                field_type: Type::Float64,
                position: AstBuilder::dummy_pos(),
                is_private: false
            },
        ],
        methods: vec![],
        doc_comment: None,
        is_exported: false,
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let output = printer.print_statement(&struct_decl);

    assert!(output.contains("Struct Point"));
    assert!(output.contains("x: float64"));
    assert!(output.contains("y: float64"));
}

#[test]
fn test_ast_printer_match_expression() {
    let match_expr = Expression::Match(MatchExpr {
        expr: Box::new(AstBuilder::identifier("x")),
        arms: vec![
            MatchExprArm {
                pattern: Pattern::Literal(LiteralValue::Integer(1), AstBuilder::dummy_pos()),
                guard: None,
                expr: AstBuilder::literal_string("one"),
                position: AstBuilder::dummy_pos(),
            },
            MatchExprArm {
                pattern: Pattern::Wildcard(AstBuilder::dummy_pos()),
                guard: None,
                expr: AstBuilder::literal_string("other"),
                position: AstBuilder::dummy_pos(),
            },
        ],
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let output = printer.print_expression(&match_expr);

    assert!(output.contains("match Ident(x)"));
    assert!(output.contains("1 => \"one\""));
    assert!(output.contains("_ => \"other\""));
}

#[test]
fn test_ast_printer_array_and_map() {
    let array_expr = AstBuilder::array_expr(vec![
        AstBuilder::literal_int(1),
        AstBuilder::literal_int(2),
        AstBuilder::literal_int(3),
    ]);

    let mut printer = AstPrinter::new();
    let array_output = printer.print_expression(&array_expr);
    assert!(array_output.contains("[1, 2, 3]"));

    let map_expr = Expression::Map(MapExpr {
        entries: vec![
            MapEntry {
                key: AstBuilder::literal_string("name"),
                value: AstBuilder::literal_string("Alice"),
                position: AstBuilder::dummy_pos(),
            },
            MapEntry {
                key: AstBuilder::literal_string("age"),
                value: AstBuilder::literal_int(30),
                position: AstBuilder::dummy_pos(),
            },
        ],
        position: AstBuilder::dummy_pos(),
    });

    let map_output = printer.print_expression(&map_expr);
    assert!(map_output.contains("\"name\": \"Alice\""));
    assert!(map_output.contains("\"age\": 30"));
}

#[test]
fn test_ast_printer_lambda_expression() {
    let lambda = Expression::Lambda(LambdaExpr {
        params: vec![
            AstBuilder::parameter("x", AstBuilder::int32_type()),
            AstBuilder::parameter("y", AstBuilder::int32_type()),
        ],
        return_type: Some(AstBuilder::int32_type()),
        body: Box::new(AstBuilder::binary_expr(
            AstBuilder::identifier("x"),
            BinaryOperator::Add,
            AstBuilder::identifier("y"),
        )),
        captures: Vec::new(),
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let output = printer.print_expression(&lambda);

    assert!(output.contains("(x: int32, y: int32) => (Ident(x) + Ident(y))"));
}

#[test]
fn test_ast_printer_async_await() {
    let async_expr = Expression::Async(AsyncExpr {
        expr: Box::new(AstBuilder::call_expr(
            AstBuilder::identifier("fetch"),
            vec![AstBuilder::literal_string("url")],
        )),
        position: AstBuilder::dummy_pos(),
    });

    let await_expr = Expression::Await(AwaitExpr {
        expr: Box::new(async_expr),
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let output = printer.print_expression(&await_expr);

    assert!(output.contains("Await(Async(Ident(fetch)(\"url\")))"));
}

#[test]
fn test_ast_printer_channel_operations() {
    // Send operation
    let send_expr = Expression::Channel(ChannelExpr {
        direction: ChannelDirection::Send,
        channel: Box::new(AstBuilder::identifier("ch")),
        value: Some(Box::new(AstBuilder::literal_int(42))),
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let send_output = printer.print_expression(&send_expr);
    assert!(send_output.contains("Ident(ch) <- 42"));

    // Receive operation
    let recv_expr = Expression::Channel(ChannelExpr {
        direction: ChannelDirection::Receive,
        channel: Box::new(AstBuilder::identifier("ch")),
        value: None,
        position: AstBuilder::dummy_pos(),
    });

    let recv_output = printer.print_expression(&recv_expr);
    assert!(recv_output.contains("<-Ident(ch)"));
}

#[test]
fn test_ast_printer_type_casting() {
    let cast_expr = Expression::Cast(CastExpr {
        expr: Box::new(AstBuilder::literal_int(42)),
        target_type: Type::Float64,
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let output = printer.print_expression(&cast_expr);

    assert!(output.contains("Cast(42 as float64)"));
}

#[test]
fn test_ast_printer_range_expressions() {
    let inclusive_range = Expression::Range(RangeExpr {
        start: Box::new(AstBuilder::literal_int(1)),
        end: Box::new(AstBuilder::literal_int(10)),
        step: None,
        inclusive: true,
        position: AstBuilder::dummy_pos(),
    });

    let exclusive_range = Expression::Range(RangeExpr {
        start: Box::new(AstBuilder::literal_int(1)),
        end: Box::new(AstBuilder::literal_int(10)),
        step: None,
        inclusive: false,
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();

    let inclusive_output = printer.print_expression(&inclusive_range);
    assert!(inclusive_output.contains("1...10"));

    let exclusive_output = printer.print_expression(&exclusive_range);
    assert!(exclusive_output.contains("1..<10"));
}

#[test]
fn test_ast_printer_control_flow() {
    let if_stmt = Statement::If(IfStmt {
        condition: AstBuilder::binary_expr(
            AstBuilder::identifier("x"),
            BinaryOperator::Greater,
            AstBuilder::literal_int(0),
        ),
        then_branch: AstBuilder::block_stmt(vec![AstBuilder::expression_stmt(
            AstBuilder::literal_string("positive"),
        )]),
        else_branch: Some(Box::new(Statement::Block(AstBuilder::block_stmt(vec![
            AstBuilder::expression_stmt(AstBuilder::literal_string("not positive")),
        ])))),
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let output = printer.print_statement(&if_stmt);

    assert!(output.contains("If ((Ident(x) > 0))"));
    assert!(output.contains("Else"));
}

#[test]
fn test_ast_printer_patterns() {
    let struct_pattern = Pattern::Struct(StructPattern {
        name: "Point".to_string(),
        fields: vec![
            FieldPattern {
                name: "x".to_string(),
                pattern: Box::new(Pattern::Identifier(
                    "px".to_string(),
                    AstBuilder::dummy_pos(),
                )),
                position: AstBuilder::dummy_pos(),
            },
            FieldPattern {
                name: "y".to_string(),
                pattern: Box::new(Pattern::Wildcard(AstBuilder::dummy_pos())),
                position: AstBuilder::dummy_pos(),
            },
        ],
        position: AstBuilder::dummy_pos(),
    });

    let mut printer = AstPrinter::new();
    let output = printer.print_pattern(&struct_pattern);

    assert!(output.contains("Point {x: px, y: _}"));
}

#[test]
fn test_ast_printer_complex_types() {
    // Array type
    let array_type = Type::Array(ArrayType {
        element_type: Box::new(Type::String),
        size: Some(10),
    });

    let mut printer = AstPrinter::new();
    let array_output = printer.print_type(&array_type);
    assert!(array_output.contains("[string; 10]"));

    // Map type
    let map_type = Type::Map(MapType {
        key_type: Box::new(Type::String),
        value_type: Box::new(Type::Int32),
    });

    let map_output = printer.print_type(&map_type);
    assert!(map_output.contains("map[string, int32]"));

    // Function type
    let func_type = Type::Function(FunctionType {
        param_types: vec![Type::Int32, Type::String],
        return_type: Some(Box::new(Type::Bool)),
        is_async: false,
    });

    let func_output = printer.print_type(&func_type);
    assert!(func_output.contains("func(int32, string): bool"));

    // Channel type
    let chan_type = Type::Channel(ChannelType {
        element_type: Box::new(Type::Int32),
        direction: ChannelDirection::Send,
    });

    let chan_output = printer.print_type(&chan_type);
    assert!(chan_output.contains("chan<- int32"));
}
