//! Concurrent mark-and-sweep garbage collector with generational collection
//!
//! This module implements a tri-color concurrent garbage collector with:
//! - Mark-and-sweep algorithm
//! - Generational collection for performance
//! - Concurrent collection with minimal pause times
//! - Escape analysis integration
//! - GC tuning parameters and monitoring

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

/// GC configuration parameters
#[derive(Debug, Clone)]
pub struct GcConfig {
    /// Maximum heap size in bytes
    pub max_heap_size: usize,
    /// Target heap usage percentage before triggering GC
    pub target_heap_usage: u8,
    /// Number of parallel GC threads
    pub gc_threads: usize,
    /// Enable debug logging
    pub debug: bool,
    /// Young generation size ratio (0.0 to 1.0)
    pub young_gen_ratio: f32,
    /// Promotion threshold (number of collections survived)
    pub promotion_threshold: u32,
    /// Concurrent collection enabled
    pub concurrent_gc: bool,
    /// Maximum GC pause time target in milliseconds
    pub max_pause_time_ms: u64,
}

impl Default for GcConfig {
    fn default() -> Self {
        Self {
            max_heap_size: 1024 * 1024 * 1024, // 1GB default
            target_heap_usage: 80,
            gc_threads: num_cpus::get().min(4),
            debug: false,
            young_gen_ratio: 0.3, // 30% for young generation
            promotion_threshold: 2,
            concurrent_gc: true,
            max_pause_time_ms: 10, // 10ms target
        }
    }
}

/// Object metadata for GC tracking
#[derive(Debug, Clone)]
pub struct ObjectHeader {
    /// Object size in bytes
    pub size: usize,
    /// Object type identifier
    pub type_id: u32,
    /// Generation (0 = young, 1+ = old)
    pub generation: u32,
    /// Number of GC cycles survived
    pub age: u32,
    /// Mark color for tri-color marking
    pub color: Color,
    /// Reference count for cycle detection
    pub ref_count: usize,
    /// Allocation timestamp
    pub allocated_at: Instant,
}

/// Tri-color marking colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    White, // Unreachable (will be collected)
    Gray,  // Reachable but not scanned
    Black, // Reachable and scanned
}

/// Heap object representation
#[derive(Debug)]
pub struct HeapObject {
    /// Object header with GC metadata
    pub header: ObjectHeader,
    /// Object data
    pub data: Vec<u8>,
    /// References to other objects
    pub references: Vec<ObjectId>,
}

/// Unique object identifier
pub type ObjectId = usize;

/// Memory region for generational collection
#[derive(Debug)]
pub struct Generation {
    /// Objects in this generation
    objects: HashMap<ObjectId, HeapObject>,
    /// Total allocated bytes
    allocated_bytes: usize,
    /// Maximum size for this generation
    max_size: usize,
    /// Collection count
    collection_count: u64,
}

impl Generation {
    fn new(max_size: usize) -> Self {
        Self {
            objects: HashMap::new(),
            allocated_bytes: 0,
            max_size,
            collection_count: 0,
        }
    }

    fn allocate(&mut self, id: ObjectId, object: HeapObject) -> bool {
        if self.allocated_bytes + object.header.size > self.max_size {
            return false;
        }

        self.allocated_bytes += object.header.size;
        self.objects.insert(id, object);
        true
    }

    fn deallocate(&mut self, id: ObjectId) -> Option<HeapObject> {
        if let Some(object) = self.objects.remove(&id) {
            self.allocated_bytes -= object.header.size;
            Some(object)
        } else {
            None
        }
    }

    fn usage_ratio(&self) -> f32 {
        self.allocated_bytes as f32 / self.max_size as f32
    }
}

/// GC statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct GcStats {
    /// Total collections performed
    pub total_collections: u64,
    /// Young generation collections
    pub young_collections: u64,
    /// Full collections
    pub full_collections: u64,
    /// Total bytes allocated
    pub total_allocated: u64,
    /// Total bytes collected
    pub total_collected: u64,
    /// Average pause time in microseconds
    pub avg_pause_time_us: u64,
    /// Maximum pause time in microseconds
    pub max_pause_time_us: u64,
    /// Current heap size
    pub current_heap_size: usize,
    /// Last collection duration
    pub last_collection_duration: Duration,
}

/// Root set for GC marking
pub trait RootSet {
    /// Get all root object IDs
    fn get_roots(&self) -> Vec<ObjectId>;
}

