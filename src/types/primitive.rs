//! Primitive types implementation for the Bulu language

use crate::ast::{LiteralValue, Type};
use crate::error::{BuluError, Result};
use std::collections::HashMap;
use std::fmt;

/// Type identifier for the type system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeId {
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

    // Composite types (using u32 as placeholder for element type)
    Array(u32),
    Slice(u32),
    Map(u32),      // composite type ID
    Function(u32), // placeholder for function signature

    // User-defined types
    Struct(u32),    // struct type ID
    Interface(u32), // interface type ID

    // Channel types
    Channel(u32), // channel type ID

    // Async types
    Promise(u32), // promise type ID

    // Result types
    Result(u32), // result type ID

    // Tuple types
    Tuple(u32), // tuple type ID

    // Special types
    Unknown,
}

/// Type information with full type details
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    Primitive(PrimitiveType),
    Array(Box<TypeInfo>),
    Slice(Box<TypeInfo>),
    Map(Box<TypeInfo>, Box<TypeInfo>),
    Function(Vec<TypeInfo>, Option<Box<TypeInfo>>), // params, return type
    Struct(String, Vec<TypeInfo>),                  // name, type args
    Interface(String, Vec<TypeInfo>),               // name, type args
    Unknown,
}

/// Primitive types in the Bulu language
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimitiveType {
    // Signed integers
    Int8,
    Int16,
    Int32,
    Int64,

    // Unsigned integers
    UInt8,
    UInt16,
    UInt32,
    UInt64,

    // Floating point
    Float32,
    Float64,

    // Other primitives
    Bool,
    Char,
    String,

    // Special types
    Any,
    Void, // For functions that don't return a value
}

impl PrimitiveType {
    /// Get the size in bytes of this primitive type
    pub fn size_bytes(&self) -> usize {
        match self {
            PrimitiveType::Int8 | PrimitiveType::UInt8 => 1,
            PrimitiveType::Int16 | PrimitiveType::UInt16 => 2,
            PrimitiveType::Int32 | PrimitiveType::UInt32 | PrimitiveType::Float32 => 4,
            PrimitiveType::Int64 | PrimitiveType::UInt64 | PrimitiveType::Float64 => 8,
            PrimitiveType::Bool => 1,
            PrimitiveType::Char => 4, // UTF-8 character
            PrimitiveType::String => std::mem::size_of::<String>(), // Pointer size
            PrimitiveType::Any => std::mem::size_of::<RuntimeValue>(), // Runtime value wrapper
            PrimitiveType::Void => 0, // Void has no size
        }
    }

    /// Check if this type is a signed integer
    pub fn is_signed_integer(&self) -> bool {
        matches!(
            self,
            PrimitiveType::Int8
                | PrimitiveType::Int16
                | PrimitiveType::Int32
                | PrimitiveType::Int64
        )
    }

    /// Check if this type is an unsigned integer
    pub fn is_unsigned_integer(&self) -> bool {
        matches!(
            self,
            PrimitiveType::UInt8
                | PrimitiveType::UInt16
                | PrimitiveType::UInt32
                | PrimitiveType::UInt64
        )
    }

    /// Check if this type is any integer type
    pub fn is_integer(&self) -> bool {
        self.is_signed_integer() || self.is_unsigned_integer()
    }

    /// Check if this type is a floating point type
    pub fn is_float(&self) -> bool {
        matches!(self, PrimitiveType::Float32 | PrimitiveType::Float64)
    }

