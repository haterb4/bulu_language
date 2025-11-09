// Goroutine runtime implementation based on M:N threading model
// Inspired by Go's runtime and Tokio's architecture

use std::collections::{HashMap, VecDeque};
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::super::compiler::ir::{IrFunction, IrProgram};
use super::super::error::Result;
use super::super::types::primitive::RuntimeValue;

// Thread-local storage for current goroutine context
thread_local! {
    static CURRENT_GOROUTINE_ID: std::cell::RefCell<Option<GoroutineId>> = std::cell::RefCell::new(None);
}

/// Set the current goroutine ID for this thread
pub fn set_current_goroutine_id(id: GoroutineId) {
    CURRENT_GOROUTINE_ID.with(|cell| {
        *cell.borrow_mut() = Some(id);
    });
}

/// Get the current goroutine ID for this thread
pub fn get_current_goroutine_id() -> Option<GoroutineId> {
    CURRENT_GOROUTINE_ID.with(|cell| *cell.borrow())
}

/// Clear the current goroutine ID for this thread
pub fn clear_current_goroutine_id() {
    CURRENT_GOROUTINE_ID.with(|cell| {
        *cell.borrow_mut() = None;
    });
}

/// Check if a result indicates the goroutine should be parked
fn should_park_goroutine(result: &RuntimeValue) -> bool {
    if let RuntimeValue::Struct { name, fields } = result {
        if name == "Result" {
            if let Some(RuntimeValue::String(error_msg)) = fields.get("error_msg") {
                return error_msg == "__GOROUTINE_PARK__";
            }
        }
    }
    false
}

/// Request to park the current goroutine for I/O
/// This is called from builtin functions when they need to wait for I/O
/// Returns true if the goroutine was parked, false if not in a goroutine context
pub fn request_park_for_io(fd: std::os::unix::io::RawFd, event: crate::runtime::netpoller::PollEvent) -> bool {
    if let Some(gid) = get_current_goroutine_id() {
        // println!("🅿️  REQUEST_PARK: Goroutine {} requesting park for fd {}", gid, fd);
        
        // Register with netpoller
        if let Some(netpoller) = crate::runtime::netpoller::get_netpoller() {
            if let Err(e) = netpoller.register(fd, gid, event) {
                eprintln!("❌ Failed to register fd {} with netpoller: {}", fd, e);
                return false;
            }
            
            // println!("🅿️  REQUEST_PARK: Goroutine {} registered with netpoller, will be parked", gid);
            // The goroutine will be parked by returning a special signal
            // For now, we just register and the blocking call will use non-blocking mode
            return true;
        }
    }
    false
}

/// Park a goroutine using the global runtime
/// This is called from worker threads when a goroutine returns a Park signal
pub(crate) fn park_goroutine_global(
    goroutine_id: GoroutineId,
    goroutine: Goroutine,
    fd: std::os::unix::io::RawFd,
    event: crate::runtime::netpoller::PollEvent,
) -> std::io::Result<()> {
    get_runtime().park_goroutine(goroutine_id, goroutine, fd, event)
}

/// Unique identifier for goroutines
pub type GoroutineId = u64;

/// Goroutine state
#[derive(Debug, Clone, PartialEq)]
pub enum GoroutineState {
    Ready,     // Ready to run
    Running,   // Currently executing
    Blocked,   // Blocked on channel operation
    Parked,    // Parked waiting for I/O
    Completed, // Finished execution
    Panicked,  // Panicked during execution
}

/// A lightweight goroutine (green thread)
#[derive(Debug)]
pub struct Goroutine {
    pub id: GoroutineId,
    pub state: GoroutineState,
    pub task: GoroutineTask,
    pub result: Option<RuntimeValue>,
    pub error: Option<String>,
}

/// Task to be executed by a goroutine
#[derive(Debug, Clone)]
pub enum GoroutineTask {
    /// Execute a function with arguments
    Function {
        name: String,
        args: Vec<RuntimeValue>,
        program: Arc<IrProgram>,
        globals: std::collections::HashMap<String, RuntimeValue>,
        struct_definitions:
            std::collections::HashMap<String, crate::runtime::interpreter::StructDefinition>,
    },
    /// Execute a closure/lambda
    Closure {
        function: Arc<IrFunction>,
        captured_vars: std::collections::HashMap<String, RuntimeValue>,
        args: Vec<RuntimeValue>,
    },
    /// Execute an expression
    Expression { expr: RuntimeValue },
}

