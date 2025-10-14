//! Channel implementation for the Bulu language
//!
//! This module provides the complete channel system including:
//! - Unbuffered and buffered channels
//! - Send and receive operations
//! - Channel closing and iteration
//! - Send-only and receive-only channel types
//! - Select statement support

use crate::error::{BuluError, Result};
use crate::types::composite::ChannelDirection;
use crate::types::primitive::{RuntimeValue, TypeId};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

/// Channel runtime representation
#[derive(Debug)]
pub struct Channel {
    inner: Arc<Mutex<ChannelInner>>,
    send_notify: Arc<Condvar>,
    recv_notify: Arc<Condvar>,
    element_type: TypeId,
    direction: ChannelDirection,
}

#[derive(Debug)]
struct ChannelInner {
    buffer: VecDeque<RuntimeValue>,
    capacity: usize, // 0 for unbuffered
    closed: bool,
    waiting_senders: usize,
    waiting_receivers: usize,
}

/// Channel operation result
#[derive(Debug, Clone, PartialEq)]
pub enum ChannelResult {
    Ok(RuntimeValue),
    Closed,
    WouldBlock,
}

/// Channel send result
#[derive(Debug, Clone, PartialEq)]
pub enum SendResult {
    Ok,
    Closed,
    WouldBlock,
}

