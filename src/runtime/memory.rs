//! Memory management with escape analysis and stack allocation
//!
//! This module provides memory management functionality including:
//! - Stack vs heap allocation decisions
//! - Escape analysis integration
//! - Memory layout optimization
//! - Integration with garbage collector

use crate::runtime::gc::{GarbageCollector, ObjectId};
use crate::runtime::safety::{SafetyChecker, SafetyResult};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Memory allocation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocStrategy {
    /// Allocate on stack (for small, non-escaping values)
    Stack,
    /// Allocate on heap (for large values or escaping values)
    Heap,
}

/// Escape analysis result
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeAnalysis {
    /// Value does not escape current scope
    NoEscape,
    /// Value escapes to parent scope
    EscapeToParent,
    /// Value escapes to heap (returned, stored in heap object, etc.)
    EscapeToHeap,
}

/// Memory layout information
#[derive(Debug, Clone)]
pub struct MemoryLayout {
    /// Size in bytes
    pub size: usize,
    /// Alignment requirement
    pub alignment: usize,
    /// Whether the type contains references
    pub contains_references: bool,
    /// Allocation strategy
    pub strategy: AllocStrategy,
}

/// Stack frame for local allocations
#[derive(Debug)]
pub struct StackFrame {
    /// Frame identifier
    pub frame_id: usize,
    /// Allocated objects in this frame
    pub allocations: Vec<StackAllocation>,
    /// Total allocated bytes
    pub allocated_bytes: usize,
}

/// Stack allocation record
#[derive(Debug)]
pub struct StackAllocation {
    /// Allocation size
    pub size: usize,
    /// Type identifier
    pub type_id: u32,
    /// Pointer to allocated memory
    pub ptr: *mut u8,
}

/// Memory manager with escape analysis and GC integration
pub struct MemoryManager {
    /// Garbage collector
    gc: Arc<Mutex<GarbageCollector>>,
    /// Stack frames
    stack_frames: Vec<StackFrame>,
    /// Next frame ID
    next_frame_id: usize,
    /// Type size cache
    type_sizes: HashMap<u32, MemoryLayout>,
    /// Escape analysis cache
    escape_cache: HashMap<(u32, usize), EscapeAnalysis>, // (type_id, scope_depth)
    /// Memory safety checker
    safety_checker: SafetyChecker,
}

