//! Async types implementation for the Bulu language
//!
//! This module provides Promise/Future types and async function type handling.

use crate::ast::nodes::PromiseType;
use crate::lexer::token::Position;
use std::fmt;

/// Promise state for runtime representation
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState<T> {
    /// Promise is still pending
    Pending,
    /// Promise has resolved with a value
    Resolved(T),
    /// Promise has been rejected with an error
    Rejected(String),
}

/// Runtime representation of a Promise
pub struct Promise<T> {
    /// Current state of the promise
    pub state: PromiseState<T>,
    /// Callbacks to execute when promise resolves
    pub then_callbacks: Vec<Box<dyn Fn(T) -> T>>,
    /// Callbacks to execute when promise rejects
    pub catch_callbacks: Vec<Box<dyn Fn(String) -> T>>,
}

impl<T> fmt::Debug for Promise<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Promise")
            .field("state", &self.state)
            .field("then_callbacks", &format!("{} callbacks", self.then_callbacks.len()))
            .field("catch_callbacks", &format!("{} callbacks", self.catch_callbacks.len()))
            .finish()
    }
}

impl<T> Promise<T> {
    /// Create a new pending promise
    pub fn new() -> Self {
        Self {
            state: PromiseState::Pending,
            then_callbacks: Vec::new(),
            catch_callbacks: Vec::new(),
        }
    }

    /// Create a resolved promise with a value
    pub fn resolved(value: T) -> Self {
        Self {
            state: PromiseState::Resolved(value),
            then_callbacks: Vec::new(),
            catch_callbacks: Vec::new(),
        }
    }

    /// Create a rejected promise with an error
    pub fn rejected(error: String) -> Self {
        Self {
            state: PromiseState::Rejected(error),
            then_callbacks: Vec::new(),
            catch_callbacks: Vec::new(),
        }
    }

    /// Check if the promise is resolved
    pub fn is_resolved(&self) -> bool {
        matches!(self.state, PromiseState::Resolved(_))
    }

    /// Check if the promise is rejected
    pub fn is_rejected(&self) -> bool {
        matches!(self.state, PromiseState::Rejected(_))
    }

    /// Check if the promise is pending
    pub fn is_pending(&self) -> bool {
        matches!(self.state, PromiseState::Pending)
    }
}

impl<T: Clone> Promise<T> {
    /// Resolve the promise with a value
    pub fn resolve(&mut self, value: T) {
        if matches!(self.state, PromiseState::Pending) {
            self.state = PromiseState::Resolved(value.clone());
            
            // Execute all then callbacks
            for callback in &self.then_callbacks {
                callback(value.clone());
            }
        }
    }

    /// Reject the promise with an error
    pub fn reject(&mut self, error: String) {
        if matches!(self.state, PromiseState::Pending) {
            self.state = PromiseState::Rejected(error.clone());
            
            // Execute all catch callbacks
            for callback in &self.catch_callbacks {
                callback(error.clone());
            }
        }
    }
}

impl<T> Default for Promise<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> fmt::Display for PromiseState<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PromiseState::Pending => write!(f, "Pending"),
            PromiseState::Resolved(value) => write!(f, "Resolved({})", value),
            PromiseState::Rejected(error) => write!(f, "Rejected({})", error),
        }
    }
}

/// Utility functions for working with async types
pub mod utils {
    use super::*;
    use crate::ast::nodes::{FunctionType, Type};

    /// Check if a type is a Promise type
    pub fn is_promise_type(ty: &Type) -> bool {
        matches!(ty, Type::Promise(_))
    }

    /// Extract the result type from a Promise type
    pub fn get_promise_result_type(ty: &Type) -> Option<&Type> {
        match ty {
            Type::Promise(promise_type) => Some(&promise_type.result_type),
            _ => None,
        }
    }

    /// Create a Promise type from a result type
    pub fn make_promise_type(result_type: Type, position: Position) -> Type {
        Type::Promise(PromiseType {
            result_type: Box::new(result_type),
            position,
        })
    }

    /// Convert an async function type to return a Promise
    pub fn make_async_function_type(func_type: &FunctionType) -> FunctionType {
        let return_type = match &func_type.return_type {
            Some(ret_type) => {
                // Wrap the return type in a Promise
                Some(Box::new(Type::Promise(PromiseType {
                    result_type: ret_type.clone(),
                    position: Position::new(0, 0, 0), // TODO: Use actual position
                })))
            }
            None => {
                // Async function with no explicit return type returns Promise<void>
                Some(Box::new(Type::Promise(PromiseType {
                    result_type: Box::new(Type::Void),
                    position: Position::new(0, 0, 0), // TODO: Use actual position
                })))
            }
        };

        FunctionType {
            param_types: func_type.param_types.clone(),
            return_type,
            is_async: true,
        }
    }

    /// Check if a function type is async
    pub fn is_async_function_type(func_type: &FunctionType) -> bool {
        func_type.is_async
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::token::Position;
    use crate::ast::nodes::{Type, FunctionType};

    #[test]
    fn test_promise_creation() {
        let promise: Promise<i32> = Promise::new();
        assert!(promise.is_pending());
        assert!(!promise.is_resolved());
        assert!(!promise.is_rejected());
    }

    #[test]
    fn test_promise_resolve() {
        let mut promise = Promise::new();
        promise.resolve(42);
        
        assert!(promise.is_resolved());
        assert!(!promise.is_pending());
        assert!(!promise.is_rejected());
        
        match promise.state {
            PromiseState::Resolved(value) => assert_eq!(value, 42),
            _ => panic!("Expected resolved state"),
        }
    }

    #[test]
    fn test_promise_reject() {
        let mut promise: Promise<i32> = Promise::new();
        promise.reject("Error occurred".to_string());
        
        assert!(promise.is_rejected());
        assert!(!promise.is_pending());
        assert!(!promise.is_resolved());
        
        match promise.state {
            PromiseState::Rejected(error) => assert_eq!(error, "Error occurred"),
            _ => panic!("Expected rejected state"),
        }
    }

    #[test]
    fn test_promise_type_creation() {
        let pos = Position::new(1, 1, 0);
        let promise_type = PromiseType {
            result_type: Box::new(Type::Int32),
            position: pos,
        };
        
        assert_eq!(*promise_type.result_type, Type::Int32);
        assert_eq!(promise_type.position, pos);
    }

    #[test]
    fn test_promise_type_utils() {
        let pos = Position::new(1, 1, 0);
        let promise_type = utils::make_promise_type(Type::String, pos);
        
        assert!(utils::is_promise_type(&promise_type));
        
        let result_type = utils::get_promise_result_type(&promise_type);
        assert_eq!(result_type, Some(&Type::String));
    }

    #[test]
    fn test_async_function_type_conversion() {
        let sync_func_type = FunctionType {
            param_types: vec![Type::Int32, Type::String],
            return_type: Some(Box::new(Type::Bool)),
            is_async: false,
        };

        let async_func_type = utils::make_async_function_type(&sync_func_type);
        
        assert!(async_func_type.is_async);
        assert_eq!(async_func_type.param_types, sync_func_type.param_types);
        
        // Check that return type is wrapped in Promise
        match async_func_type.return_type {
            Some(ret_type) => {
                match ret_type.as_ref() {
                    Type::Promise(promise_type) => {
                        assert_eq!(*promise_type.result_type, Type::Bool);
                    }
                    _ => panic!("Expected Promise return type"),
                }
            }
            None => panic!("Expected return type"),
        }
    }
}