    /// Check if this type is numeric (integer or float)
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float()
    }

    /// Check if this type can be implicitly converted to another type
    pub fn can_implicitly_convert_to(&self, other: &PrimitiveType) -> bool {
        match (self, other) {
            // Same type
            (a, b) if a == b => true,

            // Any can convert to anything
            (PrimitiveType::Any, _) => true,

            // Anything can convert to Any
            (_, PrimitiveType::Any) => true,

            // Integer widening conversions
            (
                PrimitiveType::Int8,
                PrimitiveType::Int16 | PrimitiveType::Int32 | PrimitiveType::Int64,
            ) => true,
            (PrimitiveType::Int16, PrimitiveType::Int32 | PrimitiveType::Int64) => true,
            (PrimitiveType::Int32, PrimitiveType::Int64) => true,

            (
                PrimitiveType::UInt8,
                PrimitiveType::UInt16 | PrimitiveType::UInt32 | PrimitiveType::UInt64,
            ) => true,
            (PrimitiveType::UInt16, PrimitiveType::UInt32 | PrimitiveType::UInt64) => true,
            (PrimitiveType::UInt32, PrimitiveType::UInt64) => true,

            // Float widening conversion
            (PrimitiveType::Float32, PrimitiveType::Float64) => true,

            // Integer to float conversions
            (int_type, float_type) if int_type.is_integer() && float_type.is_float() => {
                // Only allow if the integer type can fit in the float type without precision loss
                match (int_type, float_type) {
                    (
                        PrimitiveType::Int8
                        | PrimitiveType::UInt8
                        | PrimitiveType::Int16
                        | PrimitiveType::UInt16,
                        _,
                    ) => true,
                    (PrimitiveType::Int32 | PrimitiveType::UInt32, PrimitiveType::Float64) => true,
                    _ => false,
                }
            }

            _ => false,
        }
    }

    /// Check if this type can be explicitly cast to another type
    pub fn can_explicitly_cast_to(&self, other: &PrimitiveType) -> bool {
        match (self, other) {
            // Same type
            (a, b) if a == b => true,

            // Any can cast to anything
            (PrimitiveType::Any, _) => true,
            (_, PrimitiveType::Any) => true,

            // All numeric types can be cast to each other
            (a, b) if a.is_numeric() && b.is_numeric() => true,

            // Bool can be cast to integers (false = 0, true = 1)
            (PrimitiveType::Bool, int_type) if int_type.is_integer() => true,

            // Integers can be cast to bool (0 = false, non-zero = true)
            (int_type, PrimitiveType::Bool) if int_type.is_integer() => true,

            // Floats can be cast to bool (0.0 = false, non-zero = true)
            (float_type, PrimitiveType::Bool) if float_type.is_float() => true,

            // String can be cast to bool (empty = false, non-empty = true)
            (PrimitiveType::String, PrimitiveType::Bool) => true,

            // String conversions (all types can be converted to string)
            (_, PrimitiveType::String) => true,

            // Char to integer conversions
            (PrimitiveType::Char, int_type) if int_type.is_integer() => true,
            (int_type, PrimitiveType::Char) if int_type.is_integer() => true,

            _ => false,
        }
    }

    /// Get the default value for this type
    pub fn default_value(&self) -> RuntimeValue {
        match self {
            PrimitiveType::Int8 => RuntimeValue::Int8(0),
            PrimitiveType::Int16 => RuntimeValue::Int16(0),
            PrimitiveType::Int32 => RuntimeValue::Int32(0),
            PrimitiveType::Int64 => RuntimeValue::Int64(0),
            PrimitiveType::UInt8 => RuntimeValue::UInt8(0),
            PrimitiveType::UInt16 => RuntimeValue::UInt16(0),
            PrimitiveType::UInt32 => RuntimeValue::UInt32(0),
            PrimitiveType::UInt64 => RuntimeValue::UInt64(0),
            PrimitiveType::Float32 => RuntimeValue::Float32(0.0),
            PrimitiveType::Float64 => RuntimeValue::Float64(0.0),
            PrimitiveType::Bool => RuntimeValue::Bool(false),
            PrimitiveType::Char => RuntimeValue::Char('\0'),
            PrimitiveType::String => RuntimeValue::String(String::new()),
            PrimitiveType::Any => RuntimeValue::Null,
            PrimitiveType::Void => RuntimeValue::Null, // Void defaults to null
        }
    }

    /// Parse a primitive type from a string
    pub fn from_str(s: &str) -> Option<PrimitiveType> {
        match s {
            "int8" => Some(PrimitiveType::Int8),
            "int16" => Some(PrimitiveType::Int16),
            "int32" => Some(PrimitiveType::Int32),
            "int64" => Some(PrimitiveType::Int64),
            "uint8" => Some(PrimitiveType::UInt8),
            "uint16" => Some(PrimitiveType::UInt16),
            "uint32" => Some(PrimitiveType::UInt32),
            "uint64" => Some(PrimitiveType::UInt64),
            "float32" => Some(PrimitiveType::Float32),
            "float64" => Some(PrimitiveType::Float64),
            "bool" => Some(PrimitiveType::Bool),
            "char" => Some(PrimitiveType::Char),
            "string" => Some(PrimitiveType::String),
            "any" => Some(PrimitiveType::Any),
            _ => None,
        }
    }

    /// Convert AST type to TypeId (requires type registry for composite types)
    pub fn ast_type_to_type_id(ast_type: &Type) -> TypeId {
        match ast_type {
            Type::Int8 => TypeId::Int8,
            Type::Int16 => TypeId::Int16,
            Type::Int32 => TypeId::Int32,
            Type::Int64 => TypeId::Int64,
            Type::UInt8 => TypeId::UInt8,
            Type::UInt16 => TypeId::UInt16,
            Type::UInt32 => TypeId::UInt32,
            Type::UInt64 => TypeId::UInt64,
            Type::Float32 => TypeId::Float32,
            Type::Float64 => TypeId::Float64,
            Type::Bool => TypeId::Bool,
            Type::Char => TypeId::Char,
            Type::String => TypeId::String,
            Type::Any => TypeId::Any,
            Type::Array(_) => TypeId::Array(0), // Placeholder - needs type registry
            Type::Slice(_) => TypeId::Slice(0), // Placeholder - needs type registry
            Type::Map(_) => TypeId::Map(0),     // Placeholder - needs type registry
            Type::Function(_) => TypeId::Function(0), // Placeholder
            Type::Struct(_) => TypeId::Struct(0), // Placeholder - needs type registry
            Type::Interface(_) => TypeId::Interface(0), // Placeholder - needs type registry
            _ => TypeId::Unknown,
        }
    }

    /// Check if one type is assignable to another
    pub fn is_assignable(from: TypeId, to: TypeId) -> bool {
        match (from, to) {
            // Same type
            (a, b) if a == b => true,

            // Any type conversions
            (TypeId::Any, _) | (_, TypeId::Any) => true,

            // Unknown type (for error recovery)
            (TypeId::Unknown, _) | (_, TypeId::Unknown) => true,

            // Integer widening conversions
            (TypeId::Int8, TypeId::Int16 | TypeId::Int32 | TypeId::Int64) => true,
            (TypeId::Int16, TypeId::Int32 | TypeId::Int64) => true,
            (TypeId::Int32, TypeId::Int64) => true,

            (TypeId::UInt8, TypeId::UInt16 | TypeId::UInt32 | TypeId::UInt64) => true,
            (TypeId::UInt16, TypeId::UInt32 | TypeId::UInt64) => true,
            (TypeId::UInt32, TypeId::UInt64) => true,

            // Float widening conversion
            (TypeId::Float32, TypeId::Float64) => true,

            // Integer to float conversions (safe ones)
            (
                TypeId::Int8 | TypeId::UInt8 | TypeId::Int16 | TypeId::UInt16,
                TypeId::Float32 | TypeId::Float64,
            ) => true,
            (TypeId::Int32 | TypeId::UInt32, TypeId::Float64) => true,

            _ => false,
        }
    }

    /// Get the type name as a string
    pub fn type_name(type_id: TypeId) -> &'static str {
        match type_id {
            TypeId::Int8 => "int8",
            TypeId::Int16 => "int16",
            TypeId::Int32 => "int32",
            TypeId::Int64 => "int64",
            TypeId::UInt8 => "uint8",
            TypeId::UInt16 => "uint16",
            TypeId::UInt32 => "uint32",
            TypeId::UInt64 => "uint64",
            TypeId::Float32 => "float32",
            TypeId::Float64 => "float64",
            TypeId::Bool => "bool",
            TypeId::Char => "char",
            TypeId::String => "string",
            TypeId::Any => "any",
            TypeId::Array(_) => "array",
            TypeId::Slice(_) => "slice",
            TypeId::Map(_) => "map",
            TypeId::Function(_) => "function",
            TypeId::Struct(_) => "struct",
            TypeId::Interface(_) => "interface",
            TypeId::Channel(_) => "channel",
            TypeId::Unknown => "unknown",
            TypeId::Void => "void",
            TypeId::Promise(_) => "promise",
            TypeId::Result(_) => "result",
            TypeId::Tuple(_) => "tuple",
        }
    }

    /// Infer type from literal value
    pub fn infer_from_literal(literal: &LiteralValue) -> TypeId {
        match literal {
            LiteralValue::Integer(_) => TypeId::Int32, // Default integer type
            LiteralValue::Float(_) => TypeId::Float64, // Default float type
            LiteralValue::String(_) => TypeId::String,
            LiteralValue::Char(_) => TypeId::Char,
            LiteralValue::Boolean(_) => TypeId::Bool,
            LiteralValue::Null => TypeId::Any,
        }
    }

    /// Check if a TypeId is numeric
    pub fn is_numeric_type_id(type_id: TypeId) -> bool {
        matches!(
            type_id,
            TypeId::Int8
                | TypeId::Int16
                | TypeId::Int32
                | TypeId::Int64
                | TypeId::UInt8
                | TypeId::UInt16
                | TypeId::UInt32
                | TypeId::UInt64
                | TypeId::Float32
                | TypeId::Float64
        )
    }

    /// Check if a TypeId is an integer
    pub fn is_integer_type_id(type_id: TypeId) -> bool {
        matches!(
            type_id,
            TypeId::Int8
                | TypeId::Int16
                | TypeId::Int32
                | TypeId::Int64
                | TypeId::UInt8
                | TypeId::UInt16
                | TypeId::UInt32
                | TypeId::UInt64
        )
    }

    /// Get the result type of a binary operation
    pub fn binary_operation_result_type(left: TypeId, right: TypeId, op: &str) -> Result<TypeId> {
        match op {
            // Arithmetic operations
            "+" => {
                if PrimitiveType::is_numeric_type_id(left)
                    && PrimitiveType::is_numeric_type_id(right)
                {
                    // Return the "wider" type for numeric addition
                    Ok(PrimitiveType::promote_numeric_types(left, right))
                } else if left == TypeId::String || right == TypeId::String {
                    // String concatenation - allow string + any type or any type + string
                    Ok(TypeId::String)
                } else {
                    Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Cannot apply {} to {} and {}",
                            "+",
                            PrimitiveType::type_name(left),
                            PrimitiveType::type_name(right)
                        ),
                        line: 0,
                        column: 0,
                    })
                }
            }
            "-" | "*" | "/" | "%" | "**" => {
                if PrimitiveType::is_numeric_type_id(left)
                    && PrimitiveType::is_numeric_type_id(right)
                {
                    // Return the "wider" type
                    Ok(PrimitiveType::promote_numeric_types(left, right))
                } else {
                    Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Cannot apply {} to {} and {}",
                            op,
                            PrimitiveType::type_name(left),
                            PrimitiveType::type_name(right)
                        ),
                        line: 0,
                        column: 0,
                    })
                }
            }

            // Comparison operations
            "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                if PrimitiveType::is_numeric_type_id(left)
                    && PrimitiveType::is_numeric_type_id(right)
                {
                    Ok(TypeId::Bool)
                } else if left == right {
                    Ok(TypeId::Bool)
                } else {
                    Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Cannot compare {} and {}",
                            PrimitiveType::type_name(left),
                            PrimitiveType::type_name(right)
                        ),
                        line: 0,
                        column: 0,
                    })
                }
            }

            // Logical operations
            "and" | "or" => {
                if left == TypeId::Bool && right == TypeId::Bool {
                    Ok(TypeId::Bool)
                } else {
                    Err(BuluError::TypeError {
                        file: None,
                        message: format!(
                            "Logical {} requires bool operands, got {} and {}",
                            op,
                            PrimitiveType::type_name(left),
                            PrimitiveType::type_name(right)
                        ),
                        line: 0,
                        column: 0,
                    })
                }
            }

            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Unknown binary operator: {}", op),
                line: 0,
                column: 0,
            }),
        }
    }

    /// Promote two numeric types to their common type
    fn promote_numeric_types(left: TypeId, right: TypeId) -> TypeId {
        use TypeId::*;

        match (left, right) {
            // If either is float64, result is float64
            (Float64, _) | (_, Float64) => Float64,
            // If either is float32, result is float32
            (Float32, _) | (_, Float32) => Float32,
            // If either is int64, result is int64
            (Int64, _) | (_, Int64) => Int64,
            // If either is uint64, result is uint64
            (UInt64, _) | (_, UInt64) => UInt64,
            // If either is int32, result is int32
            (Int32, _) | (_, Int32) => Int32,
            // If either is uint32, result is uint32
            (UInt32, _) | (_, UInt32) => UInt32,
            // If either is int16, result is int16
            (Int16, _) | (_, Int16) => Int16,
            // If either is uint16, result is uint16
            (UInt16, _) | (_, UInt16) => UInt16,
            // If either is int8, result is int8
            (Int8, _) | (_, Int8) => Int8,
            // Default to uint8
            _ => UInt8,
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PrimitiveType::Int8 => "int8",
            PrimitiveType::Int16 => "int16",
            PrimitiveType::Int32 => "int32",
            PrimitiveType::Int64 => "int64",
            PrimitiveType::UInt8 => "uint8",
            PrimitiveType::UInt16 => "uint16",
            PrimitiveType::UInt32 => "uint32",
            PrimitiveType::UInt64 => "uint64",
            PrimitiveType::Float32 => "float32",
            PrimitiveType::Float64 => "float64",
            PrimitiveType::Bool => "bool",
            PrimitiveType::Char => "char",
            PrimitiveType::String => "string",
            PrimitiveType::Any => "any",
            PrimitiveType::Void => "void",
        };
        write!(f, "{}", name)
    }
}