impl Channel {
    /// Create a new unbuffered channel
    pub fn new_unbuffered(element_type: TypeId) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ChannelInner {
                buffer: VecDeque::new(),
                capacity: 0,
                closed: false,
                waiting_senders: 0,
                waiting_receivers: 0,
            })),
            send_notify: Arc::new(Condvar::new()),
            recv_notify: Arc::new(Condvar::new()),
            element_type,
            direction: ChannelDirection::Bidirectional,
        }
    }

    /// Create a new buffered channel
    pub fn new_buffered(element_type: TypeId, capacity: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ChannelInner {
                buffer: VecDeque::with_capacity(capacity),
                capacity,
                closed: false,
                waiting_senders: 0,
                waiting_receivers: 0,
            })),
            send_notify: Arc::new(Condvar::new()),
            recv_notify: Arc::new(Condvar::new()),
            element_type,
            direction: ChannelDirection::Bidirectional,
        }
    }

    /// Create a send-only view of this channel
    pub fn send_only(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            send_notify: Arc::clone(&self.send_notify),
            recv_notify: Arc::clone(&self.recv_notify),
            element_type: self.element_type,
            direction: ChannelDirection::SendOnly,
        }
    }

    /// Create a receive-only view of this channel
    pub fn receive_only(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            send_notify: Arc::clone(&self.send_notify),
            recv_notify: Arc::clone(&self.recv_notify),
            element_type: self.element_type,
            direction: ChannelDirection::ReceiveOnly,
        }
    }

    /// Get the element type of this channel
    pub fn element_type(&self) -> TypeId {
        self.element_type
    }

    /// Get the direction of this channel
    pub fn direction(&self) -> ChannelDirection {
        self.direction
    }

    /// Check if this channel is closed
    pub fn is_closed(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.closed
    }

    /// Get the capacity of this channel (0 for unbuffered)
    pub fn capacity(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.capacity
    }

    /// Get the current length of the channel buffer
    pub fn len(&self) -> usize {
        let inner = self.inner.lock().unwrap();
        inner.buffer.len()
    }

    /// Check if the channel buffer is empty
    pub fn is_empty(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.buffer.is_empty()
    }

    /// Check if the channel buffer is full
    pub fn is_full(&self) -> bool {
        let inner = self.inner.lock().unwrap();
        inner.capacity > 0 && inner.buffer.len() >= inner.capacity
    }

    /// Send a value to the channel (blocking)
    pub fn send(&self, value: RuntimeValue) -> Result<SendResult> {
        if self.direction == ChannelDirection::ReceiveOnly {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot send on receive-only channel".to_string(),
            });
        }

        let mut inner = self.inner.lock().unwrap();

        // Check if channel is closed
        if inner.closed {
            return Ok(SendResult::Closed);
        }

        // For unbuffered channels or when buffer is full, wait for receiver
        if inner.capacity == 0 || inner.buffer.len() >= inner.capacity {
            inner.waiting_senders += 1;

            // Wait for space or receiver
            while !inner.closed && (inner.capacity == 0 || inner.buffer.len() >= inner.capacity) {
                inner = self.send_notify.wait(inner).unwrap();
            }

            inner.waiting_senders -= 1;

            // Check if channel was closed while waiting
            if inner.closed {
                return Ok(SendResult::Closed);
            }
        }

        // Add value to buffer
        inner.buffer.push_back(value);

        // Notify waiting receivers
        drop(inner);
        self.recv_notify.notify_one();

        Ok(SendResult::Ok)
    }

    /// Try to send a value to the channel (non-blocking)
    pub fn try_send(&self, value: RuntimeValue) -> Result<SendResult> {
        if self.direction == ChannelDirection::ReceiveOnly {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot send on receive-only channel".to_string(),
            });
        }

        let mut inner = self.inner.lock().unwrap();

        // Check if channel is closed
        if inner.closed {
            return Ok(SendResult::Closed);
        }

        // Check if we can send immediately
        if inner.capacity == 0 {
            // Unbuffered channel - need a waiting receiver
            if inner.waiting_receivers > 0 {
                inner.buffer.push_back(value);
                drop(inner);
                self.recv_notify.notify_one();
                Ok(SendResult::Ok)
            } else {
                Ok(SendResult::WouldBlock)
            }
        } else {
            // Buffered channel - check if there's space
            if inner.buffer.len() < inner.capacity {
                inner.buffer.push_back(value);
                drop(inner);
                self.recv_notify.notify_one();
                Ok(SendResult::Ok)
            } else {
                Ok(SendResult::WouldBlock)
            }
        }
    }

    /// Send a value with timeout
    pub fn send_timeout(&self, value: RuntimeValue, timeout: Duration) -> Result<SendResult> {
        if self.direction == ChannelDirection::ReceiveOnly {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot send on receive-only channel".to_string(),
            });
        }

        let start = Instant::now();
        let mut inner = self.inner.lock().unwrap();

        // Check if channel is closed
        if inner.closed {
            return Ok(SendResult::Closed);
        }

        // For unbuffered channels or when buffer is full, wait for receiver
        if inner.capacity == 0 || inner.buffer.len() >= inner.capacity {
            inner.waiting_senders += 1;

            // Wait for space or receiver with timeout
            while !inner.closed && (inner.capacity == 0 || inner.buffer.len() >= inner.capacity) {
                let remaining = timeout.saturating_sub(start.elapsed());
                if remaining.is_zero() {
                    inner.waiting_senders -= 1;
                    return Ok(SendResult::WouldBlock);
                }

                let (new_inner, timeout_result) = self.send_notify.wait_timeout(inner, remaining).unwrap();
                inner = new_inner;

                if timeout_result.timed_out() {
                    inner.waiting_senders -= 1;
                    return Ok(SendResult::WouldBlock);
                }
            }

            inner.waiting_senders -= 1;

            // Check if channel was closed while waiting
            if inner.closed {
                return Ok(SendResult::Closed);
            }
        }

        // Add value to buffer
        inner.buffer.push_back(value);

        // Notify waiting receivers
        drop(inner);
        self.recv_notify.notify_one();

        Ok(SendResult::Ok)
    }

    /// Receive a value from the channel (blocking)
    pub fn receive(&self) -> Result<ChannelResult> {
        if self.direction == ChannelDirection::SendOnly {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot receive from send-only channel".to_string(),
            });
        }

        let mut inner = self.inner.lock().unwrap();

        // Wait for data or channel close
        inner.waiting_receivers += 1;

        while inner.buffer.is_empty() && !inner.closed {
            inner = self.recv_notify.wait(inner).unwrap();
        }

        inner.waiting_receivers -= 1;

        // Check if we have data
        if let Some(value) = inner.buffer.pop_front() {
            // Notify waiting senders if there's space
            if inner.capacity == 0 || inner.buffer.len() < inner.capacity {
                drop(inner);
                self.send_notify.notify_one();
            }
            Ok(ChannelResult::Ok(value))
        } else if inner.closed {
            Ok(ChannelResult::Closed)
        } else {
            // This shouldn't happen, but handle it gracefully
            Ok(ChannelResult::Closed)
        }
    }

    /// Try to receive a value from the channel (non-blocking)
    pub fn try_receive(&self) -> Result<ChannelResult> {
        if self.direction == ChannelDirection::SendOnly {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot receive from send-only channel".to_string(),
            });
        }

        let mut inner = self.inner.lock().unwrap();

        if let Some(value) = inner.buffer.pop_front() {
            // Notify waiting senders if there's space
            if inner.capacity == 0 || inner.buffer.len() < inner.capacity {
                drop(inner);
                self.send_notify.notify_one();
            }
            Ok(ChannelResult::Ok(value))
        } else if inner.closed {
            Ok(ChannelResult::Closed)
        } else {
            Ok(ChannelResult::WouldBlock)
        }
    }

    /// Receive a value with timeout
    pub fn receive_timeout(&self, timeout: Duration) -> Result<ChannelResult> {
        if self.direction == ChannelDirection::SendOnly {
            return Err(BuluError::RuntimeError {
            file: None,
                message: "Cannot receive from send-only channel".to_string(),
            });
        }

        let start = Instant::now();
        let mut inner = self.inner.lock().unwrap();

        // Wait for data or channel close with timeout
        inner.waiting_receivers += 1;

        while inner.buffer.is_empty() && !inner.closed {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                inner.waiting_receivers -= 1;
                return Ok(ChannelResult::WouldBlock);
            }

            let (new_inner, timeout_result) = self.recv_notify.wait_timeout(inner, remaining).unwrap();
            inner = new_inner;

            if timeout_result.timed_out() {
                inner.waiting_receivers -= 1;
                return Ok(ChannelResult::WouldBlock);
            }
        }

        inner.waiting_receivers -= 1;

        // Check if we have data
        if let Some(value) = inner.buffer.pop_front() {
            // Notify waiting senders if there's space
            if inner.capacity == 0 || inner.buffer.len() < inner.capacity {
                drop(inner);
                self.send_notify.notify_one();
            }
            Ok(ChannelResult::Ok(value))
        } else if inner.closed {
            Ok(ChannelResult::Closed)
        } else {
            Ok(ChannelResult::WouldBlock)
        }
    }

    /// Close the channel
    pub fn close(&self) -> Result<()> {
        let mut inner = self.inner.lock().unwrap();
        
        if inner.closed {
            return Ok(()); // Already closed
        }

        inner.closed = true;

        // Notify all waiting senders and receivers
        drop(inner);
        self.send_notify.notify_all();
        self.recv_notify.notify_all();

        Ok(())
    }

    /// Create an iterator over the channel
    pub fn iter(&self) -> ChannelIterator {
        ChannelIterator {
            channel: self.clone(),
        }
    }
}

