// Network poller for async I/O operations
// Inspired by Go's netpoller implementation

use std::collections::HashMap;
use std::os::unix::io::RawFd;
use std::sync::{Arc, Mutex};
use std::time::Duration;

// Platform-specific imports
#[cfg(unix)]
use std::os::unix::io::AsRawFd;

/// Events that can be polled
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PollEvent {
    Read,
    Write,
    ReadWrite,
}

/// State of a waiting goroutine
#[derive(Debug, Clone)]
pub struct WaitingGoroutine {
    pub goroutine_id: u64,
    pub fd: RawFd,
    pub event: PollEvent,
}

/// Network poller that manages I/O readiness
pub struct NetPoller {
    #[cfg(target_os = "linux")]
    epoll_fd: RawFd,

    #[cfg(not(target_os = "linux"))]
    poll_fds: Vec<libc::pollfd>,

    // Map of fd -> waiting goroutines
    waiting: Arc<Mutex<HashMap<RawFd, Vec<WaitingGoroutine>>>>,
}

impl NetPoller {
    pub fn new() -> std::io::Result<Self> {
        #[cfg(target_os = "linux")]
        {
            let epoll_fd = unsafe { libc::epoll_create1(0) };
            if epoll_fd < 0 {
                return Err(std::io::Error::last_os_error());
            }

            Ok(Self {
                epoll_fd,
                waiting: Arc::new(Mutex::new(HashMap::new())),
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            Ok(Self {
                poll_fds: Vec::new(),
                waiting: Arc::new(Mutex::new(HashMap::new())),
            })
        }
    }

    /// Register a file descriptor for polling
    pub fn register(&self, fd: RawFd, goroutine_id: u64, event: PollEvent) -> std::io::Result<()> {
        println!(
            "🔌 NETPOLLER: Registering fd {} for goroutine {} with event {:?}",
            fd, goroutine_id, event
        );

        let mut waiting = self.waiting.lock().unwrap();
        let entry = waiting.entry(fd).or_insert_with(Vec::new);
        entry.push(WaitingGoroutine {
            goroutine_id,
            fd,
            event,
        });

        #[cfg(target_os = "linux")]
        {
            let mut ev = libc::epoll_event {
                events: match event {
                    PollEvent::Read => libc::EPOLLIN as u32,
                    PollEvent::Write => libc::EPOLLOUT as u32,
                    PollEvent::ReadWrite => (libc::EPOLLIN | libc::EPOLLOUT) as u32,
                },
                u64: fd as u64,
            };

            let result = unsafe {
                libc::epoll_ctl(
                    self.epoll_fd,
                    libc::EPOLL_CTL_ADD,
                    fd,
                    &mut ev as *mut libc::epoll_event,
                )
            };

            if result < 0 {
                let err = std::io::Error::last_os_error();
                // If already exists, try to modify instead
                if err.raw_os_error() == Some(libc::EEXIST) {
                    let result = unsafe {
                        libc::epoll_ctl(
                            self.epoll_fd,
                            libc::EPOLL_CTL_MOD,
                            fd,
                            &mut ev as *mut libc::epoll_event,
                        )
                    };
                    if result < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                } else {
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    /// Unregister a file descriptor
    pub fn unregister(&self, fd: RawFd, goroutine_id: u64) -> std::io::Result<()> {
        println!(
            "🔌 NETPOLLER: Unregistering fd {} for goroutine {}",
            fd, goroutine_id
        );

        let mut waiting = self.waiting.lock().unwrap();
        if let Some(goroutines) = waiting.get_mut(&fd) {
            goroutines.retain(|g| g.goroutine_id != goroutine_id);
            if goroutines.is_empty() {
                waiting.remove(&fd);

                #[cfg(target_os = "linux")]
                {
                    unsafe {
                        libc::epoll_ctl(
                            self.epoll_fd,
                            libc::EPOLL_CTL_DEL,
                            fd,
                            std::ptr::null_mut(),
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Poll for ready file descriptors
    /// Returns list of goroutine IDs that should be woken up
    pub fn poll(&self, timeout: Duration) -> std::io::Result<Vec<u64>> {
        #[cfg(target_os = "linux")]
        {
            let mut events: [libc::epoll_event; 128] = unsafe { std::mem::zeroed() };
            let timeout_ms = timeout.as_millis() as i32;

            let n = unsafe {
                libc::epoll_wait(
                    self.epoll_fd,
                    events.as_mut_ptr(),
                    events.len() as i32,
                    timeout_ms,
                )
            };

            if n < 0 {
                return Err(std::io::Error::last_os_error());
            }

            let mut ready_goroutines = Vec::new();
            let waiting = self.waiting.lock().unwrap();

            for i in 0..n as usize {
                let fd = events[i].u64 as RawFd;
                if let Some(goroutines) = waiting.get(&fd) {
                    for g in goroutines {
                        println!(
                            "🔌 NETPOLLER: fd {} is ready, waking goroutine {}",
                            fd, g.goroutine_id
                        );
                        ready_goroutines.push(g.goroutine_id);
                    }
                }
            }

            Ok(ready_goroutines)
        }

        #[cfg(not(target_os = "linux"))]
        {
            // Fallback using poll() for non-Linux systems
            let waiting = self.waiting.lock().unwrap();
            let mut poll_fds: Vec<libc::pollfd> = waiting
                .iter()
                .map(|(fd, goroutines)| {
                    let events = goroutines.iter().fold(0i16, |acc, g| {
                        acc | match g.event {
                            PollEvent::Read => libc::POLLIN,
                            PollEvent::Write => libc::POLLOUT,
                            PollEvent::ReadWrite => libc::POLLIN | libc::POLLOUT,
                        }
                    });

                    libc::pollfd {
                        fd: *fd,
                        events,
                        revents: 0,
                    }
                })
                .collect();

            if poll_fds.is_empty() {
                std::thread::sleep(timeout);
                return Ok(Vec::new());
            }

            let timeout_ms = timeout.as_millis() as i32;
            let n = unsafe {
                libc::poll(
                    poll_fds.as_mut_ptr(),
                    poll_fds.len() as libc::nfds_t,
                    timeout_ms,
                )
            };

            if n < 0 {
                return Err(std::io::Error::last_os_error());
            }

            let mut ready_goroutines = Vec::new();
            for pfd in &poll_fds {
                if pfd.revents != 0 {
                    if let Some(goroutines) = waiting.get(&pfd.fd) {
                        for g in goroutines {
                            println!(
                                "🔌 NETPOLLER: fd {} is ready, waking goroutine {}",
                                pfd.fd, g.goroutine_id
                            );
                            ready_goroutines.push(g.goroutine_id);
                        }
                    }
                }
            }

            Ok(ready_goroutines)
        }
    }
}

impl Drop for NetPoller {
    fn drop(&mut self) {
        #[cfg(target_os = "linux")]
        {
            unsafe {
                libc::close(self.epoll_fd);
            }
        }
    }
}

// Global netpoller instance
static mut GLOBAL_NETPOLLER: Option<Arc<NetPoller>> = None;
static NETPOLLER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the global netpoller
pub fn init_netpoller() {
    NETPOLLER_INIT.call_once(|| match NetPoller::new() {
        Ok(poller) => {
            unsafe {
                GLOBAL_NETPOLLER = Some(Arc::new(poller));
            }
            // println!("🔌 NETPOLLER: Initialized");
        }
        Err(e) => {
            eprintln!("❌ Failed to initialize netpoller: {}", e);
        }
    });
}

/// Get the global netpoller
pub fn get_netpoller() -> Option<Arc<NetPoller>> {
    unsafe { GLOBAL_NETPOLLER.as_ref().map(Arc::clone) }
}
