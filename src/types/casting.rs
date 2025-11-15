//! Type casting implementation for the Bulu language

use crate::error::{Result, BuluError};
use crate::types::primitive::{PrimitiveType, RuntimeValue, TypeId};

/// Type casting utilities
pub struct TypeCaster;

impl TypeCaster {
    /// Perform explicit type casting
    pub fn cast_value(value: RuntimeValue, target_type: TypeId) -> Result<RuntimeValue> {
        let target_primitive = match target_type {
            TypeId::Int8 => PrimitiveType::Int8,
            TypeId::Int16 => PrimitiveType::Int16,
            TypeId::Int32 => PrimitiveType::Int32,
            TypeId::Int64 => PrimitiveType::Int64,
            TypeId::UInt8 => PrimitiveType::UInt8,
            TypeId::UInt16 => PrimitiveType::UInt16,
            TypeId::UInt32 => PrimitiveType::UInt32,
            TypeId::UInt64 => PrimitiveType::UInt64,
            TypeId::Float32 => PrimitiveType::Float32,
            TypeId::Float64 => PrimitiveType::Float64,
            TypeId::Bool => PrimitiveType::Bool,
            TypeId::Char => PrimitiveType::Char,
            TypeId::String => PrimitiveType::String,
            TypeId::Any => PrimitiveType::Any,
            _ => {
                return Err(BuluError::TypeError { stack: Vec::new(),
                    file: None,
                    message: format!("Cannot cast to {}", PrimitiveType::type_name(target_type)),
                    line: 0,
                    column: 0,
                });
            }
        };
        
        value.cast_to(target_primitive)
    }
    
    /// Check if a cast is valid
    pub fn is_cast_valid(from_type: TypeId, to_type: TypeId) -> bool {
        let from_primitive = match from_type {
            TypeId::Int8 => PrimitiveType::Int8,
            TypeId::Int16 => PrimitiveType::Int16,
            TypeId::Int32 => PrimitiveType::Int32,
            TypeId::Int64 => PrimitiveType::Int64,
            TypeId::UInt8 => PrimitiveType::UInt8,
            TypeId::UInt16 => PrimitiveType::UInt16,
            TypeId::UInt32 => PrimitiveType::UInt32,
            TypeId::UInt64 => PrimitiveType::UInt64,
            TypeId::Float32 => PrimitiveType::Float32,
            TypeId::Float64 => PrimitiveType::Float64,
            TypeId::Bool => PrimitiveType::Bool,
            TypeId::Char => PrimitiveType::Char,
            TypeId::String => PrimitiveType::String,
            TypeId::Any => PrimitiveType::Any,
            _ => return false,
        };
        
        let to_primitive = match to_type {
            TypeId::Int8 => PrimitiveType::Int8,
            TypeId::Int16 => PrimitiveType::Int16,
            TypeId::Int32 => PrimitiveType::Int32,
            TypeId::Int64 => PrimitiveType::Int64,
            TypeId::UInt8 => PrimitiveType::UInt8,
            TypeId::UInt16 => PrimitiveType::UInt16,
            TypeId::UInt32 => PrimitiveType::UInt32,
            TypeId::UInt64 => PrimitiveType::UInt64,
            TypeId::Float32 => PrimitiveType::Float32,
            TypeId::Float64 => PrimitiveType::Float64,
            TypeId::Bool => PrimitiveType::Bool,
            TypeId::Char => PrimitiveType::Char,
            TypeId::String => PrimitiveType::String,
            TypeId::Any => PrimitiveType::Any,
            _ => return false,
        };
        
        from_primitive.can_explicitly_cast_to(&to_primitive)
    }
}