impl Clone for Channel {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::clone(&self.inner),
            send_notify: Arc::clone(&self.send_notify),
            recv_notify: Arc::clone(&self.recv_notify),
            element_type: self.element_type,
            direction: self.direction,
        }
    }
}

/// Iterator over channel values
pub struct ChannelIterator {
    channel: Channel,
}

impl Iterator for ChannelIterator {
    type Item = RuntimeValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self.channel.receive() {
            Ok(ChannelResult::Ok(value)) => Some(value),
            Ok(ChannelResult::Closed) | Ok(ChannelResult::WouldBlock) => None,
            Err(_) => None,
        }
    }
}

/// Channel registry for managing channels in the runtime
#[derive(Debug, Default)]
pub struct ChannelRegistry {
    channels: std::collections::HashMap<usize, Channel>,
    next_id: usize,
}

impl ChannelRegistry {
    pub fn new() -> Self {
        Self {
            channels: std::collections::HashMap::new(),
            next_id: 1,
        }
    }

    /// Register a new channel and return its ID
    pub fn register(&mut self, channel: Channel) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.channels.insert(id, channel);
        id
    }

    /// Get a channel by ID
    pub fn get(&self, id: usize) -> Option<&Channel> {
        self.channels.get(&id)
    }

    /// Remove a channel by ID
    pub fn remove(&mut self, id: usize) -> Option<Channel> {
        self.channels.remove(&id)
    }

    /// Get all channel IDs
    pub fn get_all_ids(&self) -> Vec<usize> {
        self.channels.keys().copied().collect()
    }

    /// Close all channels
    pub fn close_all(&mut self) {
        for channel in self.channels.values() {
            let _ = channel.close();
        }
    }
}

