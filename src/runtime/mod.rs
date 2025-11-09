//! Runtime module for the Bulu language
//! 
//! This module provides the runtime system including
//! garbage collection, concurrency support, built-in functions,
//! and error handling.

pub mod gc;
// pub mod scheduler; // Removed - using new goroutine system
pub mod goroutine;
pub mod netpoller;
pub mod io_state;
pub mod async_executor;
pub mod syscall_thread;
pub mod builtins;
pub mod memory;
pub mod error_handler;
pub mod channels;
pub mod sync;
pub mod promises;
pub mod safety;
pub mod safe_collections;
pub mod interpreter;
pub mod module;
pub mod ast_interpreter;

#[cfg(test)]
mod test_import_export;

// pub use scheduler::Scheduler; // Removed - using new goroutine system
pub use gc::GarbageCollector;
pub use error_handler::{ErrorHandler, RuntimeError, ErrorType, ErrorFormatter};
pub use channels::{Channel, ChannelRegistry, ChannelResult, SendResult};
pub use sync::{Lock, LockRegistry, LockGuard, AtomicOperations, sleep, yield_now, timer};
pub use promises::{PromiseRegistry, RuntimePromise, PromiseState};
pub use safety::{SafetyChecker, SafetyError, SafetyResult, safe_array_get, safe_array_get_mut, 
                 safe_slice, safe_slice_mut, safe_deref, safe_deref_mut, set_max_stack_size, get_max_stack_size};
pub use safe_collections::{SafeArray, SafeSlice, SafeSliceMut, SafeString};
pub use interpreter::Interpreter;
pub use crate::types::primitive::RuntimeValue;
pub use module::{ModuleResolver, Module};
pub use ast_interpreter::{AstInterpreter, Environment};