/// Runtime value representation with type information
#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeValue {
    // Signed integers
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),

    // Unsigned integers
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),

    // Floating point
    Float32(f32),
    Float64(f64),

    // Other primitives
    Bool(bool),
    Char(char),
    String(String),

    // Synchronization primitives
    Lock(usize), // Lock ID in registry

    // Concurrency primitives
    Channel(u32),   // Channel ID
    Goroutine(u32), // Goroutine ID

    // Async primitives
    Promise(u32), // Promise ID

    // Collection types
    Array(Vec<RuntimeValue>),                             // Array of values
    Slice(Vec<RuntimeValue>),                             // Slice of values (dynamic array)
    Tuple(Vec<RuntimeValue>),                             // Tuple of values
    Map(std::collections::HashMap<String, RuntimeValue>), // Map/dictionary
    Range(i64, i64, Option<i64>),                         // Range (start, end, step)
    Integer(i64),                                         // Generic integer for compatibility
    Byte(u8),

    // Function references
    Function(String), // Function name or identifier

    // Method references
    MethodRef {
        object: Box<RuntimeValue>,
        method_name: String,
        source_register: Option<u32>, // Track the register that contains the original struct
    },

    // Struct instances
    Struct {
        name: String,
        fields: std::collections::HashMap<String, RuntimeValue>,
    },

    // Global references
    Global(String), // Global variable name
    
    // Special values
    Null,
}

