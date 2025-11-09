// Syscall thread pool for blocking I/O operations
// Inspired by Go's syscall thread mechanism

use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use super::goroutine::{Goroutine, GoroutineId};
use super::super::types::primitive::RuntimeValue;
use super::super::error::Result;

/// A blocking I/O operation to be executed
pub enum BlockingOp {
    /// TCP accept operation
    TcpAccept {
        listener: Arc<Mutex<std::net::TcpListener>>,
    },
    /// TCP read operation  
    TcpRead {
        stream: Arc<Mutex<std::net::TcpStream>>,
        buffer_size: usize,
    },
    /// TCP write operation
    TcpWrite {
        stream: Arc<Mutex<std::net::TcpStream>>,
        data: Vec<u8>,
    },
}

/// Result of a blocking operation
pub struct BlockingResult {
    pub goroutine_id: GoroutineId,
    pub result: Result<RuntimeValue>,
}

/// A task for a syscall thread
pub struct SyscallTask {
    pub goroutine_id: GoroutineId,
    pub operation: BlockingOp,
    pub response_channel: Option<std::sync::mpsc::Sender<Result<RuntimeValue>>>,
}

/// Pool of threads for blocking syscalls
pub struct SyscallThreadPool {
    // Queue of pending syscall tasks
    task_queue: Arc<Mutex<VecDeque<SyscallTask>>>,
    
    // Queue of completed results
    result_queue: Arc<Mutex<VecDeque<BlockingResult>>>,
    
    // Worker threads
    workers: Vec<JoinHandle<()>>,
    
    // Synchronization
    task_condvar: Arc<Condvar>,
    shutdown: Arc<AtomicBool>,
    
    // Stats
    active_syscalls: Arc<AtomicU64>,
}