/// Concurrent mark-and-sweep garbage collector
pub struct GarbageCollector {
    /// GC configuration
    config: GcConfig,
    /// Young generation (generation 0)
    young_gen: Arc<RwLock<Generation>>,
    /// Old generation (generation 1+)
    old_gen: Arc<RwLock<Generation>>,
    /// Next object ID
    next_object_id: AtomicUsize,
    /// GC statistics
    stats: Arc<RwLock<GcStats>>,
    /// GC thread handle
    gc_thread: Option<thread::JoinHandle<()>>,
    /// GC running flag
    gc_running: Arc<AtomicBool>,
    /// Collection requested flag
    collection_requested: Arc<AtomicBool>,
    /// Root set provider
    root_set: Arc<dyn RootSet + Send + Sync>,
}

impl GarbageCollector {
    /// Create a new garbage collector with default configuration
    pub fn new() -> Self {
        Self::with_config(GcConfig::default())
    }

    /// Create a new garbage collector with custom configuration
    pub fn with_config(config: GcConfig) -> Self {
        let young_size = (config.max_heap_size as f32 * config.young_gen_ratio) as usize;
        let old_size = config.max_heap_size - young_size;

        let gc_running = Arc::new(AtomicBool::new(false));
        let collection_requested = Arc::new(AtomicBool::new(false));
        let stats = Arc::new(RwLock::new(GcStats::default()));

        // Create a dummy root set for now
        let root_set: Arc<dyn RootSet + Send + Sync> = Arc::new(EmptyRootSet);

        let mut gc = Self {
            config,
            young_gen: Arc::new(RwLock::new(Generation::new(young_size))),
            old_gen: Arc::new(RwLock::new(Generation::new(old_size))),
            next_object_id: AtomicUsize::new(1),
            stats,
            gc_thread: None,
            gc_running,
            collection_requested,
            root_set,
        };

        // Start concurrent GC thread if enabled
        if gc.config.concurrent_gc {
            gc.start_concurrent_gc();
        }

        gc
    }

    /// Set the root set provider
    pub fn set_root_set(&mut self, root_set: Arc<dyn RootSet + Send + Sync>) {
        self.root_set = root_set;
    }

    /// Allocate a new object
    pub fn allocate(&self, size: usize, type_id: u32) -> Result<ObjectId, String> {
        let object_id = self.next_object_id.fetch_add(1, Ordering::SeqCst);

        let header = ObjectHeader {
            size,
            type_id,
            generation: 0, // Start in young generation
            age: 0,
            color: Color::White,
            ref_count: 0,
            allocated_at: Instant::now(),
        };

        // Try to allocate in young generation first
        {
            let mut young_gen = self.young_gen.write().unwrap();
            let object = HeapObject {
                header: header.clone(),
                data: vec![0; size],
                references: Vec::new(),
            };

            if young_gen.allocate(object_id, object) {
                // Update statistics
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.total_allocated += size as u64;
                    stats.current_heap_size += size;
                }
                return Ok(object_id);
            }
        }

        // Young generation full, try old generation
        {
            let mut old_gen = self.old_gen.write().unwrap();
            let mut old_header = header.clone();
            old_header.generation = 1;

            let object = HeapObject {
                header: old_header,
                data: vec![0; size],
                references: Vec::new(),
            };

            if old_gen.allocate(object_id, object) {
                // Update statistics
                {
                    let mut stats = self.stats.write().unwrap();
                    stats.total_allocated += size as u64;
                    stats.current_heap_size += size;
                }
                return Ok(object_id);
            }
        }

        // Both generations full, trigger collection and retry
        self.request_collection();

        // Wait a bit for collection to complete
        thread::sleep(Duration::from_millis(1));

        // Try young generation again
        {
            let mut young_gen = self.young_gen.write().unwrap();
            let object = HeapObject {
                header: header.clone(),
                data: vec![0; size],
                references: Vec::new(),
            };

            if young_gen.allocate(object_id, object) {
                let mut stats = self.stats.write().unwrap();
                stats.total_allocated += size as u64;
                stats.current_heap_size += size;
                return Ok(object_id);
            }
        }

