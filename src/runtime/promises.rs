//! Promise runtime implementation for the Bulu language
//!
//! This module provides the runtime support for Promise/Future types and async operations.

use crate::types::primitive::RuntimeValue as Value;
use std::collections::HashMap;

/// Promise state for runtime representation
#[derive(Debug, Clone, PartialEq)]
pub enum PromiseState {
    /// Promise is still pending
    Pending,
    /// Promise has resolved with a value
    Resolved(Value),
    /// Promise has been rejected with an error
    Rejected(String),
}

/// Runtime representation of a Promise
pub struct RuntimePromise {
    /// Unique identifier for this promise
    pub id: usize,
    /// Current state of the promise
    pub state: PromiseState,
    /// Callbacks to execute when promise resolves
    pub then_callbacks: Vec<Box<dyn Fn(Value) -> Value + Send + Sync>>,
    /// Callbacks to execute when promise rejects
    pub catch_callbacks: Vec<Box<dyn Fn(String) -> Value + Send + Sync>>,
}

impl std::fmt::Debug for RuntimePromise {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimePromise")
            .field("id", &self.id)
            .field("state", &self.state)
            .field(
                "then_callbacks",
                &format!("{} callbacks", self.then_callbacks.len()),
            )
            .field(
                "catch_callbacks",
                &format!("{} callbacks", self.catch_callbacks.len()),
            )
            .finish()
    }
}

impl Clone for RuntimePromise {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            state: self.state.clone(),
            then_callbacks: Vec::new(),  // Don't clone callbacks
            catch_callbacks: Vec::new(), // Don't clone callbacks
        }
    }
}

impl RuntimePromise {
    /// Create a new pending promise
    pub fn new(id: usize) -> Self {
        Self {
            id,
            state: PromiseState::Pending,
            then_callbacks: Vec::new(),
            catch_callbacks: Vec::new(),
        }
    }

    /// Create a resolved promise with a value
    pub fn resolved(id: usize, value: Value) -> Self {
        Self {
            id,
            state: PromiseState::Resolved(value),
            then_callbacks: Vec::new(),
            catch_callbacks: Vec::new(),
        }
    }

