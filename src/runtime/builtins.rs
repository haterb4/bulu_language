//! Built-in functions for the Bulu language
//!
//! This module implements all core built-in functions including:
//! - Type conversion functions (int32(), float64(), string(), etc.)
//! - Memory functions (len(), cap(), clone(), sizeof())
//! - Array/slice operations (append(), make(), copy())
//! - Map and channel operations (make(), delete())
//! - I/O functions (print(), println(), printf(), input())

use crate::error::{BuluError, Result};
use crate::types::primitive::{PrimitiveType, RuntimeValue, TypeId};

use crate::runtime::channels::{Channel, ChannelRegistry};
use crate::runtime::promises::PromiseRegistry;
use crate::runtime::sync::{sleep, timer, yield_now, AtomicOperations, LockRegistry};
use std::collections::HashMap;
use std::io::{self, Write};
use std::sync::{Arc, Mutex};

/// Runtime value for collections and composite types
#[derive(Debug, Clone, PartialEq)]
pub enum CollectionValue {
    Array(Vec<RuntimeValue>),
    Slice(Vec<RuntimeValue>),
    Map(HashMap<String, RuntimeValue>),
    Channel(usize), // Channel ID in registry
}

/// Built-in function registry
pub struct BuiltinRegistry {
    functions: HashMap<String, BuiltinFunction>,
    channel_registry: Arc<Mutex<ChannelRegistry>>,
    lock_registry: Arc<Mutex<LockRegistry>>,
    promise_registry: Arc<Mutex<PromiseRegistry>>,
}

/// Built-in function type
pub type BuiltinFunction = fn(&[RuntimeValue]) -> Result<RuntimeValue>;

impl BuiltinRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            functions: HashMap::new(),
            channel_registry: Arc::new(Mutex::new(ChannelRegistry::new())),
            lock_registry: Arc::new(Mutex::new(LockRegistry::new())),
            promise_registry: Arc::new(Mutex::new(PromiseRegistry::new())),
        };

        // Register all built-in functions
        registry.register_type_conversion_functions();
        registry.register_memory_functions();
        registry.register_collection_functions();
        registry.register_io_functions();
        registry.register_utility_functions();
        registry.register_channel_functions();
        registry.register_synchronization_functions();

        registry
    }

    /// Register a built-in function
    pub fn register(&mut self, name: &str, func: BuiltinFunction) {
        self.functions.insert(name.to_string(), func);
    }

    /// Get a built-in function by name
    pub fn get(&self, name: &str) -> Option<&BuiltinFunction> {
        self.functions.get(name)
    }

    /// Check if a function is a built-in
    pub fn is_builtin(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    /// Get all built-in function names
    pub fn get_all_names(&self) -> Vec<String> {
        self.functions.keys().cloned().collect()
    }

    /// Get the channel registry
    pub fn channel_registry(&self) -> Arc<Mutex<ChannelRegistry>> {
        Arc::clone(&self.channel_registry)
    }

    /// Get the lock registry
    pub fn lock_registry(&self) -> Arc<Mutex<LockRegistry>> {
        Arc::clone(&self.lock_registry)
    }

    /// Get the promise registry
    pub fn promise_registry(&self) -> Arc<Mutex<PromiseRegistry>> {
        Arc::clone(&self.promise_registry)
    }

    /// Register type conversion functions
    fn register_type_conversion_functions(&mut self) {
        self.register("int8", builtin_int8);
        self.register("int16", builtin_int16);
        self.register("int32", builtin_int32);
        self.register("int64", builtin_int64);
        self.register("uint8", builtin_uint8);
        self.register("uint16", builtin_uint16);
        self.register("uint32", builtin_uint32);
        self.register("uint64", builtin_uint64);
        self.register("float32", builtin_float32);
        self.register("float64", builtin_float64);
        self.register("byte", builtin_int8);
        self.register("rune", builtin_int8);
        self.register("bool", builtin_bool);
        self.register("char", builtin_char);
        self.register("string", builtin_string);
    }

    /// Register memory functions
    fn register_memory_functions(&mut self) {
        self.register("len", builtin_len);
        self.register("cap", builtin_cap);
        self.register("clone", builtin_clone);
        self.register("sizeof", builtin_sizeof);
    }

    /// Register collection functions
    fn register_collection_functions(&mut self) {
        self.register("make", builtin_make);
        self.register("append", builtin_append);
        self.register("copy", builtin_copy);
        self.register("delete", builtin_delete);
        self.register("__range_to_array", builtin_range_to_array);
    }

    /// Register I/O functions
    fn register_io_functions(&mut self) {
        self.register("print", builtin_print);
        self.register("println", builtin_println);
        self.register("printf", builtin_printf);
        self.register("input", builtin_input);
    }

    /// Register utility functions
    fn register_utility_functions(&mut self) {
        self.register("typeof", builtin_typeof);
        self.register("instanceof", builtin_instanceof);
        self.register("panic", builtin_panic);
        self.register("assert", builtin_assert);
        self.register("recover", builtin_recover);
    }

    /// Register channel functions
    fn register_channel_functions(&mut self) {
        self.register("close", builtin_close);
        self.register("send", builtin_send);
        self.register("recv", builtin_recv);
    }

    /// Register synchronization functions
    fn register_synchronization_functions(&mut self) {
        self.register("lock", builtin_lock);
        self.register("sleep", builtin_sleep);
        self.register("yield", builtin_yield);
        self.register("timer", builtin_timer);
        self.register("atomic_load", builtin_atomic_load);
        self.register("atomic_store", builtin_atomic_store);
        self.register("atomic_add", builtin_atomic_add);
        self.register("atomic_sub", builtin_atomic_sub);
        self.register("atomic_cas", builtin_atomic_cas);
    }
}

// ============================================================================
// TYPE CONVERSION FUNCTIONS
// ============================================================================

/// Convert value to int8
pub fn builtin_int8(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "int8() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Int8)
}

/// Convert value to int16
pub fn builtin_int16(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "int16() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Int16)
}

/// Convert value to int32
pub fn builtin_int32(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "int32() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Int32)
}

/// Convert value to int64
pub fn builtin_int64(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "int64() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Int64)
}

/// Convert value to uint8
pub fn builtin_uint8(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "uint8() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::UInt8)
}

/// Convert value to uint16
pub fn builtin_uint16(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "uint16() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::UInt16)
}

/// Convert value to uint32
pub fn builtin_uint32(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "uint32() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::UInt32)
}

/// Convert value to uint64
pub fn builtin_uint64(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "uint64() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::UInt64)
}

/// Convert value to float32
pub fn builtin_float32(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "float32() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Float32)
}

/// Convert value to float64
pub fn builtin_float64(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "float64() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Float64)
}

/// Convert value to bool
pub fn builtin_bool(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "bool() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Bool)
}

/// Convert value to char
pub fn builtin_char(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "char() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::Char)
}

