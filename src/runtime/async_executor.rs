// Async executor for goroutines
// This provides the infrastructure for suspendable I/O operations

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Wake, Waker};

use super::super::error::Result;
use super::super::types::primitive::RuntimeValue;
use super::netpoller::{NetPoller, PollEvent};

/// A task that can be executed by the async executor
pub struct Task {
    pub id: u64,
    pub future: Mutex<Pin<Box<dyn Future<Output = Result<RuntimeValue>> + Send>>>,
}

impl Task {
    pub fn new(
        id: u64,
        future: Pin<Box<dyn Future<Output = Result<RuntimeValue>> + Send>>,
    ) -> Self {
        Self {
            id,
            future: Mutex::new(future),
        }
    }
}

/// Waker for goroutine tasks
struct GoroutineWaker {
    task_id: u64,
    executor: Arc<Mutex<AsyncExecutor>>,
}

impl Wake for GoroutineWaker {
    fn wake(self: Arc<Self>) {
        println!("🔔 WAKER: Waking task {}", self.task_id);
        if let Ok(mut executor) = self.executor.lock() {
            executor.wake_task(self.task_id);
        }
    }
}

/// Async executor for goroutines
pub struct AsyncExecutor {
    tasks: HashMap<u64, Arc<Task>>,
    ready_queue: Vec<u64>,
    netpoller: Option<Arc<NetPoller>>,
    next_task_id: u64,
}

impl AsyncExecutor {
    pub fn new(netpoller: Option<Arc<NetPoller>>) -> Self {
        Self {
            tasks: HashMap::new(),
            ready_queue: Vec::new(),
            netpoller,
            next_task_id: 1,
        }
    }

    /// Spawn a new async task
    pub fn spawn(
        &mut self,
        future: Pin<Box<dyn Future<Output = Result<RuntimeValue>> + Send>>,
    ) -> u64 {
        let task_id = self.next_task_id;
        self.next_task_id += 1;

        let task = Arc::new(Task::new(task_id, future));
        self.tasks.insert(task_id, task);
        self.ready_queue.push(task_id);

        // println!("📋 EXECUTOR: Spawned task {}", task_id);
        task_id
    }

    /// Wake a task and add it to the ready queue
    fn wake_task(&mut self, task_id: u64) {
        if self.tasks.contains_key(&task_id) {
            if !self.ready_queue.contains(&task_id) {
                self.ready_queue.push(task_id);
                println!("🔔 EXECUTOR: Task {} added to ready queue", task_id);
            }
        }
    }

    /// Run the executor until all tasks complete
    pub fn run(&mut self) -> Vec<Result<RuntimeValue>> {
        let mut results = Vec::new();

        while !self.tasks.is_empty() {
            // Get next ready task
            if let Some(task_id) = self.ready_queue.pop() {
                if let Some(task) = self.tasks.get(&task_id) {
                    let task = Arc::clone(task);

                    // Create waker for this task
                    let waker = Arc::new(GoroutineWaker {
                        task_id,
                        executor: Arc::new(Mutex::new(AsyncExecutor::new(self.netpoller.clone()))),
                    });
                    let waker = Waker::from(waker);
                    let mut context = Context::from_waker(&waker);

                    // Poll the future
                    if let Ok(mut future) = task.future.lock() {
                        match future.as_mut().poll(&mut context) {
                            Poll::Ready(result) => {
                                println!("✅ EXECUTOR: Task {} completed", task_id);
                                self.tasks.remove(&task_id);
                                results.push(result);
                            }
                            Poll::Pending => {
                                println!("⏸️  EXECUTOR: Task {} pending", task_id);
                                // Task is waiting, will be woken up later
                            }
                        }
                    };
                }
            } else {
                // No ready tasks, poll netpoller if available
                if let Some(ref netpoller) = self.netpoller {
                    match netpoller.poll(std::time::Duration::from_millis(10)) {
                        Ok(ready_tasks) => {
                            for task_id in ready_tasks {
                                self.wake_task(task_id);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ EXECUTOR: Netpoller error: {}", e);
                        }
                    }
                }

                // If still no ready tasks and no tasks at all, we're done
                if self.ready_queue.is_empty() && self.tasks.is_empty() {
                    break;
                }

                // Small sleep to avoid busy waiting
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }

        results
    }
}

/// Future for TCP accept operation
pub struct TcpAcceptFuture {
    listener: Arc<Mutex<std::net::TcpListener>>,
    fd: std::os::unix::io::RawFd,
    task_id: u64,
    registered: bool,
}

impl TcpAcceptFuture {
    pub fn new(
        listener: Arc<Mutex<std::net::TcpListener>>,
        fd: std::os::unix::io::RawFd,
        task_id: u64,
    ) -> Self {
        Self {
            listener,
            fd,
            task_id,
            registered: false,
        }
    }
}

impl Future for TcpAcceptFuture {
    type Output = std::io::Result<(std::net::TcpStream, std::net::SocketAddr)>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Try to accept
        let accept_result = if let Ok(listener) = self.listener.lock() {
            listener.accept()
        } else {
            return Poll::Pending;
        };

        match accept_result {
            Ok(result) => {
                println!("✅ TCP_ACCEPT_FUTURE: Connection accepted");
                // Unregister from netpoller
                if self.registered {
                    if let Some(netpoller) = crate::runtime::netpoller::get_netpoller() {
                        let _ = netpoller.unregister(self.fd, self.task_id);
                    }
                }
                Poll::Ready(Ok(result))
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // Register with netpoller if not already registered
                if !self.registered {
                    if let Some(netpoller) = crate::runtime::netpoller::get_netpoller() {
                        if let Ok(()) = netpoller.register(self.fd, self.task_id, PollEvent::Read) {
                            println!(
                                "🔌 TCP_ACCEPT_FUTURE: Registered fd {} with netpoller",
                                self.fd
                            );
                            self.registered = true;
                        }
                    }
                }

                // Wake us up when ready
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(e) => {
                // Real error
                if self.registered {
                    if let Some(netpoller) = crate::runtime::netpoller::get_netpoller() {
                        let _ = netpoller.unregister(self.fd, self.task_id);
                    }
                }
                Poll::Ready(Err(e))
            }
        }
    }
}