impl RuntimeValue {
    /// Get the type of this runtime value
    pub fn get_type(&self) -> PrimitiveType {
        match self {
            RuntimeValue::Int8(_) => PrimitiveType::Int8,
            RuntimeValue::Int16(_) => PrimitiveType::Int16,
            RuntimeValue::Int32(_) => PrimitiveType::Int32,
            RuntimeValue::Int64(_) => PrimitiveType::Int64,
            RuntimeValue::UInt8(_) => PrimitiveType::UInt8,
            RuntimeValue::UInt16(_) => PrimitiveType::UInt16,
            RuntimeValue::UInt32(_) => PrimitiveType::UInt32,
            RuntimeValue::UInt64(_) => PrimitiveType::UInt64,
            RuntimeValue::Float32(_) => PrimitiveType::Float32,
            RuntimeValue::Float64(_) => PrimitiveType::Float64,
            RuntimeValue::Bool(_) => PrimitiveType::Bool,
            RuntimeValue::Char(_) => PrimitiveType::Char,
            RuntimeValue::String(_) => PrimitiveType::String,
            RuntimeValue::Lock(_) => PrimitiveType::Any, // Locks are treated as Any type
            RuntimeValue::Channel(_) => PrimitiveType::Any, // Channels are treated as Any type
            RuntimeValue::Goroutine(_) => PrimitiveType::Any, // Goroutines are treated as Any type
            RuntimeValue::Promise(_) => PrimitiveType::Any, // Promises are treated as Any type
            RuntimeValue::Array(_) => PrimitiveType::Any, // Arrays are treated as Any type
            RuntimeValue::Slice(_) => PrimitiveType::Any, // Slices are treated as Any type
            RuntimeValue::Tuple(_) => PrimitiveType::Any, // Tuples are treated as Any type
            RuntimeValue::Map(_) => PrimitiveType::Any,  // Maps are treated as Any type
            RuntimeValue::Range(_, _, _) => PrimitiveType::Any, // Ranges are treated as Any type
            RuntimeValue::Integer(_) => PrimitiveType::Int64, // Generic integer maps to Int64
            RuntimeValue::Byte(_) => PrimitiveType::UInt8, // Byte maps to UInt8
            RuntimeValue::Function(_) => PrimitiveType::Any, // Functions are treated as Any type
            RuntimeValue::MethodRef { .. } => PrimitiveType::Any, // Method refs are treated as Any type
            RuntimeValue::Struct { .. } => PrimitiveType::Any, // Structs are treated as Any type
            RuntimeValue::Global(_) => PrimitiveType::Any, // Global refs are treated as Any type
            RuntimeValue::Null => PrimitiveType::Any,
        }
    }