/// Convert value to string
pub fn builtin_string(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "string() expects exactly 1 argument".to_string(),
        });
    }

    args[0].cast_to(PrimitiveType::String)
}

// ============================================================================
// MEMORY FUNCTIONS
// ============================================================================

/// Get length of array, slice, string, or map
pub fn builtin_len(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "len() expects exactly 1 argument".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::String(s) => Ok(RuntimeValue::Int32(s.len() as i32)),
        RuntimeValue::Array(arr) => Ok(RuntimeValue::Int32(arr.len() as i32)),
        RuntimeValue::Slice(slice) => Ok(RuntimeValue::Int32(slice.len() as i32)),
        RuntimeValue::Map(map) => Ok(RuntimeValue::Int32(map.len() as i32)),
        RuntimeValue::Channel(channel_id) => {
            // Get channel length from global registry
            let registry = crate::runtime::interpreter::get_global_channel_registry().lock().unwrap();
            if let Some(channel) = registry.get(*channel_id) {
                Ok(RuntimeValue::Int32(channel.len() as i32))
            } else {
                Err(BuluError::RuntimeError {
                    file: None,
                    message: "Channel not found".to_string(),
                })
            }
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: format!("len() not supported for type: {:?}", args[0].get_type()),
        }),
    }
}

/// Get capacity of slice or channel
pub fn builtin_cap(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "cap() expects exactly 1 argument".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::Slice(slice) => Ok(RuntimeValue::Int32(slice.capacity() as i32)),
        RuntimeValue::Array(arr) => Ok(RuntimeValue::Int32(arr.len() as i32)), // Arrays have fixed capacity
        RuntimeValue::Channel(channel_id) => {
            // Get channel capacity from global registry
            let registry = crate::runtime::interpreter::get_global_channel_registry().lock().unwrap();
            if let Some(channel) = registry.get(*channel_id) {
                Ok(RuntimeValue::Int32(channel.capacity() as i32))
            } else {
                Err(BuluError::RuntimeError {
                    file: None,
                    message: "Channel not found".to_string(),
                })
            }
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: format!("cap() not supported for type: {:?}", args[0].get_type()),
        }),
    }
}

/// Deep clone a value
pub fn builtin_clone(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "clone() expects exactly 1 argument".to_string(),
        });
    }

    // For primitive types, cloning is just copying
    Ok(args[0].clone())
}

/// Get size in bytes of a value
pub fn builtin_sizeof(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "sizeof() expects exactly 1 argument".to_string(),
        });
    }

    let size = match &args[0] {
        RuntimeValue::Int8(_) => 1,
        RuntimeValue::Int16(_) => 2,
        RuntimeValue::Int32(_) => 4,
        RuntimeValue::Int64(_) => 8,
        RuntimeValue::UInt8(_) => 1,
        RuntimeValue::UInt16(_) => 2,
        RuntimeValue::UInt32(_) => 4,
        RuntimeValue::UInt64(_) => 8,
        RuntimeValue::Float32(_) => 4,
        RuntimeValue::Float64(_) => 8,
        RuntimeValue::Bool(_) => 1,
        RuntimeValue::Char(_) => 4, // UTF-8 character
        RuntimeValue::String(s) => s.len(),
        RuntimeValue::Lock(_) => std::mem::size_of::<usize>(), // Lock ID size
        RuntimeValue::Channel(_) => std::mem::size_of::<u32>(), // Channel ID size
        RuntimeValue::Goroutine(_) => std::mem::size_of::<u32>(), // Goroutine ID size
        RuntimeValue::Promise(_) => std::mem::size_of::<u32>(), // Promise ID size
        RuntimeValue::Array(arr) => arr.len() * std::mem::size_of::<RuntimeValue>(), // Array size
        RuntimeValue::Slice(slice) => slice.len() * std::mem::size_of::<RuntimeValue>(), // Slice size
        RuntimeValue::Map(map) => {
            map.len() * (std::mem::size_of::<String>() + std::mem::size_of::<RuntimeValue>())
        } // Map size
        RuntimeValue::Integer(_) => 8, // Generic integer is 64-bit
        RuntimeValue::Byte(_) => 1,    // Byte is 1 byte
        RuntimeValue::Null => 0,
        RuntimeValue::Function(_) => std::mem::size_of::<String>(), // Function refs are pointer-sized
        RuntimeValue::MethodRef { .. } => std::mem::size_of::<String>() * 2, // Object + method name
        RuntimeValue::Struct { fields, .. } => {
            // Estimate struct size as sum of field sizes
            fields.values().map(|v| estimate_value_size(v)).sum::<usize>()
        }
        RuntimeValue::Global(_) => std::mem::size_of::<String>(), // Global refs are pointer-sized
    };

    Ok(RuntimeValue::Int32(size as i32))
}

fn estimate_value_size(value: &RuntimeValue) -> usize {
    match value {
        RuntimeValue::Int8(_) => 1,
        RuntimeValue::Int16(_) => 2,
        RuntimeValue::Int32(_) => 4,
        RuntimeValue::Int64(_) => 8,
        RuntimeValue::UInt8(_) => 1,
        RuntimeValue::UInt16(_) => 2,
        RuntimeValue::UInt32(_) => 4,
        RuntimeValue::UInt64(_) => 8,
        RuntimeValue::Float32(_) => 4,
        RuntimeValue::Float64(_) => 8,
        RuntimeValue::Byte(_) => 1,
        RuntimeValue::Bool(_) => 1,
        RuntimeValue::Char(_) => 4,
        RuntimeValue::String(s) => s.len(),
        _ => 8, // Default size for complex types
    }
}

// ============================================================================
// COLLECTION FUNCTIONS
// ============================================================================

