//! Safe collection types with runtime bounds checking
//!
//! This module provides safe wrappers around arrays and slices that
//! perform runtime bounds checking to prevent buffer overflows and
//! out-of-bounds access.

use crate::runtime::safety::{SafetyChecker, SafetyResult};
use crate::types::primitive::TypeId;
use std::fmt;
use std::ops::{Index, IndexMut};

/// Safe array wrapper with runtime bounds checking
#[derive(Debug, Clone)]
pub struct SafeArray<T> {
    data: Vec<T>,
    element_type: TypeId,
    safety_checker: SafetyChecker,
}

impl<T> SafeArray<T> {
    /// Create a new safe array with the given capacity
    pub fn new(capacity: usize, element_type: TypeId) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            element_type,
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Create a safe array from existing data
    pub fn from_vec(data: Vec<T>, element_type: TypeId) -> Self {
        Self {
            data,
            element_type,
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Get the length of the array
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the array is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the capacity of the array
    pub fn capacity(&self) -> usize {
        self.data.capacity()
    }

    /// Get element type
    pub fn element_type(&self) -> TypeId {
        self.element_type
    }

    /// Safe get with bounds checking
    pub fn get(&self, index: usize) -> SafetyResult<&T> {
        self.safety_checker.check_bounds(index, self.data.len(), "array")?;
        Ok(&self.data[index])
    }

    /// Safe mutable get with bounds checking
    pub fn get_mut(&mut self, index: usize) -> SafetyResult<&mut T> {
        self.safety_checker.check_bounds(index, self.data.len(), "array")?;
        Ok(&mut self.data[index])
    }

    /// Safe set with bounds checking
    pub fn set(&mut self, index: usize, value: T) -> SafetyResult<()> {
        self.safety_checker.check_bounds(index, self.data.len(), "array")?;
        self.data[index] = value;
        Ok(())
    }

    /// Push an element to the end of the array
    pub fn push(&mut self, value: T) {
        self.data.push(value);
    }

    /// Pop an element from the end of the array
    pub fn pop(&mut self) -> Option<T> {
        self.data.pop()
    }

    /// Create a safe slice with bounds checking
    pub fn slice(&self, start: usize, end: usize) -> SafetyResult<SafeSlice<T>> {
        self.safety_checker.check_slice_bounds(start, end, self.data.len(), "array")?;
        Ok(SafeSlice {
            data: &self.data[start..end],
            element_type: self.element_type,
            safety_checker: self.safety_checker.clone(),
        })
    }

    /// Create a mutable safe slice with bounds checking
    pub fn slice_mut(&mut self, start: usize, end: usize) -> SafetyResult<SafeSliceMut<T>> {
        self.safety_checker.check_slice_bounds(start, end, self.data.len(), "array")?;
        let len = self.data.len();
        Ok(SafeSliceMut {
            data: &mut self.data[start..end],
            element_type: self.element_type,
            safety_checker: self.safety_checker.clone(),
            _original_len: len,
        })
    }

    /// Get the underlying data as a slice (unsafe)
    pub fn as_slice(&self) -> &[T] {
        &self.data
    }

    /// Get the underlying data as a mutable slice (unsafe)
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data
    }

    /// Configure safety checking
    pub fn set_bounds_checking(&mut self, enabled: bool) {
        self.safety_checker.set_bounds_checking(enabled);
    }

    /// Check if bounds checking is enabled
    pub fn bounds_checking_enabled(&self) -> bool {
        let (bounds, _, _) = self.safety_checker.get_settings();
        bounds
    }
}

impl<T> Index<usize> for SafeArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Ok(value) => value,
            Err(e) => panic!("Array index out of bounds: {}", e),
        }
    }
}

impl<T> IndexMut<usize> for SafeArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            Ok(value) => value,
            Err(e) => panic!("Array index out of bounds: {}", e),
        }
    }
}

/// Safe slice wrapper with runtime bounds checking
#[derive(Debug)]
pub struct SafeSlice<'a, T> {
    data: &'a [T],
    element_type: TypeId,
    safety_checker: SafetyChecker,
}