impl MemoryManager {
    /// Create a new memory manager
    pub fn new() -> Self {
        Self {
            gc: Arc::new(Mutex::new(GarbageCollector::new())),
            stack_frames: Vec::new(),
            next_frame_id: 0,
            type_sizes: HashMap::new(),
            escape_cache: HashMap::new(),
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Create memory manager with custom GC
    pub fn with_gc(gc: GarbageCollector) -> Self {
        Self {
            gc: Arc::new(Mutex::new(gc)),
            stack_frames: Vec::new(),
            next_frame_id: 0,
            type_sizes: HashMap::new(),
            escape_cache: HashMap::new(),
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Register type layout information
    pub fn register_type_layout(&mut self, type_id: u32, layout: MemoryLayout) {
        self.type_sizes.insert(type_id, layout);
    }

    /// Get type layout
    pub fn get_type_layout(&self, type_id: u32) -> Option<&MemoryLayout> {
        self.type_sizes.get(&type_id)
    }

    /// Perform escape analysis for a type in a given scope
    pub fn analyze_escape(&mut self, type_id: u32, scope_depth: usize, context: &EscapeContext) -> EscapeAnalysis {
        // For now, don't cache based on context since different contexts can have different results
        // In a real implementation, we'd need a more sophisticated cache key
        let result = self.perform_escape_analysis(type_id, scope_depth, context);
        result
    }

    /// Perform actual escape analysis
    fn perform_escape_analysis(&self, type_id: u32, _scope_depth: usize, context: &EscapeContext) -> EscapeAnalysis {
        // Get type layout
        let layout = match self.type_sizes.get(&type_id) {
            Some(layout) => layout,
            None => return EscapeAnalysis::EscapeToHeap, // Conservative default
        };

        // Large objects always escape to heap
        if layout.size > 128 {
            return EscapeAnalysis::EscapeToHeap;
        }

        // Check escape conditions
        match context {
            EscapeContext::LocalVariable => {
                // Local variables don't escape unless they contain references
                if layout.contains_references {
                    EscapeAnalysis::EscapeToParent
                } else {
                    EscapeAnalysis::NoEscape
                }
            }
            EscapeContext::FunctionReturn => {
                // Returned values escape to caller
                EscapeAnalysis::EscapeToHeap
            }
            EscapeContext::HeapStore => {
                // Values stored in heap objects escape
                EscapeAnalysis::EscapeToHeap
            }
            EscapeContext::ClosureCapture => {
                // Captured by closure - escapes to heap
                EscapeAnalysis::EscapeToHeap
            }
            EscapeContext::ChannelSend => {
                // Sent through channel - escapes to heap
                EscapeAnalysis::EscapeToHeap
            }
        }
    }

    /// Allocate memory based on escape analysis
    pub fn allocate(&mut self, type_id: u32, context: &EscapeContext) -> Result<AllocationResult, String> {
        let layout = self.type_sizes.get(&type_id)
            .ok_or_else(|| format!("Unknown type ID: {}", type_id))?
            .clone();

        let scope_depth = self.stack_frames.len();
        let escape_analysis = self.analyze_escape(type_id, scope_depth, context);

        match (layout.strategy, escape_analysis) {
            (AllocStrategy::Stack, EscapeAnalysis::NoEscape) => {
                self.allocate_on_stack(layout.size, type_id)
            }
            _ => {
                self.allocate_on_heap(layout.size, type_id)
            }
        }
    }

    /// Allocate on stack
    fn allocate_on_stack(&mut self, size: usize, type_id: u32) -> Result<AllocationResult, String> {
        // Ensure we have a current stack frame
        if self.stack_frames.is_empty() {
            return Err("No active stack frame for allocation".to_string());
        }

        // Allocate memory (simplified - in reality would use proper stack allocation)
        let ptr = unsafe {
            let layout = std::alloc::Layout::from_size_align(size, 8)
                .map_err(|_| "Invalid allocation layout")?;
            std::alloc::alloc(layout)
        };

        if ptr.is_null() {
            return Err("Stack allocation failed".to_string());
        }

        // Record allocation in current frame
        let frame = self.stack_frames.last_mut().unwrap();
        frame.allocations.push(StackAllocation {
            size,
            type_id,
            ptr,
        });
        frame.allocated_bytes += size;

        Ok(AllocationResult::Stack(ptr))
    }

    /// Allocate on heap via GC
    fn allocate_on_heap(&self, size: usize, type_id: u32) -> Result<AllocationResult, String> {
        let gc = self.gc.lock().unwrap();
        let object_id = gc.allocate(size, type_id)?;
        Ok(AllocationResult::Heap(object_id))
    }

    /// Enter a new scope (create stack frame)
    pub fn enter_scope(&mut self) -> usize {
        let frame_id = self.next_frame_id;
        self.next_frame_id += 1;

        let frame = StackFrame {
            frame_id,
            allocations: Vec::new(),
            allocated_bytes: 0,
        };

        self.stack_frames.push(frame);
        frame_id
    }

    /// Exit scope (cleanup stack frame)
    pub fn exit_scope(&mut self) -> Result<(), String> {
        if let Some(frame) = self.stack_frames.pop() {
            // Deallocate all stack allocations in this frame
            for allocation in frame.allocations {
                unsafe {
                    let layout = std::alloc::Layout::from_size_align(allocation.size, 8)
                        .map_err(|_| "Invalid deallocation layout")?;
                    std::alloc::dealloc(allocation.ptr, layout);
                }
            }
            Ok(())
        } else {
            Err("No active scope to exit".to_string())
        }
    }

    /// Get current scope depth
    pub fn scope_depth(&self) -> usize {
        self.stack_frames.len()
    }

    /// Get memory statistics
    pub fn get_memory_stats(&self) -> MemoryStats {
        let stack_bytes: usize = self.stack_frames.iter()
            .map(|frame| frame.allocated_bytes)
            .sum();

        let (heap_used, heap_total) = {
            let gc = self.gc.lock().unwrap();
            gc.heap_usage()
        };

        MemoryStats {
            stack_bytes,
            heap_used,
            heap_total,
            stack_frames: self.stack_frames.len(),
            gc_stats: {
                let gc = self.gc.lock().unwrap();
                gc.get_stats()
            },
        }
    }

    /// Force garbage collection
    pub fn force_gc(&self) {
        let gc = self.gc.lock().unwrap();
        gc.force_collect();
    }

    /// Get garbage collector reference
    pub fn gc(&self) -> Arc<Mutex<GarbageCollector>> {
        Arc::clone(&self.gc)
    }

    /// Check array bounds access
    pub fn check_array_bounds(&self, index: usize, length: usize, type_name: &str) -> SafetyResult<()> {
        self.safety_checker.check_bounds(index, length, type_name)
    }

    /// Check slice bounds access
    pub fn check_slice_bounds(&self, start: usize, end: usize, length: usize, type_name: &str) -> SafetyResult<()> {
        self.safety_checker.check_slice_bounds(start, end, length, type_name)
    }

    /// Check for null pointer dereference
    pub fn check_null_pointer<T>(&self, ptr: *const T, operation: &str, location: &str) -> SafetyResult<()> {
        self.safety_checker.check_null_pointer(ptr, operation, location)
    }

    /// Check for buffer overflow
    pub fn check_buffer_access(&self, offset: usize, size: usize, buffer_size: usize, operation: &str) -> SafetyResult<()> {
        self.safety_checker.check_buffer_access(offset, size, buffer_size, operation)
    }

    /// Check for stack overflow
    pub fn check_stack_overflow(&self) -> SafetyResult<()> {
        self.safety_checker.check_stack_overflow()
    }

    /// Get safety checker reference
    pub fn safety_checker(&self) -> &SafetyChecker {
        &self.safety_checker
    }

    /// Get mutable safety checker reference
    pub fn safety_checker_mut(&mut self) -> &mut SafetyChecker {
        &mut self.safety_checker
    }
}

impl Drop for MemoryManager {
    fn drop(&mut self) {
        // Clean up all remaining stack frames
        while !self.stack_frames.is_empty() {
            let _ = self.exit_scope();
        }
    }
}

/// Escape analysis context
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EscapeContext {
    /// Local variable declaration
    LocalVariable,
    /// Function return value
    FunctionReturn,
    /// Stored in heap object
    HeapStore,
    /// Captured by closure
    ClosureCapture,
    /// Sent through channel
    ChannelSend,
}

/// Allocation result
#[derive(Debug)]
pub enum AllocationResult {
    /// Allocated on stack
    Stack(*mut u8),
    /// Allocated on heap (GC managed)
    Heap(ObjectId),
}

/// Memory statistics
#[derive(Debug, Clone)]
pub struct MemoryStats {
    /// Bytes allocated on stack
    pub stack_bytes: usize,
    /// Bytes used on heap
    pub heap_used: usize,
    /// Total heap capacity
    pub heap_total: usize,
    /// Number of active stack frames
    pub stack_frames: usize,
    /// Garbage collector statistics
    pub gc_stats: crate::runtime::gc::GcStats,
}

/// Initialize default type layouts
pub fn init_default_type_layouts(memory_manager: &mut MemoryManager) {
    use crate::types::primitive::PrimitiveType;

    // Primitive types - small and stack-allocated
    let primitive_types = [
        (1, PrimitiveType::Int8, 1),
        (2, PrimitiveType::Int16, 2),
        (3, PrimitiveType::Int32, 4),
        (4, PrimitiveType::Int64, 8),
        (5, PrimitiveType::UInt8, 1),
        (6, PrimitiveType::UInt16, 2),
        (7, PrimitiveType::UInt32, 4),
        (8, PrimitiveType::UInt64, 8),
        (9, PrimitiveType::Float32, 4),
        (10, PrimitiveType::Float64, 8),
        (11, PrimitiveType::Bool, 1),
        (12, PrimitiveType::Char, 4),
    ];

    for (type_id, _prim_type, size) in primitive_types {
        memory_manager.register_type_layout(type_id, MemoryLayout {
            size,
            alignment: size.min(8),
            contains_references: false,
            strategy: AllocStrategy::Stack,
        });
    }

    // String type - contains references, heap allocated
    memory_manager.register_type_layout(13, MemoryLayout {
        size: std::mem::size_of::<String>(),
        alignment: 8,
        contains_references: true,
        strategy: AllocStrategy::Heap,
    });

    // Any type - contains references, heap allocated
    memory_manager.register_type_layout(14, MemoryLayout {
        size: std::mem::size_of::<crate::types::primitive::RuntimeValue>(),
        alignment: 8,
        contains_references: true,
        strategy: AllocStrategy::Heap,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_manager_creation() {
        let mm = MemoryManager::new();
        assert_eq!(mm.scope_depth(), 0);
    }

    #[test]
    fn test_scope_management() {
        let mut mm = MemoryManager::new();
        
        let frame_id = mm.enter_scope();
        assert_eq!(mm.scope_depth(), 1);
        assert_eq!(frame_id, 0);
        
        mm.exit_scope().unwrap();
        assert_eq!(mm.scope_depth(), 0);
    }

    #[test]
    fn test_type_layout_registration() {
        let mut mm = MemoryManager::new();
        
        let layout = MemoryLayout {
            size: 4,
            alignment: 4,
            contains_references: false,
            strategy: AllocStrategy::Stack,
        };
        
        mm.register_type_layout(1, layout.clone());
        
        let retrieved = mm.get_type_layout(1).unwrap();
        assert_eq!(retrieved.size, 4);
        assert_eq!(retrieved.strategy, AllocStrategy::Stack);
    }

    #[test]
    fn test_escape_analysis() {
        let mut mm = MemoryManager::new();
        
        // Register a small type
        mm.register_type_layout(1, MemoryLayout {
            size: 4,
            alignment: 4,
            contains_references: false,
            strategy: AllocStrategy::Stack,
        });
        
        // Local variable should not escape
        let result = mm.analyze_escape(1, 0, &EscapeContext::LocalVariable);
        assert_eq!(result, EscapeAnalysis::NoEscape);
        
        // Function return should escape to heap
        let result = mm.analyze_escape(1, 0, &EscapeContext::FunctionReturn);
        assert_eq!(result, EscapeAnalysis::EscapeToHeap);
    }

    #[test]
    fn test_default_type_layouts() {
        let mut mm = MemoryManager::new();
        init_default_type_layouts(&mut mm);
        
        // Check int32 layout
        let layout = mm.get_type_layout(3).unwrap();
        assert_eq!(layout.size, 4);
        assert_eq!(layout.strategy, AllocStrategy::Stack);
        assert!(!layout.contains_references);
        
        // Check string layout
        let layout = mm.get_type_layout(13).unwrap();
        assert_eq!(layout.strategy, AllocStrategy::Heap);
        assert!(layout.contains_references);
    }
}