/// Create collections (arrays, slices, maps, channels)
/// This is a simplified version that handles basic channel creation
/// Built-in make function with Go-like type syntax
/// This function should be called with proper type information from the parser
pub fn builtin_make(args: &[RuntimeValue]) -> Result<RuntimeValue> {

    // This function should not be called directly for type-based make() calls
    // The interpreter's evaluate_make_call() handles proper type-based make() syntax
    // This function is kept for backward compatibility with non-type-based calls

    if args.is_empty() {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "make() expects at least 1 argument".to_string(),
        });
    }

    // Check if the first argument is a type identifier
    if let RuntimeValue::String(type_name) = &args[0] {
        // Handle slice types
        if type_name.starts_with("slice_") {
            let len = if args.len() > 1 {
                extract_size_arg(&args[1], "slice length")?
            } else {
                0 // Empty slice by default
            };
            
            let cap = if args.len() > 2 {
                extract_size_arg(&args[2], "slice capacity")?
            } else {
                len // Capacity equals length by default
            };
            
            // Create slice with specified length, filled with default values for the type
            let default_value = get_default_value_for_slice_type(type_name);
            let slice = vec![default_value; len];
            return Ok(RuntimeValue::Slice(slice));
        }
        
        // Handle channel types
        if type_name.starts_with("chan_") {
            let capacity = if args.len() > 1 {
                Some(extract_size_arg(&args[1], "channel capacity")?)
            } else {
                None
            };
            
            // Extract the element type from chan_TYPE
            let element_type = match type_name.strip_prefix("chan_") {
                Some("int8") => TypeId::Int8,
                Some("int16") => TypeId::Int16,
                Some("int32") => TypeId::Int32,
                Some("int64") => TypeId::Int64,
                Some("uint8") => TypeId::UInt8,
                Some("uint16") => TypeId::UInt16,
                Some("uint32") => TypeId::UInt32,
                Some("uint64") => TypeId::UInt64,
                Some("float32") => TypeId::Float32,
                Some("float64") => TypeId::Float64,
                Some("byte") => TypeId::Int8,
                Some("bool") => TypeId::Bool,
                Some("char") => TypeId::Char,
                Some("string") => TypeId::String,
                _ => TypeId::Any,
            };
            
            return builtin_make_chan(element_type, &args[1..]);
        }
        
        // Handle other type names
        match type_name.as_str() {
            "chan" => {
                // Untyped channel (chan any)
                let capacity = if args.len() > 1 {
                    Some(extract_size_arg(&args[1], "channel capacity")?)
                } else {
                    None
                };
                return builtin_make_chan(TypeId::Any, &args[1..]);
            }
            // Primitive types - return zero values
            "int8" => {
                return Ok(RuntimeValue::Int8(0));
            }
            "int16" => {
                return Ok(RuntimeValue::Int16(0));
            }
            "int32" => {
                return Ok(RuntimeValue::Int32(0));
            }
            "uint8" => {
                return Ok(RuntimeValue::UInt8(0));
            }
            "uint16" => {
                return Ok(RuntimeValue::UInt16(0));
            }
            "uint32" => {
                return Ok(RuntimeValue::UInt32(0));
            }
            "byte" => {
                return Ok(RuntimeValue::Byte(0));
            }
            "int64" => {
                return Ok(RuntimeValue::Int64(0));
            }
            "uint64" => {
                return Ok(RuntimeValue::UInt64(0));
            }
            "float32" => {
                return Ok(RuntimeValue::Float32(0.0));
            }
            "float64" => {
                return Ok(RuntimeValue::Float64(0.0));
            }
            "bool" => {
                return Ok(RuntimeValue::Bool(false));
            }
            "string" | "char" => {
                return Ok(RuntimeValue::String(String::new()));
            }
            "any" => {
                return Ok(RuntimeValue::Null);
            }
            _ => {}
        }
    }

    // Handle generic slice types (slice_TypeName)
    if let RuntimeValue::String(type_name) = &args[0] {
        if type_name.starts_with("slice_") {
            let len = if args.len() > 1 {
                extract_size_arg(&args[1], "slice length")?
            } else {
                0 // Empty slice by default
            };
            
            // Create slice with specified length, filled with zero values
            let slice = vec![RuntimeValue::Null; len];
            return Ok(RuntimeValue::Slice(slice));
        }
        
        // Handle generic channel types (chan_TypeName)
        if type_name.starts_with("chan_") {
            let capacity = if args.len() > 1 {
                Some(extract_size_arg(&args[1], "channel capacity")?)
            } else {
                None
            };
            
            // For unknown channel types, default to Any
            return builtin_make_chan(TypeId::Any, &args[1..]);
        }
    }

    // For direct builtin calls without type information, we can only infer from patterns
    match args.len() {
        1 => {
            // Single argument - assume it's a map
            let map = std::collections::HashMap::new();
            Ok(RuntimeValue::Map(map))
        }
        2 => {
            // Two arguments - assume it's a slice with length
            let size = extract_size_arg(&args[1], "size")?;
            let slice = vec![RuntimeValue::Null; size];
            Ok(RuntimeValue::Slice(slice))
        }
        3 => {
            // Three arguments - assume it's a slice with length and capacity
            let len = extract_size_arg(&args[1], "length")?;
            let _cap = extract_size_arg(&args[2], "capacity")?;
            let slice = vec![RuntimeValue::Null; len];
            Ok(RuntimeValue::Slice(slice))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "make() takes 1-3 arguments".to_string(),
        }),
    }
}

/// Get default value for a slice type
fn get_default_value_for_slice_type(slice_type: &str) -> RuntimeValue {
    if let Some(element_type) = slice_type.strip_prefix("slice_") {
        match element_type {
            "int8" => RuntimeValue::Int8(0),
            "int16" => RuntimeValue::Int16(0),
            "int32" => RuntimeValue::Int32(0),
            "int64" => RuntimeValue::Int64(0),
            "uint8" => RuntimeValue::UInt8(0),
            "uint16" => RuntimeValue::UInt16(0),
            "uint32" => RuntimeValue::UInt32(0),
            "uint64" => RuntimeValue::UInt64(0),
            "float32" => RuntimeValue::Float32(0.0),
            "float64" => RuntimeValue::Float64(0.0),
            "byte" => RuntimeValue::Byte(0),
            "bool" => RuntimeValue::Bool(false),
            "char" => RuntimeValue::Char('\0'),
            "string" => RuntimeValue::String(String::new()),
            "any" | "unknown" => RuntimeValue::Null,
            // For user-defined types (structs, interfaces), use Null as placeholder
            _ => RuntimeValue::Null,
        }
    } else {
        RuntimeValue::Null
    }
}

/// Helper function to extract size arguments
fn extract_size_arg(arg: &RuntimeValue, context: &str) -> Result<usize> {
    match arg {
        RuntimeValue::Int32(s) if *s >= 0 => Ok(*s as usize),
        RuntimeValue::Int64(s) if *s >= 0 => Ok(*s as usize),
        RuntimeValue::UInt32(s) => Ok(*s as usize),
        RuntimeValue::UInt64(s) => Ok(*s as usize),
        RuntimeValue::Integer(s) if *s >= 0 => Ok(*s as usize),
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: format!("{} must be a non-negative integer", context),
        }),
    }
}

/// Create a channel with make(chan T) or make(chan T, capacity)
pub fn builtin_make_channel(
    element_type: TypeId,
    capacity: Option<usize>,
    registry: Arc<Mutex<ChannelRegistry>>,
) -> Result<RuntimeValue> {
    let channel = if let Some(cap) = capacity {
        if cap == 0 {
            Channel::new_unbuffered(element_type)
        } else {
            Channel::new_buffered(element_type, cap)
        }
    } else {
        Channel::new_unbuffered(element_type)
    };

    let mut reg = registry.lock().unwrap();
    let channel_id = reg.register(channel);

    // Return the channel ID wrapped in a special runtime value
    // In a full implementation, this would be a proper Channel runtime value
    Ok(RuntimeValue::Int32(channel_id as i32))
}

