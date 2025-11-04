//! Composite types implementation for the Bulu language

// use crate::error::{Result, BuluError};
use crate::types::primitive::{TypeId, PrimitiveType};
use std::collections::HashMap;

/// Composite type information with proper type tracking
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CompositeTypeId {
    Array(Box<TypeId>),
    Slice(Box<TypeId>),
    Map(Box<TypeId>, Box<TypeId>), // key type, value type
    Tuple(Vec<TypeId>), // tuple element types
    Struct(StructTypeInfo),
    Interface(InterfaceTypeInfo),
    Channel(ChannelTypeInfo),
    Promise(Box<TypeId>), // result type
}

/// Struct type information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructTypeInfo {
    pub name: String,
    pub fields: Vec<StructField>,
    pub type_params: Vec<String>, // Generic type parameters
}

/// Struct field information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StructField {
    pub name: String,
    pub field_type: TypeId,
}

/// Interface type information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceTypeInfo {
    pub name: String,
    pub methods: Vec<InterfaceMethod>,
    pub type_params: Vec<String>, // Generic type parameters
}

/// Interface method signature
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InterfaceMethod {
    pub name: String,
    pub param_types: Vec<TypeId>,
    pub return_type: Option<TypeId>,
}

/// Channel type information
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChannelTypeInfo {
    pub element_type: TypeId,
    pub direction: ChannelDirection,
    pub buffered: bool,
    pub capacity: Option<usize>,
}

/// Channel direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelDirection {
    Bidirectional, // chan T
    SendOnly,      // chan<- T
    ReceiveOnly,   // <-chan T
}

