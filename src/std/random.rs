// std.random module - Random number generation and utilities
// Requirements: 7.1.6

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

/// Random number generator with various distribution support
pub struct Random {
    seed: u64,
}

impl Random {
    /// Create a new random generator with current time as seed
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { seed }
    }
    
    /// Create a new random generator with specific seed
    pub fn with_seed(seed: u64) -> Self {
        Self { seed }
    }
    
    /// Generate next random u64 using linear congruential generator
    fn next_u64(&mut self) -> u64 {
        // LCG parameters (same as used by glibc)
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        self.seed
    }
    
    /// Generate random float between 0.0 and 1.0
    pub fn random(&mut self) -> f64 {
        let value = self.next_u64();
        (value as f64) / (u64::MAX as f64)
    }
    
    /// Generate random integer between min and max (inclusive)
    pub fn random_int(&mut self, min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        
        let range = (max - min + 1) as f64;
        min + (self.random() * range) as i32
    }
    
    /// Generate random float between min and max
    pub fn random_float(&mut self, min: f64, max: f64) -> f64 {
        min + self.random() * (max - min)
    }
    
    /// Generate random boolean
    pub fn random_bool(&mut self) -> bool {
        self.random() < 0.5
    }
    
    /// Generate random bytes
    pub fn random_bytes(&mut self, count: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(count);
        for _ in 0..count {
            bytes.push((self.next_u64() & 0xFF) as u8);
        }
        bytes
    }
    
    /// Choose random element from slice
    pub fn choose<'a, T>(&mut self, items: &'a [T]) -> Option<&'a T> {
        if items.is_empty() {
            None
        } else {
            let index = self.random_int(0, items.len() as i32 - 1) as usize;
            items.get(index)
        }
    }
    
    /// Shuffle array in place using Fisher-Yates algorithm
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        for i in (1..items.len()).rev() {
            let j = self.random_int(0, i as i32) as usize;
            items.swap(i, j);
        }
    }
    
    /// Generate random string of specified length using given charset
    pub fn random_string(&mut self, length: usize, charset: &str) -> String {
        let chars: Vec<char> = charset.chars().collect();
        if chars.is_empty() {
            return String::new();
        }
        
        let mut result = String::with_capacity(length);
        for _ in 0..length {
            if let Some(&ch) = self.choose(&chars) {
                result.push(ch);
            }
        }
        result
    }
    
    /// Generate random alphanumeric string
    pub fn random_alphanumeric(&mut self, length: usize) -> String {
        const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
        self.random_string(length, CHARSET)
    }
    
    /// Generate random alphabetic string
    pub fn random_alpha(&mut self, length: usize) -> String {
        const CHARSET: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
        self.random_string(length, CHARSET)
    }
    
    /// Generate random numeric string
    pub fn random_numeric(&mut self, length: usize) -> String {
        const CHARSET: &str = "0123456789";
        self.random_string(length, CHARSET)
    }
    
    /// Generate random UUID v4 (pseudo-random)
    pub fn random_uuid(&mut self) -> String {
        let bytes = self.random_bytes(16);
        
        format!(
            "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-4{:01x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            bytes[0], bytes[1], bytes[2], bytes[3],
            bytes[4], bytes[5],
            bytes[6] & 0x0f, bytes[7],
            (bytes[8] & 0x3f) | 0x80, bytes[9],
            bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
        )
    }
}

impl Default for Random {
    fn default() -> Self {
        Self::new()
    }
}

/// Global random functions (stateless, using system entropy)
pub mod global {
    use super::*;
    
    /// Generate random float between 0.0 and 1.0 (stateless)
    pub fn random() -> f64 {
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let hash = hasher.finish();
        (hash as f64) / (u64::MAX as f64)
    }
    
    /// Generate random integer between min and max (inclusive, stateless)
    pub fn random_int(min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        
        let range = (max - min + 1) as f64;
        min + (random() * range) as i32
    }
    
    /// Generate random float between min and max (stateless)
    pub fn random_float(min: f64, max: f64) -> f64 {
        min + random() * (max - min)
    }
    
    /// Generate random boolean (stateless)
    pub fn random_bool() -> bool {
        random() < 0.5
    }
    