/// Append elements to a slice
pub fn builtin_append(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() < 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "append() expects at least 2 arguments".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::Slice(slice) => {
            let mut new_slice = slice.clone();
            // Append all remaining arguments to the slice
            for arg in &args[1..] {
                new_slice.push(arg.clone());
            }
            Ok(RuntimeValue::Slice(new_slice))
        }
        RuntimeValue::Array(array) => {
            let mut new_array = array.clone();
            // Append all remaining arguments to the array
            for arg in &args[1..] {
                new_array.push(arg.clone());
            }
            Ok(RuntimeValue::Array(new_array))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "append() first argument must be a slice or array".to_string(),
        }),
    }
}

/// Copy elements from source to destination slice
pub fn builtin_copy(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "copy() expects exactly 2 arguments (dst, src)".to_string(),
        });
    }

    let src_elements = match &args[1] {
        RuntimeValue::Slice(slice) => slice,
        RuntimeValue::Array(array) => array,
        _ => {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "copy() source must be a slice or array".to_string(),
            })
        }
    };

    match &args[0] {
        RuntimeValue::Slice(dst_slice) => {
            let mut new_dst = dst_slice.clone();
            let copy_count = std::cmp::min(new_dst.len(), src_elements.len());

            for i in 0..copy_count {
                new_dst[i] = src_elements[i].clone();
            }

            Ok(RuntimeValue::Int32(copy_count as i32))
        }
        RuntimeValue::Array(dst_array) => {
            let mut new_dst = dst_array.clone();
            let copy_count = std::cmp::min(new_dst.len(), src_elements.len());

            for i in 0..copy_count {
                new_dst[i] = src_elements[i].clone();
            }

            Ok(RuntimeValue::Int32(copy_count as i32))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "copy() destination must be a slice or array".to_string(),
        }),
    }
}

/// Delete key from map
pub fn builtin_delete(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "delete() expects exactly 2 arguments (map, key)".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::Map(map) => {
            let key = match &args[1] {
                RuntimeValue::String(s) => s.clone(),
                _ => args[1].to_string(), // Convert other types to string keys
            };

            let mut new_map = map.clone();
            new_map.remove(&key);
            Ok(RuntimeValue::Map(new_map))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "delete() first argument must be a map".to_string(),
        }),
    }
}

/// Convert range to array: __range_to_array(start, end, inclusive)
pub fn builtin_range_to_array(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 3 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "__range_to_array requires exactly 3 arguments".to_string(),
        });
    }

    let start = match &args[0] {
        RuntimeValue::Int64(i) => *i,
        RuntimeValue::Int32(i) => *i as i64,
        RuntimeValue::Integer(i) => *i,
        _ => return Err(BuluError::RuntimeError {
            file: None,
            message: "Range start must be an integer".to_string(),
        }),
    };

    let end = match &args[1] {
        RuntimeValue::Int64(i) => *i,
        RuntimeValue::Int32(i) => *i as i64,
        RuntimeValue::Integer(i) => *i,
        _ => return Err(BuluError::RuntimeError {
            file: None,
            message: "Range end must be an integer".to_string(),
        }),
    };

    let inclusive = match &args[2] {
        RuntimeValue::Bool(b) => *b,
        _ => return Err(BuluError::RuntimeError {
            file: None,
            message: "Range inclusive flag must be boolean".to_string(),
        }),
    };

    // Create array from range
    let mut array = Vec::new();
    let actual_end = if inclusive { end + 1 } else { end };
    
    for i in start..actual_end {
        array.push(RuntimeValue::Int64(i));
    }

    Ok(RuntimeValue::Array(array))
}

// ============================================================================
// I/O FUNCTIONS
// ============================================================================

/// Print values to stdout
pub fn builtin_print(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", format_runtime_value(arg));
    }
    io::stdout().flush().map_err(|e| BuluError::RuntimeError {
            file: None,
        message: format!("Failed to flush stdout: {}", e),
    })?;

    Ok(RuntimeValue::Null)
}

/// Print values to stdout with newline
pub fn builtin_println(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{}", format_runtime_value(arg));
    }
    println!();

    Ok(RuntimeValue::Null)
}

/// Formatted print with basic format specifier support
pub fn builtin_printf(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.is_empty() {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "printf() expects at least 1 argument (format string)".to_string(),
        });
    }

    if let RuntimeValue::String(format_str) = &args[0] {
        let formatted = format_string_with_args(format_str, &args[1..])?;
        print!("{}", formatted);
        io::stdout().flush().map_err(|e| BuluError::RuntimeError {
            file: None,
            message: format!("Failed to flush stdout: {}", e),
        })?;
    } else {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "printf() first argument must be a string".to_string(),
        });
    }

    Ok(RuntimeValue::Null)
}

/// Read input from stdin
pub fn builtin_input(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    // Print prompt if provided
    if !args.is_empty() {
        if let RuntimeValue::String(prompt) = &args[0] {
            print!("{}", prompt);
            io::stdout().flush().map_err(|e| BuluError::RuntimeError {
            file: None,
                message: format!("Failed to flush stdout: {}", e),
            })?;
        }
    }

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| BuluError::RuntimeError {
            file: None,
            message: format!("Failed to read input: {}", e),
        })?;

    // Remove trailing newline
    if input.ends_with('\n') {
        input.pop();
        if input.ends_with('\r') {
            input.pop();
        }
    }

    Ok(RuntimeValue::String(input))
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get the type name of a value
pub fn builtin_typeof(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "typeof() expects exactly 1 argument".to_string(),
        });
    }

    let type_name = match &args[0] {
        RuntimeValue::Int8(_) => "int8",
        RuntimeValue::Int16(_) => "int16",
        RuntimeValue::Int32(_) => "int32",
        RuntimeValue::Int64(_) => "int64",
        RuntimeValue::UInt8(_) => "uint8",
        RuntimeValue::UInt16(_) => "uint16",
        RuntimeValue::UInt32(_) => "uint32",
        RuntimeValue::UInt64(_) => "uint64",
        RuntimeValue::Float32(_) => "float32",
        RuntimeValue::Float64(_) => "float64",
        RuntimeValue::Bool(_) => "bool",
        RuntimeValue::Char(_) => "char",
        RuntimeValue::String(_) => "string",
        RuntimeValue::Lock(_) => "lock",
        RuntimeValue::Channel(_) => "channel",
        RuntimeValue::Goroutine(_) => "goroutine",
        RuntimeValue::Promise(_) => "promise",
        RuntimeValue::Array(_) => "array",
        RuntimeValue::Slice(_) => "slice",
        RuntimeValue::Map(_) => "map",
        RuntimeValue::Integer(_) => "integer",
        RuntimeValue::Byte(_) => "byte",
        RuntimeValue::Function(_) => "function",
        RuntimeValue::MethodRef { .. } => "method",
        RuntimeValue::Struct { name, .. } => name,
        RuntimeValue::Global(_) => "global",
        RuntimeValue::Null => "null",
    };

    Ok(RuntimeValue::String(type_name.to_string()))
}

