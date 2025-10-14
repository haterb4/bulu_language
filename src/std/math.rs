// std.math module - Mathematical functions
// Requirements: 7.1.5

/// Mathematical constants
pub mod constants {
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;
    pub const TAU: f64 = std::f64::consts::TAU; // 2 * PI
    pub const PHI: f64 = 1.618033988749895; // Golden ratio
    pub const SQRT_2: f64 = std::f64::consts::SQRT_2;
    pub const SQRT_3: f64 = 1.7320508075688772;
    pub const LN_2: f64 = std::f64::consts::LN_2;
    pub const LN_10: f64 = std::f64::consts::LN_10;
    pub const LOG2_E: f64 = std::f64::consts::LOG2_E;
    pub const LOG10_E: f64 = std::f64::consts::LOG10_E;
}

/// Basic mathematical operations
pub struct Math;

impl Math {
    /// Absolute value
    pub fn abs(x: f64) -> f64 {
        x.abs()
    }
    
    /// Absolute value for integers
    pub fn abs_i32(x: i32) -> i32 {
        x.abs()
    }
    
    /// Absolute value for i64
    pub fn abs_i64(x: i64) -> i64 {
        x.abs()
    }
    
    /// Sign function (-1, 0, or 1)
    pub fn sign(x: f64) -> f64 {
        if x > 0.0 { 1.0 }
        else if x < 0.0 { -1.0 }
        else { 0.0 }
    }
    
    /// Maximum of two values
    pub fn max(a: f64, b: f64) -> f64 {
        a.max(b)
    }
    
    /// Minimum of two values
    pub fn min(a: f64, b: f64) -> f64 {
        a.min(b)
    }
    
    /// Clamp value between min and max
    pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
        if value < min { min }
        else if value > max { max }
        else { value }
    }
    
    /// Square root
    pub fn sqrt(x: f64) -> f64 {
        x.sqrt()
    }
    
    /// Cube root
    pub fn cbrt(x: f64) -> f64 {
        x.cbrt()
    }
    
    /// Power function (x^y)
    pub fn pow(x: f64, y: f64) -> f64 {
        x.powf(y)
    }
    
    /// Integer power (x^n)
    pub fn powi(x: f64, n: i32) -> f64 {
        x.powi(n)
    }
    
    /// Exponential function (e^x)
    pub fn exp(x: f64) -> f64 {
        x.exp()
    }
    
    /// Exponential function base 2 (2^x)
    pub fn exp2(x: f64) -> f64 {
        x.exp2()
    }
    
    /// Natural logarithm
    pub fn ln(x: f64) -> f64 {
        x.ln()
    }
    
    /// Logarithm base 2
    pub fn log2(x: f64) -> f64 {
        x.log2()
    }
    
    /// Logarithm base 10
    pub fn log10(x: f64) -> f64 {
        x.log10()
    }
    
    /// Logarithm with custom base
    pub fn log(x: f64, base: f64) -> f64 {
        x.log(base)
    }
    
    /// Floor function (round down)
    pub fn floor(x: f64) -> f64 {
        x.floor()
    }
    
    /// Ceiling function (round up)
    pub fn ceil(x: f64) -> f64 {
        x.ceil()
    }
    
    /// Round to nearest integer
    pub fn round(x: f64) -> f64 {
        x.round()
    }
    
    /// Truncate (remove fractional part)
    pub fn trunc(x: f64) -> f64 {
        x.trunc()
    }
    
    /// Fractional part
    pub fn fract(x: f64) -> f64 {
        x.fract()
    }
    
    /// Modulo operation
    pub fn modulo(x: f64, y: f64) -> f64 {
        x % y
    }
    
    /// Remainder (IEEE 754)
    pub fn remainder(x: f64, y: f64) -> f64 {
        x.rem_euclid(y)
    }
    
    /// Check if value is NaN
    pub fn is_nan(x: f64) -> bool {
        x.is_nan()
    }
    
    /// Check if value is infinite
    pub fn is_infinite(x: f64) -> bool {
        x.is_infinite()
    }
    
    /// Check if value is finite
    pub fn is_finite(x: f64) -> bool {
        x.is_finite()
    }
}

