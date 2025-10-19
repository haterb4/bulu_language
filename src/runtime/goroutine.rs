// Goroutine runtime implementation based on M:N threading model
// Inspired by Go's runtime and Tokio's architecture

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar, atomic::{AtomicU64, AtomicBool, Ordering}};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use super::super::error::Result;
use super::super::types::primitive::RuntimeValue;
use super::super::compiler::ir::{IrFunction, IrProgram};

/// Unique identifier for goroutines
pub type GoroutineId = u64;

/// Goroutine state
#[derive(Debug, Clone, PartialEq)]
pub enum GoroutineState {
    Ready,      // Ready to run
    Running,    // Currently executing
    Blocked,    // Blocked on channel operation
    Completed,  // Finished execution
    Panicked,   // Panicked during execution
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
    },
    /// Execute a closure/lambda
    Closure {
        function: Arc<IrFunction>,
        captured_vars: std::collections::HashMap<String, RuntimeValue>,
        args: Vec<RuntimeValue>,
    },
    /// Execute an expression
    Expression {
        expr: RuntimeValue,
    },
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

        let mut workers = Vec::new();

        // Spawn worker threads
        for worker_id in 0..num_workers {
            let global_queue = Arc::clone(&global_queue);
            let local_queue = Arc::clone(&local_queues[worker_id]);
            let other_queues: Vec<_> = local_queues.iter()
                .enumerate()
                .filter(|(i, _)| *i != worker_id)
                .map(|(_, q)| Arc::clone(q))
                .collect();
            let shutdown = Arc::clone(&shutdown);
            let condvar = Arc::clone(&condvar);
            let stats = Arc::clone(&stats);

            let handle = thread::Builder::new()
                .name(format!("goroutine-worker-{}", worker_id))
                .spawn(move || {
                    Self::worker_loop(
                        worker_id,
                        global_queue,
                        local_queue,
                        other_queues,
                        shutdown,
                        condvar,
                        stats,
                    );
                })
                .expect("Failed to spawn goroutine worker thread");

            workers.push(handle);
        }

        Self {
            global_queue,
            local_queues,
            workers,
            shutdown,
            condvar,
            next_id: AtomicU64::new(1),
            stats,
            num_workers,
        }
    }

    /// Spawn a new goroutine
    pub fn spawn(&self, task: GoroutineTask) -> GoroutineId {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let goroutine = Goroutine::new(id, task);

        // Add to global queue
        {
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

    /// Worker thread main loop
    fn worker_loop(
        worker_id: usize,
        global_queue: Arc<Mutex<WorkQueue>>,
        local_queue: Arc<Mutex<WorkQueue>>,
        other_queues: Vec<Arc<Mutex<WorkQueue>>>,
        shutdown: Arc<AtomicBool>,
        condvar: Arc<Condvar>,
        stats: Arc<Mutex<RuntimeStats>>,
    ) {
        println!("Goroutine worker {} started", worker_id);

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
                // Execute the goroutine
                g.state = GoroutineState::Running;
                
                match Self::execute_goroutine(&mut g) {
                    Ok(result) => {
                        g.state = GoroutineState::Completed;
                        g.result = Some(result);
                        
                        // Update stats
                        let mut stats = stats.lock().unwrap();
                        stats.active_goroutines -= 1;
                        stats.completed_goroutines += 1;
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
                let _guard = condvar.wait_timeout(
                    global_queue.lock().unwrap(),
                    Duration::from_millis(10)
                ).unwrap();
            }
        }

        println!("Goroutine worker {} shutting down", worker_id);
    }

    /// Execute a single goroutine
    fn execute_goroutine(goroutine: &mut Goroutine) -> Result<RuntimeValue> {
        match &goroutine.task {
            GoroutineTask::Function { name, args, program } => {
                // Create a minimal interpreter for this goroutine
                let mut interpreter = crate::runtime::interpreter::Interpreter::new_for_goroutine();
                interpreter.set_program(program.clone());
                
                // Execute the function
                if let Some(function) = program.functions.iter().find(|f| f.name == *name) {
                    interpreter.call_function(function, args.clone())
                } else {
                    // Try built-in functions
                    interpreter.call_builtin_function(name, args)
                }
            }
            GoroutineTask::Closure { function, captured_vars, args } => {
                // Create interpreter with captured variables
                let mut interpreter = crate::runtime::interpreter::Interpreter::new_for_goroutine();
                
                // Set captured variables in the interpreter context
                for (name, value) in captured_vars {
                    interpreter.set_global(name.clone(), value.clone());
                }
                
                // Execute the closure
                interpreter.call_function(function, args.clone())
            }
            GoroutineTask::Expression { expr } => {
                // Execute simple expression
                match expr {
                    RuntimeValue::Function(func_name) => {
                        let mut interpreter = crate::runtime::interpreter::Interpreter::new_for_goroutine();
                        interpreter.call_builtin_function(func_name, &[])
                    }
                    _ => Ok(expr.clone())
                }
            }
        }
    }

    /// Get runtime statistics
    pub fn stats(&self) -> RuntimeStats {
        self.stats.lock().unwrap().clone()
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
        GLOBAL_RUNTIME.as_ref().expect("Goroutine runtime not initialized. Call init_runtime() first.")
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