/// Check if a value is an instance of a specific type
pub fn builtin_instanceof(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "instanceof() expects exactly 2 arguments (value, type_name)".to_string(),
        });
    }

    let value = &args[0];
    let type_name = match &args[1] {
        RuntimeValue::String(s) => s,
        _ => {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "instanceof() second argument must be a string (type name)".to_string(),
            });
        }
    };

    let actual_type = match value {
        RuntimeValue::Int8(_) => "int8",
        RuntimeValue::Int16(_) => "int16",
        RuntimeValue::Int32(_) => "int32",
        RuntimeValue::Int64(_) => "int64",
        RuntimeValue::UInt8(_) => "uint8",
        RuntimeValue::UInt16(_) => "uint16",
        RuntimeValue::UInt32(_) => "uint32",
        RuntimeValue::UInt64(_) => "uint64",
        RuntimeValue::Float32(_) => "float32",
        RuntimeValue::Float64(_) => "float64",
        RuntimeValue::Bool(_) => "bool",
        RuntimeValue::Char(_) => "char",
        RuntimeValue::String(_) => "string",
        RuntimeValue::Lock(_) => "lock",
        RuntimeValue::Channel(_) => "channel",
        RuntimeValue::Goroutine(_) => "goroutine",
        RuntimeValue::Promise(_) => "promise",
        RuntimeValue::Array(_) => "array",
        RuntimeValue::Slice(_) => "slice",
        RuntimeValue::Map(_) => "map",
        RuntimeValue::Integer(_) => "integer",
        RuntimeValue::Byte(_) => "byte",
        RuntimeValue::Function(_) => "function",
        RuntimeValue::MethodRef { .. } => "method",
        RuntimeValue::Struct { name, .. } => name,
        RuntimeValue::Global(_) => "global",
        RuntimeValue::Null => "null",
    };

    // Check for exact type match
    if actual_type == type_name {
        return Ok(RuntimeValue::Bool(true));
    }

    // Check for type category matches
    let is_match = match type_name.as_str() {
        "integer" => matches!(
            value,
            RuntimeValue::Int8(_)
                | RuntimeValue::Int16(_)
                | RuntimeValue::Int32(_)
                | RuntimeValue::Int64(_)
                | RuntimeValue::UInt8(_)
                | RuntimeValue::UInt16(_)
                | RuntimeValue::UInt32(_)
                | RuntimeValue::UInt64(_)
        ),
        "float" => matches!(value, RuntimeValue::Float32(_) | RuntimeValue::Float64(_)),
        "number" | "numeric" => matches!(
            value,
            RuntimeValue::Int8(_)
                | RuntimeValue::Int16(_)
                | RuntimeValue::Int32(_)
                | RuntimeValue::Int64(_)
                | RuntimeValue::UInt8(_)
                | RuntimeValue::UInt16(_)
                | RuntimeValue::UInt32(_)
                | RuntimeValue::UInt64(_)
                | RuntimeValue::Float32(_)
                | RuntimeValue::Float64(_)
        ),
        "signed" => matches!(
            value,
            RuntimeValue::Int8(_)
                | RuntimeValue::Int16(_)
                | RuntimeValue::Int32(_)
                | RuntimeValue::Int64(_)
        ),
        "unsigned" => matches!(
            value,
            RuntimeValue::UInt8(_)
                | RuntimeValue::UInt16(_)
                | RuntimeValue::UInt32(_)
                | RuntimeValue::UInt64(_)
        ),
        "primitive" => !matches!(value, RuntimeValue::Null),
        "any" => true, // Everything is an instance of 'any'
        _ => false,
    };

    Ok(RuntimeValue::Bool(is_match))
}

/// Panic with a message
pub fn builtin_panic(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    let message = if args.is_empty() {
        "panic".to_string()
    } else {
        format_runtime_value(&args[0])
    };

    Err(BuluError::RuntimeError {
            file: None,
        message: format!("panic: {}", message),
    })
}

/// Assert a condition
pub fn builtin_assert(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.is_empty() {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "assert() expects at least 1 argument".to_string(),
        });
    }

    let condition = args[0].is_truthy();
    if !condition {
        let message = if args.len() > 1 {
            format_runtime_value(&args[1])
        } else {
            "assertion failed".to_string()
        };

        return Err(BuluError::RuntimeError {
            file: None,
            message: format!("assertion failed: {}", message),
        });
    }

    Ok(RuntimeValue::Null)
}

/// Thread-local panic state for recover mechanism
thread_local! {
    static PANIC_STATE: std::cell::RefCell<Option<RuntimeValue>> = std::cell::RefCell::new(None);
}

/// Set panic state (used internally by panic mechanism)
pub fn set_panic_state(value: RuntimeValue) {
    PANIC_STATE.with(|state| {
        *state.borrow_mut() = Some(value);
    });
}

/// Clear panic state (used internally by panic mechanism)
pub fn clear_panic_state() {
    PANIC_STATE.with(|state| {
        *state.borrow_mut() = None;
    });
}

/// Recover from a panic (used in defer blocks)
pub fn builtin_recover(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if !args.is_empty() {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "recover() expects no arguments".to_string(),
        });
    }

    // Check if we're currently in a panic state using thread-local storage
    PANIC_STATE.with(|state| {
        let mut panic_state = state.borrow_mut();
        if let Some(panic_value) = panic_state.take() {
            // We were in a panic, return the panic value and clear the state
            Ok(panic_value)
        } else {
            // Not in a panic, return null
            Ok(RuntimeValue::Null)
        }
    })
}

// ============================================================================
// SYNCHRONIZATION FUNCTIONS
// ============================================================================

/// Create a new lock
pub fn builtin_lock(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if !args.is_empty() {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "lock() expects no arguments".to_string(),
        });
    }

    // Use a global lock registry for simplicity
    // In a full implementation, this would be passed from the interpreter context
    thread_local! {
        static LOCK_REGISTRY: std::cell::RefCell<LockRegistry> = std::cell::RefCell::new(LockRegistry::new());
    }

    LOCK_REGISTRY.with(|registry| {
        let mut reg = registry.borrow_mut();
        let lock_id = reg.create_lock();
        Ok(RuntimeValue::Lock(lock_id))
    })
}

/// Sleep for the specified number of milliseconds
pub fn builtin_sleep(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "sleep() expects exactly 1 argument (milliseconds)".to_string(),
        });
    }

    let milliseconds = match &args[0] {
        RuntimeValue::Int32(ms) => *ms as u64,
        RuntimeValue::Int64(ms) => *ms as u64,
        RuntimeValue::UInt32(ms) => *ms as u64,
        RuntimeValue::UInt64(ms) => *ms,
        _ => {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "sleep() argument must be a number (milliseconds)".to_string(),
            });
        }
    };

    sleep(milliseconds);
    Ok(RuntimeValue::Null)
}

/// Yield execution to other goroutines
pub fn builtin_yield(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if !args.is_empty() {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "yield() expects no arguments".to_string(),
        });
    }

    yield_now();
    Ok(RuntimeValue::Null)
}