impl SyscallThreadPool {
    /// Create a new syscall thread pool
    pub fn new(num_threads: usize) -> Self {
        let task_queue = Arc::new(Mutex::new(VecDeque::new()));
        let result_queue = Arc::new(Mutex::new(VecDeque::new()));
        let task_condvar = Arc::new(Condvar::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let active_syscalls = Arc::new(AtomicU64::new(0));
        
        let mut workers = Vec::new();
        
        // Spawn syscall worker threads
        for worker_id in 0..num_threads {
            let task_queue = Arc::clone(&task_queue);
            let result_queue = Arc::clone(&result_queue);
            let task_condvar = Arc::clone(&task_condvar);
            let shutdown = Arc::clone(&shutdown);
            let active_syscalls = Arc::clone(&active_syscalls);
            
            let handle = thread::Builder::new()
                .name(format!("syscall-worker-{}", worker_id))
                .spawn(move || {
                    Self::worker_loop(
                        worker_id,
                        task_queue,
                        result_queue,
                        task_condvar,
                        shutdown,
                        active_syscalls,
                    );
                })
                .expect("Failed to spawn syscall worker thread");
            
            workers.push(handle);
        }
        
        // println!("🔧 SYSCALL_POOL: Initialized with {} threads", num_threads);
        
        Self {
            task_queue,
            result_queue,
            workers,
            task_condvar,
            shutdown,
            active_syscalls,
        }
    }
    
    /// Submit a blocking operation
    pub fn submit(&self, task: SyscallTask) {
        // println!("📤 SYSCALL_POOL: Submitting task for goroutine {}", task.goroutine_id);
        
        let mut queue = self.task_queue.lock().unwrap();
        queue.push_back(task);
        drop(queue);
        
        self.task_condvar.notify_one();
    }
    
    /// Try to get a completed result
    pub fn try_get_result(&self) -> Option<BlockingResult> {
        let mut queue = self.result_queue.lock().unwrap();
        queue.pop_front()
    }
    
    /// Get number of active syscalls
    pub fn active_count(&self) -> u64 {
        self.active_syscalls.load(Ordering::Relaxed)
    }
    
    /// Worker loop for syscall threads
    fn worker_loop(
        worker_id: usize,
        task_queue: Arc<Mutex<VecDeque<SyscallTask>>>,
        result_queue: Arc<Mutex<VecDeque<BlockingResult>>>,
        task_condvar: Arc<Condvar>,
        shutdown: Arc<AtomicBool>,
        active_syscalls: Arc<AtomicU64>,
    ) {
        // println!("🔧 SYSCALL_WORKER {}: Started", worker_id);
        
        while !shutdown.load(Ordering::Relaxed) {
            // Wait for a task
            let task = {
                let mut queue = task_queue.lock().unwrap();
                
                while queue.is_empty() && !shutdown.load(Ordering::Relaxed) {
                    queue = task_condvar.wait(queue).unwrap();
                }
                
                if shutdown.load(Ordering::Relaxed) {
                    break;
                }
                
                queue.pop_front()
            };
            
            if let Some(task) = task {
                // println!("🔧 SYSCALL_WORKER {}: Executing blocking op for goroutine {}", 
                //     worker_id, task.goroutine_id);
                
                active_syscalls.fetch_add(1, Ordering::Relaxed);
                
                // Execute the blocking operation
                let result = Self::execute_blocking_op(task.operation);
                
                active_syscalls.fetch_sub(1, Ordering::Relaxed);
                
                // If there's a response channel, send the result directly
                if let Some(response_channel) = task.response_channel {
                    let _ = response_channel.send(result.clone());
                    // println!("✅ SYSCALL_WORKER {}: Sent result to goroutine {} via channel", 
                    //     worker_id, task.goroutine_id);
                } else {
                    // Otherwise, store in the result queue (old behavior)
                    let blocking_result = BlockingResult {
                        goroutine_id: task.goroutine_id,
                        result,
                    };
                    
                    let mut results = result_queue.lock().unwrap();
                    results.push_back(blocking_result);
                    
                    // println!("✅ SYSCALL_WORKER {}: Completed op for goroutine {}", 
                    //     worker_id, task.goroutine_id);
                }
            }
        }
        
        // println!("🔧 SYSCALL_WORKER {}: Shutting down", worker_id);
    }
    
    /// Execute a blocking operation
    fn execute_blocking_op(operation: BlockingOp) -> Result<RuntimeValue> {
        match operation {
            BlockingOp::TcpAccept { listener } => {
                // println!("🌐 SYSCALL: Executing blocking TCP accept");
                
                // This will block until a connection arrives
                let listener = listener.lock().unwrap();
                
                // Ensure the listener is in blocking mode
                listener.set_nonblocking(false).ok();
                
                match listener.accept() {
                    Ok((stream, peer_addr)) => {
                        // println!("🌐 SYSCALL: TCP accept succeeded from {}", peer_addr);
                        
                        // Store connection and create result
                        let connection_id = format!("conn_{}", peer_addr);
                        
                        // Store in TCP connections registry
                        if let Ok(mut connections) = crate::runtime::builtins::get_tcp_connections().lock() {
                            connections.insert(
                                connection_id.clone(),
                                Arc::new(Mutex::new(stream))
                            );
                        }
                        
                        // Create TcpConnection struct
                        let mut connection_fields = std::collections::HashMap::new();
                        connection_fields.insert(
                            "peer_addr".to_string(),
                            RuntimeValue::String(peer_addr.to_string()),
                        );
                        connection_fields.insert(
                            "connection_id".to_string(),
                            RuntimeValue::String(connection_id),
                        );
                        
                        let connection = RuntimeValue::Struct {
                            name: "TcpConnection".to_string(),
                            fields: connection_fields,
                        };
                        
                        // Create Result struct
                        let mut result_fields = std::collections::HashMap::new();
                        result_fields.insert("is_ok".to_string(), RuntimeValue::Bool(true));
                        result_fields.insert("value".to_string(), connection);
                        result_fields.insert("error_msg".to_string(), RuntimeValue::String("".to_string()));
                        
                        Ok(RuntimeValue::Struct {
                            name: "Result".to_string(),
                            fields: result_fields,
                        })
                    }
                    Err(e) => {
                        println!("❌ SYSCALL: TCP accept failed: {}", e);
                        
                        let mut result_fields = std::collections::HashMap::new();
                        result_fields.insert("is_ok".to_string(), RuntimeValue::Bool(false));
                        result_fields.insert("value".to_string(), RuntimeValue::Null);
                        result_fields.insert("error_msg".to_string(), RuntimeValue::String(e.to_string()));
                        
                        Ok(RuntimeValue::Struct {
                            name: "Result".to_string(),
                            fields: result_fields,
                        })
                    }
                }
            }
            BlockingOp::TcpRead { stream, buffer_size } => {
                use std::io::Read;
                
                // println!("🌐 SYSCALL: Executing blocking TCP read");
                
                let mut stream = stream.lock().unwrap();
                
                // Ensure the stream is in blocking mode
                stream.set_nonblocking(false).ok();
                
                let mut buffer = vec![0u8; buffer_size];
                
                match stream.read(&mut buffer) {
                    Ok(n) => {
                        // println!("🌐 SYSCALL: TCP read succeeded, {} bytes", n);
                        
                        buffer.truncate(n);
                        
                        // Convert buffer to RuntimeValue array
                        let byte_array: Vec<RuntimeValue> = buffer.iter()
                            .map(|&b| RuntimeValue::UInt8(b))
                            .collect();
                        
                        // Store the read data globally for string conversion
                        let data = String::from_utf8_lossy(&buffer).to_string();
                        
                        // Store in the global LAST_READ_DATA for string conversion
                        let last_read = crate::runtime::builtins::get_last_read_data();
                        if let Ok(mut last) = last_read.lock() {
                            *last = data.clone();
                        }
                        
                        // Create a custom result that includes both the byte count and the data
                        let mut result_fields = std::collections::HashMap::new();
                        result_fields.insert("is_ok".to_string(), RuntimeValue::Bool(true));
                        result_fields.insert("value".to_string(), RuntimeValue::Int64(n as i64));
                        result_fields.insert("data".to_string(), RuntimeValue::Array(byte_array));
                        result_fields.insert("error_msg".to_string(), RuntimeValue::String("".to_string()));
                        
                        Ok(RuntimeValue::Struct {
                            name: "Result".to_string(),
                            fields: result_fields,
                        })
                    }
                    Err(e) => {
                        println!("❌ SYSCALL: TCP read failed: {}", e);
                        
                        let mut result_fields = std::collections::HashMap::new();
                        result_fields.insert("is_ok".to_string(), RuntimeValue::Bool(false));
                        result_fields.insert("value".to_string(), RuntimeValue::Null);
                        result_fields.insert("error_msg".to_string(), RuntimeValue::String(e.to_string()));
                        
                        Ok(RuntimeValue::Struct {
                            name: "Result".to_string(),
                            fields: result_fields,
                        })
                    }
                }
            }
            BlockingOp::TcpWrite { stream, data } => {
                use std::io::Write;
                
                // println!("🌐 SYSCALL: Executing blocking TCP write");
                
                let mut stream = stream.lock().unwrap();
                
                // Ensure the stream is in blocking mode
                stream.set_nonblocking(false).ok();
                
                match stream.write_all(&data) {
                    Ok(()) => {
                        // println!("🌐 SYSCALL: TCP write succeeded, {} bytes", data.len());
                        
                        let mut result_fields = std::collections::HashMap::new();
                        result_fields.insert("is_ok".to_string(), RuntimeValue::Bool(true));
                        result_fields.insert("value".to_string(), RuntimeValue::Int64(data.len() as i64));
                        result_fields.insert("error_msg".to_string(), RuntimeValue::String("".to_string()));
                        
                        Ok(RuntimeValue::Struct {
                            name: "Result".to_string(),
                            fields: result_fields,
                        })
                    }
                    Err(e) => {
                        println!("❌ SYSCALL: TCP write failed: {}", e);
                        
                        let mut result_fields = std::collections::HashMap::new();
                        result_fields.insert("is_ok".to_string(), RuntimeValue::Bool(false));
                        result_fields.insert("value".to_string(), RuntimeValue::Null);
                        result_fields.insert("error_msg".to_string(), RuntimeValue::String(e.to_string()));
                        
                        Ok(RuntimeValue::Struct {
                            name: "Result".to_string(),
                            fields: result_fields,
                        })
                    }
                }
            }
        }
    }
    
    /// Shutdown the pool
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Relaxed);
        self.task_condvar.notify_all();
    }
}

