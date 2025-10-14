// std.arrays module - Array operation utilities
// Requirements: 7.1.4

use std::collections::HashMap;
use std::hash::Hash;

/// Array manipulation utilities
pub struct ArrayUtils;

impl ArrayUtils {
    /// Get length of array/slice
    pub fn len<T>(arr: &[T]) -> usize {
        arr.len()
    }
    
    /// Check if array is empty
    pub fn is_empty<T>(arr: &[T]) -> bool {
        arr.is_empty()
    }
    
    /// Get first element
    pub fn first<T>(arr: &[T]) -> Option<&T> {
        arr.first()
    }
    
    /// Get last element
    pub fn last<T>(arr: &[T]) -> Option<&T> {
        arr.last()
    }
    
    /// Get element at index safely
    pub fn get<T>(arr: &[T], index: usize) -> Option<&T> {
        arr.get(index)
    }
    
    /// Check if array contains element
    pub fn contains<T: PartialEq>(arr: &[T], item: &T) -> bool {
        arr.contains(item)
    }
    
    /// Find index of first occurrence
    pub fn index_of<T: PartialEq>(arr: &[T], item: &T) -> Option<usize> {
        arr.iter().position(|x| x == item)
    }
    
    /// Find index of last occurrence
    pub fn last_index_of<T: PartialEq>(arr: &[T], item: &T) -> Option<usize> {
        arr.iter().rposition(|x| x == item)
    }
    
    /// Count occurrences of element
    pub fn count<T: PartialEq>(arr: &[T], item: &T) -> usize {
        arr.iter().filter(|&x| x == item).count()
    }
    
    /// Reverse array (returns new array)
    pub fn reverse<T: Clone>(arr: &[T]) -> Vec<T> {
        let mut result = arr.to_vec();
        result.reverse();
        result
    }
    
    /// Sort array (returns new array)
    pub fn sort<T: Clone + Ord>(arr: &[T]) -> Vec<T> {
        let mut result = arr.to_vec();
        result.sort();
        result
    }
    
    /// Sort array by key function
    pub fn sort_by<T: Clone, K: Ord, F>(arr: &[T], key_fn: F) -> Vec<T>
    where
        F: Fn(&T) -> K,
    {
        let mut result = arr.to_vec();
        result.sort_by_key(key_fn);
        result
    }
    
    /// Remove duplicates (preserves order)
    pub fn unique<T: Clone + PartialEq>(arr: &[T]) -> Vec<T> {
        let mut result = Vec::new();
        for item in arr {
            if !result.contains(item) {
                result.push(item.clone());
            }
        }
        result
    }
    
    /// Remove duplicates using hash set (faster but doesn't preserve order)
    pub fn unique_unordered<T: Clone + Hash + Eq>(arr: &[T]) -> Vec<T> {
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        
        for item in arr {
            if seen.insert(item.clone()) {
                result.push(item.clone());
            }
        }
        
        result
    }
    
    /// Filter array by predicate
    pub fn filter<T: Clone, F>(arr: &[T], predicate: F) -> Vec<T>
    where
        F: Fn(&T) -> bool,
    {
        arr.iter().filter(|&x| predicate(x)).cloned().collect()
    }
    
    /// Map array to new array
    pub fn map<T, U, F>(arr: &[T], mapper: F) -> Vec<U>
    where
        F: Fn(&T) -> U,
    {
        arr.iter().map(mapper).collect()
    }
    
    /// Reduce array to single value
    pub fn reduce<T, F>(arr: &[T], initial: T, reducer: F) -> T
    where
        F: Fn(T, &T) -> T,
    {
        arr.iter().fold(initial, reducer)
    }
    
    /// Find first element matching predicate
    pub fn find<T, F>(arr: &[T], predicate: F) -> Option<&T>
    where
        F: Fn(&T) -> bool,
    {
        arr.iter().find(|&x| predicate(x))
    }
    
    /// Check if any element matches predicate
    pub fn any<T, F>(arr: &[T], predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        arr.iter().any(predicate)
    }
    
    /// Check if all elements match predicate
    pub fn all<T, F>(arr: &[T], predicate: F) -> bool
    where
        F: Fn(&T) -> bool,
    {
        arr.iter().all(predicate)
    }
    
    /// Take first n elements
    pub fn take<T: Clone>(arr: &[T], n: usize) -> Vec<T> {
        arr.iter().take(n).cloned().collect()
    }
    
    /// Skip first n elements
    pub fn skip<T: Clone>(arr: &[T], n: usize) -> Vec<T> {
        arr.iter().skip(n).cloned().collect()
    }
    
    /// Get slice of array
    pub fn slice<T: Clone>(arr: &[T], start: usize, end: usize) -> Vec<T> {
        let end = end.min(arr.len());
        if start >= arr.len() {
            return Vec::new();
        }
        arr[start..end].to_vec()
    }
    
    /// Concatenate two arrays
    pub fn concat<T: Clone>(arr1: &[T], arr2: &[T]) -> Vec<T> {
        let mut result = arr1.to_vec();
        result.extend_from_slice(arr2);
        result
    }
    
