//! Type system implementation for the Bulu language
//!
//! This module provides the complete type system including:
//! - Primitive types (integers, floats, bool, char, string)
//! - Composite types (arrays, slices, maps)
//! - User-defined types (structs, interfaces)
//! - Generic types and type parameters
//! - Type checking and inference
//! - Type casting and conversions

pub mod primitive;
pub mod composite;
pub mod checker;
pub mod casting;
pub mod generics;
pub mod async_types;

pub use primitive::*;
pub use composite::*;
pub use checker::*;
pub use casting::*;
pub use generics::*;
pub use async_types::*;