    /// Choose random element from slice (stateless)
    pub fn choose<T>(items: &[T]) -> Option<&T> {
        if items.is_empty() {
            None
        } else {
            let index = random_int(0, items.len() as i32 - 1) as usize;
            items.get(index)
        }
    }
    
    /// Generate random alphanumeric string (stateless)
    pub fn random_alphanumeric(length: usize) -> String {
        let mut rng = Random::new();
        rng.random_alphanumeric(length)
    }
    
    /// Generate random UUID v4 (stateless)
    pub fn random_uuid() -> String {
        let mut rng = Random::new();
        rng.random_uuid()
    }
}

/// Probability distributions
pub mod distributions {
    use super::Random;
    
    impl Random {
        /// Generate random number from normal distribution (Box-Muller transform)
        pub fn normal(&mut self, mean: f64, std_dev: f64) -> f64 {
            // Box-Muller transform
            static mut SPARE: Option<f64> = None;
            static mut HAS_SPARE: bool = false;
            
            unsafe {
                if HAS_SPARE {
                    HAS_SPARE = false;
                    return mean + std_dev * SPARE.unwrap();
                }
                
                HAS_SPARE = true;
                let u = self.random();
                let v = self.random();
                let mag = std_dev * (-2.0 * u.ln()).sqrt();
                SPARE = Some(mag * (2.0 * std::f64::consts::PI * v).cos());
                mean + mag * (2.0 * std::f64::consts::PI * v).sin()
            }
        }
        
        /// Generate random number from exponential distribution
        pub fn exponential(&mut self, lambda: f64) -> f64 {
            -self.random().ln() / lambda
        }
        
        /// Generate random number from uniform distribution
        pub fn uniform(&mut self, min: f64, max: f64) -> f64 {
            self.random_float(min, max)
        }
        
        /// Generate random integer from Poisson distribution (approximation)
        pub fn poisson(&mut self, lambda: f64) -> u32 {
            if lambda < 30.0 {
                // Use Knuth's algorithm for small lambda
                let l = (-lambda).exp();
                let mut k = 0;
                let mut p = 1.0;
                
                loop {
                    k += 1;
                    p *= self.random();
                    if p <= l {
                        break;
                    }
                }
                
                (k - 1) as u32
            } else {
                // Use normal approximation for large lambda
                let normal_val = self.normal(lambda, lambda.sqrt());
                normal_val.max(0.0).round() as u32
            }
        }
        
        /// Generate random integer from binomial distribution
        pub fn binomial(&mut self, n: u32, p: f64) -> u32 {
            let mut count = 0;
            for _ in 0..n {
                if self.random() < p {
                    count += 1;
                }
            }
            count
        }
        