    /// Flatten array of arrays
    pub fn flatten<T: Clone>(arr: &[Vec<T>]) -> Vec<T> {
        arr.iter().flat_map(|v| v.iter()).cloned().collect()
    }
    
    /// Zip two arrays together
    pub fn zip<T: Clone, U: Clone>(arr1: &[T], arr2: &[U]) -> Vec<(T, U)> {
        arr1.iter().zip(arr2.iter()).map(|(a, b)| (a.clone(), b.clone())).collect()
    }
    
    /// Group array elements by key function
    pub fn group_by<T: Clone, K: Hash + Eq, F>(arr: &[T], key_fn: F) -> HashMap<K, Vec<T>>
    where
        F: Fn(&T) -> K,
    {
        let mut groups = HashMap::new();
        
        for item in arr {
            let key = key_fn(item);
            groups.entry(key).or_insert_with(Vec::new).push(item.clone());
        }
        
        groups
    }
    
    /// Partition array into two based on predicate
    pub fn partition<T: Clone, F>(arr: &[T], predicate: F) -> (Vec<T>, Vec<T>)
    where
        F: Fn(&T) -> bool,
    {
        let mut true_items = Vec::new();
        let mut false_items = Vec::new();
        
        for item in arr {
            if predicate(item) {
                true_items.push(item.clone());
            } else {
                false_items.push(item.clone());
            }
        }
        
        (true_items, false_items)
    }
    
    /// Create chunks of specified size
    pub fn chunk<T: Clone>(arr: &[T], size: usize) -> Vec<Vec<T>> {
        if size == 0 {
            return vec![arr.to_vec()];
        }
        
        arr.chunks(size).map(|chunk| chunk.to_vec()).collect()
    }
    
    /// Rotate array left by n positions
    pub fn rotate_left<T: Clone>(arr: &[T], n: usize) -> Vec<T> {
        if arr.is_empty() {
            return Vec::new();
        }
        
        let n = n % arr.len();
        let mut result = arr.to_vec();
        result.rotate_left(n);
        result
    }
    
    /// Rotate array right by n positions
    pub fn rotate_right<T: Clone>(arr: &[T], n: usize) -> Vec<T> {
        if arr.is_empty() {
            return Vec::new();
        }
        
        let n = n % arr.len();
        let mut result = arr.to_vec();
        result.rotate_right(n);
        result
    }
    
    /// Shuffle array randomly
    pub fn shuffle<T: Clone>(arr: &[T]) -> Vec<T> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut result = arr.to_vec();
        let len = result.len();
        
        // Simple pseudo-random shuffle using hash
        for i in 0..len {
            let mut hasher = DefaultHasher::new();
            i.hash(&mut hasher);
            let j = (hasher.finish() as usize) % len;
            result.swap(i, j);
        }
        