impl Goroutine {
    pub fn new(id: GoroutineId, task: GoroutineTask) -> Self {
        Self {
            id,
            state: GoroutineState::Ready,
            task,
            result: None,
            error: None,
        }
    }
}

/// Work-stealing queue for goroutines
#[derive(Debug)]
pub struct WorkQueue {
    queue: VecDeque<Goroutine>,
}

impl WorkQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub fn push(&mut self, goroutine: Goroutine) {
        self.queue.push_back(goroutine);
    }

    pub fn pop(&mut self) -> Option<Goroutine> {
        self.queue.pop_front()
    }

    pub fn steal(&mut self) -> Option<Goroutine> {
        self.queue.pop_back()
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

/// Statistics for the goroutine runtime
#[derive(Debug, Default, Clone)]
pub struct RuntimeStats {
    pub total_goroutines: u64,
    pub active_goroutines: u64,
    pub completed_goroutines: u64,
    pub panicked_goroutines: u64,
    pub worker_threads: usize,
}

/// Goroutine runtime - manages the execution of goroutines
pub struct GoroutineRuntime {
    // Core components
    global_queue: Arc<Mutex<WorkQueue>>,
    local_queues: Vec<Arc<Mutex<WorkQueue>>>,
    parked_queue: Arc<Mutex<HashMap<GoroutineId, Goroutine>>>,

    // Worker management
    workers: Vec<JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,

    // Synchronization
    condvar: Arc<Condvar>,

    // State
    next_id: AtomicU64,
    stats: Arc<Mutex<RuntimeStats>>,

    // Configuration
    num_workers: usize,
    
    // Network poller
    netpoller: Option<Arc<crate::runtime::netpoller::NetPoller>>,
}

impl GoroutineRuntime {
    /// Create a new goroutine runtime with specified number of workers
    pub fn new(num_workers: usize) -> Self {
        let num_workers = if num_workers == 0 {
            std::cmp::max(1, num_cpus::get())
        } else {
            num_workers
        };

        let global_queue = Arc::new(Mutex::new(WorkQueue::new()));
        let mut local_queues = Vec::new();

        // Create local queues for each worker
        for _ in 0..num_workers {
            local_queues.push(Arc::new(Mutex::new(WorkQueue::new())));
        }

        let shutdown = Arc::new(AtomicBool::new(false));
        let condvar = Arc::new(Condvar::new());
        let stats = Arc::new(Mutex::new(RuntimeStats {
            worker_threads: num_workers,
            ..Default::default()
        }));
        let parked_queue = Arc::new(Mutex::new(HashMap::new()));

        let mut workers = Vec::new();

        // Spawn worker threads
        for worker_id in 0..num_workers {
            let global_queue = Arc::clone(&global_queue);
            let local_queue = Arc::clone(&local_queues[worker_id]);
            let other_queues: Vec<_> = local_queues
                .iter()
                .enumerate()
                .filter(|(i, _)| *i != worker_id)
                .map(|(_, q)| Arc::clone(q))
                .collect();
            let shutdown = Arc::clone(&shutdown);
            let condvar = Arc::clone(&condvar);
            let stats = Arc::clone(&stats);

            let parked_queue_clone = Arc::clone(&parked_queue);
            
            let handle = thread::Builder::new()
                .name(format!("goroutine-worker-{}", worker_id))
                .spawn(move || {
                    Self::worker_loop(
                        worker_id,
                        global_queue,
                        local_queue,
                        other_queues,
                        parked_queue_clone,
                        shutdown,
                        condvar,
                        stats,
                    );
                })
                .expect("Failed to spawn goroutine worker thread");

            workers.push(handle);
        }

        // Initialize netpoller and syscall pool
        crate::runtime::netpoller::init_netpoller();
        let netpoller = crate::runtime::netpoller::get_netpoller();
        
        crate::runtime::syscall_thread::init_syscall_pool(4); // 4 syscall threads
        
        let runtime = Self {
            global_queue,
            local_queues,
            parked_queue,
            workers,
            shutdown: Arc::clone(&shutdown),
            condvar: Arc::clone(&condvar),
            next_id: AtomicU64::new(1),
            stats,
            num_workers,
            netpoller: netpoller.clone(),
        };
        
        // Start netpoller thread if available
        if let Some(poller) = netpoller {
            let parked_queue = Arc::clone(&runtime.parked_queue);
            let global_queue = Arc::clone(&runtime.global_queue);
            let shutdown = Arc::clone(&runtime.shutdown);
            let condvar = Arc::clone(&runtime.condvar);
            let stats = Arc::clone(&runtime.stats);
            
            thread::Builder::new()
                .name("netpoller".to_string())
                .spawn(move || {
                    // println!("🔌 NETPOLLER: Thread started (monitoring mode)");
                    while !shutdown.load(Ordering::Relaxed) {
                        // Poll for ready file descriptors
                        match poller.poll(Duration::from_millis(100)) {
                            Ok(ready_goroutines) => {
                                if !ready_goroutines.is_empty() {
                                    // Just log that fds are ready
                                    // The goroutines polling these fds will detect readiness on their next attempt
                                    // println!("🔌 NETPOLLER: {} fds ready for I/O", ready_goroutines.len());
                                }
                            }
                            Err(e) => {
                                eprintln!("❌ NETPOLLER: Poll error: {}", e);
                            }
                        }
                    }
                    // println!("🔌 NETPOLLER: Thread shutting down");
                })
                .expect("Failed to spawn netpoller thread");
        }
        
        // Start syscall result checker thread
        {
            let global_queue = Arc::clone(&runtime.global_queue);
            let shutdown = Arc::clone(&runtime.shutdown);
            let condvar = Arc::clone(&runtime.condvar);
            let parked_queue = Arc::clone(&runtime.parked_queue);
            
            thread::Builder::new()
                .name("syscall-checker".to_string())
                .spawn(move || {
                    // println!("🔧 SYSCALL_CHECKER: Thread started");
                    while !shutdown.load(Ordering::Relaxed) {
                        // Check for completed syscall results
                        if let Some(result) = crate::runtime::syscall_thread::try_get_blocking_result() {
                            println!("✅ SYSCALL_CHECKER: Got result for goroutine {}", result.goroutine_id);
                            
                            // Remove from parked queue if it's there
                            let mut parked = parked_queue.lock().unwrap();
                            if let Some(mut goroutine) = parked.remove(&result.goroutine_id) {
                                // Store the result in the goroutine somehow
                                // For now, we'll need to modify the goroutine to carry the result
                                goroutine.state = GoroutineState::Ready;
                                goroutine.result = Some(result.result.unwrap_or(RuntimeValue::Null));
                                
                                drop(parked);
                                
                                // Add back to global queue
                                let mut queue = global_queue.lock().unwrap();
                                queue.push(goroutine);
                                drop(queue);
                                
                                // Notify workers
                                condvar.notify_one();
                            }
                        }
                        
                        // Sleep briefly
                        std::thread::sleep(Duration::from_millis(1));
                    }
                    // println!("🔧 SYSCALL_CHECKER: Thread shutting down");
                })
                .expect("Failed to spawn syscall checker thread");
        }
        
        runtime
    }

    /// Spawn a new goroutine
    pub fn spawn(&self, task: GoroutineTask) -> GoroutineId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let goroutine = Goroutine::new(id, task);

        // println!(
        //     "📋 SPAWN: Creating goroutine {} on thread {:?}",
        //     id,
        //     std::thread::current().id()
        // );

        // Try to add to a local queue first for better cache locality
        if !self.local_queues.is_empty() {
            let queue_index = (id as usize) % self.local_queues.len();
            if let Ok(mut queue) = self.local_queues[queue_index].try_lock() {
                // println!(
                //     "📋 SPAWN: Added goroutine {} to local queue {}",
                //     id, queue_index
                // );
                queue.push(goroutine);
            } else {
                // If local queue is locked, add to global queue
                // println!(
                //     "📋 SPAWN: Local queue locked, adding goroutine {} to global queue",
                //     id
                // );
                let mut queue = self.global_queue.lock().unwrap();
                queue.push(goroutine);
            }
        } else {
            // No local queues, add to global queue
            // println!(
            //     "📋 SPAWN: No local queues, adding goroutine {} to global queue",
            //     id
            // );
            let mut queue = self.global_queue.lock().unwrap();
            queue.push(goroutine);
        }

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.total_goroutines += 1;
            stats.active_goroutines += 1;
        }

        // Notify workers
        self.condvar.notify_one();

        id
    }
    
    /// Park a goroutine waiting for I/O
    pub fn park_goroutine(&self, goroutine_id: GoroutineId, mut goroutine: Goroutine, fd: std::os::unix::io::RawFd, event: crate::runtime::netpoller::PollEvent) -> std::io::Result<()> {
        // println!("🅿️  PARK: Parking goroutine {} waiting for fd {}", goroutine_id, fd);
        
        // Register with netpoller
        if let Some(ref netpoller) = self.netpoller {
            netpoller.register(fd, goroutine_id, event)?;
        }
        
        // Mark as parked and store
        goroutine.state = GoroutineState::Parked;
        let mut parked = self.parked_queue.lock().unwrap();
        parked.insert(goroutine_id, goroutine);
        
        // Update stats
        let mut stats = self.stats.lock().unwrap();
        stats.active_goroutines -= 1;
        
        Ok(())
    }
    
    /// Unpark a goroutine and make it ready to run
    pub fn unpark_goroutine(&self, goroutine_id: GoroutineId, fd: std::os::unix::io::RawFd) -> std::io::Result<()> {
        // println!("🔓 UNPARK: Unparking goroutine {}", goroutine_id);
        
        // Unregister from netpoller
        if let Some(ref netpoller) = self.netpoller {
            netpoller.unregister(fd, goroutine_id)?;
        }
        
        // Remove from parked queue
        let mut parked = self.parked_queue.lock().unwrap();
        if let Some(mut goroutine) = parked.remove(&goroutine_id) {
            // Mark as ready
            goroutine.state = GoroutineState::Ready;
            
            // Add back to global queue
            drop(parked); // Release lock before acquiring another
            let mut queue = self.global_queue.lock().unwrap();
            queue.push(goroutine);
            
            // Update stats
            let mut stats = self.stats.lock().unwrap();
            stats.active_goroutines += 1;
            
            // Notify workers
            self.condvar.notify_one();
        }
        
        Ok(())
    }
    
    /// Worker thread main loop
    fn worker_loop(
        worker_id: usize,
        global_queue: Arc<Mutex<WorkQueue>>,
        local_queue: Arc<Mutex<WorkQueue>>,
        other_queues: Vec<Arc<Mutex<WorkQueue>>>,
        parked_queue: Arc<Mutex<HashMap<GoroutineId, Goroutine>>>,
        shutdown: Arc<AtomicBool>,
        condvar: Arc<Condvar>,
        stats: Arc<Mutex<RuntimeStats>>,
    ) {
        // println!(
        //     "⚙️  WORKER {}: Started on thread {:?}",
        //     worker_id,
        //     std::thread::current().id()
        // );

        while !shutdown.load(Ordering::Relaxed) {
            // Try to get work in order of preference:
            // 1. Local queue
            // 2. Global queue
            // 3. Steal from other workers
            let mut goroutine = None;

            // Try local queue first
            if let Ok(mut queue) = local_queue.try_lock() {
                goroutine = queue.pop();
            }

            // Try global queue
            if goroutine.is_none() {
                if let Ok(mut queue) = global_queue.try_lock() {
                    goroutine = queue.pop();
                }
            }

            // Try work stealing
            if goroutine.is_none() {
                for other_queue in &other_queues {
                    if let Ok(mut queue) = other_queue.try_lock() {
                        if let Some(stolen) = queue.steal() {
                            goroutine = Some(stolen);
                            break;
                        }
                    }
                }
            }

            if let Some(mut g) = goroutine {
                // println!(
                //     "⚙️  WORKER {}: Found goroutine {} to execute",
                //     worker_id, g.id
                // );

                // Execute the goroutine
                g.state = GoroutineState::Running;

                match Self::execute_goroutine(&mut g) {
                    Ok(result) => {
                        // Check if the goroutine should be parked (syscall in progress)
                        if should_park_goroutine(&result) {
                            // println!("🅿️  WORKER {}: Parking goroutine {} for I/O", worker_id, g.id);
                            
                            // Park the goroutine
                            g.state = GoroutineState::Parked;
                            let mut parked = parked_queue.lock().unwrap();
                            parked.insert(g.id, g);
                            
                            // Update stats
                            let mut stats = stats.lock().unwrap();
                            stats.active_goroutines -= 1;
                        } else {
                            // Normal completion
                            g.state = GoroutineState::Completed;
                            g.result = Some(result);

                            // Update stats
                            let mut stats = stats.lock().unwrap();
                            stats.active_goroutines -= 1;
                            stats.completed_goroutines += 1;
                        }
                    }
                    Err(e) => {
                        g.state = GoroutineState::Panicked;
                        g.error = Some(format!("{:?}", e));

                        // Update stats
                        let mut stats = stats.lock().unwrap();
                        stats.active_goroutines -= 1;
                        stats.panicked_goroutines += 1;

                        eprintln!("Goroutine {} panicked: {:?}", g.id, e);
                    }
                }
            } else {
                // No work available, wait for notification
                let _guard = condvar
                    .wait_timeout(global_queue.lock().unwrap(), Duration::from_millis(10))
                    .unwrap();
            }
        }

        println!("Goroutine worker {} shutting down", worker_id);
    }

    /// Execute a single goroutine with better error handling
    fn execute_goroutine(goroutine: &mut Goroutine) -> Result<RuntimeValue> {
        // println!(
        //     "🔄 GOROUTINE {}: Starting execution on thread {:?}",
        //     goroutine.id,
        //     std::thread::current().id()
        // );

        // Set the goroutine context for this thread
        set_current_goroutine_id(goroutine.id);
        crate::runtime::builtins::set_goroutine_context(true);

        match &goroutine.task {
            GoroutineTask::Function {
                name,
                args,
                program,
                globals,
                struct_definitions,
            } => {
                // println!(
                //     "🔄 GOROUTINE {}: Executing function '{}' with {} args",
                //     goroutine.id,
                //     name,
                //     args.len()
                // );

                // Create an interpreter with the globals and struct definitions from the parent context
                let mut interpreter =
                    crate::runtime::interpreter::Interpreter::new_for_goroutine_with_context(
                        globals.clone(),
                        struct_definitions.clone(),
                    );
                interpreter.set_program(program.clone());

                // Execute the function with proper error handling
                if let Some(function) = program.functions.iter().find(|f| f.name == *name) {
                    // println!(
                    //     "🔄 GOROUTINE {}: Found function '{}', executing...",
                    //     goroutine.id, name
                    // );
                    // Use the normal IR execution method
                    match interpreter.call_function(function, args.clone()) {
                        Ok(result) => {
                            // println!(
                            //     "🔄 GOROUTINE {}: Function '{}' completed successfully",
                            //     goroutine.id, name
                            // );
                            Ok(result)
                        }
                        Err(e) => {
                            // Log the error but don't panic the goroutine
                            eprintln!(
                                "❌ Goroutine {} function execution error: {:?}",
                                goroutine.id, e
                            );
                            Ok(RuntimeValue::Null)
                        }
                    }
                } else {
                    println!(
                        "🔄 GOROUTINE {}: Function '{}' not found, trying builtins",
                        goroutine.id, name
                    );
                    // Try built-in functions
                    match interpreter.call_builtin_function(name, args) {
                        Ok(result) => Ok(result),
                        Err(e) => {
                            eprintln!(
                                "❌ Goroutine {} builtin function error: {:?}",
                                goroutine.id, e
                            );
                            Ok(RuntimeValue::Null)
                        }
                    }
                }
            }
            GoroutineTask::Closure {
                function,
                captured_vars,
                args,
            } => {
                // Create interpreter with captured variables
                let mut interpreter = crate::runtime::interpreter::Interpreter::new_for_goroutine();

                // Set captured variables in the interpreter context
                for (name, value) in captured_vars {
                    interpreter.set_global(name.clone(), value.clone());
                }

                // Execute the closure with error handling
                match interpreter.execute_function_safely(function, args.clone()) {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        eprintln!(
                            "Goroutine {} closure execution error: {:?}",
                            goroutine.id, e
                        );
                        Ok(RuntimeValue::Null)
                    }
                }
            }
            GoroutineTask::Expression { expr } => {
                // Execute simple expression
                let result = match expr {
                    RuntimeValue::Function(func_name) => {
                        let mut interpreter =
                            crate::runtime::interpreter::Interpreter::new_for_goroutine();
                        match interpreter.call_builtin_function(func_name, &[]) {
                            Ok(result) => Ok(result),
                            Err(e) => {
                                eprintln!("Goroutine {} expression error: {:?}", goroutine.id, e);
                                Ok(RuntimeValue::Null)
                            }
                        }
                    }
                    _ => Ok(expr.clone()),
                };
                
                // Clean up context
                clear_current_goroutine_id();
                result
            }
        }
    }

    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStats {
        let mut stats = self.stats.lock().unwrap().clone();
        stats.worker_threads = self.num_workers;
        stats
    }

    /// Get the number of worker threads
    pub fn worker_count(&self) -> usize {
        self.num_workers
    }

    /// Get the number of local queues
    pub fn queue_count(&self) -> usize {
        self.local_queues.len()
    }

    /// Shutdown the runtime
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.condvar.notify_all();
    }
}

