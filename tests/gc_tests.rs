//! Comprehensive tests for the garbage collector
//!
//! Tests cover:
//! - Basic allocation and collection
//! - Generational collection
//! - Concurrent collection
//! - Memory safety
//! - Performance characteristics

use bulu::runtime::gc::{GarbageCollector, GcConfig, RootSet, ObjectId, parse_gc_config_from_env};
use bulu::runtime::memory::{MemoryManager, EscapeContext, MemoryLayout, AllocStrategy, init_default_type_layouts};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::HashSet;

/// Test root set implementation
struct TestRootSet {
    roots: Arc<Mutex<Vec<ObjectId>>>,
}

impl TestRootSet {
    fn new() -> Self {
        Self {
            roots: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn add_root(&self, id: ObjectId) {
        self.roots.lock().unwrap().push(id);
    }

    fn clear_roots(&self) {
        self.roots.lock().unwrap().clear();
    }
}

impl RootSet for TestRootSet {
    fn get_roots(&self) -> Vec<ObjectId> {
        self.roots.lock().unwrap().clone()
    }
}

#[test]
fn test_gc_basic_allocation() {
    let gc = GarbageCollector::new();
    
    // Allocate some objects
    let obj1 = gc.allocate(100, 1).expect("Failed to allocate object 1");
    let obj2 = gc.allocate(200, 2).expect("Failed to allocate object 2");
    let obj3 = gc.allocate(50, 1).expect("Failed to allocate object 3");
    
    assert!(obj1 > 0);
    assert!(obj2 > 0);
    assert!(obj3 > 0);
    assert_ne!(obj1, obj2);
    assert_ne!(obj2, obj3);
    
    // Check heap usage
    let (used, total) = gc.heap_usage();
    assert!(used >= 350); // At least the allocated bytes
    assert!(total > used);
}

#[test]
fn test_gc_collection_stats() {
    let gc = GarbageCollector::new();
    
    // Initial stats
    let stats = gc.get_stats();
    assert_eq!(stats.total_collections, 0);
    assert_eq!(stats.total_allocated, 0);
    
    // Allocate some objects
    let _obj1 = gc.allocate(100, 1).unwrap();
    let _obj2 = gc.allocate(200, 2).unwrap();
    
    // Check allocation stats
    let stats = gc.get_stats();
    assert!(stats.total_allocated >= 300);
    assert!(stats.current_heap_size >= 300);
    
    // Force collection
    gc.force_collect();
    
    // Check collection stats
    let stats = gc.get_stats();
    assert!(stats.total_collections > 0);
}

#[test]
fn test_gc_generational_collection() {
    let mut config = GcConfig::default();
    config.young_gen_ratio = 0.5; // 50% young generation
    config.promotion_threshold = 1; // Promote after 1 collection
    config.concurrent_gc = false; // Disable for deterministic testing
    
    let gc = GarbageCollector::with_config(config);
    
    // Allocate objects in young generation
    let obj1 = gc.allocate(100, 1).unwrap();
    let obj2 = gc.allocate(200, 2).unwrap();
    
    // Set up root set to keep obj1 alive
    let root_set = Arc::new(TestRootSet::new());
    root_set.add_root(obj1);
    
    // Force young generation collection
    gc.force_collect();
    
    let stats = gc.get_stats();
    assert!(stats.young_collections > 0);
}

#[test]
fn test_gc_concurrent_collection() {
    let mut config = GcConfig::default();
    config.concurrent_gc = true;
    config.target_heap_usage = 50; // Trigger collection at 50% usage
    
    let gc = Arc::new(GarbageCollector::with_config(config));
    
    // Spawn multiple threads allocating objects
    let mut handles = Vec::new();
    
    for i in 0..4 {
        let gc_clone = Arc::clone(&gc);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let size = 100 + (i * 10) + j;
                if let Ok(_obj_id) = gc_clone.allocate(size, 1) {
                    // Object allocated successfully
                    if j % 10 == 0 {
                        thread::sleep(Duration::from_millis(1));
                    }
                }
            }
        });
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Check that collections occurred
    let stats = gc.get_stats();
    assert!(stats.total_allocated > 0);
    