        /// Generate random integer from geometric distribution
        pub fn geometric(&mut self, p: f64) -> u32 {
            if p <= 0.0 || p >= 1.0 {
                return 1;
            }
            
            let u = self.random();
            ((u.ln() / (1.0 - p).ln()).floor() as u32) + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_random_generator() {
        let mut rng = Random::with_seed(12345);
        
        // Test basic random generation
        let r1 = rng.random();
        let r2 = rng.random();
        assert!(r1 >= 0.0 && r1 <= 1.0);
        assert!(r2 >= 0.0 && r2 <= 1.0);
        assert_ne!(r1, r2); // Should be different
        
        // Test deterministic behavior with same seed
        let mut rng1 = Random::with_seed(54321);
        let mut rng2 = Random::with_seed(54321);
        assert_eq!(rng1.random(), rng2.random());
    }
    
    #[test]
    fn test_random_int() {
        let mut rng = Random::new();
        
        for _ in 0..100 {
            let val = rng.random_int(1, 10);
            assert!(val >= 1 && val <= 10);
        }
        
        // Test edge cases
        assert_eq!(rng.random_int(5, 5), 5);
        assert_eq!(rng.random_int(10, 5), 10); // min >= max
    }
    
    #[test]
    fn test_random_float() {
        let mut rng = Random::new();
        
        for _ in 0..100 {
            let val = rng.random_float(1.0, 10.0);
            assert!(val >= 1.0 && val <= 10.0);
        }
    }
    
    #[test]
    fn test_random_bool() {
        let mut rng = Random::new();
        let mut true_count = 0;
        let mut false_count = 0;
        
        for _ in 0..1000 {
            if rng.random_bool() {
                true_count += 1;
            } else {
                false_count += 1;
            }
        }
        
        // Should be roughly balanced (allow some variance)
        assert!(true_count > 300 && true_count < 700);
        assert!(false_count > 300 && false_count < 700);
    }
    
    #[test]
    fn test_random_bytes() {
        let mut rng = Random::new();
        let bytes = rng.random_bytes(10);
        assert_eq!(bytes.len(), 10);
        
        // Check that not all bytes are the same
        let first = bytes[0];
        assert!(bytes.iter().any(|&b| b != first));
    }
    
    #[test]
    fn test_choose() {
        let mut rng = Random::new();
        let items = vec![1, 2, 3, 4, 5];
        
        for _ in 0..100 {
            let chosen = rng.choose(&items);
            assert!(chosen.is_some());
            assert!(items.contains(chosen.unwrap()));
        }
        
        // Test empty slice
        let empty: Vec<i32> = vec![];
        assert!(rng.choose(&empty).is_none());
    }
    
    #[test]
    fn test_shuffle() {
        let mut rng = Random::with_seed(12345);
        let mut items = vec![1, 2, 3, 4, 5];
        let original = items.clone();
        
        rng.shuffle(&mut items);
        
        // Should contain same elements
        assert_eq!(items.len(), original.len());
        for &item in &original {
            assert!(items.contains(&item));
        }
        
        // Should be different order (with high probability)
        // Note: There's a small chance they could be the same
    }
    
    #[test]
    fn test_random_strings() {
        let mut rng = Random::new();
        
        let alpha = rng.random_alphanumeric(10);
        assert_eq!(alpha.len(), 10);
        assert!(alpha.chars().all(|c| c.is_alphanumeric()));
        
        let numeric = rng.random_numeric(5);
        assert_eq!(numeric.len(), 5);
        assert!(numeric.chars().all(|c| c.is_numeric()));
        
        let custom = rng.random_string(8, "ABC");
        assert_eq!(custom.len(), 8);
        assert!(custom.chars().all(|c| "ABC".contains(c)));
    }
    
    #[test]
    fn test_random_uuid() {
        let mut rng = Random::new();
        let uuid = rng.random_uuid();
        
        // Check format: xxxxxxxx-xxxx-4xxx-xxxx-xxxxxxxxxxxx
        assert_eq!(uuid.len(), 36);
        assert_eq!(uuid.chars().nth(8), Some('-'));
        assert_eq!(uuid.chars().nth(13), Some('-'));
        assert_eq!(uuid.chars().nth(14), Some('4')); // Version 4
        assert_eq!(uuid.chars().nth(18), Some('-'));
        assert_eq!(uuid.chars().nth(23), Some('-'));
    }
    
    #[test]
    fn test_global_functions() {
        // Test that global functions work
        let r = global::random();
        assert!(r >= 0.0 && r <= 1.0);
        
        let i = global::random_int(1, 10);
        assert!(i >= 1 && i <= 10);
        
        let f = global::random_float(1.0, 10.0);
        assert!(f >= 1.0 && f <= 10.0);
        
        let _b = global::random_bool();
        
        let items = vec![1, 2, 3, 4, 5];
        let chosen = global::choose(&items);
        assert!(chosen.is_some());
        
        let uuid = global::random_uuid();
        assert_eq!(uuid.len(), 36);
    }
    
    #[test]
    fn test_distributions() {
        let mut rng = Random::new();
        
        // Test normal distribution
        let mut sum = 0.0;
        let n = 1000;
        for _ in 0..n {
            sum += rng.normal(0.0, 1.0);
        }
        let mean = sum / n as f64;
        assert!(mean.abs() < 0.2); // Should be close to 0
        
        // Test exponential distribution
        let exp_val = rng.exponential(1.0);
        assert!(exp_val >= 0.0);
        
        // Test uniform distribution
        let uniform_val = rng.uniform(5.0, 10.0);
        assert!(uniform_val >= 5.0 && uniform_val <= 10.0);
        
        // Test Poisson distribution
        let poisson_val = rng.poisson(5.0);
        assert!(poisson_val < 100); // Reasonable upper bound
        
        // Test binomial distribution
        let binomial_val = rng.binomial(10, 0.5);
        assert!(binomial_val <= 10);
        
        // Test geometric distribution
        let geometric_val = rng.geometric(0.3);
        assert!(geometric_val >= 1);
    }
}