    /// Check if this value is truthy
    pub fn is_truthy(&self) -> bool {
        match self {
            RuntimeValue::Bool(b) => *b,
            RuntimeValue::Int8(i) => *i != 0,
            RuntimeValue::Int16(i) => *i != 0,
            RuntimeValue::Int32(i) => *i != 0,
            RuntimeValue::Int64(i) => *i != 0,
            RuntimeValue::UInt8(i) => *i != 0,
            RuntimeValue::UInt16(i) => *i != 0,
            RuntimeValue::UInt32(i) => *i != 0,
            RuntimeValue::UInt64(i) => *i != 0,
            RuntimeValue::Float32(f) => *f != 0.0,
            RuntimeValue::Float64(f) => *f != 0.0,
            RuntimeValue::Char(c) => *c != '\0',
            RuntimeValue::String(s) => !s.is_empty(),
            RuntimeValue::Lock(_) => true, // Locks are always truthy (they exist)
            RuntimeValue::Channel(_) => true, // Channels are always truthy (they exist)
            RuntimeValue::Goroutine(_) => true, // Goroutines are always truthy (they exist)
            RuntimeValue::Promise(_) => true, // Promises are always truthy (they exist)
            RuntimeValue::Array(arr) => !arr.is_empty(), // Arrays are truthy if not empty
            RuntimeValue::Slice(slice) => !slice.is_empty(), // Slices are truthy if not empty
            RuntimeValue::Tuple(tuple) => !tuple.is_empty(), // Tuples are truthy if not empty
            RuntimeValue::Map(map) => !map.is_empty(), // Maps are truthy if not empty
            RuntimeValue::Range(start, end, _) => start != end, // Ranges are truthy if not empty
            RuntimeValue::Integer(i) => *i != 0, // Generic integer
            RuntimeValue::Byte(b) => *b != 0, // Byte is truthy if not zero
            RuntimeValue::Function(_) => true, // Functions are always truthy (they exist)
            RuntimeValue::MethodRef { .. } => true, // Method refs are always truthy (they exist)
            RuntimeValue::Struct { .. } => true, // Structs are always truthy (they exist)
            RuntimeValue::Global(_) => true, // Global refs are always truthy (they exist)
            RuntimeValue::Null => false,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            RuntimeValue::Int8(i) => i.to_string(),
            RuntimeValue::Int16(i) => i.to_string(),
            RuntimeValue::Int32(i) => i.to_string(),
            RuntimeValue::Int64(i) => i.to_string(),
            RuntimeValue::UInt8(i) => i.to_string(),
            RuntimeValue::UInt16(i) => i.to_string(),
            RuntimeValue::UInt32(i) => i.to_string(),
            RuntimeValue::UInt64(i) => i.to_string(),
            RuntimeValue::Float32(f) => f.to_string(),
            RuntimeValue::Float64(f) => f.to_string(),
            RuntimeValue::Bool(b) => b.to_string(),
            RuntimeValue::Char(c) => c.to_string(),
            RuntimeValue::String(s) => s.clone(),
            RuntimeValue::Lock(id) => format!("Lock({})", id),
            RuntimeValue::Channel(id) => format!("Channel({})", id),
            RuntimeValue::Goroutine(id) => format!("Goroutine({})", id),
            RuntimeValue::Promise(id) => format!("Promise({})", id),
            RuntimeValue::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            RuntimeValue::Range(start, end, step) => {
                if let Some(s) = step {
                    format!("{}..{}:{}", start, end, s)
                } else {
                    format!("{}..{}", start, end)
                }
            }
            RuntimeValue::Slice(slice) => {
                let elements: Vec<String> = slice.iter().map(|v| v.to_string()).collect();
                format!("[{}]", elements.join(", "))
            }
            RuntimeValue::Tuple(tuple) => {
                let elements: Vec<String> = tuple.iter().map(|v| v.to_string()).collect();
                format!("({})", elements.join(", "))
            }
            RuntimeValue::Map(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{{{}}}", pairs.join(", "))
            }
            RuntimeValue::Integer(i) => i.to_string(),
            RuntimeValue::Byte(b) => b.to_string(),
            RuntimeValue::Function(name) => format!("Function({})", name),
            RuntimeValue::MethodRef { method_name, .. } => format!("Method({})", method_name),
            RuntimeValue::Struct { name, fields } => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                format!("{}{{ {} }}", name, field_strs.join(", "))
            }
            RuntimeValue::Global(name) => format!("global:{}", name),
            RuntimeValue::Null => "null".to_string(),
        }
    }

    /// Attempt to cast this value to another type
    pub fn cast_to(&self, target_type: PrimitiveType) -> Result<RuntimeValue> {
        if !self.get_type().can_explicitly_cast_to(&target_type) {
            return Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot cast {} to {}", self.get_type(), target_type),
                line: 0,
                column: 0,
            });
        }

        match target_type {
            PrimitiveType::Int8 => Ok(RuntimeValue::Int8(self.to_i8()?)),
            PrimitiveType::Int16 => Ok(RuntimeValue::Int16(self.to_i16()?)),
            PrimitiveType::Int32 => Ok(RuntimeValue::Int32(self.to_i32()?)),
            PrimitiveType::Int64 => Ok(RuntimeValue::Int64(self.to_i64()?)),
            PrimitiveType::UInt8 => Ok(RuntimeValue::UInt8(self.to_u8()?)),
            PrimitiveType::UInt16 => Ok(RuntimeValue::UInt16(self.to_u16()?)),
            PrimitiveType::UInt32 => Ok(RuntimeValue::UInt32(self.to_u32()?)),
            PrimitiveType::UInt64 => Ok(RuntimeValue::UInt64(self.to_u64()?)),
            PrimitiveType::Float32 => Ok(RuntimeValue::Float32(self.to_f32()?)),
            PrimitiveType::Float64 => Ok(RuntimeValue::Float64(self.to_f64()?)),
            PrimitiveType::Bool => Ok(RuntimeValue::Bool(self.is_truthy())),
            PrimitiveType::Char => Ok(RuntimeValue::Char(self.to_char()?)),
            PrimitiveType::String => Ok(RuntimeValue::String(self.to_string())),
            PrimitiveType::Any => Ok(self.clone()),
            PrimitiveType::Void => Ok(RuntimeValue::Null), // Void casts to null
        }
    }

    // Helper methods for type conversions
    fn to_i8(&self) -> Result<i8> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i),
            RuntimeValue::Int16(i) => Ok(*i as i8),
            RuntimeValue::Int32(i) => Ok(*i as i8),
            RuntimeValue::Int64(i) => Ok(*i as i8),
            RuntimeValue::UInt8(i) => Ok(*i as i8),
            RuntimeValue::UInt16(i) => Ok(*i as i8),
            RuntimeValue::UInt32(i) => Ok(*i as i8),
            RuntimeValue::UInt64(i) => Ok(*i as i8),
            RuntimeValue::Float32(f) => Ok(*f as i8),
            RuntimeValue::Float64(f) => Ok(*f as i8),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32 as i8),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to int8", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_i16(&self) -> Result<i16> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as i16),
            RuntimeValue::Int16(i) => Ok(*i),
            RuntimeValue::Int32(i) => Ok(*i as i16),
            RuntimeValue::Int64(i) => Ok(*i as i16),
            RuntimeValue::UInt8(i) => Ok(*i as i16),
            RuntimeValue::UInt16(i) => Ok(*i as i16),
            RuntimeValue::UInt32(i) => Ok(*i as i16),
            RuntimeValue::UInt64(i) => Ok(*i as i16),
            RuntimeValue::Float32(f) => Ok(*f as i16),
            RuntimeValue::Float64(f) => Ok(*f as i16),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32 as i16),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to int16", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_i32(&self) -> Result<i32> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as i32),
            RuntimeValue::Int16(i) => Ok(*i as i32),
            RuntimeValue::Int32(i) => Ok(*i),
            RuntimeValue::Int64(i) => Ok(*i as i32),
            RuntimeValue::UInt8(i) => Ok(*i as i32),
            RuntimeValue::UInt16(i) => Ok(*i as i32),
            RuntimeValue::UInt32(i) => Ok(*i as i32),
            RuntimeValue::UInt64(i) => Ok(*i as i32),
            RuntimeValue::Float32(f) => Ok(*f as i32),
            RuntimeValue::Float64(f) => Ok(*f as i32),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32 as i32),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to int32", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_i64(&self) -> Result<i64> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as i64),
            RuntimeValue::Int16(i) => Ok(*i as i64),
            RuntimeValue::Int32(i) => Ok(*i as i64),
            RuntimeValue::Int64(i) => Ok(*i),
            RuntimeValue::UInt8(i) => Ok(*i as i64),
            RuntimeValue::UInt16(i) => Ok(*i as i64),
            RuntimeValue::UInt32(i) => Ok(*i as i64),
            RuntimeValue::UInt64(i) => Ok(*i as i64),
            RuntimeValue::Float32(f) => Ok(*f as i64),
            RuntimeValue::Float64(f) => Ok(*f as i64),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32 as i64),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to int64", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_u8(&self) -> Result<u8> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as u8),
            RuntimeValue::Int16(i) => Ok(*i as u8),
            RuntimeValue::Int32(i) => Ok(*i as u8),
            RuntimeValue::Int64(i) => Ok(*i as u8),
            RuntimeValue::UInt8(i) => Ok(*i),
            RuntimeValue::UInt16(i) => Ok(*i as u8),
            RuntimeValue::UInt32(i) => Ok(*i as u8),
            RuntimeValue::UInt64(i) => Ok(*i as u8),
            RuntimeValue::Float32(f) => Ok(*f as u8),
            RuntimeValue::Float64(f) => Ok(*f as u8),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32 as u8),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to uint8", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_u16(&self) -> Result<u16> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as u16),
            RuntimeValue::Int16(i) => Ok(*i as u16),
            RuntimeValue::Int32(i) => Ok(*i as u16),
            RuntimeValue::Int64(i) => Ok(*i as u16),
            RuntimeValue::UInt8(i) => Ok(*i as u16),
            RuntimeValue::UInt16(i) => Ok(*i),
            RuntimeValue::UInt32(i) => Ok(*i as u16),
            RuntimeValue::UInt64(i) => Ok(*i as u16),
            RuntimeValue::Float32(f) => Ok(*f as u16),
            RuntimeValue::Float64(f) => Ok(*f as u16),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32 as u16),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to uint16", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_u32(&self) -> Result<u32> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as u32),
            RuntimeValue::Int16(i) => Ok(*i as u32),
            RuntimeValue::Int32(i) => Ok(*i as u32),
            RuntimeValue::Int64(i) => Ok(*i as u32),
            RuntimeValue::UInt8(i) => Ok(*i as u32),
            RuntimeValue::UInt16(i) => Ok(*i as u32),
            RuntimeValue::UInt32(i) => Ok(*i),
            RuntimeValue::UInt64(i) => Ok(*i as u32),
            RuntimeValue::Float32(f) => Ok(*f as u32),
            RuntimeValue::Float64(f) => Ok(*f as u32),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to uint32", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_u64(&self) -> Result<u64> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as u64),
            RuntimeValue::Int16(i) => Ok(*i as u64),
            RuntimeValue::Int32(i) => Ok(*i as u64),
            RuntimeValue::Int64(i) => Ok(*i as u64),
            RuntimeValue::UInt8(i) => Ok(*i as u64),
            RuntimeValue::UInt16(i) => Ok(*i as u64),
            RuntimeValue::UInt32(i) => Ok(*i as u64),
            RuntimeValue::UInt64(i) => Ok(*i),
            RuntimeValue::Float32(f) => Ok(*f as u64),
            RuntimeValue::Float64(f) => Ok(*f as u64),
            RuntimeValue::Bool(b) => Ok(if *b { 1 } else { 0 }),
            RuntimeValue::Char(c) => Ok(*c as u32 as u64),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to uint64", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_f32(&self) -> Result<f32> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as f32),
            RuntimeValue::Int16(i) => Ok(*i as f32),
            RuntimeValue::Int32(i) => Ok(*i as f32),
            RuntimeValue::Int64(i) => Ok(*i as f32),
            RuntimeValue::UInt8(i) => Ok(*i as f32),
            RuntimeValue::UInt16(i) => Ok(*i as f32),
            RuntimeValue::UInt32(i) => Ok(*i as f32),
            RuntimeValue::UInt64(i) => Ok(*i as f32),
            RuntimeValue::Float32(f) => Ok(*f),
            RuntimeValue::Float64(f) => Ok(*f as f32),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to float32", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_f64(&self) -> Result<f64> {
        match self {
            RuntimeValue::Int8(i) => Ok(*i as f64),
            RuntimeValue::Int16(i) => Ok(*i as f64),
            RuntimeValue::Int32(i) => Ok(*i as f64),
            RuntimeValue::Int64(i) => Ok(*i as f64),
            RuntimeValue::UInt8(i) => Ok(*i as f64),
            RuntimeValue::UInt16(i) => Ok(*i as f64),
            RuntimeValue::UInt32(i) => Ok(*i as f64),
            RuntimeValue::UInt64(i) => Ok(*i as f64),
            RuntimeValue::Float32(f) => Ok(*f as f64),
            RuntimeValue::Float64(f) => Ok(*f),
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to float64", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }

    fn to_char(&self) -> Result<char> {
        match self {
            RuntimeValue::Char(c) => Ok(*c),
            RuntimeValue::UInt32(i) => char::from_u32(*i).ok_or_else(|| BuluError::TypeError {
                file: None,
                message: format!("Invalid Unicode code point: {}", i),
                line: 0,
                column: 0,
            }),
            RuntimeValue::Int32(i) if *i >= 0 => {
                char::from_u32(*i as u32).ok_or_else(|| BuluError::TypeError {
                    file: None,
                    message: format!("Invalid Unicode code point: {}", i),
                    line: 0,
                    column: 0,
                })
            }
            _ => Err(BuluError::TypeError {
                file: None,
                message: format!("Cannot convert {} to char", self.get_type()),
                line: 0,
                column: 0,
            }),
        }
    }
}