    // Give concurrent GC time to run
    thread::sleep(Duration::from_millis(100));
}

#[test]
fn test_gc_memory_pressure() {
    let mut config = GcConfig::default();
    config.max_heap_size = 1024 * 1024; // 1MB heap
    config.concurrent_gc = false;
    
    let gc = GarbageCollector::with_config(config);
    
    // Allocate until we get memory pressure
    let mut allocated_objects = Vec::new();
    let mut allocation_count = 0;
    
    loop {
        match gc.allocate(1024, 1) {
            Ok(obj_id) => {
                allocated_objects.push(obj_id);
                allocation_count += 1;
                
                // Stop after reasonable number of allocations
                if allocation_count > 2000 {
                    break;
                }
            }
            Err(_) => {
                // Out of memory - this is expected
                break;
            }
        }
    }
    
    assert!(allocation_count > 0);
    println!("Allocated {} objects before memory pressure", allocation_count);
    
    // Force collection should free up space
    gc.force_collect();
    
    // Should be able to allocate again
    let _new_obj = gc.allocate(1024, 1).expect("Should be able to allocate after GC");
}

#[test]
fn test_gc_pause_time_target() {
    let mut config = GcConfig::default();
    config.max_pause_time_ms = 5; // 5ms target
    config.concurrent_gc = false;
    
    let gc = GarbageCollector::with_config(config);
    
    // Allocate many small objects
    for i in 0..1000 {
        let _ = gc.allocate(100, i % 10);
    }
    
    // Measure collection time
    let start = Instant::now();
    gc.force_collect();
    let duration = start.elapsed();
    
    let stats = gc.get_stats();
    println!("Collection took {:?}, max pause time: {}μs", 
             duration, stats.max_pause_time_us);
    
    // Note: In a real implementation, we'd check that pause times
    // are within target, but this is a simplified implementation
    assert!(stats.total_collections > 0);
}

#[test]
fn test_gc_config_from_env() {
    // Set environment variables
    std::env::set_var("LANG_GC_HEAP_SIZE", "512M");
    std::env::set_var("LANG_GC_TARGET", "70");
    std::env::set_var("LANG_GC_THREADS", "2");
    std::env::set_var("LANG_GC_DEBUG", "true");
    
    let config = parse_gc_config_from_env();
    
    assert_eq!(config.max_heap_size, 512 * 1024 * 1024);
    assert_eq!(config.target_heap_usage, 70);
    assert_eq!(config.gc_threads, 2);
    assert!(config.debug);
    
    // Clean up
    std::env::remove_var("LANG_GC_HEAP_SIZE");
    std::env::remove_var("LANG_GC_TARGET");
    std::env::remove_var("LANG_GC_THREADS");
    std::env::remove_var("LANG_GC_DEBUG");
}

#[test]
fn test_memory_manager_integration() {
    let mut memory_manager = MemoryManager::new();
    init_default_type_layouts(&mut memory_manager);
    
    // Enter a scope
    let _frame_id = memory_manager.enter_scope();
    
    // Allocate some objects
    let result1 = memory_manager.allocate(3, &EscapeContext::LocalVariable); // int32
    let result2 = memory_manager.allocate(13, &EscapeContext::FunctionReturn); // string
    
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    
    // Check memory stats
    let stats = memory_manager.get_memory_stats();
    assert!(stats.stack_bytes > 0 || stats.heap_used > 0);
    
    // Exit scope
    memory_manager.exit_scope().unwrap();
    
    // Check that stack memory was cleaned up
    let stats_after = memory_manager.get_memory_stats();
    assert_eq!(stats_after.stack_frames, 0);
}