/// Trigonometric functions
pub struct Trig;

impl Trig {
    /// Sine
    pub fn sin(x: f64) -> f64 {
        x.sin()
    }
    
    /// Cosine
    pub fn cos(x: f64) -> f64 {
        x.cos()
    }
    
    /// Tangent
    pub fn tan(x: f64) -> f64 {
        x.tan()
    }
    
    /// Arcsine
    pub fn asin(x: f64) -> f64 {
        x.asin()
    }
    
    /// Arccosine
    pub fn acos(x: f64) -> f64 {
        x.acos()
    }
    
    /// Arctangent
    pub fn atan(x: f64) -> f64 {
        x.atan()
    }
    
    /// Arctangent of y/x (handles quadrants correctly)
    pub fn atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }
    
    /// Hyperbolic sine
    pub fn sinh(x: f64) -> f64 {
        x.sinh()
    }
    
    /// Hyperbolic cosine
    pub fn cosh(x: f64) -> f64 {
        x.cosh()
    }
    
    /// Hyperbolic tangent
    pub fn tanh(x: f64) -> f64 {
        x.tanh()
    }
    
    /// Inverse hyperbolic sine
    pub fn asinh(x: f64) -> f64 {
        x.asinh()
    }
    
    /// Inverse hyperbolic cosine
    pub fn acosh(x: f64) -> f64 {
        x.acosh()
    }
    
    /// Inverse hyperbolic tangent
    pub fn atanh(x: f64) -> f64 {
        x.atanh()
    }
    
    /// Convert degrees to radians
    pub fn to_radians(degrees: f64) -> f64 {
        degrees.to_radians()
    }
    
    /// Convert radians to degrees
    pub fn to_degrees(radians: f64) -> f64 {
        radians.to_degrees()
    }
}

/// Statistical functions
pub struct Stats;

impl Stats {
    /// Calculate mean (average)
    pub fn mean(values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            None
        } else {
            Some(values.iter().sum::<f64>() / values.len() as f64)
        }
    }
    
    /// Calculate median
    pub fn median(values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }
        
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let len = sorted.len();
        if len % 2 == 0 {
            Some((sorted[len / 2 - 1] + sorted[len / 2]) / 2.0)
        } else {
            Some(sorted[len / 2])
        }
    }
    
    /// Calculate mode (most frequent value)
    pub fn mode(values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }
        
        let mut counts = std::collections::HashMap::new();
        for &value in values {
            *counts.entry(value.to_bits()).or_insert(0) += 1;
        }
        
        counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(bits, _)| f64::from_bits(bits))
    }
    
    /// Calculate variance
    pub fn variance(values: &[f64]) -> Option<f64> {
        if values.len() < 2 {
            return None;
        }
        
        let mean = Self::mean(values)?;
        let sum_squared_diff: f64 = values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum();
        
        Some(sum_squared_diff / (values.len() - 1) as f64)
    }
    
    /// Calculate standard deviation
    pub fn std_dev(values: &[f64]) -> Option<f64> {
        Self::variance(values).map(|v| v.sqrt())
    }
    
    /// Calculate range (max - min)
    pub fn range(values: &[f64]) -> Option<f64> {
        if values.is_empty() {
            return None;
        }
        
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        
        Some(max - min)
    }
    
    /// Calculate percentile
    pub fn percentile(values: &[f64], p: f64) -> Option<f64> {
        if values.is_empty() || p < 0.0 || p > 100.0 {
            return None;
        }
        
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let index = (p / 100.0) * (sorted.len() - 1) as f64;
        let lower = index.floor() as usize;
        let upper = index.ceil() as usize;
        
        if lower == upper {
            Some(sorted[lower])
        } else {
            let weight = index - lower as f64;
            Some(sorted[lower] * (1.0 - weight) + sorted[upper] * weight)
        }
    }
    
    /// Calculate correlation coefficient between two datasets
    pub fn correlation(x: &[f64], y: &[f64]) -> Option<f64> {
        if x.len() != y.len() || x.len() < 2 {
            return None;
        }
        
        let mean_x = Self::mean(x)?;
        let mean_y = Self::mean(y)?;
        
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;
        
        for i in 0..x.len() {
            let dx = x[i] - mean_x;
            let dy = y[i] - mean_y;
            sum_xy += dx * dy;
            sum_x2 += dx * dx;
            sum_y2 += dy * dy;
        }
        
        let denominator = (sum_x2 * sum_y2).sqrt();
        if denominator == 0.0 {
            None
        } else {
            Some(sum_xy / denominator)
        }
    }
}