impl<'a, T> SafeSlice<'a, T> {
    /// Create a new safe slice
    pub fn new(data: &'a [T], element_type: TypeId) -> Self {
        Self {
            data,
            element_type,
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Get the length of the slice
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the slice is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get element type
    pub fn element_type(&self) -> TypeId {
        self.element_type
    }

    /// Safe get with bounds checking
    pub fn get(&self, index: usize) -> SafetyResult<&T> {
        self.safety_checker.check_bounds(index, self.data.len(), "slice")?;
        Ok(&self.data[index])
    }

    /// Create a sub-slice with bounds checking
    pub fn slice(&self, start: usize, end: usize) -> SafetyResult<SafeSlice<T>> {
        self.safety_checker.check_slice_bounds(start, end, self.data.len(), "slice")?;
        Ok(SafeSlice {
            data: &self.data[start..end],
            element_type: self.element_type,
            safety_checker: self.safety_checker.clone(),
        })
    }

    /// Get the underlying data as a slice (unsafe)
    pub fn as_slice(&self) -> &[T] {
        self.data
    }

    /// Configure safety checking
    pub fn set_bounds_checking(&mut self, enabled: bool) {
        self.safety_checker.set_bounds_checking(enabled);
    }
}

impl<'a, T> Index<usize> for SafeSlice<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Ok(value) => value,
            Err(e) => panic!("Slice index out of bounds: {}", e),
        }
    }
}

/// Safe mutable slice wrapper with runtime bounds checking
#[derive(Debug)]
pub struct SafeSliceMut<'a, T> {
    data: &'a mut [T],
    element_type: TypeId,
    safety_checker: SafetyChecker,
    _original_len: usize, // For debugging purposes
}

impl<'a, T> SafeSliceMut<'a, T> {
    /// Create a new safe mutable slice
    pub fn new(data: &'a mut [T], element_type: TypeId) -> Self {
        let len = data.len();
        Self {
            data,
            element_type,
            safety_checker: SafetyChecker::new(),
            _original_len: len,
        }
    }

    /// Get the length of the slice
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if the slice is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get element type
    pub fn element_type(&self) -> TypeId {
        self.element_type
    }

    /// Safe get with bounds checking
    pub fn get(&self, index: usize) -> SafetyResult<&T> {
        self.safety_checker.check_bounds(index, self.data.len(), "slice")?;
        Ok(&self.data[index])
    }

    /// Safe mutable get with bounds checking
    pub fn get_mut(&mut self, index: usize) -> SafetyResult<&mut T> {
        self.safety_checker.check_bounds(index, self.data.len(), "slice")?;
        Ok(&mut self.data[index])
    }

    /// Safe set with bounds checking
    pub fn set(&mut self, index: usize, value: T) -> SafetyResult<()> {
        self.safety_checker.check_bounds(index, self.data.len(), "slice")?;
        self.data[index] = value;
        Ok(())
    }

    /// Create a sub-slice with bounds checking
    pub fn slice(&self, start: usize, end: usize) -> SafetyResult<SafeSlice<T>> {
        self.safety_checker.check_slice_bounds(start, end, self.data.len(), "slice")?;
        Ok(SafeSlice {
            data: &self.data[start..end],
            element_type: self.element_type,
            safety_checker: self.safety_checker.clone(),
        })
    }

    /// Create a mutable sub-slice with bounds checking
    pub fn slice_mut(&mut self, start: usize, end: usize) -> SafetyResult<SafeSliceMut<T>> {
        self.safety_checker.check_slice_bounds(start, end, self.data.len(), "slice")?;
        let len = self.data.len();
        Ok(SafeSliceMut {
            data: &mut self.data[start..end],
            element_type: self.element_type,
            safety_checker: self.safety_checker.clone(),
            _original_len: len,
        })
    }

    /// Get the underlying data as a slice (unsafe)
    pub fn as_slice(&self) -> &[T] {
        self.data
    }

    /// Get the underlying data as a mutable slice (unsafe)
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        self.data
    }

    /// Configure safety checking
    pub fn set_bounds_checking(&mut self, enabled: bool) {
        self.safety_checker.set_bounds_checking(enabled);
    }
}

impl<'a, T> Index<usize> for SafeSliceMut<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match self.get(index) {
            Ok(value) => value,
            Err(e) => panic!("Slice index out of bounds: {}", e),
        }
    }
}

impl<'a, T> IndexMut<usize> for SafeSliceMut<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match self.get_mut(index) {
            Ok(value) => value,
            Err(e) => panic!("Slice index out of bounds: {}", e),
        }
    }
}

/// Safe string wrapper with bounds checking for character access
#[derive(Debug, Clone)]
pub struct SafeString {
    data: String,
    safety_checker: SafetyChecker,
}

