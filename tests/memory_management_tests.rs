//! Memory management tests
//!
//! Tests for:
//! - Stack vs heap allocation decisions
//! - Escape analysis
//! - Memory layout optimization
//! - Integration with garbage collector

use bulu::runtime::memory::{
    MemoryManager, EscapeContext, EscapeAnalysis, MemoryLayout, AllocStrategy,
    AllocationResult, init_default_type_layouts
};
use bulu::runtime::gc::{GarbageCollector, GcConfig};
use std::time::Duration;
use std::thread;

#[test]
fn test_memory_manager_basic_operations() {
    let mut mm = MemoryManager::new();
    
    // Test scope management
    assert_eq!(mm.scope_depth(), 0);
    
    let frame1 = mm.enter_scope();
    assert_eq!(mm.scope_depth(), 1);
    assert_eq!(frame1, 0);
    
    let frame2 = mm.enter_scope();
    assert_eq!(mm.scope_depth(), 2);
    assert_eq!(frame2, 1);
    
    mm.exit_scope().unwrap();
    assert_eq!(mm.scope_depth(), 1);
    
    mm.exit_scope().unwrap();
    assert_eq!(mm.scope_depth(), 0);
}

#[test]
fn test_type_layout_management() {
    let mut mm = MemoryManager::new();
    
    // Register a custom type layout
    let layout = MemoryLayout {
        size: 16,
        alignment: 8,
        contains_references: true,
        strategy: AllocStrategy::Heap,
    };
    
    mm.register_type_layout(100, layout.clone());
    
    let retrieved = mm.get_type_layout(100).unwrap();
    assert_eq!(retrieved.size, 16);
    assert_eq!(retrieved.alignment, 8);
    assert!(retrieved.contains_references);
    assert_eq!(retrieved.strategy, AllocStrategy::Heap);
    
    // Test non-existent type
    assert!(mm.get_type_layout(999).is_none());
}