/// Number theory functions
pub struct NumberTheory;

impl NumberTheory {
    /// Greatest Common Divisor
    pub fn gcd(mut a: i64, mut b: i64) -> i64 {
        while b != 0 {
            let temp = b;
            b = a % b;
            a = temp;
        }
        a.abs()
    }
    
    /// Least Common Multiple
    pub fn lcm(a: i64, b: i64) -> i64 {
        if a == 0 || b == 0 {
            0
        } else {
            (a.abs() / Self::gcd(a, b)) * b.abs()
        }
    }
    
    /// Check if number is prime
    pub fn is_prime(n: i64) -> bool {
        if n < 2 {
            return false;
        }
        if n == 2 {
            return true;
        }
        if n % 2 == 0 {
            return false;
        }
        
        let sqrt_n = (n as f64).sqrt() as i64;
        for i in (3..=sqrt_n).step_by(2) {
            if n % i == 0 {
                return false;
            }
        }
        
        true
    }
    
    /// Generate prime numbers up to n (Sieve of Eratosthenes)
    pub fn primes_up_to(n: usize) -> Vec<usize> {
        if n < 2 {
            return Vec::new();
        }
        
        let mut is_prime = vec![true; n + 1];
        is_prime[0] = false;
        is_prime[1] = false;
        
        for i in 2..=((n as f64).sqrt() as usize) {
            if is_prime[i] {
                for j in ((i * i)..=n).step_by(i) {
                    is_prime[j] = false;
                }
            }
        }
        
        (2..=n).filter(|&i| is_prime[i]).collect()
    }
    
    /// Calculate factorial
    pub fn factorial(n: u64) -> u64 {
        if n <= 1 {
            1
        } else {
            (2..=n).product()
        }
    }
    
    /// Calculate combinations (n choose k)
    pub fn combinations(n: u64, k: u64) -> u64 {
        if k > n {
            0
        } else if k == 0 || k == n {
            1
        } else {
            let k = k.min(n - k); // Take advantage of symmetry
            (1..=k).fold(1, |acc, i| acc * (n - i + 1) / i)
        }
    }
    
    /// Calculate permutations (n P k)
    pub fn permutations(n: u64, k: u64) -> u64 {
        if k > n {
            0
        } else {
            (n - k + 1..=n).product()
        }
    }
    