        result
    }
    
    /// Binary search in sorted array
    pub fn binary_search<T: Ord>(arr: &[T], item: &T) -> Result<usize, usize> {
        arr.binary_search(item)
    }
    
    /// Get minimum element
    pub fn min<T: Ord>(arr: &[T]) -> Option<&T> {
        arr.iter().min()
    }
    
    /// Get maximum element
    pub fn max<T: Ord>(arr: &[T]) -> Option<&T> {
        arr.iter().max()
    }
    
    /// Sum numeric array
    pub fn sum<T>(arr: &[T]) -> T
    where
        T: Clone + std::iter::Sum,
    {
        arr.iter().cloned().sum()
    }
    
    /// Calculate average of numeric array
    pub fn average(arr: &[f64]) -> Option<f64> {
        if arr.is_empty() {
            None
        } else {
            Some(arr.iter().sum::<f64>() / arr.len() as f64)
        }
    }
    
    /// Calculate median of numeric array
    pub fn median(arr: &[f64]) -> Option<f64> {
        if arr.is_empty() {
            return None;
        }
        
        let mut sorted = arr.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let len = sorted.len();
        if len % 2 == 0 {
            Some((sorted[len / 2 - 1] + sorted[len / 2]) / 2.0)
        } else {
            Some(sorted[len / 2])
        }
    }
    
    /// Create range array
    pub fn range(start: i32, end: i32) -> Vec<i32> {
        (start..end).collect()
    }
    
    /// Create range array with step
    pub fn range_step(start: i32, end: i32, step: i32) -> Vec<i32> {
        if step == 0 {
            return Vec::new();
        }
        
        let mut result = Vec::new();
        let mut current = start;
        
        if step > 0 {
            while current < end {
                result.push(current);
                current += step;
            }
        } else {
            while current > end {
                result.push(current);
                current += step;
            }
        }
        
        result
    }
    
    /// Fill array with value
    pub fn fill<T: Clone>(value: T, count: usize) -> Vec<T> {
        vec![value; count]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_operations() {
        let arr = vec![1, 2, 3, 4, 5];
        
        assert_eq!(ArrayUtils::len(&arr), 5);
        assert!(!ArrayUtils::is_empty(&arr));
        assert_eq!(ArrayUtils::first(&arr), Some(&1));
        assert_eq!(ArrayUtils::last(&arr), Some(&5));
        assert_eq!(ArrayUtils::get(&arr, 2), Some(&3));
    }
    
    #[test]
    fn test_searching() {
        let arr = vec![1, 2, 3, 2, 4];
        
        assert!(ArrayUtils::contains(&arr, &2));
        assert_eq!(ArrayUtils::index_of(&arr, &2), Some(1));
        assert_eq!(ArrayUtils::last_index_of(&arr, &2), Some(3));
        assert_eq!(ArrayUtils::count(&arr, &2), 2);
    }
    
    #[test]
    fn test_transformations() {
        let arr = vec![3, 1, 4, 1, 5];
        
        assert_eq!(ArrayUtils::reverse(&arr), vec![5, 1, 4, 1, 3]);
        assert_eq!(ArrayUtils::sort(&arr), vec![1, 1, 3, 4, 5]);
        assert_eq!(ArrayUtils::unique(&arr), vec![3, 1, 4, 5]);
    }
    
    #[test]
    fn test_functional_operations() {
        let arr = vec![1, 2, 3, 4, 5];
        
        let evens = ArrayUtils::filter(&arr, |&x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4]);
        
        let doubled = ArrayUtils::map(&arr, |&x| x * 2);
        assert_eq!(doubled, vec![2, 4, 6, 8, 10]);
        
        let sum = ArrayUtils::reduce(&arr, 0, |acc, &x| acc + x);
        assert_eq!(sum, 15);
        
        assert!(ArrayUtils::any(&arr, |&x| x > 3));
        assert!(ArrayUtils::all(&arr, |&x| x > 0));
    }
    
    #[test]
    fn test_slicing() {
        let arr = vec![1, 2, 3, 4, 5];
        
        assert_eq!(ArrayUtils::take(&arr, 3), vec![1, 2, 3]);
        assert_eq!(ArrayUtils::skip(&arr, 2), vec![3, 4, 5]);
        assert_eq!(ArrayUtils::slice(&arr, 1, 4), vec![2, 3, 4]);
    }
    
    #[test]
    fn test_concatenation() {
        let arr1 = vec![1, 2, 3];
        let arr2 = vec![4, 5, 6];
        
        assert_eq!(ArrayUtils::concat(&arr1, &arr2), vec![1, 2, 3, 4, 5, 6]);
        
        let nested = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        assert_eq!(ArrayUtils::flatten(&nested), vec![1, 2, 3, 4, 5, 6]);
        
        let zipped = ArrayUtils::zip(&arr1, &arr2);
        assert_eq!(zipped, vec![(1, 4), (2, 5), (3, 6)]);
    }
    
    #[test]
    fn test_grouping() {
        let arr = vec![1, 2, 3, 4, 5, 6];
        
        let groups = ArrayUtils::group_by(&arr, |&x| x % 2);
        assert_eq!(groups.get(&0), Some(&vec![2, 4, 6]));
        assert_eq!(groups.get(&1), Some(&vec![1, 3, 5]));
        
        let (evens, odds) = ArrayUtils::partition(&arr, |&x| x % 2 == 0);
        assert_eq!(evens, vec![2, 4, 6]);
        assert_eq!(odds, vec![1, 3, 5]);
    }
    
    #[test]
    fn test_chunking() {
        let arr = vec![1, 2, 3, 4, 5, 6, 7];
        let chunks = ArrayUtils::chunk(&arr, 3);
        assert_eq!(chunks, vec![vec![1, 2, 3], vec![4, 5, 6], vec![7]]);
    }
    
    #[test]
    fn test_rotation() {
        let arr = vec![1, 2, 3, 4, 5];
        
        assert_eq!(ArrayUtils::rotate_left(&arr, 2), vec![3, 4, 5, 1, 2]);
        assert_eq!(ArrayUtils::rotate_right(&arr, 2), vec![4, 5, 1, 2, 3]);
    }
    
    #[test]
    fn test_statistics() {
        let arr = vec![1, 2, 3, 4, 5];
        let float_arr = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        assert_eq!(ArrayUtils::min(&arr), Some(&1));
        assert_eq!(ArrayUtils::max(&arr), Some(&5));
        assert_eq!(ArrayUtils::sum(&arr), 15);
        assert_eq!(ArrayUtils::average(&float_arr), Some(3.0));
        assert_eq!(ArrayUtils::median(&float_arr), Some(3.0));
    }
    
    #[test]
    fn test_range() {
        assert_eq!(ArrayUtils::range(1, 5), vec![1, 2, 3, 4]);
        assert_eq!(ArrayUtils::range_step(0, 10, 2), vec![0, 2, 4, 6, 8]);
        assert_eq!(ArrayUtils::range_step(10, 0, -2), vec![10, 8, 6, 4, 2]);
    }
    
    #[test]
    fn test_fill() {
        assert_eq!(ArrayUtils::fill(42, 3), vec![42, 42, 42]);
        assert_eq!(ArrayUtils::fill("hello".to_string(), 2), vec!["hello".to_string(), "hello".to_string()]);
    }
}