/// Create a timer channel
pub fn builtin_timer(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "timer() expects exactly 1 argument (milliseconds)".to_string(),
        });
    }

    let milliseconds = match &args[0] {
        RuntimeValue::Int32(ms) => *ms as u64,
        RuntimeValue::Int64(ms) => *ms as u64,
        RuntimeValue::UInt32(ms) => *ms as u64,
        RuntimeValue::UInt64(ms) => *ms,
        _ => {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "timer() argument must be a number (milliseconds)".to_string(),
            });
        }
    };

    timer(milliseconds)
}

/// Atomic load operation
pub fn builtin_atomic_load(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_load() expects exactly 1 argument".to_string(),
        });
    }

    AtomicOperations::atomic_load(&args[0])
}

/// Atomic store operation
pub fn builtin_atomic_store(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_store() expects exactly 2 arguments (target, value)".to_string(),
        });
    }

    // Validate that both arguments are compatible atomic types
    match (&args[0], &args[1]) {
        (RuntimeValue::Int32(_), RuntimeValue::Int32(_))
        | (RuntimeValue::Int64(_), RuntimeValue::Int64(_))
        | (RuntimeValue::UInt32(_), RuntimeValue::UInt32(_))
        | (RuntimeValue::UInt64(_), RuntimeValue::UInt64(_))
        | (RuntimeValue::Bool(_), RuntimeValue::Bool(_)) => {
            // In a real implementation, this would perform an atomic store operation
            // For now, we simulate the operation and return null to indicate success
            Ok(RuntimeValue::Null)
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_store() requires compatible atomic types".to_string(),
        }),
    }
}

/// Atomic add operation
pub fn builtin_atomic_add(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_add() expects exactly 2 arguments (target, value)".to_string(),
        });
    }

    // Perform atomic addition and return the old value
    match (&args[0], &args[1]) {
        (RuntimeValue::Int32(target), RuntimeValue::Int32(_value)) => {
            let old_value = *target;
            // In a real implementation, this would be an atomic fetch_add operation
            Ok(RuntimeValue::Int32(old_value))
        }
        (RuntimeValue::Int64(target), RuntimeValue::Int64(_value)) => {
            let old_value = *target;
            Ok(RuntimeValue::Int64(old_value))
        }
        (RuntimeValue::UInt32(target), RuntimeValue::UInt32(_value)) => {
            let old_value = *target;
            Ok(RuntimeValue::UInt32(old_value))
        }
        (RuntimeValue::UInt64(target), RuntimeValue::UInt64(_value)) => {
            let old_value = *target;
            Ok(RuntimeValue::UInt64(old_value))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_add() requires compatible numeric types".to_string(),
        }),
    }
}

/// Atomic subtract operation
pub fn builtin_atomic_sub(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_sub() expects exactly 2 arguments (target, value)".to_string(),
        });
    }

    // Perform atomic subtraction and return the old value
    match (&args[0], &args[1]) {
        (RuntimeValue::Int32(target), RuntimeValue::Int32(_value)) => {
            let old_value = *target;
            // In a real implementation, this would be an atomic fetch_sub operation
            Ok(RuntimeValue::Int32(old_value))
        }
        (RuntimeValue::Int64(target), RuntimeValue::Int64(_value)) => {
            let old_value = *target;
            Ok(RuntimeValue::Int64(old_value))
        }
        (RuntimeValue::UInt32(target), RuntimeValue::UInt32(_value)) => {
            let old_value = *target;
            Ok(RuntimeValue::UInt32(old_value))
        }
        (RuntimeValue::UInt64(target), RuntimeValue::UInt64(_value)) => {
            let old_value = *target;
            Ok(RuntimeValue::UInt64(old_value))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_sub() requires compatible numeric types".to_string(),
        }),
    }
}

/// Atomic compare-and-swap operation
pub fn builtin_atomic_cas(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 3 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_cas() expects exactly 3 arguments (target, expected, desired)"
                .to_string(),
        });
    }

    // Perform atomic compare-and-swap and return the old value
    match (&args[0], &args[1], &args[2]) {
        (
            RuntimeValue::Int32(target),
            RuntimeValue::Int32(_expected),
            RuntimeValue::Int32(_desired),
        ) => {
            let old_value = *target;
            // In a real implementation, this would be an atomic compare_exchange operation
            // Return the old value regardless of whether the swap succeeded
            Ok(RuntimeValue::Int32(old_value))
        }
        (
            RuntimeValue::Int64(target),
            RuntimeValue::Int64(_expected),
            RuntimeValue::Int64(_desired),
        ) => {
            let old_value = *target;
            Ok(RuntimeValue::Int64(old_value))
        }
        (
            RuntimeValue::UInt32(target),
            RuntimeValue::UInt32(_expected),
            RuntimeValue::UInt32(_desired),
        ) => {
            let old_value = *target;
            Ok(RuntimeValue::UInt32(old_value))
        }
        (
            RuntimeValue::UInt64(target),
            RuntimeValue::UInt64(_expected),
            RuntimeValue::UInt64(_desired),
        ) => {
            let old_value = *target;
            Ok(RuntimeValue::UInt64(old_value))
        }
        (
            RuntimeValue::Bool(target),
            RuntimeValue::Bool(_expected),
            RuntimeValue::Bool(_desired),
        ) => {
            let old_value = *target;
            Ok(RuntimeValue::Bool(old_value))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "atomic_cas() requires all arguments to be the same atomic type".to_string(),
        }),
    }
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Format a runtime value for display
pub fn format_runtime_value(value: &RuntimeValue) -> String {
    match value {
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
        RuntimeValue::Lock(id) => format!("lock({})", id),
        RuntimeValue::Channel(id) => format!("channel({})", id),
        RuntimeValue::Goroutine(id) => format!("goroutine({})", id),
        RuntimeValue::Promise(id) => format!("promise({})", id),
        RuntimeValue::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(|v| format_runtime_value(v)).collect();
            format!("[{}]", elements.join(", "))
        }
        RuntimeValue::Slice(slice) => {
            let elements: Vec<String> = slice.iter().map(|v| format_runtime_value(v)).collect();
            format!("[{}]", elements.join(", "))
        }
        RuntimeValue::Map(map) => {
            let pairs: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, format_runtime_value(v)))
                .collect();
            format!("{{{}}}", pairs.join(", "))
        }
        RuntimeValue::Integer(i) => i.to_string(),
        RuntimeValue::Byte(b) => b.to_string(),
        RuntimeValue::Function(name) => format!("function({})", name),
        RuntimeValue::MethodRef { method_name, .. } => format!("method({})", method_name),
        RuntimeValue::Struct { name, fields } => {
            let field_strs: Vec<String> = fields.iter()
                .map(|(k, v)| format!("{}: {}", k, format_runtime_value(v)))
                .collect();
            format!("{}{{ {} }}", name, field_strs.join(", "))
        }
        RuntimeValue::Global(name) => format!("global:{}", name),
        RuntimeValue::Null => "null".to_string(),
    }
}