impl fmt::Display for RuntimeValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeValue::Int8(i) => write!(f, "{}", i),
            RuntimeValue::Int16(i) => write!(f, "{}", i),
            RuntimeValue::Int32(i) => write!(f, "{}", i),
            RuntimeValue::Int64(i) => write!(f, "{}", i),
            RuntimeValue::UInt8(i) => write!(f, "{}", i),
            RuntimeValue::UInt16(i) => write!(f, "{}", i),
            RuntimeValue::UInt32(i) => write!(f, "{}", i),
            RuntimeValue::UInt64(i) => write!(f, "{}", i),
            RuntimeValue::Float32(f_val) => write!(f, "{}", f_val),
            RuntimeValue::Float64(f_val) => write!(f, "{}", f_val),
            RuntimeValue::Bool(b) => write!(f, "{}", b),
            RuntimeValue::Char(c) => write!(f, "{}", c),
            RuntimeValue::String(s) => write!(f, "{}", s),
            RuntimeValue::Lock(id) => write!(f, "lock({})", id),
            RuntimeValue::Channel(id) => write!(f, "channel({})", id),
            RuntimeValue::Goroutine(id) => write!(f, "goroutine({})", id),
            RuntimeValue::Promise(id) => write!(f, "promise({})", id),
            RuntimeValue::Array(arr) => {
                let elements: Vec<String> = arr.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            RuntimeValue::Range(start, end, step) => {
                if let Some(s) = step {
                    write!(f, "{}..{}:{}", start, end, s)
                } else {
                    write!(f, "{}..{}", start, end)
                }
            }
            RuntimeValue::Slice(slice) => {
                let elements: Vec<String> = slice.iter().map(|v| v.to_string()).collect();
                write!(f, "[{}]", elements.join(", "))
            }
            RuntimeValue::Tuple(tuple) => {
                let elements: Vec<String> = tuple.iter().map(|v| v.to_string()).collect();
                write!(f, "({})", elements.join(", "))
            }
            RuntimeValue::Map(map) => {
                let pairs: Vec<String> = map
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                write!(f, "{{{}}}", pairs.join(", "))
            }
            RuntimeValue::Integer(i) => write!(f, "{}", i),
            RuntimeValue::Byte(b) => write!(f, "{}", b),
            RuntimeValue::Function(name) => write!(f, "function({})", name),
            RuntimeValue::MethodRef { method_name, .. } => write!(f, "method({})", method_name),
            RuntimeValue::Struct { name, fields } => {
                let field_strs: Vec<String> = fields.iter()
                    .map(|(k, v)| format!("{}: {}", k, v.to_string()))
                    .collect();
                write!(f, "{}{{ {} }}", name, field_strs.join(", "))
            }
            RuntimeValue::Global(name) => write!(f, "global:{}", name),
            RuntimeValue::Null => write!(f, "null"),
        }
    }
}