impl Drop for GoroutineRuntime {
    fn drop(&mut self) {
        self.shutdown();

        // Wait for all workers to finish
        while let Some(handle) = self.workers.pop() {
            let _ = handle.join();
        }
    }
}

/// Global goroutine runtime instance
static mut GLOBAL_RUNTIME: Option<GoroutineRuntime> = None;
static RUNTIME_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global goroutine runtime
pub fn init_runtime(num_workers: Option<usize>) {
    RUNTIME_INIT.call_once(|| {
        let workers = num_workers.unwrap_or_else(|| std::cmp::max(1, num_cpus::get()));
        unsafe {
            GLOBAL_RUNTIME = Some(GoroutineRuntime::new(workers));
        }
    });
}

/// Get the global goroutine runtime
pub fn get_runtime() -> &'static GoroutineRuntime {
    unsafe {
        match GLOBAL_RUNTIME.as_ref() {
            Some(runtime) => runtime,
            None => panic!("Goroutine runtime not initialized. Call init_runtime() first."),
        }
    }
}

/// Spawn a goroutine using the global runtime
pub fn spawn(task: GoroutineTask) -> GoroutineId {
    get_runtime().spawn(task)
}

/// Get runtime statistics
pub fn runtime_stats() -> RuntimeStats {
    get_runtime().stats()
}