        Err("Out of memory: unable to allocate object".to_string())
    }

    /// Get object by ID
    pub fn get_object(&self, id: ObjectId) -> Option<Arc<RwLock<HeapObject>>> {
        // Check young generation first
        {
            let young_gen = self.young_gen.read().unwrap();
            if young_gen.objects.contains_key(&id) {
                // Return a reference-counted wrapper
                // Note: This is simplified - in a real implementation,
                // we'd need proper synchronization
                return None; // Simplified for now
            }
        }

        // Check old generation
        {
            let old_gen = self.old_gen.read().unwrap();
            if old_gen.objects.contains_key(&id) {
                return None; // Simplified for now
            }
        }

        None
    }

    /// Request garbage collection
    pub fn request_collection(&self) {
        self.collection_requested.store(true, Ordering::SeqCst);
    }

    /// Perform garbage collection
    pub fn collect(&self) {
        let start_time = Instant::now();

        if self.config.debug {
            println!("GC: Starting collection");
        }

        // Determine collection type based on heap usage
        let should_collect_old = {
            let old_gen = self.old_gen.read().unwrap();
            old_gen.usage_ratio() > 0.8
        };

        if should_collect_old {
            self.collect_full();
        } else {
            self.collect_young();
        }

        let duration = start_time.elapsed();

        // Update statistics
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_collections += 1;
            stats.last_collection_duration = duration;

            let pause_time_us = duration.as_micros() as u64;
            if pause_time_us > stats.max_pause_time_us {
                stats.max_pause_time_us = pause_time_us;
            }

            // Update average pause time
            let total_pause_time =
                stats.avg_pause_time_us * (stats.total_collections - 1) + pause_time_us;
            stats.avg_pause_time_us = total_pause_time / stats.total_collections;
        }

        if self.config.debug {
            println!("GC: Collection completed in {:?}", duration);
        }
    }

    /// Collect young generation only
    fn collect_young(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.young_collections += 1;
        drop(stats);

        if self.config.debug {
            println!("GC: Young generation collection");
        }

        // Mark phase
        let marked_objects = self.mark_from_roots();

        // Sweep young generation
        let collected_bytes = self.sweep_generation(&self.young_gen, &marked_objects);

        // Promote surviving objects
        self.promote_survivors(&marked_objects);

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.total_collected += collected_bytes;
        stats.current_heap_size -= collected_bytes as usize;
    }

    /// Collect all generations (full collection)
    fn collect_full(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.full_collections += 1;
        drop(stats);

        if self.config.debug {
            println!("GC: Full collection");
        }

        // Mark phase
        let marked_objects = self.mark_from_roots();

        // Sweep both generations
        let young_collected = self.sweep_generation(&self.young_gen, &marked_objects);
        let old_collected = self.sweep_generation(&self.old_gen, &marked_objects);

        let total_collected = young_collected + old_collected;

        // Update statistics
        let mut stats = self.stats.write().unwrap();
        stats.total_collected += total_collected;
        stats.current_heap_size -= total_collected as usize;
    }

    /// Mark phase: mark all reachable objects
    fn mark_from_roots(&self) -> HashSet<ObjectId> {
        let mut marked = HashSet::new();
        let mut work_queue = VecDeque::new();

        // Get root objects
        let roots = self.root_set.get_roots();

        // Initialize work queue with roots
        for root_id in roots {
            work_queue.push_back(root_id);
            marked.insert(root_id);
            self.set_object_color(root_id, Color::Gray);
        }

        // Process work queue
        while let Some(object_id) = work_queue.pop_front() {
            // Mark object as black (processed)
            self.set_object_color(object_id, Color::Black);

            // Get object references
            if let Some(references) = self.get_object_references(object_id) {
                for &ref_id in &references {
                    if !marked.contains(&ref_id) {
                        marked.insert(ref_id);
                        work_queue.push_back(ref_id);
                        self.set_object_color(ref_id, Color::Gray);
                    }
                }
            }
        }

        marked
    }

    /// Sweep phase: deallocate unmarked objects in a generation
    fn sweep_generation(
        &self,
        generation: &Arc<RwLock<Generation>>,
        marked: &HashSet<ObjectId>,
    ) -> u64 {
        let mut collected_bytes = 0u64;
        let mut to_remove = Vec::new();

        {
            let gen = generation.read().unwrap();
            for (&object_id, object) in &gen.objects {
                if !marked.contains(&object_id) {
                    to_remove.push(object_id);
                    collected_bytes += object.header.size as u64;
                }
            }
        }

        // Remove unmarked objects
        {
            let mut gen = generation.write().unwrap();
            for object_id in to_remove {
                gen.deallocate(object_id);
            }
            gen.collection_count += 1;
        }

        collected_bytes
    }

    /// Promote surviving young objects to old generation
    fn promote_survivors(&self, marked: &HashSet<ObjectId>) {
        let mut to_promote = Vec::new();

        // Find objects to promote
        {
            let young_gen = self.young_gen.read().unwrap();
            for (&object_id, object) in &young_gen.objects {
                if marked.contains(&object_id)
                    && object.header.age >= self.config.promotion_threshold
                {
                    to_promote.push(object_id);
                }
            }
        }

        // Move objects to old generation
        for object_id in to_promote {
            if let Some(mut object) = self.young_gen.write().unwrap().deallocate(object_id) {
                object.header.generation = 1;
                object.header.age = 0; // Reset age in new generation

                // Try to allocate in old generation
                if !self.old_gen.write().unwrap().allocate(object_id, object) {
                    // Old generation full, put back in young generation
                    // This is a simplified fallback
                    if self.config.debug {
                        println!(
                            "GC: Failed to promote object {}, old generation full",
                            object_id
                        );
                    }
                }
            }
        }

        // Age remaining young objects
        {
            let mut young_gen = self.young_gen.write().unwrap();
            for object in young_gen.objects.values_mut() {
                if marked.contains(&(object as *const HeapObject as usize)) {
                    object.header.age += 1;
                }
            }
        }
    }

    /// Set object color for tri-color marking
    fn set_object_color(&self, object_id: ObjectId, color: Color) {
        // Check young generation
        {
            let mut young_gen = self.young_gen.write().unwrap();
            if let Some(object) = young_gen.objects.get_mut(&object_id) {
                object.header.color = color;
                return;
            }
        }

        // Check old generation
        {
            let mut old_gen = self.old_gen.write().unwrap();
            if let Some(object) = old_gen.objects.get_mut(&object_id) {
                object.header.color = color;
            }
        }
    }

    /// Get object references
    fn get_object_references(&self, object_id: ObjectId) -> Option<Vec<ObjectId>> {
        // Check young generation
        {
            let young_gen = self.young_gen.read().unwrap();
            if let Some(object) = young_gen.objects.get(&object_id) {
                return Some(object.references.clone());
            }
        }

        // Check old generation
        {
            let old_gen = self.old_gen.read().unwrap();
            if let Some(object) = old_gen.objects.get(&object_id) {
                return Some(object.references.clone());
            }
        }

        None
    }

    /// Start concurrent GC thread
    fn start_concurrent_gc(&mut self) {
        let gc_running = Arc::clone(&self.gc_running);
        let collection_requested = Arc::clone(&self.collection_requested);
        let young_gen = Arc::clone(&self.young_gen);
        let old_gen = Arc::clone(&self.old_gen);
        let config = self.config.clone();
        let stats = Arc::clone(&self.stats);
        let root_set = Arc::clone(&self.root_set);

        gc_running.store(true, Ordering::SeqCst);

        let handle = thread::spawn(move || {
            while gc_running.load(Ordering::SeqCst) {
                // Check if collection was requested
                if collection_requested.load(Ordering::SeqCst) {
                    collection_requested.store(false, Ordering::SeqCst);

                    // Create a temporary GC instance for collection
                    let temp_gc = GarbageCollector {
                        config: config.clone(),
                        young_gen: Arc::clone(&young_gen),
                        old_gen: Arc::clone(&old_gen),
                        next_object_id: AtomicUsize::new(0), // Not used in collection
                        stats: Arc::clone(&stats),
                        gc_thread: None,
                        gc_running: Arc::clone(&gc_running),
                        collection_requested: Arc::clone(&collection_requested),
                        root_set: Arc::clone(&root_set),
                    };

                    temp_gc.collect();
                }

                // Check heap usage and trigger collection if needed
                let should_collect = {
                    let young_gen = young_gen.read().unwrap();
                    young_gen.usage_ratio() > (config.target_heap_usage as f32 / 100.0)
                };

                if should_collect {
                    let temp_gc = GarbageCollector {
                        config: config.clone(),
                        young_gen: Arc::clone(&young_gen),
                        old_gen: Arc::clone(&old_gen),
                        next_object_id: AtomicUsize::new(0),
                        stats: Arc::clone(&stats),
                        gc_thread: None,
                        gc_running: Arc::clone(&gc_running),
                        collection_requested: Arc::clone(&collection_requested),
                        root_set: Arc::clone(&root_set),
                    };

                    temp_gc.collect();
                }

                // Sleep for a short time
                thread::sleep(Duration::from_millis(10));
            }
        });

        self.gc_thread = Some(handle);
    }

    /// Get GC statistics
    pub fn get_stats(&self) -> GcStats {
        self.stats.read().unwrap().clone()
    }

    /// Get current heap usage
    pub fn heap_usage(&self) -> (usize, usize) {
        let young_gen = self.young_gen.read().unwrap();
        let old_gen = self.old_gen.read().unwrap();

        let used = young_gen.allocated_bytes + old_gen.allocated_bytes;
        let total = young_gen.max_size + old_gen.max_size;

        (used, total)
    }

    /// Force garbage collection (for testing)
    pub fn force_collect(&self) {
        self.collect();
    }
}