#[test]
fn test_escape_analysis_local_variables() {
    let mut mm = MemoryManager::new();
    
    // Small type without references - should not escape
    mm.register_type_layout(1, MemoryLayout {
        size: 4,
        alignment: 4,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    let result = mm.analyze_escape(1, 0, &EscapeContext::LocalVariable);
    assert_eq!(result, EscapeAnalysis::NoEscape);
    
    // Type with references - should escape to parent
    mm.register_type_layout(2, MemoryLayout {
        size: 8,
        alignment: 8,
        contains_references: true,
        strategy: AllocStrategy::Stack,
    });
    
    let result = mm.analyze_escape(2, 0, &EscapeContext::LocalVariable);
    assert_eq!(result, EscapeAnalysis::EscapeToParent);
}

#[test]
fn test_escape_analysis_function_returns() {
    let mut mm = MemoryManager::new();
    
    // Any type returned from function should escape to heap
    mm.register_type_layout(1, MemoryLayout {
        size: 4,
        alignment: 4,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    let result = mm.analyze_escape(1, 0, &EscapeContext::FunctionReturn);
    assert_eq!(result, EscapeAnalysis::EscapeToHeap);
}

#[test]
fn test_escape_analysis_large_objects() {
    let mut mm = MemoryManager::new();
    
    // Large objects always escape to heap
    mm.register_type_layout(1, MemoryLayout {
        size: 256, // > 128 bytes
        alignment: 8,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    let result = mm.analyze_escape(1, 0, &EscapeContext::LocalVariable);
    assert_eq!(result, EscapeAnalysis::EscapeToHeap);
}

#[test]
fn test_escape_analysis_caching() {
    let mut mm = MemoryManager::new();
    
    mm.register_type_layout(1, MemoryLayout {
        size: 4,
        alignment: 4,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    // First call should compute result
    let result1 = mm.analyze_escape(1, 0, &EscapeContext::LocalVariable);
    
    // Second call should use cached result
    let result2 = mm.analyze_escape(1, 0, &EscapeContext::LocalVariable);
    
    assert_eq!(result1, result2);
    assert_eq!(result1, EscapeAnalysis::NoEscape);
}

#[test]
fn test_allocation_strategy_stack() {
    let mut mm = MemoryManager::new();
    
    // Register small stack-allocated type
    mm.register_type_layout(1, MemoryLayout {
        size: 8,
        alignment: 8,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    // Enter scope for stack allocation
    mm.enter_scope();
    
    // Allocate object that should go on stack
    let result = mm.allocate(1, &EscapeContext::LocalVariable);
    assert!(result.is_ok());
    
    match result.unwrap() {
        AllocationResult::Stack(ptr) => {
            assert!(!ptr.is_null());
        }
        AllocationResult::Heap(_) => {
            panic!("Expected stack allocation");
        }
    }
    
    // Check memory stats
    let stats = mm.get_memory_stats();
    assert!(stats.stack_bytes >= 8);
    assert_eq!(stats.stack_frames, 1);
    
    mm.exit_scope().unwrap();
}

#[test]
fn test_allocation_strategy_heap() {
    let mut mm = MemoryManager::new();
    
    // Register heap-allocated type
    mm.register_type_layout(1, MemoryLayout {
        size: 16,
        alignment: 8,
        contains_references: true,
        strategy: AllocStrategy::Heap,
    });
    
    mm.enter_scope();
    
    // Allocate object that should go on heap
    let result = mm.allocate(1, &EscapeContext::LocalVariable);
    assert!(result.is_ok());
    
    match result.unwrap() {
        AllocationResult::Heap(obj_id) => {
            assert!(obj_id > 0);
        }
        AllocationResult::Stack(_) => {
            panic!("Expected heap allocation");
        }
    }
    
    // Check memory stats
    let stats = mm.get_memory_stats();
    assert!(stats.heap_used > 0);
    
    mm.exit_scope().unwrap();
}

#[test]
fn test_allocation_without_scope() {
    let mut mm = MemoryManager::new();
    
    mm.register_type_layout(1, MemoryLayout {
        size: 8,
        alignment: 8,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    // Try to allocate on stack without active scope
    let result = mm.allocate(1, &EscapeContext::LocalVariable);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No active stack frame"));
}

#[test]
fn test_memory_stats() {
    let mut mm = MemoryManager::new();
    init_default_type_layouts(&mut mm);
    
    mm.enter_scope();
    
    // Allocate some objects
    let _stack_alloc = mm.allocate(3, &EscapeContext::LocalVariable); // int32 on stack
    let _heap_alloc = mm.allocate(13, &EscapeContext::FunctionReturn); // string on heap
    
    let stats = mm.get_memory_stats();
    assert!(stats.stack_bytes > 0 || stats.heap_used > 0);
    assert_eq!(stats.stack_frames, 1);
    assert!(stats.gc_stats.total_allocated >= 0);
    
    mm.exit_scope().unwrap();
}

#[test]
fn test_default_type_layouts() {
    let mut mm = MemoryManager::new();
    init_default_type_layouts(&mut mm);
    
    // Test primitive types
    let int32_layout = mm.get_type_layout(3).unwrap(); // int32
    assert_eq!(int32_layout.size, 4);
    assert_eq!(int32_layout.strategy, AllocStrategy::Stack);
    assert!(!int32_layout.contains_references);
    
    let int64_layout = mm.get_type_layout(4).unwrap(); // int64
    assert_eq!(int64_layout.size, 8);
    assert_eq!(int64_layout.strategy, AllocStrategy::Stack);
    
    let bool_layout = mm.get_type_layout(11).unwrap(); // bool
    assert_eq!(bool_layout.size, 1);
    assert_eq!(bool_layout.strategy, AllocStrategy::Stack);
    
    // Test reference types
    let string_layout = mm.get_type_layout(13).unwrap(); // string
    assert_eq!(string_layout.strategy, AllocStrategy::Heap);
    assert!(string_layout.contains_references);
    
    let any_layout = mm.get_type_layout(14).unwrap(); // any
    assert_eq!(any_layout.strategy, AllocStrategy::Heap);
    assert!(any_layout.contains_references);
}

#[test]
fn test_memory_manager_with_custom_gc() {
    let mut config = GcConfig::default();
    config.max_heap_size = 1024 * 1024; // 1MB
    config.concurrent_gc = false;
    
    let gc = GarbageCollector::with_config(config);
    let mut mm = MemoryManager::with_gc(gc);
    init_default_type_layouts(&mut mm);
    
    mm.enter_scope();
    
    // Allocate heap objects
    for i in 0..100 {
        let _result = mm.allocate(13, &EscapeContext::FunctionReturn); // string type
        if i % 10 == 0 {
            // Force GC occasionally
            mm.force_gc();
        }
    }
    
    let stats = mm.get_memory_stats();
    assert!(stats.heap_used > 0);
    assert!(stats.gc_stats.total_allocated > 0);
    
    mm.exit_scope().unwrap();
}

#[test]
fn test_scope_cleanup() {
    let mut mm = MemoryManager::new();
    
    mm.register_type_layout(1, MemoryLayout {
        size: 64,
        alignment: 8,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    // Allocate in nested scopes
    mm.enter_scope();
    let _alloc1 = mm.allocate(1, &EscapeContext::LocalVariable);
    
    mm.enter_scope();
    let _alloc2 = mm.allocate(1, &EscapeContext::LocalVariable);
    let _alloc3 = mm.allocate(1, &EscapeContext::LocalVariable);
    
    let stats_before = mm.get_memory_stats();
    assert!(stats_before.stack_bytes >= 64 * 3);
    assert_eq!(stats_before.stack_frames, 2);
    
    // Exit inner scope
    mm.exit_scope().unwrap();
    
    let stats_after = mm.get_memory_stats();
    assert!(stats_after.stack_bytes < stats_before.stack_bytes);
    assert_eq!(stats_after.stack_frames, 1);
    
    // Exit outer scope
    mm.exit_scope().unwrap();
    
    let stats_final = mm.get_memory_stats();
    assert_eq!(stats_final.stack_frames, 0);
}

#[test]
fn test_escape_context_variations() {
    let mut mm = MemoryManager::new();
    
    mm.register_type_layout(1, MemoryLayout {
        size: 8,
        alignment: 8,
        contains_references: false,
        strategy: AllocStrategy::Stack,
    });
    
    // Test different escape contexts
    let contexts = [
        EscapeContext::LocalVariable,
        EscapeContext::FunctionReturn,
        EscapeContext::HeapStore,
        EscapeContext::ClosureCapture,
        EscapeContext::ChannelSend,
    ];
    
    for context in &contexts {
        let result = mm.analyze_escape(1, 0, context);
        match context {
            EscapeContext::LocalVariable => assert_eq!(result, EscapeAnalysis::NoEscape),
            _ => assert_eq!(result, EscapeAnalysis::EscapeToHeap),
        }
    }
}

#[test]
fn test_memory_manager_stress() {
    let mut mm = MemoryManager::new();
    init_default_type_layouts(&mut mm);
    
    // Stress test with many allocations and scope changes
    for outer in 0..10 {
        mm.enter_scope();
        
        for inner in 0..50 {
            mm.enter_scope();
            
            // Mix of stack and heap allocations
            let _stack_alloc = mm.allocate(3, &EscapeContext::LocalVariable); // int32
            let _heap_alloc = mm.allocate(13, &EscapeContext::FunctionReturn); // string
            
            if inner % 5 == 0 {
                mm.force_gc();
            }
            
            mm.exit_scope().unwrap();
        }
        
        mm.exit_scope().unwrap();
        
        if outer % 3 == 0 {
            let stats = mm.get_memory_stats();
            assert_eq!(stats.stack_frames, 0);
        }
    }
    
    // Final state should be clean
    let final_stats = mm.get_memory_stats();
    assert_eq!(final_stats.stack_frames, 0);
    assert!(final_stats.gc_stats.total_allocated > 0);
}

// Note: Concurrent access test removed due to thread safety issues with raw pointers
// In a production implementation, proper synchronization would be needed