/// Select operation for multiplexing channels
pub struct SelectOperation {
    pub channel_id: usize,
    pub is_send: bool,
    pub send_value: Option<RuntimeValue>,
}

/// Result of a select operation
pub enum SelectResult {
    Ready(usize, ChannelResult), // channel_id, result
    Timeout,
    Error(BuluError),
}

/// Select implementation for channel multiplexing
pub fn select_channels(
    registry: &ChannelRegistry,
    operations: &[SelectOperation],
    timeout: Option<Duration>,
) -> SelectResult {
    let start = Instant::now();

    loop {
        // Try all operations non-blocking
        for (index, op) in operations.iter().enumerate() {
            if let Some(channel) = registry.get(op.channel_id) {
                if op.is_send {
                    if let Some(ref value) = op.send_value {
                        match channel.try_send(value.clone()) {
                            Ok(SendResult::Ok) => {
                                return SelectResult::Ready(index, ChannelResult::Ok(RuntimeValue::Null));
                            }
                            Ok(SendResult::Closed) => {
                                return SelectResult::Ready(index, ChannelResult::Closed);
                            }
                            Ok(SendResult::WouldBlock) => continue,
                            Err(e) => return SelectResult::Error(e),
                        }
                    }
                } else {
                    match channel.try_receive() {
                        Ok(ChannelResult::Ok(value)) => {
                            return SelectResult::Ready(index, ChannelResult::Ok(value));
                        }
                        Ok(ChannelResult::Closed) => {
                            return SelectResult::Ready(index, ChannelResult::Closed);
                        }
                        Ok(ChannelResult::WouldBlock) => continue,
                        Err(e) => return SelectResult::Error(e),
                    }
                }
            }
        }

        // Check timeout
        if let Some(timeout_duration) = timeout {
            if start.elapsed() >= timeout_duration {
                return SelectResult::Timeout;
            }
        }

        // Sleep briefly before trying again
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::primitive::TypeId;

    #[test]
    fn test_unbuffered_channel_creation() {
        let channel = Channel::new_unbuffered(TypeId::Int32);
        assert_eq!(channel.capacity(), 0);
        assert_eq!(channel.len(), 0);
        assert!(channel.is_empty());
        assert!(!channel.is_closed());
        assert_eq!(channel.direction(), ChannelDirection::Bidirectional);
    }

    #[test]
    fn test_buffered_channel_creation() {
        let channel = Channel::new_buffered(TypeId::String, 5);
        assert_eq!(channel.capacity(), 5);
        assert_eq!(channel.len(), 0);
        assert!(channel.is_empty());
        assert!(!channel.is_full());
        assert!(!channel.is_closed());
    }

    #[test]
    fn test_channel_directions() {
        let channel = Channel::new_unbuffered(TypeId::Int32);
        
        let send_only = channel.send_only();
        assert_eq!(send_only.direction(), ChannelDirection::SendOnly);
        
        let recv_only = channel.receive_only();
        assert_eq!(recv_only.direction(), ChannelDirection::ReceiveOnly);
    }

    #[test]
    fn test_buffered_channel_send_receive() {
        let channel = Channel::new_buffered(TypeId::Int32, 2);
        
        // Send values
        assert_eq!(channel.try_send(RuntimeValue::Int32(1)).unwrap(), SendResult::Ok);
        assert_eq!(channel.try_send(RuntimeValue::Int32(2)).unwrap(), SendResult::Ok);
        assert_eq!(channel.len(), 2);
        assert!(channel.is_full());
        
        // Buffer is full, should block
        assert_eq!(channel.try_send(RuntimeValue::Int32(3)).unwrap(), SendResult::WouldBlock);
        
        // Receive values
        match channel.try_receive().unwrap() {
            ChannelResult::Ok(RuntimeValue::Int32(1)) => {},
            other => panic!("Expected Ok(Int32(1)), got {:?}", other),
        }
        
        match channel.try_receive().unwrap() {
            ChannelResult::Ok(RuntimeValue::Int32(2)) => {},
            other => panic!("Expected Ok(Int32(2)), got {:?}", other),
        }
        
        assert_eq!(channel.len(), 0);
        assert!(channel.is_empty());
        
        // Channel is empty, should block
        assert_eq!(channel.try_receive().unwrap(), ChannelResult::WouldBlock);
    }

    #[test]
    fn test_channel_close() {
        let channel = Channel::new_buffered(TypeId::Int32, 1);
        
        // Send a value
        assert_eq!(channel.try_send(RuntimeValue::Int32(42)).unwrap(), SendResult::Ok);
        
        // Close the channel
        channel.close().unwrap();
        assert!(channel.is_closed());
        
        // Can still receive existing values
        match channel.try_receive().unwrap() {
            ChannelResult::Ok(RuntimeValue::Int32(42)) => {},
            other => panic!("Expected Ok(Int32(42)), got {:?}", other),
        }
        
        // Further receives return Closed
        assert_eq!(channel.try_receive().unwrap(), ChannelResult::Closed);
        
        // Sends to closed channel return Closed
        assert_eq!(channel.try_send(RuntimeValue::Int32(1)).unwrap(), SendResult::Closed);
    }

    #[test]
    fn test_send_only_channel_restrictions() {
        let channel = Channel::new_unbuffered(TypeId::Int32);
        let send_only = channel.send_only();
        
        // Can send
        assert!(send_only.try_send(RuntimeValue::Int32(1)).is_ok());
        
        // Cannot receive
        assert!(send_only.try_receive().is_err());
    }

    #[test]
    fn test_receive_only_channel_restrictions() {
        let channel = Channel::new_buffered(TypeId::Int32, 1);
        let recv_only = channel.receive_only();
        
        // Send through original channel
        channel.try_send(RuntimeValue::Int32(42)).unwrap();
        
        // Can receive
        assert!(recv_only.try_receive().is_ok());
        
        // Cannot send
        assert!(recv_only.try_send(RuntimeValue::Int32(1)).is_err());
    }

    #[test]
    fn test_channel_iterator() {
        let channel = Channel::new_buffered(TypeId::Int32, 3);
        
        // Send some values
        channel.try_send(RuntimeValue::Int32(1)).unwrap();
        channel.try_send(RuntimeValue::Int32(2)).unwrap();
        channel.try_send(RuntimeValue::Int32(3)).unwrap();
        
        // Close the channel
        channel.close().unwrap();
        
        // Iterate over values
        let values: Vec<_> = channel.iter().collect();
        assert_eq!(values.len(), 3);
        
        match (&values[0], &values[1], &values[2]) {
            (RuntimeValue::Int32(1), RuntimeValue::Int32(2), RuntimeValue::Int32(3)) => {},
            other => panic!("Expected [1, 2, 3], got {:?}", other),
        }
    }

    #[test]
    fn test_channel_registry() {
        let mut registry = ChannelRegistry::new();
        
        let channel1 = Channel::new_unbuffered(TypeId::Int32);
        let channel2 = Channel::new_buffered(TypeId::String, 5);
        
        let id1 = registry.register(channel1);
        let id2 = registry.register(channel2);
        
        assert_ne!(id1, id2);
        assert!(registry.get(id1).is_some());
        assert!(registry.get(id2).is_some());
        assert!(registry.get(999).is_none());
        
        let removed = registry.remove(id1);
        assert!(removed.is_some());
        assert!(registry.get(id1).is_none());
    }
}