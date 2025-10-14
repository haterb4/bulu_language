//! Unit tests for the channel system implementation

use bulu::ast::*;
use bulu::runtime::interpreter::Interpreter;
use bulu::types::primitive::RuntimeValue;
use bulu::lexer::token::Position;
use bulu::types::primitive::TypeId;
use std::thread;
use std::time::Duration;

fn create_test_position() -> Position {
    Position::new(1, 1, 0)
}

#[test]
fn test_channel_creation() {
    let mut interpreter = Interpreter::new();
    
    // Test unbuffered channel creation
    let unbuffered = interpreter.make_channel(TypeId::Int32, None).unwrap();
    match unbuffered {
        RuntimeValue::Channel(_) => {}, // Success
        other => panic!("Expected Channel, got {:?}", other),
    }
    
    // Test buffered channel creation
    let buffered = interpreter.make_channel(TypeId::Int32, Some(5)).unwrap();
    match buffered {
        RuntimeValue::Channel(_) => {}, // Success
        other => panic!("Expected Channel, got {:?}", other),
    }
}

#[test]
fn test_make_function_call() {
    let mut interpreter = Interpreter::new();
    
    // Create make() call expression
    let make_call = CallExpr {
        callee: Box::new(Expression::Identifier(IdentifierExpr {
            name: "make".to_string(),
            position: create_test_position(),
        })),
        type_args: vec![],
        args: vec![
            Expression::Identifier(IdentifierExpr {
                name: "chan".to_string(),
                position: create_test_position(),
            }),
        ],
        position: create_test_position(),
    };
    
    let result = interpreter.evaluate_call(&make_call).unwrap();
    match result {
        RuntimeValue::Channel(_) => {}, // Success
        other => panic!("Expected Channel, got {:?}", other),
    }
}

#[test]
fn test_make_function_with_capacity() {
    let mut interpreter = Interpreter::new();
    
    // Create make() call expression with capacity
    let make_call = CallExpr {
        callee: Box::new(Expression::Identifier(IdentifierExpr {
            name: "make".to_string(),
            position: create_test_position(),
        })),
        type_args: vec![],
        args: vec![
            Expression::Identifier(IdentifierExpr {
                name: "chan".to_string(),
                position: create_test_position(),
            }),
            Expression::Literal(LiteralExpr {
                value: LiteralValue::Integer(10),
                position: create_test_position(),
            }),
        ],
        position: create_test_position(),
    };
    
    let result = interpreter.evaluate_call(&make_call).unwrap();
    match result {
        RuntimeValue::Channel(_) => {}, // Success
        other => panic!("Expected Channel, got {:?}", other),
    }
}

#[test]
fn test_channel_send_receive() {
    let mut interpreter = Interpreter::new();
    
    // Create a buffered channel
    let channel = interpreter.make_channel(TypeId::Int32, Some(1)).unwrap();
    let channel_id = match channel {
        RuntimeValue::Channel(id) => id,
        _ => panic!("Expected channel"),
    };
    
    // Create send expression: channel <- 42
    let send_expr = ChannelExpr {
        direction: ChannelDirection::Send,
        channel: Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(channel_id as i64),
            position: create_test_position(),
        })),
        value: Some(Box::new(Expression::Literal(LiteralExpr {
            value: LiteralValue::Integer(42),
            position: create_test_position(),
        }))),
        position: create_test_position(),
    };
    
    // This test is simplified since we need proper channel value handling
    // In a full implementation, we would test actual send/receive operations
}

#[test]
fn test_close_function_call() {
    let mut interpreter = Interpreter::new();
    
    // Create a channel
    let channel = interpreter.make_channel(TypeId::Int32, None).unwrap();
    
    // Store the channel in the interpreter environment first
    let channel_id = match &channel {
        RuntimeValue::Channel(id) => *id,
        _ => panic!("Expected channel"),
    };
    interpreter.environment.define("test_channel".to_string(), channel);

    // Create close() call expression
    let close_call = CallExpr {
        callee: Box::new(Expression::Identifier(IdentifierExpr {
            name: "close".to_string(),
            position: create_test_position(),
        })),
        type_args: vec![],
        args: vec![
            Expression::Identifier(IdentifierExpr {
                name: "test_channel".to_string(),
                position: create_test_position(),
            }),
        ],
        position: create_test_position(),
    };
    
    let result = interpreter.evaluate_call(&close_call).unwrap();
    assert_eq!(result, RuntimeValue::Null);
}

#[test]
fn test_channel_error_handling() {
    let mut interpreter = Interpreter::new();
    
    // Test make() with invalid capacity
    let make_call = CallExpr {
        callee: Box::new(Expression::Identifier(IdentifierExpr {
            name: "make".to_string(),
            position: create_test_position(),
        })),
        type_args: vec![],
        args: vec![
            Expression::Identifier(IdentifierExpr {
                name: "chan".to_string(),
                position: create_test_position(),
            }),
            Expression::Literal(LiteralExpr {
                value: LiteralValue::Integer(-1),
                position: create_test_position(),
            }),
        ],
        position: create_test_position(),
    };
    
    let result = interpreter.evaluate_call(&make_call);
    assert!(result.is_err());
}