impl Drop for SyscallThreadPool {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// Global syscall thread pool
static mut GLOBAL_SYSCALL_POOL: Option<Arc<SyscallThreadPool>> = None;
static SYSCALL_POOL_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global syscall thread pool
pub fn init_syscall_pool(num_threads: usize) {
    SYSCALL_POOL_INIT.call_once(|| {
        unsafe {
            GLOBAL_SYSCALL_POOL = Some(Arc::new(SyscallThreadPool::new(num_threads)));
        }
    });
}

/// Get the global syscall thread pool
pub fn get_syscall_pool() -> Option<Arc<SyscallThreadPool>> {
    unsafe { GLOBAL_SYSCALL_POOL.as_ref().map(Arc::clone) }
}

/// Submit a blocking operation to the syscall pool
pub fn submit_blocking_op(goroutine_id: GoroutineId, operation: BlockingOp) {
    init_syscall_pool(4); // Initialize with 4 threads if not already done
    
    if let Some(pool) = get_syscall_pool() {
        pool.submit(SyscallTask {
            goroutine_id,
            operation,
            response_channel: None,
        });
    }
}

/// Submit a blocking operation and wait for the result synchronously
pub fn submit_blocking_op_sync(goroutine_id: GoroutineId, operation: BlockingOp) -> Result<RuntimeValue> {
    init_syscall_pool(4); // Initialize with 4 threads if not already done
    
    // Create a channel for the response
    let (tx, rx) = std::sync::mpsc::channel();
    
    if let Some(pool) = get_syscall_pool() {
        pool.submit(SyscallTask {
            goroutine_id,
            operation,
            response_channel: Some(tx),
        });
        
        // Wait for the result
        match rx.recv() {
            Ok(result) => result,
            Err(_) => Err(crate::error::BuluError::RuntimeError {
                file: None,
                message: "Failed to receive syscall result".to_string(),
            }),
        }
    } else {
        Err(crate::error::BuluError::RuntimeError {
            file: None,
            message: "Syscall pool not initialized".to_string(),
        })
    }
}

/// Try to get a completed blocking operation result
pub fn try_get_blocking_result() -> Option<BlockingResult> {
    if let Some(pool) = get_syscall_pool() {
        pool.try_get_result()
    } else {
        None
    }
}