/// Format a string with arguments using basic format specifiers
fn format_string_with_args(format_str: &str, args: &[RuntimeValue]) -> Result<String> {
    let mut result = String::new();
    let mut chars = format_str.chars().peekable();
    let mut arg_index = 0;

    while let Some(ch) = chars.next() {
        if ch == '%' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '%' {
                    // Escaped %
                    chars.next();
                    result.push('%');
                } else {
                    // Format specifier
                    chars.next(); // consume the format character

                    if arg_index >= args.len() {
                        return Err(BuluError::RuntimeError {
            file: None,
                            message: format!("printf: not enough arguments for format string"),
                        });
                    }

                    let formatted = match next_ch {
                        'd' | 'i' => {
                            // Integer format
                            match &args[arg_index] {
                                RuntimeValue::Int8(i) => i.to_string(),
                                RuntimeValue::Int16(i) => i.to_string(),
                                RuntimeValue::Int32(i) => i.to_string(),
                                RuntimeValue::Int64(i) => i.to_string(),
                                RuntimeValue::UInt8(i) => i.to_string(),
                                RuntimeValue::UInt16(i) => i.to_string(),
                                RuntimeValue::UInt32(i) => i.to_string(),
                                RuntimeValue::UInt64(i) => i.to_string(),
                                _ => format_runtime_value(&args[arg_index]),
                            }
                        }
                        'f' => {
                            // Float format
                            match &args[arg_index] {
                                RuntimeValue::Float32(f) => format!("{:.6}", f),
                                RuntimeValue::Float64(f) => format!("{:.6}", f),
                                _ => format_runtime_value(&args[arg_index]),
                            }
                        }
                        'g' => {
                            // General float format
                            match &args[arg_index] {
                                RuntimeValue::Float32(f) => f.to_string(),
                                RuntimeValue::Float64(f) => f.to_string(),
                                _ => format_runtime_value(&args[arg_index]),
                            }
                        }
                        's' => {
                            // String format
                            match &args[arg_index] {
                                RuntimeValue::String(s) => s.clone(),
                                _ => format_runtime_value(&args[arg_index]),
                            }
                        }
                        'c' => {
                            // Character format
                            match &args[arg_index] {
                                RuntimeValue::Char(c) => c.to_string(),
                                RuntimeValue::Int32(i) => {
                                    if let Some(c) = char::from_u32(*i as u32) {
                                        c.to_string()
                                    } else {
                                        "?".to_string()
                                    }
                                }
                                _ => format_runtime_value(&args[arg_index]),
                            }
                        }
                        'b' => {
                            // Boolean format
                            match &args[arg_index] {
                                RuntimeValue::Bool(b) => b.to_string(),
                                _ => args[arg_index].is_truthy().to_string(),
                            }
                        }
                        'x' => {
                            // Hexadecimal format
                            match &args[arg_index] {
                                RuntimeValue::Int8(i) => format!("{:x}", i),
                                RuntimeValue::Int16(i) => format!("{:x}", i),
                                RuntimeValue::Int32(i) => format!("{:x}", i),
                                RuntimeValue::Int64(i) => format!("{:x}", i),
                                RuntimeValue::UInt8(i) => format!("{:x}", i),
                                RuntimeValue::UInt16(i) => format!("{:x}", i),
                                RuntimeValue::UInt32(i) => format!("{:x}", i),
                                RuntimeValue::UInt64(i) => format!("{:x}", i),
                                _ => format_runtime_value(&args[arg_index]),
                            }
                        }
                        'o' => {
                            // Octal format
                            match &args[arg_index] {
                                RuntimeValue::Int8(i) => format!("{:o}", i),
                                RuntimeValue::Int16(i) => format!("{:o}", i),
                                RuntimeValue::Int32(i) => format!("{:o}", i),
                                RuntimeValue::Int64(i) => format!("{:o}", i),
                                RuntimeValue::UInt8(i) => format!("{:o}", i),
                                RuntimeValue::UInt16(i) => format!("{:o}", i),
                                RuntimeValue::UInt32(i) => format!("{:o}", i),
                                RuntimeValue::UInt64(i) => format!("{:o}", i),
                                _ => format_runtime_value(&args[arg_index]),
                            }
                        }
                        'v' => {
                            // Default format (like %s but for any type)
                            format_runtime_value(&args[arg_index])
                        }
                        _ => {
                            // Unknown format specifier, just use the character
                            format!("%{}", next_ch)
                        }
                    };

                    result.push_str(&formatted);
                    arg_index += 1;
                }
            } else {
                // % at end of string
                result.push('%');
            }
        } else {
            result.push(ch);
        }
    }

    Ok(result)
}

// ============================================================================
// CHANNEL FUNCTIONS
// ============================================================================

/// Close a channel
pub fn builtin_close(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "close() expects exactly 1 argument (channel)".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::Channel(channel_id) => {
            // Use a global channel registry for simplicity
            // In a full implementation, this would be passed from the interpreter context
            thread_local! {
                static CHANNEL_REGISTRY: std::cell::RefCell<ChannelRegistry> = std::cell::RefCell::new(ChannelRegistry::new());
            }

            CHANNEL_REGISTRY.with(|registry| {
                let reg = registry.borrow();
                if let Some(channel) = reg.get(*channel_id as usize) {
                    channel.close()?;
                    Ok(RuntimeValue::Null)
                } else {
                    Err(BuluError::RuntimeError {
            file: None,
                        message: format!("Channel {} not found", channel_id),
                    })
                }
            })
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "close() expects a channel argument".to_string(),
        }),
    }
}

/// Send a value to a channel (used internally by channel operations)
pub fn channel_send(
    channel_id: usize,
    value: RuntimeValue,
    registry: Arc<Mutex<ChannelRegistry>>,
) -> Result<RuntimeValue> {
    let reg = registry.lock().unwrap();
    if let Some(channel) = reg.get(channel_id) {
        match channel.send(value)? {
            crate::runtime::channels::SendResult::Ok => Ok(RuntimeValue::Bool(true)),
            crate::runtime::channels::SendResult::Closed => Ok(RuntimeValue::Bool(false)),
            crate::runtime::channels::SendResult::WouldBlock => Ok(RuntimeValue::Bool(false)),
        }
    } else {
        Err(BuluError::RuntimeError {
            file: None,
            message: format!("Invalid channel ID: {}", channel_id),
        })
    }
}

