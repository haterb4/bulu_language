// I/O state machine for resumable operations
// This allows goroutines to be parked and resumed

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::types::primitive::RuntimeValue;

/// State of an I/O operation that can be resumed
#[derive(Debug, Clone)]
pub enum IoOperationState {
    /// TCP accept operation
    TcpAccept {
        listener_id: String,
        attempt: u32,
    },
    /// TCP read operation
    TcpRead {
        connection_id: String,
        buffer_size: usize,
        attempt: u32,
    },
    /// TCP write operation
    TcpWrite {
        connection_id: String,
        data: Vec<u8>,
        bytes_written: usize,
        attempt: u32,
    },
}

/// Registry of pending I/O operations for goroutines
pub struct IoStateRegistry {
    states: HashMap<u64, IoOperationState>, // goroutine_id -> state
}

impl IoStateRegistry {
    pub fn new() -> Self {
        Self {
            states: HashMap::new(),
        }
    }
    
    pub fn register(&mut self, goroutine_id: u64, state: IoOperationState) {
        self.states.insert(goroutine_id, state);
    }
    
    pub fn get(&self, goroutine_id: u64) -> Option<&IoOperationState> {
        self.states.get(&goroutine_id)
    }
    
    pub fn remove(&mut self, goroutine_id: u64) -> Option<IoOperationState> {
        self.states.remove(&goroutine_id)
    }
}

// Global I/O state registry
static mut GLOBAL_IO_REGISTRY: Option<Arc<Mutex<IoStateRegistry>>> = None;
static IO_REGISTRY_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global I/O state registry
pub fn init_io_registry() {
    IO_REGISTRY_INIT.call_once(|| {
        unsafe {
            GLOBAL_IO_REGISTRY = Some(Arc::new(Mutex::new(IoStateRegistry::new())));
        }
    });
}

/// Get the global I/O state registry
pub fn get_io_registry() -> Option<Arc<Mutex<IoStateRegistry>>> {
    unsafe { GLOBAL_IO_REGISTRY.as_ref().map(Arc::clone) }
}

/// Register an I/O operation for a goroutine
pub fn register_io_operation(goroutine_id: u64, state: IoOperationState) {
    init_io_registry();
    if let Some(registry) = get_io_registry() {
        if let Ok(mut reg) = registry.lock() {
            reg.register(goroutine_id, state);
        }
    }
}

/// Get the I/O operation state for a goroutine
pub fn get_io_operation(goroutine_id: u64) -> Option<IoOperationState> {
    if let Some(registry) = get_io_registry() {
        if let Ok(reg) = registry.lock() {
            return reg.get(goroutine_id).cloned();
        }
    }
    None
}

/// Remove and return the I/O operation state for a goroutine
pub fn remove_io_operation(goroutine_id: u64) -> Option<IoOperationState> {
    if let Some(registry) = get_io_registry() {
        if let Ok(mut reg) = registry.lock() {
            return reg.remove(goroutine_id);
        }
    }
    None
}