impl SafeString {
    /// Create a new safe string
    pub fn new() -> Self {
        Self {
            data: String::new(),
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Create a safe string from existing string
    pub fn from_string(data: String) -> Self {
        Self {
            data,
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Create a safe string from string slice
    pub fn from_str(s: &str) -> Self {
        Self {
            data: s.to_string(),
            safety_checker: SafetyChecker::new(),
        }
    }

    /// Get the length of the string in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get the length of the string in characters
    pub fn char_len(&self) -> usize {
        self.data.chars().count()
    }

    /// Check if the string is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Safe character access by index
    pub fn char_at(&self, index: usize) -> SafetyResult<char> {
        let chars: Vec<char> = self.data.chars().collect();
        self.safety_checker.check_bounds(index, chars.len(), "string")?;
        Ok(chars[index])
    }

    /// Safe substring with bounds checking
    pub fn substring(&self, start: usize, end: usize) -> SafetyResult<String> {
        let chars: Vec<char> = self.data.chars().collect();
        self.safety_checker.check_slice_bounds(start, end, chars.len(), "string")?;
        Ok(chars[start..end].iter().collect())
    }

    /// Safe byte access with bounds checking
    pub fn byte_at(&self, index: usize) -> SafetyResult<u8> {
        let bytes = self.data.as_bytes();
        self.safety_checker.check_bounds(index, bytes.len(), "string_bytes")?;
        Ok(bytes[index])
    }

    /// Safe byte slice with bounds checking
    pub fn byte_slice(&self, start: usize, end: usize) -> SafetyResult<&[u8]> {
        let bytes = self.data.as_bytes();
        self.safety_checker.check_slice_bounds(start, end, bytes.len(), "string_bytes")?;
        Ok(&bytes[start..end])
    }

    /// Push a character to the string
    pub fn push(&mut self, ch: char) {
        self.data.push(ch);
    }

    /// Push a string to the string
    pub fn push_str(&mut self, s: &str) {
        self.data.push_str(s);
    }

    /// Get the underlying string (unsafe)
    pub fn as_str(&self) -> &str {
        &self.data
    }

    /// Get the underlying string (unsafe)
    pub fn into_string(self) -> String {
        self.data
    }

    /// Configure safety checking
    pub fn set_bounds_checking(&mut self, enabled: bool) {
        self.safety_checker.set_bounds_checking(enabled);
    }
}

impl fmt::Display for SafeString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Default for SafeString {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::primitive::TypeId;

    #[test]
    fn test_safe_array_basic_operations() {
        let mut array = SafeArray::from_vec(vec![1, 2, 3, 4, 5], TypeId::Int32);
        
        // Test length and capacity
        assert_eq!(array.len(), 5);
        assert!(!array.is_empty());
        
        // Test safe get
        assert_eq!(*array.get(0).unwrap(), 1);
        assert_eq!(*array.get(4).unwrap(), 5);
        
        // Test out of bounds
        assert!(array.get(5).is_err());
        assert!(array.get(100).is_err());
        
        // Test safe set
        assert!(array.set(2, 10).is_ok());
        assert_eq!(*array.get(2).unwrap(), 10);
        
        // Test out of bounds set
        assert!(array.set(5, 20).is_err());
    }

    #[test]
    fn test_safe_array_indexing() {
        let array = SafeArray::from_vec(vec![10, 20, 30], TypeId::Int32);
        
        // Test index operator
        assert_eq!(array[0], 10);
        assert_eq!(array[1], 20);
        assert_eq!(array[2], 30);
    }

    #[test]
    #[should_panic(expected = "Array index out of bounds")]
    fn test_safe_array_index_panic() {
        let array = SafeArray::from_vec(vec![1, 2, 3], TypeId::Int32);
        let _ = array[5]; // Should panic
    }

    #[test]
    fn test_safe_array_slicing() {
        let array = SafeArray::from_vec(vec![1, 2, 3, 4, 5, 6], TypeId::Int32);
        
        // Test valid slicing
        let slice = array.slice(1, 4).unwrap();
        assert_eq!(slice.len(), 3);
        assert_eq!(*slice.get(0).unwrap(), 2);
        assert_eq!(*slice.get(2).unwrap(), 4);
        
        // Test invalid slicing
        assert!(array.slice(0, 10).is_err());
        assert!(array.slice(5, 3).is_err());
        assert!(array.slice(10, 15).is_err());
    }

    #[test]
    fn test_safe_array_push_pop() {
        let mut array = SafeArray::new(0, TypeId::Int32);
        
        // Test push
        array.push(1);
        array.push(2);
        array.push(3);
        assert_eq!(array.len(), 3);
        assert_eq!(*array.get(1).unwrap(), 2);
        
        // Test pop
        assert_eq!(array.pop(), Some(3));
        assert_eq!(array.len(), 2);
        assert_eq!(array.pop(), Some(2));
        assert_eq!(array.pop(), Some(1));
        assert_eq!(array.pop(), None);
        assert!(array.is_empty());
    }

    #[test]
    fn test_safe_slice_operations() {
        let data = vec![10, 20, 30, 40, 50];
        let slice = SafeSlice::new(&data, TypeId::Int32);
        
        // Test basic operations
        assert_eq!(slice.len(), 5);
        assert!(!slice.is_empty());
        assert_eq!(*slice.get(2).unwrap(), 30);
        
        // Test out of bounds
        assert!(slice.get(5).is_err());
        
        // Test sub-slicing
        let sub_slice = slice.slice(1, 4).unwrap();
        assert_eq!(sub_slice.len(), 3);
        assert_eq!(*sub_slice.get(0).unwrap(), 20);
        assert_eq!(*sub_slice.get(2).unwrap(), 40);
    }

    #[test]
    fn test_safe_slice_mut_operations() {
        let mut data = vec![1, 2, 3, 4, 5];
        let mut slice = SafeSliceMut::new(&mut data, TypeId::Int32);
        
        // Test mutable operations
        assert!(slice.set(2, 100).is_ok());
        assert_eq!(*slice.get(2).unwrap(), 100);
        
        // Test mutable get
        *slice.get_mut(0).unwrap() = 200;
        assert_eq!(*slice.get(0).unwrap(), 200);
        
        // Test out of bounds
        assert!(slice.set(5, 300).is_err());
        assert!(slice.get_mut(5).is_err());
    }

    #[test]
    fn test_safe_string_operations() {
        let mut safe_str = SafeString::from_str("Hello, World!");
        
        // Test length operations
        assert_eq!(safe_str.len(), 13); // bytes
        assert_eq!(safe_str.char_len(), 13); // characters (ASCII)
        assert!(!safe_str.is_empty());
        
        // Test character access
        assert_eq!(safe_str.char_at(0).unwrap(), 'H');
        assert_eq!(safe_str.char_at(7).unwrap(), 'W');
        assert!(safe_str.char_at(13).is_err());
        
        // Test substring
        let sub = safe_str.substring(0, 5).unwrap();
        assert_eq!(sub, "Hello");
        
        let sub2 = safe_str.substring(7, 12).unwrap();
        assert_eq!(sub2, "World");
        
        // Test invalid substring
        assert!(safe_str.substring(0, 20).is_err());
        assert!(safe_str.substring(10, 5).is_err());
        
        // Test byte access
        assert_eq!(safe_str.byte_at(0).unwrap(), b'H');
        assert!(safe_str.byte_at(13).is_err());
        
        // Test push operations
        safe_str.push('!');
        assert_eq!(safe_str.char_len(), 14);
        
        safe_str.push_str(" How are you?");
        assert!(safe_str.as_str().ends_with("How are you?"));
    }

    #[test]
    fn test_safe_string_unicode() {
        let safe_str = SafeString::from_str("Hello, 世界!");
        
        // Unicode string has different byte and character lengths
        assert_eq!(safe_str.len(), 14); // bytes (世界 takes 6 bytes total)
        assert_eq!(safe_str.char_len(), 10); // characters
        
        // Test character access with unicode
        assert_eq!(safe_str.char_at(7).unwrap(), '世');
        assert_eq!(safe_str.char_at(8).unwrap(), '界');
        
        // Test substring with unicode
        let sub = safe_str.substring(7, 9).unwrap();
        assert_eq!(sub, "世界");
    }

    #[test]
    fn test_bounds_checking_configuration() {
        let mut array = SafeArray::from_vec(vec![1, 2, 3], TypeId::Int32);
        
        // Bounds checking enabled by default
        assert!(array.bounds_checking_enabled());
        assert!(array.get(5).is_err());
        
        // Disable bounds checking
        array.set_bounds_checking(false);
        assert!(!array.bounds_checking_enabled());
        
        // Note: Even with bounds checking disabled, Rust's built-in bounds checking
        // will still prevent actual memory safety violations, but our custom
        // SafetyError won't be returned.
    }

    #[test]
    fn test_element_type_tracking() {
        let int_array = SafeArray::from_vec(vec![1, 2, 3], TypeId::Int32);
        assert_eq!(int_array.element_type(), TypeId::Int32);
        
        let float_array = SafeArray::from_vec(vec![1.0, 2.0], TypeId::Float64);
        assert_eq!(float_array.element_type(), TypeId::Float64);
        
        let slice = int_array.slice(0, 2).unwrap();
        assert_eq!(slice.element_type(), TypeId::Int32);
    }
}