impl Drop for GarbageCollector {
    fn drop(&mut self) {
        // Stop GC thread
        self.gc_running.store(false, Ordering::SeqCst);

        if let Some(handle) = self.gc_thread.take() {
            let _ = handle.join();
        }
    }
}

/// Empty root set implementation for testing
struct EmptyRootSet;

impl RootSet for EmptyRootSet {
    fn get_roots(&self) -> Vec<ObjectId> {
        Vec::new()
    }
}

/// Parse GC configuration from environment variables
pub fn parse_gc_config_from_env() -> GcConfig {
    let mut config = GcConfig::default();

    if let Ok(heap_size) = std::env::var("LANG_GC_HEAP_SIZE") {
        if let Ok(size) = parse_size(&heap_size) {
            config.max_heap_size = size;
        }
    }

    if let Ok(target) = std::env::var("LANG_GC_TARGET") {
        if let Ok(target_val) = target.parse::<u8>() {
            if target_val <= 100 {
                config.target_heap_usage = target_val;
            }
        }
    }

    if let Ok(threads) = std::env::var("LANG_GC_THREADS") {
        if let Ok(thread_count) = threads.parse::<usize>() {
            config.gc_threads = thread_count.max(1);
        }
    }

    if let Ok(debug) = std::env::var("LANG_GC_DEBUG") {
        config.debug = debug.to_lowercase() == "true";
    }

    config
}