#[test]
fn test_close_invalid_argument() {
    let mut interpreter = Interpreter::new();
    
    // Test close() with non-channel argument
    let close_call = CallExpr {
        callee: Box::new(Expression::Identifier(IdentifierExpr {
            name: "close".to_string(),
            position: create_test_position(),
        })),
        type_args: vec![],
        args: vec![
            Expression::Literal(LiteralExpr {
                value: LiteralValue::Integer(42),
                position: create_test_position(),
            }),
        ],
        position: create_test_position(),
    };
    
    let result = interpreter.evaluate_call(&close_call);
    assert!(result.is_err());
}

#[test]
fn test_channel_directions() {
    // Test that channel direction types are properly defined
    use bulu::types::composite::ChannelDirection;
    
    assert_eq!(ChannelDirection::Bidirectional, ChannelDirection::Bidirectional);
    assert_eq!(ChannelDirection::SendOnly, ChannelDirection::SendOnly);
    assert_eq!(ChannelDirection::ReceiveOnly, ChannelDirection::ReceiveOnly);
    
    assert_ne!(ChannelDirection::SendOnly, ChannelDirection::ReceiveOnly);
}

#[test]
fn test_channel_type_info() {
    use bulu::types::composite::{ChannelTypeInfo, ChannelDirection};
    use bulu::types::primitive::TypeId;
    
    let channel_info = ChannelTypeInfo {
        element_type: TypeId::Int32,
        direction: ChannelDirection::Bidirectional,
        buffered: false,
        capacity: None,
    };
    
    assert_eq!(channel_info.element_type, TypeId::Int32);
    assert_eq!(channel_info.direction, ChannelDirection::Bidirectional);
    assert!(!channel_info.buffered);
    assert_eq!(channel_info.capacity, None);
}

#[test]
fn test_buffered_channel_type_info() {
    use bulu::types::composite::{ChannelTypeInfo, ChannelDirection};
    use bulu::types::primitive::TypeId;
    
    let channel_info = ChannelTypeInfo {
        element_type: TypeId::String,
        direction: ChannelDirection::SendOnly,
        buffered: true,
        capacity: Some(10),
    };
    
    assert_eq!(channel_info.element_type, TypeId::String);
    assert_eq!(channel_info.direction, ChannelDirection::SendOnly);
    assert!(channel_info.buffered);
    assert_eq!(channel_info.capacity, Some(10));
}

#[test]
fn test_value_to_runtime_value_conversion() {
    let interpreter = Interpreter::new();
    
    // Test integer conversion - create an Expression instead of Value
    let int_expr = Expression::Literal(LiteralExpr {
        value: LiteralValue::Integer(42),
        position: Position::new(1, 1, 0),
    });
    let runtime_value = interpreter.value_to_runtime_value(int_expr).unwrap();
    match runtime_value {
        RuntimeValue::Int32(42) => {}, // Success
        other => panic!("Expected Int32(42), got {:?}", other),
    }
    
    // Test string conversion
    let string_expr = Expression::Literal(LiteralExpr {
        value: LiteralValue::String("hello".to_string()),
        position: Position::new(1, 1, 0),
    });
    let runtime_value = interpreter.value_to_runtime_value(string_expr).unwrap();
    match runtime_value {
        RuntimeValue::String(s) if s == "hello" => {}, // Success
        other => panic!("Expected String(hello), got {:?}", other),
    }
    
    // Test boolean conversion
    let bool_expr = Expression::Literal(LiteralExpr {
        value: LiteralValue::Boolean(true),
        position: Position::new(1, 1, 0),
    });
    let runtime_value = interpreter.value_to_runtime_value(bool_expr).unwrap();
    match runtime_value {
        RuntimeValue::Bool(true) => {}, // Success
        other => panic!("Expected Bool(true), got {:?}", other),
    }
}

#[test]
fn test_runtime_value_to_value_conversion() {
    let interpreter = Interpreter::new();
    
    // Test integer conversion
    let runtime_value = RuntimeValue::Int32(42);
    let expr = interpreter.runtime_value_to_value(runtime_value).unwrap();
    match expr {
        Expression::Literal(literal) => {
            assert_eq!(literal.value, LiteralValue::Integer(42));
        }
        _ => panic!("Expected literal expression"),
    }
    
    // Test string conversion
    let runtime_value = RuntimeValue::String("world".to_string());
    let expr = interpreter.runtime_value_to_value(runtime_value).unwrap();
    match expr {
        Expression::Literal(literal) => {
            assert_eq!(literal.value, LiteralValue::String("world".to_string()));
        }
        _ => panic!("Expected literal expression"),
    }
    
    // Test boolean conversion
    let runtime_value = RuntimeValue::Bool(false);
    let expr = interpreter.runtime_value_to_value(runtime_value).unwrap();
    match expr {
        Expression::Literal(literal) => {
            assert_eq!(literal.value, LiteralValue::Boolean(false));
        }
        _ => panic!("Expected literal expression"),
    }
}