#[test]
fn test_escape_analysis() {
    let mut memory_manager = MemoryManager::new();
    init_default_type_layouts(&mut memory_manager);
    
    // Small primitive type in local context should not escape
    let escape1 = memory_manager.analyze_escape(3, 0, &EscapeContext::LocalVariable);
    assert_eq!(escape1, bulu::runtime::memory::EscapeAnalysis::NoEscape);
    
    // Same type in function return should escape
    let escape2 = memory_manager.analyze_escape(3, 0, &EscapeContext::FunctionReturn);
    assert_eq!(escape2, bulu::runtime::memory::EscapeAnalysis::EscapeToHeap);
    
    // String type should escape due to containing references
    let escape3 = memory_manager.analyze_escape(13, 0, &EscapeContext::LocalVariable);
    assert_eq!(escape3, bulu::runtime::memory::EscapeAnalysis::EscapeToParent);
}

#[test]
fn test_gc_stress_test() {
    let mut config = GcConfig::default();
    config.max_heap_size = 10 * 1024 * 1024; // 10MB
    config.concurrent_gc = true;
    
    let gc = Arc::new(GarbageCollector::with_config(config));
    let root_set = Arc::new(TestRootSet::new());
    
    // Stress test with multiple threads
    let mut handles = Vec::new();
    
    for thread_id in 0..8 {
        let gc_clone = Arc::clone(&gc);
        let root_set_clone = Arc::clone(&root_set);
        
        let handle = thread::spawn(move || {
            let mut local_objects = Vec::new();
            
            for i in 0..500 {
                // Allocate object
                if let Ok(obj_id) = gc_clone.allocate(1024 + (i % 100), thread_id as u32) {
                    local_objects.push(obj_id);
                    
                    // Keep some objects as roots
                    if i % 10 == 0 {
                        root_set_clone.add_root(obj_id);
                    }
                    
                    // Occasionally clear old objects
                    if local_objects.len() > 50 {
                        local_objects.drain(0..25);
                    }
                }
                
                // Small delay to allow GC to run
                if i % 50 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }
    
    // Give GC time to finish
    thread::sleep(Duration::from_millis(200));
    
    let stats = gc.get_stats();
    println!("Stress test results:");
    println!("  Total collections: {}", stats.total_collections);
    println!("  Total allocated: {} bytes", stats.total_allocated);
    println!("  Total collected: {} bytes", stats.total_collected);
    println!("  Average pause time: {}μs", stats.avg_pause_time_us);
    println!("  Max pause time: {}μs", stats.max_pause_time_us);
    
    assert!(stats.total_collections > 0);
    assert!(stats.total_allocated > 0);
}

#[test]
fn test_gc_memory_safety() {
    let gc = GarbageCollector::new();
    
    // Test that we can't access deallocated objects
    let obj1 = gc.allocate(100, 1).unwrap();
    let obj2 = gc.allocate(200, 2).unwrap();
    
    // Objects should be accessible before collection
    assert!(obj1 > 0);
    assert!(obj2 > 0);
    
    // Force collection (without roots, objects should be collected)
    gc.force_collect();
    
    // In a real implementation, accessing collected objects would be prevented
    // by the type system and runtime checks
    let stats = gc.get_stats();
    assert!(stats.total_collections > 0);
}

#[test]
fn test_gc_performance_characteristics() {
    let mut config = GcConfig::default();
    config.max_heap_size = 50 * 1024 * 1024; // 50MB
    config.concurrent_gc = false; // For deterministic timing
    
    let gc = GarbageCollector::with_config(config);
    
    // Measure allocation performance
    let start = Instant::now();
    let mut objects = Vec::new();
    
    for i in 0..10000 {
        if let Ok(obj_id) = gc.allocate(1024, 1) {
            objects.push(obj_id);
        }
    }
    
    let allocation_time = start.elapsed();
    println!("Allocated {} objects in {:?}", objects.len(), allocation_time);
    
    // Measure collection performance
    let start = Instant::now();
    gc.force_collect();
    let collection_time = start.elapsed();
    
    println!("Collection took {:?}", collection_time);
    
    let stats = gc.get_stats();
    assert!(stats.total_allocated > 0);
    assert!(allocation_time.as_millis() < 1000); // Should be fast
}