impl TypeInfo {
    /// Check if this type is assignable to another type
    pub fn is_assignable_to(&self, other: &TypeInfo) -> bool {
        match (self, other) {
            // Same types
            (a, b) if a == b => true,

            // Unknown type (for error recovery)
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => true,

            // Primitive type assignability
            (TypeInfo::Primitive(from), TypeInfo::Primitive(to)) => {
                let from_id = PrimitiveType::to_type_id(*from);
                let to_id = PrimitiveType::to_type_id(*to);
                PrimitiveType::is_assignable(from_id, to_id)
            }

            // Array to slice conversion
            (TypeInfo::Array(elem1), TypeInfo::Slice(elem2)) => elem1.is_assignable_to(elem2),

            // Function type assignability (contravariant parameters, covariant return)
            (TypeInfo::Function(params1, ret1), TypeInfo::Function(params2, ret2)) => {
                // Check parameter count
                if params1.len() != params2.len() {
                    return false;
                }

                // Parameters are contravariant (can accept more specific types)
                for (p1, p2) in params1.iter().zip(params2.iter()) {
                    if !p2.is_assignable_to(p1) {
                        return false;
                    }
                }

                // Return type is covariant (can return more specific types)
                match (ret1, ret2) {
                    (Some(r1), Some(r2)) => r1.is_assignable_to(r2),
                    (None, None) => true,
                    _ => false,
                }
            }

            _ => false,
        }
    }
}

impl PrimitiveType {
    /// Convert PrimitiveType to TypeId
    pub fn to_type_id(self) -> TypeId {
        match self {
            PrimitiveType::Int8 => TypeId::Int8,
            PrimitiveType::Int16 => TypeId::Int16,
            PrimitiveType::Int32 => TypeId::Int32,
            PrimitiveType::Int64 => TypeId::Int64,
            PrimitiveType::UInt8 => TypeId::UInt8,
            PrimitiveType::UInt16 => TypeId::UInt16,
            PrimitiveType::UInt32 => TypeId::UInt32,
            PrimitiveType::UInt64 => TypeId::UInt64,
            PrimitiveType::Float32 => TypeId::Float32,
            PrimitiveType::Float64 => TypeId::Float64,
            PrimitiveType::Bool => TypeId::Bool,
            PrimitiveType::Char => TypeId::Char,
            PrimitiveType::String => TypeId::String,
            PrimitiveType::Any => TypeId::Any,
            PrimitiveType::Void => TypeId::Void,
        }
    }
}