/// Type registry for managing composite types
#[derive(Debug, Default)]
pub struct TypeRegistry {
    /// Maps composite type IDs to unique integers for TypeId enum
    composite_types: HashMap<CompositeTypeId, u32>,
    /// Reverse mapping from integers to composite types
    type_lookup: HashMap<u32, CompositeTypeId>,
    /// Next available ID
    next_id: u32,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            composite_types: HashMap::new(),
            type_lookup: HashMap::new(),
            next_id: 1, // Start from 1, 0 is reserved
        }
    }

    /// Register a composite type and get its ID
    pub fn register_composite_type(&mut self, composite_type: CompositeTypeId) -> u32 {
        if let Some(&id) = self.composite_types.get(&composite_type) {
            return id;
        }

        let id = self.next_id;
        self.next_id += 1;
        
        self.composite_types.insert(composite_type.clone(), id);
        self.type_lookup.insert(id, composite_type);
        
        id
    }

    /// Get composite type by ID
    pub fn get_composite_type(&self, id: u32) -> Option<&CompositeTypeId> {
        self.type_lookup.get(&id)
    }

    /// Register an array type
    pub fn register_array_type(&mut self, element_type: TypeId) -> u32 {
        let composite_type = CompositeTypeId::Array(Box::new(element_type));
        self.register_composite_type(composite_type)
    }

    /// Register a slice type
    pub fn register_slice_type(&mut self, element_type: TypeId) -> u32 {
        let composite_type = CompositeTypeId::Slice(Box::new(element_type));
        self.register_composite_type(composite_type)
    }

    /// Register a map type
    pub fn register_map_type(&mut self, key_type: TypeId, value_type: TypeId) -> u32 {
        let composite_type = CompositeTypeId::Map(Box::new(key_type), Box::new(value_type));
        self.register_composite_type(composite_type)
    }

    /// Register a tuple type
    pub fn register_tuple_type(&mut self, element_types: Vec<TypeId>) -> u32 {
        let composite_type = CompositeTypeId::Tuple(element_types);
        self.register_composite_type(composite_type)
    }

    /// Register a struct type
    pub fn register_struct_type(&mut self, struct_info: StructTypeInfo) -> u32 {
        let composite_type = CompositeTypeId::Struct(struct_info);
        self.register_composite_type(composite_type)
    }

    /// Register an interface type
    pub fn register_interface_type(&mut self, interface_info: InterfaceTypeInfo) -> u32 {
        let composite_type = CompositeTypeId::Interface(interface_info);
        self.register_composite_type(composite_type)
    }

    /// Register a channel type
    pub fn register_channel_type(&mut self, channel_info: ChannelTypeInfo) -> u32 {
        let composite_type = CompositeTypeId::Channel(channel_info);
        self.register_composite_type(composite_type)
    }

    /// Register a promise type
    pub fn register_promise_type(&mut self, result_type: TypeId) -> u32 {
        let composite_type = CompositeTypeId::Promise(Box::new(result_type));
        self.register_composite_type(composite_type)
    }

    /// Get the element type of an array or slice
    pub fn get_element_type(&self, type_id: TypeId) -> Option<TypeId> {
        match type_id {
            TypeId::Array(id) | TypeId::Slice(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    match composite_type {
                        CompositeTypeId::Array(element_type) | CompositeTypeId::Slice(element_type) => {
                            Some(**element_type)
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get the key and value types of a map
    pub fn get_map_types(&self, type_id: TypeId) -> Option<(TypeId, TypeId)> {
        match type_id {
            TypeId::Map(composite_id) => {
                if let Some(composite_type) = self.get_composite_type(composite_id) {
                    if let CompositeTypeId::Map(key_type, value_type) = composite_type {
                        Some((**key_type, **value_type))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get struct information by type ID
    pub fn get_struct_info(&self, type_id: TypeId) -> Option<&StructTypeInfo> {
        match type_id {
            TypeId::Struct(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Struct(struct_info) = composite_type {
                        Some(struct_info)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get interface information by type ID
    pub fn get_interface_info(&self, type_id: TypeId) -> Option<&InterfaceTypeInfo> {
        match type_id {
            TypeId::Interface(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Interface(interface_info) = composite_type {
                        Some(interface_info)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Get channel information by type ID
    pub fn get_channel_info(&self, type_id: TypeId) -> Option<&ChannelTypeInfo> {
        match type_id {
            TypeId::Channel(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Channel(channel_info) = composite_type {
                        Some(channel_info)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Check if a struct has a specific field
    pub fn struct_has_field(&self, type_id: TypeId, field_name: &str) -> bool {
        if let Some(struct_info) = self.get_struct_info(type_id) {
            struct_info.fields.iter().any(|field| field.name == field_name)
        } else {
            false
        }
    }

    /// Get the type of a struct field
    pub fn get_struct_field_type(&self, type_id: TypeId, field_name: &str) -> Option<TypeId> {
        if let Some(struct_info) = self.get_struct_info(type_id) {
            struct_info.fields.iter()
                .find(|field| field.name == field_name)
                .map(|field| field.field_type)
        } else {
            None
        }
    }

    /// Check if a type implements an interface (duck typing)
    pub fn implements_interface(&self, type_id: TypeId, interface_id: TypeId) -> bool {
        if let Some(_interface_info) = self.get_interface_info(interface_id) {
            // For now, we'll implement a simple check
            // In a full implementation, this would check if the type has all required methods
            match type_id {
                TypeId::Struct(_) => {
                    // Check if struct has all required methods
                    // This is a simplified implementation
                    true // TODO: Implement proper method checking
                }
                _ => false,
            }
        } else {
            false
        }
    }

    /// Get a human-readable name for a composite type
    pub fn get_type_name(&self, type_id: TypeId) -> String {
        match type_id {
            TypeId::Array(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Array(element_type) = composite_type {
                        format!("[]{}", self.get_type_name(**element_type))
                    } else {
                        "array".to_string()
                    }
                } else {
                    "array".to_string()
                }
            }
            TypeId::Slice(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Slice(element_type) = composite_type {
                        format!("[]{}", self.get_type_name(**element_type))
                    } else {
                        "slice".to_string()
                    }
                } else {
                    "slice".to_string()
                }
            }
            TypeId::Map(composite_id) => {
                if let Some(composite_type) = self.get_composite_type(composite_id) {
                    if let CompositeTypeId::Map(key_type, value_type) = composite_type {
                        format!("map[{}]{}", 
                            self.get_type_name(**key_type),
                            self.get_type_name(**value_type))
                    } else {
                        "map".to_string()
                    }
                } else {
                    "map".to_string()
                }
            }
            TypeId::Struct(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Struct(struct_info) = composite_type {
                        struct_info.name.clone()
                    } else {
                        "struct".to_string()
                    }
                } else {
                    "struct".to_string()
                }
            }
            TypeId::Interface(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Interface(interface_info) = composite_type {
                        interface_info.name.clone()
                    } else {
                        "interface".to_string()
                    }
                } else {
                    "interface".to_string()
                }
            }
            TypeId::Channel(id) => {
                if let Some(composite_type) = self.get_composite_type(id) {
                    if let CompositeTypeId::Channel(channel_info) = composite_type {
                        match channel_info.direction {
                            ChannelDirection::Bidirectional => {
                                format!("chan {}", self.get_type_name(channel_info.element_type))
                            }
                            ChannelDirection::SendOnly => {
                                format!("chan<- {}", self.get_type_name(channel_info.element_type))
                            }
                            ChannelDirection::ReceiveOnly => {
                                format!("<-chan {}", self.get_type_name(channel_info.element_type))
                            }
                        }
                    } else {
                        "channel".to_string()
                    }
                } else {
                    "channel".to_string()
                }
            }
            _ => PrimitiveType::type_name(type_id).to_string(),
        }
    }
}

/// Utility functions for working with composite types
impl CompositeTypeId {
    /// Check if this composite type is assignable to another
    pub fn is_assignable_to(&self, other: &CompositeTypeId) -> bool {
        match (self, other) {
            // Same array types
            (CompositeTypeId::Array(a), CompositeTypeId::Array(b)) => {
                PrimitiveType::is_assignable(**a, **b)
            }
            // Same slice types
            (CompositeTypeId::Slice(a), CompositeTypeId::Slice(b)) => {
                PrimitiveType::is_assignable(**a, **b)
            }
            // Array to slice (if element types are compatible)
            (CompositeTypeId::Array(a), CompositeTypeId::Slice(b)) => {
                PrimitiveType::is_assignable(**a, **b)
            }
            // Same map types
            (CompositeTypeId::Map(k1, v1), CompositeTypeId::Map(k2, v2)) => {
                PrimitiveType::is_assignable(**k1, **k2) && PrimitiveType::is_assignable(**v1, **v2)
            }
            // Same struct types
            (CompositeTypeId::Struct(s1), CompositeTypeId::Struct(s2)) => {
                s1.name == s2.name && s1.type_params == s2.type_params
            }
            // Same interface types
            (CompositeTypeId::Interface(i1), CompositeTypeId::Interface(i2)) => {
                i1.name == i2.name && i1.type_params == i2.type_params
            }
            // Struct to interface (duck typing)
            (CompositeTypeId::Struct(_), CompositeTypeId::Interface(_)) => {
                // TODO: Implement proper duck typing check
                true // For now, assume all structs can implement any interface
            }
            // Channel type compatibility
            (CompositeTypeId::Channel(c1), CompositeTypeId::Channel(c2)) => {
                // Element types must be compatible
                if !PrimitiveType::is_assignable(c1.element_type, c2.element_type) {
                    return false;
                }
                
                // Direction compatibility
                match (c1.direction, c2.direction) {
                    // Same direction is always compatible
                    (a, b) if a == b => true,
                    // Bidirectional can be assigned to send-only or receive-only
                    (ChannelDirection::Bidirectional, _) => true,
                    // Send-only and receive-only cannot be assigned to bidirectional
                    (_, ChannelDirection::Bidirectional) => false,
                    // Send-only and receive-only are not compatible with each other
                    _ => false,
                }
            }
            _ => false,
        }
    }
}