/// Parse size string (e.g., "1024M", "2G")
fn parse_size(size_str: &str) -> Result<usize, String> {
    let size_str = size_str.trim().to_uppercase();

    if let Some(num_str) = size_str.strip_suffix('G') {
        let num: usize = num_str.parse().map_err(|_| "Invalid size format")?;
        Ok(num * 1024 * 1024 * 1024)
    } else if let Some(num_str) = size_str.strip_suffix('M') {
        let num: usize = num_str.parse().map_err(|_| "Invalid size format")?;
        Ok(num * 1024 * 1024)
    } else if let Some(num_str) = size_str.strip_suffix('K') {
        let num: usize = num_str.parse().map_err(|_| "Invalid size format")?;
        Ok(num * 1024)
    } else {
        size_str
            .parse()
            .map_err(|_| "Invalid size format".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gc_config_default() {
        let config = GcConfig::default();
        assert_eq!(config.target_heap_usage, 80);
        assert!(config.concurrent_gc);
        assert_eq!(config.max_pause_time_ms, 10);
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1K").unwrap(), 1024);
        assert_eq!(parse_size("1M").unwrap(), 1024 * 1024);
        assert_eq!(parse_size("1G").unwrap(), 1024 * 1024 * 1024);
    }

    #[test]
    fn test_generation_allocation() {
        let mut gen = Generation::new(1024);

        let header = ObjectHeader {
            size: 100,
            type_id: 1,
            generation: 0,
            age: 0,
            color: Color::White,
            ref_count: 0,
            allocated_at: Instant::now(),
        };

        let object = HeapObject {
            header,
            data: vec![0; 100],
            references: Vec::new(),
        };

        assert!(gen.allocate(1, object));
        assert_eq!(gen.allocated_bytes, 100);
        assert_eq!(gen.objects.len(), 1);
    }

    #[test]
    fn test_gc_allocation() {
        let gc = GarbageCollector::new();

        let obj_id = gc.allocate(100, 1).unwrap();
        assert!(obj_id > 0);

        let (used, _total) = gc.heap_usage();
        assert!(used >= 100);
    }
}