    /// Calculate Fibonacci number
    pub fn fibonacci(n: u64) -> u64 {
        if n <= 1 {
            n
        } else {
            let mut a = 0;
            let mut b = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

/// Random number generation utilities
pub struct Random;

impl Random {
    /// Generate random float between 0.0 and 1.0
    pub fn random() -> f64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        use std::time::{SystemTime, UNIX_EPOCH};
        
        let mut hasher = DefaultHasher::new();
        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos().hash(&mut hasher);
        let hash = hasher.finish();
        
        (hash as f64) / (u64::MAX as f64)
    }
    
    /// Generate random integer between min and max (inclusive)
    pub fn random_int(min: i32, max: i32) -> i32 {
        if min >= max {
            return min;
        }
        
        let range = (max - min + 1) as f64;
        min + (Self::random() * range) as i32
    }
    
    /// Generate random float between min and max
    pub fn random_float(min: f64, max: f64) -> f64 {
        min + Self::random() * (max - min)
    }
    
    /// Generate random boolean
    pub fn random_bool() -> bool {
        Self::random() < 0.5
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_math() {
        assert_eq!(Math::abs(-5.0), 5.0);
        assert_eq!(Math::abs_i32(-10), 10);
        assert_eq!(Math::sign(-3.0), -1.0);
        assert_eq!(Math::sign(0.0), 0.0);
        assert_eq!(Math::sign(3.0), 1.0);
        
        assert_eq!(Math::max(3.0, 7.0), 7.0);
        assert_eq!(Math::min(3.0, 7.0), 3.0);
        assert_eq!(Math::clamp(5.0, 1.0, 10.0), 5.0);
        assert_eq!(Math::clamp(-5.0, 1.0, 10.0), 1.0);
        assert_eq!(Math::clamp(15.0, 1.0, 10.0), 10.0);
    }
    
    #[test]
    fn test_powers_and_roots() {
        assert_eq!(Math::sqrt(9.0), 3.0);
        assert_eq!(Math::cbrt(8.0), 2.0);
        assert_eq!(Math::pow(2.0, 3.0), 8.0);
        assert_eq!(Math::powi(2.0, 3), 8.0);
        
        assert!((Math::exp(1.0) - constants::E).abs() < 1e-10);
        assert_eq!(Math::exp2(3.0), 8.0);
        assert!((Math::ln(constants::E) - 1.0).abs() < 1e-10);
        assert_eq!(Math::log2(8.0), 3.0);
        assert_eq!(Math::log10(1000.0), 3.0);
    }
    
    #[test]
    fn test_rounding() {
        assert_eq!(Math::floor(3.7), 3.0);
        assert_eq!(Math::ceil(3.2), 4.0);
        assert_eq!(Math::round(3.5), 4.0);
        assert_eq!(Math::trunc(3.7), 3.0);
        assert!((Math::fract(3.7) - 0.7).abs() < 1e-10);
    }
    
    #[test]
    fn test_trigonometry() {
        assert!((Trig::sin(constants::PI / 2.0) - 1.0).abs() < 1e-10);
        assert!((Trig::cos(0.0) - 1.0).abs() < 1e-10);
        assert!((Trig::tan(constants::PI / 4.0) - 1.0).abs() < 1e-10);
        
        assert!((Trig::asin(1.0) - constants::PI / 2.0).abs() < 1e-10);
        assert!((Trig::acos(1.0) - 0.0).abs() < 1e-10);
        assert!((Trig::atan(1.0) - constants::PI / 4.0).abs() < 1e-10);
        
        assert_eq!(Trig::to_radians(180.0), constants::PI);
        assert_eq!(Trig::to_degrees(constants::PI), 180.0);
    }
    
    #[test]
    fn test_statistics() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        
        assert_eq!(Stats::mean(&data), Some(3.0));
        assert_eq!(Stats::median(&data), Some(3.0));
        assert_eq!(Stats::range(&data), Some(4.0));
        
        let variance = Stats::variance(&data).unwrap();
        assert!((variance - 2.5).abs() < 1e-10);
        
        let std_dev = Stats::std_dev(&data).unwrap();
        assert!((std_dev - variance.sqrt()).abs() < 1e-10);
        
        assert_eq!(Stats::percentile(&data, 50.0), Some(3.0));
    }
    
    #[test]
    fn test_number_theory() {
        assert_eq!(NumberTheory::gcd(12, 18), 6);
        assert_eq!(NumberTheory::lcm(12, 18), 36);
        
        assert!(NumberTheory::is_prime(17));
        assert!(!NumberTheory::is_prime(15));
        
        let primes = NumberTheory::primes_up_to(20);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19]);
        
        assert_eq!(NumberTheory::factorial(5), 120);
        assert_eq!(NumberTheory::combinations(5, 2), 10);
        assert_eq!(NumberTheory::permutations(5, 2), 20);
        assert_eq!(NumberTheory::fibonacci(10), 55);
    }
    
    #[test]
    fn test_random() {
        // Test that random generates values in expected range
        for _ in 0..100 {
            let r = Random::random();
            assert!(r >= 0.0 && r <= 1.0);
            
            let ri = Random::random_int(1, 10);
            assert!(ri >= 1 && ri <= 10);
            
            let rf = Random::random_float(1.0, 10.0);
            assert!(rf >= 1.0 && rf <= 10.0);
        }
    }
    
    #[test]
    fn test_constants() {
        assert!((constants::PI - 3.141592653589793).abs() < 1e-10);
        assert!((constants::E - 2.718281828459045).abs() < 1e-10);
        assert!((constants::TAU - 2.0 * constants::PI).abs() < 1e-10);
    }
}