/// Receive a value from a channel (used internally by channel operations)
pub fn channel_receive(
    channel_id: usize,
    registry: Arc<Mutex<ChannelRegistry>>,
) -> Result<RuntimeValue> {
    let reg = registry.lock().unwrap();
    if let Some(channel) = reg.get(channel_id) {
        match channel.receive()? {
            crate::runtime::channels::ChannelResult::Ok(value) => Ok(value),
            crate::runtime::channels::ChannelResult::Closed => Ok(RuntimeValue::Null),
            crate::runtime::channels::ChannelResult::WouldBlock => Ok(RuntimeValue::Null),
        }
    } else {
        Err(BuluError::RuntimeError {
            file: None,
            message: format!("Invalid channel ID: {}", channel_id),
        })
    }
}

/// Try to send a value to a channel (non-blocking)
pub fn channel_try_send(
    channel_id: usize,
    value: RuntimeValue,
    registry: Arc<Mutex<ChannelRegistry>>,
) -> Result<RuntimeValue> {
    let reg = registry.lock().unwrap();
    if let Some(channel) = reg.get(channel_id) {
        match channel.try_send(value)? {
            crate::runtime::channels::SendResult::Ok => Ok(RuntimeValue::Bool(true)),
            crate::runtime::channels::SendResult::Closed => Ok(RuntimeValue::Bool(false)),
            crate::runtime::channels::SendResult::WouldBlock => Ok(RuntimeValue::Bool(false)),
        }
    } else {
        Err(BuluError::RuntimeError {
            file: None,
            message: format!("Invalid channel ID: {}", channel_id),
        })
    }
}

/// Try to receive a value from a channel (non-blocking)
pub fn channel_try_receive(
    channel_id: usize,
    registry: Arc<Mutex<ChannelRegistry>>,
) -> Result<(RuntimeValue, bool)> {
    let reg = registry.lock().unwrap();
    if let Some(channel) = reg.get(channel_id) {
        match channel.try_receive()? {
            crate::runtime::channels::ChannelResult::Ok(value) => Ok((value, true)),
            crate::runtime::channels::ChannelResult::Closed => Ok((RuntimeValue::Null, false)),
            crate::runtime::channels::ChannelResult::WouldBlock => Ok((RuntimeValue::Null, false)),
        }
    } else {
        Err(BuluError::RuntimeError {
            file: None,
            message: format!("Invalid channel ID: {}", channel_id),
        })
    }
}

/// Close a channel by ID
pub fn channel_close(
    channel_id: usize,
    registry: Arc<Mutex<ChannelRegistry>>,
) -> Result<RuntimeValue> {
    let reg = registry.lock().unwrap();
    if let Some(channel) = reg.get(channel_id) {
        channel.close()?;
        Ok(RuntimeValue::Null)
    } else {
        Err(BuluError::RuntimeError {
            file: None,
            message: format!("Invalid channel ID: {}", channel_id),
        })
    }
}

// ============================================================================
// DEFAULT INSTANCE
// ============================================================================

/// Get the default built-in registry
pub fn get_default_builtins() -> BuiltinRegistry {
    BuiltinRegistry::new()
}
// ============================================================================
// CHANNEL FUNCTIONS
// ============================================================================

/// Send a value to a channel
pub fn builtin_send(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 2 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "send() expects exactly 2 arguments (channel, value)".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::Channel(channel_id) => {
            let _value = &args[1];
            // For now, simulate successful send
            // In a full implementation, this would use the actual channel registry
            // and perform the send operation
            println!("Sending value to channel {}", channel_id);
            Ok(RuntimeValue::Null)
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "send() first argument must be a channel".to_string(),
        }),
    }
}

/// Receive a value from a channel
pub fn builtin_recv(args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.len() != 1 {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "recv() expects exactly 1 argument (channel)".to_string(),
        });
    }

    match &args[0] {
        RuntimeValue::Channel(channel_id) => {
            // For now, simulate successful receive
            // In a full implementation, this would use the actual channel registry
            // and perform the receive operation
            println!("Receiving value from channel {}", channel_id);
            Ok(RuntimeValue::String("received_value".to_string()))
        }
        _ => Err(BuluError::RuntimeError {
            file: None,
            message: "recv() argument must be a channel".to_string(),
        }),
    }
}

/// Make slice: make([]T, len) or make([]T, len, cap)
pub fn builtin_make_slice(element_type: TypeId, args: &[RuntimeValue]) -> Result<RuntimeValue> {
    if args.is_empty() {
        return Err(BuluError::RuntimeError {
            file: None,
            message: "make([]T, len) requires length argument".to_string(),
        });
    }

    let len = extract_size_arg(&args[0], "slice length")?;
    let _cap = if args.len() > 1 {
        extract_size_arg(&args[1], "slice capacity")?
    } else {
        len
    };

    // Create slice with default values based on element type
    let default_value = get_default_value_for_type(element_type);
    let slice = vec![default_value; len];
    Ok(RuntimeValue::Slice(slice))
}

/// Make map: make(map[K]V) or make(map[K]V, initialCapacity)
pub fn builtin_make_map(
    key_type: TypeId,
    value_type: TypeId,
    args: &[RuntimeValue],
) -> Result<RuntimeValue> {
    let _initial_capacity = if !args.is_empty() {
        Some(extract_size_arg(&args[0], "map initial capacity")?)
    } else {
        None
    };

    // For now, we use string keys regardless of key_type
    // In a full implementation, we'd support different key types
    let _ = (key_type, value_type);
    let map = std::collections::HashMap::new();
    Ok(RuntimeValue::Map(map))
}

/// Make channel: make(chan T) or make(chan T, capacity)
pub fn builtin_make_chan(element_type: TypeId, args: &[RuntimeValue]) -> Result<RuntimeValue> {
    let capacity = if !args.is_empty() {
        Some(extract_size_arg(&args[0], "channel capacity")?)
    } else {
        None
    };

    // Create channel in the global registry
    let buffer_size = capacity.unwrap_or(0);
    let channel_id = {
        let mut registry = crate::runtime::interpreter::get_global_channel_registry().lock().unwrap();
        let id = registry.create_channel(element_type, buffer_size);

        id
    };
    
    Ok(RuntimeValue::Channel(channel_id))
}

/// Get default value for a type
fn get_default_value_for_type(type_id: TypeId) -> RuntimeValue {
    match type_id {
        TypeId::Int8 => RuntimeValue::Int8(0),
        TypeId::Int16 => RuntimeValue::Int16(0),
        TypeId::Int32 => RuntimeValue::Int32(0),
        TypeId::Int64 => RuntimeValue::Int64(0),
        TypeId::UInt8 => RuntimeValue::UInt8(0),
        TypeId::UInt16 => RuntimeValue::UInt16(0),
        TypeId::UInt32 => RuntimeValue::UInt32(0),
        TypeId::UInt64 => RuntimeValue::UInt64(0),
        TypeId::Float32 => RuntimeValue::Float32(0.0),
        TypeId::Float64 => RuntimeValue::Float64(0.0),
        TypeId::Bool => RuntimeValue::Bool(false),
        TypeId::Char => RuntimeValue::Char('\0'),
        TypeId::String => RuntimeValue::String(String::new()),
        _ => RuntimeValue::Null,
    }
}