    /// Create a rejected promise with an error
    pub fn rejected(id: usize, error: String) -> Self {
        Self {
            id,
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

    /// Resolve the promise with a value
    pub fn resolve(&mut self, value: Value) {
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

    /// Get the resolved value if the promise is resolved
    pub fn get_value(&self) -> Option<&Value> {
        match &self.state {
            PromiseState::Resolved(value) => Some(value),
            _ => None,
        }
    }

    /// Get the error message if the promise is rejected
    pub fn get_error(&self) -> Option<&String> {
        match &self.state {
            PromiseState::Rejected(error) => Some(error),
            _ => None,
        }
    }
}

/// Promise registry for managing runtime promises
#[derive(Debug)]
pub struct PromiseRegistry {
    /// Map of promise ID to promise
    promises: HashMap<usize, RuntimePromise>,
    /// Next available promise ID
    next_id: usize,
}

impl PromiseRegistry {
    /// Create a new promise registry
    pub fn new() -> Self {
        Self {
            promises: HashMap::new(),
            next_id: 1,
        }
    }

    /// Create a new pending promise
    pub fn create_promise(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let promise = RuntimePromise::new(id);
        self.promises.insert(id, promise);

        id
    }

    /// Create a resolved promise
    pub fn create_resolved_promise(&mut self, value: Value) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let promise = RuntimePromise::resolved(id, value);
        self.promises.insert(id, promise);

        id
    }

    /// Create a rejected promise
    pub fn create_rejected_promise(&mut self, error: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;

        let promise = RuntimePromise::rejected(id, error);
        self.promises.insert(id, promise);

        id
    }

    /// Get a promise by ID
    pub fn get_promise(&self, id: usize) -> Option<&RuntimePromise> {
        self.promises.get(&id)
    }

    /// Get a mutable promise by ID
    pub fn get_promise_mut(&mut self, id: usize) -> Option<&mut RuntimePromise> {
        self.promises.get_mut(&id)
    }

    /// Resolve a promise with a value
    pub fn resolve_promise(&mut self, id: usize, value: Value) -> Result<(), String> {
        if let Some(promise) = self.promises.get_mut(&id) {
            promise.resolve(value);
            Ok(())
        } else {
            Err(format!("Promise with ID {} not found", id))
        }
    }

    /// Reject a promise with an error
    pub fn reject_promise(&mut self, id: usize, error: String) -> Result<(), String> {
        if let Some(promise) = self.promises.get_mut(&id) {
            promise.reject(error);
            Ok(())
        } else {
            Err(format!("Promise with ID {} not found", id))
        }
    }

    /// Check if a promise exists
    pub fn has_promise(&self, id: usize) -> bool {
        self.promises.contains_key(&id)
    }

    /// Remove a promise from the registry
    pub fn remove_promise(&mut self, id: usize) -> Option<RuntimePromise> {
        self.promises.remove(&id)
    }

    /// Get the number of promises in the registry
    pub fn len(&self) -> usize {
        self.promises.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.promises.is_empty()
    }
}

impl Default for PromiseRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Promise utility functions
pub mod utils {
    use super::*;

    /// Create a Promise.all equivalent that waits for all promises to resolve
    pub fn promise_all(
        registry: &PromiseRegistry,
        promise_ids: &[usize],
    ) -> Result<Vec<Value>, String> {
        let mut results = Vec::new();

        for &id in promise_ids {
            if let Some(promise) = registry.get_promise(id) {
                match &promise.state {
                    PromiseState::Resolved(value) => results.push(value.clone()),
                    PromiseState::Rejected(error) => return Err(error.clone()),
                    PromiseState::Pending => return Err("Promise is still pending".to_string()),
                }
            } else {
                return Err(format!("Promise with ID {} not found", id));
            }
        }

        Ok(results)
    }

    /// Create a Promise.race equivalent that returns the first resolved/rejected promise
    pub fn promise_race(
        registry: &PromiseRegistry,
        promise_ids: &[usize],
    ) -> Result<Value, String> {
        for &id in promise_ids {
            if let Some(promise) = registry.get_promise(id) {
                match &promise.state {
                    PromiseState::Resolved(value) => return Ok(value.clone()),
                    PromiseState::Rejected(error) => return Err(error.clone()),
                    PromiseState::Pending => continue,
                }
            }
        }

        Err("All promises are still pending".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_promise_creation() {
        let mut registry = PromiseRegistry::new();
        let id = registry.create_promise();

        assert_eq!(id, 1);
        assert!(registry.has_promise(id));

        let promise = registry.get_promise(id).unwrap();
        assert!(promise.is_pending());
        assert!(!promise.is_resolved());
        assert!(!promise.is_rejected());
    }

    #[test]
    fn test_promise_resolve() {
        let mut registry = PromiseRegistry::new();
        let id = registry.create_promise();

        let value = Value::Int64(42);
        registry.resolve_promise(id, value.clone()).unwrap();

        let promise = registry.get_promise(id).unwrap();
        assert!(promise.is_resolved());
        assert_eq!(promise.get_value(), Some(&value));
    }

    #[test]
    fn test_promise_reject() {
        let mut registry = PromiseRegistry::new();
        let id = registry.create_promise();

        let error = "Something went wrong".to_string();
        registry.reject_promise(id, error.clone()).unwrap();

        let promise = registry.get_promise(id).unwrap();
        assert!(promise.is_rejected());
        assert_eq!(promise.get_error(), Some(&error));
    }

    #[test]
    fn test_resolved_promise_creation() {
        let mut registry = PromiseRegistry::new();
        let value = Value::String("Hello".to_string());
        let id = registry.create_resolved_promise(value.clone());

        let promise = registry.get_promise(id).unwrap();
        assert!(promise.is_resolved());
        assert_eq!(promise.get_value(), Some(&value));
    }

    #[test]
    fn test_rejected_promise_creation() {
        let mut registry = PromiseRegistry::new();
        let error = "Error occurred".to_string();
        let id = registry.create_rejected_promise(error.clone());

        let promise = registry.get_promise(id).unwrap();
        assert!(promise.is_rejected());
        assert_eq!(promise.get_error(), Some(&error));
    }

    #[test]
    fn test_promise_all_success() {
        let mut registry = PromiseRegistry::new();

        let id1 = registry.create_resolved_promise(Value::Int64(1));
        let id2 = registry.create_resolved_promise(Value::Int64(2));
        let id3 = registry.create_resolved_promise(Value::Int64(3));

        let result = utils::promise_all(&registry, &[id1, id2, id3]).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], Value::Int64(1));
        assert_eq!(result[1], Value::Int64(2));
        assert_eq!(result[2], Value::Int64(3));
    }

    #[test]
    fn test_promise_all_failure() {
        let mut registry = PromiseRegistry::new();

        let id1 = registry.create_resolved_promise(Value::Int64(1));
        let id2 = registry.create_rejected_promise("Error".to_string());
        let id3 = registry.create_resolved_promise(Value::Int64(3));

        let result = utils::promise_all(&registry, &[id1, id2, id3]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Error");
    }
}