/// Wait for all active goroutines to complete
/// This function blocks until all goroutines have finished executing
pub fn wait_all() {
    // println!("⏳ Waiting for all goroutines to complete...");

    let runtime = get_runtime();
    let start = std::time::Instant::now();
    let timeout = Duration::from_secs(30); // 30 second timeout

    loop {
        let stats = runtime.stats();

        if stats.active_goroutines == 0 {
            println!("✅ All goroutines completed!");
            println!(
                "   Total: {}, Completed: {}, Panicked: {}",
                stats.total_goroutines, stats.completed_goroutines, stats.panicked_goroutines
            );
            break;
        }

        if start.elapsed() > timeout {
            println!(
                "⚠️  Timeout waiting for goroutines. Still active: {}",
                stats.active_goroutines
            );
            break;
        }

        // Sleep briefly to avoid busy waiting
        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Wait for all active goroutines with a custom timeout
pub fn wait_all_timeout(timeout: Duration) -> bool {
    let runtime = get_runtime();
    let start = std::time::Instant::now();

    loop {
        let stats = runtime.stats();

        if stats.active_goroutines == 0 {
            return true;
        }

        if start.elapsed() > timeout {
            return false;
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

/// Park the current goroutine waiting for I/O on a file descriptor
/// This should be called from within a goroutine when it needs to wait for I/O
pub fn park_current(goroutine_id: GoroutineId, goroutine: Goroutine, fd: std::os::unix::io::RawFd, event: crate::runtime::netpoller::PollEvent) -> std::io::Result<()> {
    get_runtime().park_goroutine(goroutine_id, goroutine, fd, event)
}

/// Unpark a goroutine that was waiting for I/O
pub fn unpark(goroutine_id: GoroutineId, fd: std::os::unix::io::RawFd) -> std::io::Result<()> {
    get_runtime().unpark_goroutine(goroutine_id, fd)
}
