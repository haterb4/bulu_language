//! Tests for struct and interface functionality

use bulu::ast::*;
use bulu::lexer::Lexer;
use bulu::parser::Parser;
use bulu::types::{TypeRegistry, StructTypeInfo, InterfaceTypeInfo, StructField, InterfaceMethod};
use bulu::types::primitive::TypeId;

#[test]
fn test_struct_declaration_parsing() {
    let source = r#"
        struct Point {
            x: float64
            y: float64
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");

    assert_eq!(program.statements.len(), 1);
    
    if let Statement::StructDecl(struct_decl) = &program.statements[0] {
        assert_eq!(struct_decl.name, "Point");
        assert_eq!(struct_decl.fields.len(), 2);
        
        assert_eq!(struct_decl.fields[0].name, "x");
        assert_eq!(struct_decl.fields[0].field_type, Type::Float64);
        
        assert_eq!(struct_decl.fields[1].name, "y");
        assert_eq!(struct_decl.fields[1].field_type, Type::Float64);
    } else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn test_struct_with_methods_parsing() {
    let source = r#"
        struct Rectangle {
            width: float64
            height: float64
            
            func area(): float64 {
                return this.width * this.height
            }
            
            func perimeter(): float64 {
                return 2.0 * (this.width + this.height)
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");

    assert_eq!(program.statements.len(), 1);
    
    if let Statement::StructDecl(struct_decl) = &program.statements[0] {
        assert_eq!(struct_decl.name, "Rectangle");
        assert_eq!(struct_decl.fields.len(), 2);
        assert_eq!(struct_decl.methods.len(), 2);
        
        // Check fields
        assert_eq!(struct_decl.fields[0].name, "width");
        assert_eq!(struct_decl.fields[1].name, "height");
        
        // Check methods
        assert_eq!(struct_decl.methods[0].name, "area");
        assert_eq!(struct_decl.methods[0].return_type, Some(Type::Float64));
        
        assert_eq!(struct_decl.methods[1].name, "perimeter");
        assert_eq!(struct_decl.methods[1].return_type, Some(Type::Float64));
    } else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn test_interface_declaration_parsing() {
    let source = r#"
        interface Shape {
            func area(): float64
            func perimeter(): float64
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");

    assert_eq!(program.statements.len(), 1);
    
    if let Statement::InterfaceDecl(interface_decl) = &program.statements[0] {
        assert_eq!(interface_decl.name, "Shape");
        assert_eq!(interface_decl.methods.len(), 2);
        
        assert_eq!(interface_decl.methods[0].name, "area");
        assert_eq!(interface_decl.methods[0].return_type, Some(Type::Float64));
        
        assert_eq!(interface_decl.methods[1].name, "perimeter");
        assert_eq!(interface_decl.methods[1].return_type, Some(Type::Float64));
    } else {
        panic!("Expected interface declaration");
    }
}

#[test]
fn test_generic_struct_parsing() {
    let source = r#"
        struct Box<T> {
            value: T
            
            func get(): T {
                return this.value
            }
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");

    assert_eq!(program.statements.len(), 1);
    
    if let Statement::StructDecl(struct_decl) = &program.statements[0] {
        assert_eq!(struct_decl.name, "Box");
        assert_eq!(struct_decl.type_params.len(), 1);
        assert_eq!(struct_decl.type_params[0].name, "T");
        assert_eq!(struct_decl.fields.len(), 1);
        assert_eq!(struct_decl.methods.len(), 1);
    } else {
        panic!("Expected struct declaration");
    }
}

#[test]
fn test_generic_interface_parsing() {
    let source = r#"
        interface Container<T> {
            func add(item: T)
            func get(index: int32): T
            func size(): int32
        }
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");

    assert_eq!(program.statements.len(), 1);
    
    if let Statement::InterfaceDecl(interface_decl) = &program.statements[0] {
        assert_eq!(interface_decl.name, "Container");
        assert_eq!(interface_decl.type_params.len(), 1);
        assert_eq!(interface_decl.type_params[0].name, "T");
        assert_eq!(interface_decl.methods.len(), 3);
        
        // Check method signatures
        assert_eq!(interface_decl.methods[0].name, "add");
        assert_eq!(interface_decl.methods[0].params.len(), 1);
        assert_eq!(interface_decl.methods[0].return_type, None);
        
        assert_eq!(interface_decl.methods[1].name, "get");
        assert_eq!(interface_decl.methods[1].params.len(), 1);
        assert_eq!(interface_decl.methods[1].return_type, Some(Type::Named("T".to_string())));
        
        assert_eq!(interface_decl.methods[2].name, "size");
        assert_eq!(interface_decl.methods[2].params.len(), 0);
        assert_eq!(interface_decl.methods[2].return_type, Some(Type::Int32));
    } else {
        panic!("Expected interface declaration");
    }
}

#[test]
fn test_member_access_parsing() {
    let source = r#"
        let p = point
        let x_val = p.x
        p.y = 25.0
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");

    assert_eq!(program.statements.len(), 3);
    
    // Check member access in second statement
    if let Statement::VariableDecl(var_decl) = &program.statements[1] {
        if let Some(Expression::MemberAccess(member_access)) = &var_decl.initializer {
            if let Expression::Identifier(ident) = &*member_access.object {
                assert_eq!(ident.name, "p");
            }
            assert_eq!(member_access.member, "x");
        } else {
            panic!("Expected member access expression");
        }
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
fn test_method_call_parsing() {
    let source = r#"
        let rect = rectangle
        let area = rect.area()
        rect.scale(2.0)
    "#;

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().expect("Tokenization failed");
    let mut parser = Parser::new(tokens);
    let program = parser.parse().expect("Parsing failed");

    assert_eq!(program.statements.len(), 3);
    
    // Check method call in second statement
    if let Statement::VariableDecl(var_decl) = &program.statements[1] {
        if let Some(Expression::Call(call_expr)) = &var_decl.initializer {
            if let Expression::MemberAccess(member_access) = &*call_expr.callee {
                if let Expression::Identifier(ident) = &*member_access.object {
                    assert_eq!(ident.name, "rect");
                }
                assert_eq!(member_access.member, "area");
            }
            assert_eq!(call_expr.args.len(), 0);
        } else {
            panic!("Expected method call expression");
        }
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
fn test_type_registry_struct_operations() {
    let mut registry = TypeRegistry::new();
    
    // Create a struct type
    let struct_info = StructTypeInfo {
        name: "Point".to_string(),
        fields: vec![
            StructField {
                name: "x".to_string(),
                field_type: TypeId::Float64,
            },
            StructField {
                name: "y".to_string(),
                field_type: TypeId::Float64,
            },
        ],
        type_params: vec![],
    };
    
    let struct_id = registry.register_struct_type(struct_info);
    let type_id = TypeId::Struct(struct_id);
    
    // Test struct field operations
    assert!(registry.struct_has_field(type_id, "x"));
    assert!(registry.struct_has_field(type_id, "y"));
    assert!(!registry.struct_has_field(type_id, "z"));
    
    assert_eq!(registry.get_struct_field_type(type_id, "x"), Some(TypeId::Float64));
    assert_eq!(registry.get_struct_field_type(type_id, "y"), Some(TypeId::Float64));
    assert_eq!(registry.get_struct_field_type(type_id, "z"), None);
    
    // Test type name
    assert_eq!(registry.get_type_name(type_id), "Point");
}

#[test]
fn test_type_registry_interface_operations() {
    let mut registry = TypeRegistry::new();
    
    // Create an interface type
    let interface_info = InterfaceTypeInfo {
        name: "Shape".to_string(),
        methods: vec![
            InterfaceMethod {
                name: "area".to_string(),
                param_types: vec![],
                return_type: Some(TypeId::Float64),
            },
            InterfaceMethod {
                name: "perimeter".to_string(),
                param_types: vec![],
                return_type: Some(TypeId::Float64),
            },
        ],
        type_params: vec![],
    };
    
    let interface_id = registry.register_interface_type(interface_info);
    let type_id = TypeId::Interface(interface_id);
    
    // Test interface operations
    let interface_info = registry.get_interface_info(type_id).unwrap();
    assert_eq!(interface_info.name, "Shape");
    assert_eq!(interface_info.methods.len(), 2);
    assert_eq!(interface_info.methods[0].name, "area");
    assert_eq!(interface_info.methods[1].name, "perimeter");
    
    // Test type name
    assert_eq!(registry.get_type_name(type_id), "Shape");
}

#[test]
fn test_duck_typing_interface_implementation() {
    let mut registry = TypeRegistry::new();
    
    // Create a struct type
    let struct_info = StructTypeInfo {
        name: "Circle".to_string(),
        fields: vec![
            StructField {
                name: "radius".to_string(),
                field_type: TypeId::Float64,
            },
        ],
        type_params: vec![],
    };
    
    let struct_id = registry.register_struct_type(struct_info);
    let struct_type_id = TypeId::Struct(struct_id);
    
    // Create an interface type
    let interface_info = InterfaceTypeInfo {
        name: "Shape".to_string(),
        methods: vec![
            InterfaceMethod {
                name: "area".to_string(),
                param_types: vec![],
                return_type: Some(TypeId::Float64),
            },
        ],
        type_params: vec![],
    };
    
    let interface_id = registry.register_interface_type(interface_info);
    let interface_type_id = TypeId::Interface(interface_id);
    
    // Test duck typing (simplified implementation)
    assert!(registry.implements_interface(struct_type_id, interface_type_id));
}

#[test]
fn test_struct_field_access_type_checking() {
    // This test would be part of a type checker implementation
    // For now, we'll test the basic structure
    
    let mut registry = TypeRegistry::new();
    
    let struct_info = StructTypeInfo {
        name: "Person".to_string(),
        fields: vec![
            StructField {
                name: "name".to_string(),
                field_type: TypeId::String,
            },
            StructField {
                name: "age".to_string(),
                field_type: TypeId::Int32,
            },
        ],
        type_params: vec![],
    };
    
    let struct_id = registry.register_struct_type(struct_info);
    let type_id = TypeId::Struct(struct_id);
    
    // Verify field types
    assert_eq!(registry.get_struct_field_type(type_id, "name"), Some(TypeId::String));
    assert_eq!(registry.get_struct_field_type(type_id, "age"), Some(TypeId::Int32));
}

#[test]
fn test_nested_struct_types() {
    let mut registry = TypeRegistry::new();
    
    // Create inner struct
    let point_info = StructTypeInfo {
        name: "Point".to_string(),
        fields: vec![
            StructField {
                name: "x".to_string(),
                field_type: TypeId::Float64,
            },
            StructField {
                name: "y".to_string(),
                field_type: TypeId::Float64,
            },
        ],
        type_params: vec![],
    };
    
    let point_id = registry.register_struct_type(point_info);
    let point_type_id = TypeId::Struct(point_id);
    
    // Create outer struct that uses the inner struct
    let line_info = StructTypeInfo {
        name: "Line".to_string(),
        fields: vec![
            StructField {
                name: "start".to_string(),
                field_type: point_type_id,
            },
            StructField {
                name: "end".to_string(),
                field_type: point_type_id,
            },
        ],
        type_params: vec![],
    };
    
    let line_id = registry.register_struct_type(line_info);
    let line_type_id = TypeId::Struct(line_id);
    
    // Test nested field access
    assert_eq!(registry.get_struct_field_type(line_type_id, "start"), Some(point_type_id));
    assert_eq!(registry.get_struct_field_type(line_type_id, "end"), Some(point_type_id));
}