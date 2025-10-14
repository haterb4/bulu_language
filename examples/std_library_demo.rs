// Standard Library Demo
// This example demonstrates the usage of all core standard library modules

use bulu::std::{io, fmt, strings, arrays, math, random, time};
use std::collections::HashMap;

fn main() {
    println!("=== Bulu Standard Library Demo ===\n");
    
    // I/O Module Demo
    println!("1. I/O Module Demo:");
    io::print(&["Hello".to_string(), "from".to_string(), "std.io!".to_string()]);
    println!();
    io::println(&["This".to_string(), "is".to_string(), "println!".to_string()]);
    io::printf("Formatted output: {0} + {1} = {2}\n", &["5".to_string(), "3".to_string(), "8".to_string()]).unwrap();
    println!();
    
    // String Formatting Demo
    println!("2. String Formatting Demo:");
    let formatted = fmt::format_positional("Hello {0}, you are {1} years old!", &["Alice".to_string(), "30".to_string()]);
    println!("{}", formatted);
    
    let mut named_args = HashMap::new();
    named_args.insert("name".to_string(), "Bob".to_string());
    named_args.insert("score".to_string(), "95".to_string());
    let named_formatted = fmt::format_named("Congratulations {name}! Your score is {score}%", &named_args);
    println!("{}", named_formatted);
    
    let advanced = fmt::format_advanced("Number: {0:05d}, Pi: {1:.2f}", &["42".to_string(), "3.14159".to_string()]);
    println!("{}", advanced);
    println!();
    
    // String Manipulation Demo
    println!("3. String Manipulation Demo:");
    let text = "  Hello, World!  ";
    println!("Original: '{}'", text);
    println!("Trimmed: '{}'", strings::StringUtils::trim(text));
    println!("Uppercase: '{}'", strings::StringUtils::to_upper(text));
    println!("Title Case: '{}'", strings::StringUtils::title_case("hello world from rust"));
    
    let words = strings::StringUtils::split("apple,banana,cherry", ",");
    println!("Split result: {:?}", words);
    
    let joined = strings::StringUtils::join(&words, " | ");
    println!("Joined: {}", joined);
    
    println!("Contains 'World': {}", strings::StringUtils::contains(text, "World"));
    println!("Length: {} characters", strings::StringUtils::len(text));
    println!();
    
    // Array Operations Demo
    println!("4. Array Operations Demo:");
    let numbers = vec![3, 1, 4, 1, 5, 9, 2, 6, 5, 3];
    println!("Original array: {:?}", numbers);
    
    let sorted = arrays::ArrayUtils::sort(&numbers);
    println!("Sorted: {:?}", sorted);
    
    let unique = arrays::ArrayUtils::unique(&numbers);
    println!("Unique elements: {:?}", unique);
    
    let evens = arrays::ArrayUtils::filter(&numbers, |&x| x % 2 == 0);
    println!("Even numbers: {:?}", evens);
    
    let doubled = arrays::ArrayUtils::map(&numbers, |&x| x * 2);
    println!("Doubled: {:?}", doubled);
    
    let sum = arrays::ArrayUtils::reduce(&numbers, 0, |acc, &x| acc + x);
    println!("Sum: {}", sum);
    
    let chunks = arrays::ArrayUtils::chunk(&numbers, 3);
    println!("Chunks of 3: {:?}", chunks);
    println!();
    
    // Math Operations Demo
    println!("5. Math Operations Demo:");
    println!("Constants:");
    println!("  PI = {:.6}", math::constants::PI);
    println!("  E = {:.6}", math::constants::E);
    println!("  Golden Ratio = {:.6}", math::constants::PHI);
    
    println!("Basic Math:");
    println!("  abs(-5.7) = {}", math::Math::abs(-5.7));
    println!("  sqrt(16) = {}", math::Math::sqrt(16.0));
    println!("  pow(2, 8) = {}", math::Math::pow(2.0, 8.0));
    println!("  max(10, 20) = {}", math::Math::max(10.0, 20.0));
    
    println!("Trigonometry:");
    println!("  sin(π/2) = {:.6}", math::Trig::sin(math::constants::PI / 2.0));
    println!("  cos(0) = {:.6}", math::Trig::cos(0.0));
    println!("  tan(π/4) = {:.6}", math::Trig::tan(math::constants::PI / 4.0));
    
    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    println!("Statistics for {:?}:", data);
    println!("  Mean: {:.2}", math::Stats::mean(&data).unwrap());
    println!("  Median: {:.2}", math::Stats::median(&data).unwrap());
    println!("  Std Dev: {:.2}", math::Stats::std_dev(&data).unwrap());
    
    println!("Number Theory:");
    println!("  GCD(48, 18) = {}", math::NumberTheory::gcd(48, 18));
    println!("  LCM(12, 15) = {}", math::NumberTheory::lcm(12, 15));
    println!("  Is 17 prime? {}", math::NumberTheory::is_prime(17));
    println!("  Factorial(6) = {}", math::NumberTheory::factorial(6));
    println!("  Fibonacci(10) = {}", math::NumberTheory::fibonacci(10));
    
    let primes = math::NumberTheory::primes_up_to(30);
    println!("  Primes up to 30: {:?}", primes);
    println!();
    
    // Random Numbers Demo
    println!("6. Random Numbers Demo:");
    let mut rng = random::Random::new();
    println!("Random numbers:");
    for i in 0..5 {
        let r = rng.random();
        let ri = rng.random_int(1, 100);
        let rf = rng.random_float(0.0, 10.0);
        let rb = rng.random_bool();
        println!("  {}: random={:.3}, int={}, float={:.2}, bool={}", i+1, r, ri, rf, rb);
    }
    
    println!("Random strings:");
    println!("  Alphanumeric: {}", rng.random_alphanumeric(8));
    println!("  Numeric: {}", rng.random_numeric(6));
    println!("  UUID: {}", rng.random_uuid());
    
    let items = vec!["apple", "banana", "cherry", "date", "elderberry"];
    println!("  Random choice: {:?}", rng.choose(&items));
    
    println!("Probability distributions:");
    println!("  Normal(0,1): {:.3}", rng.normal(0.0, 1.0));
    println!("  Exponential(1): {:.3}", rng.exponential(1.0));
    println!("  Poisson(3): {}", rng.poisson(3.0));
    println!();
    
    // Time and Date Demo
    println!("7. Time and Date Demo:");
    let now = time::Time::now();
    println!("Current time:");
    println!("  Timestamp: {}", now.timestamp());
    println!("  ISO 8601: {}", now.format_iso8601());
    println!("  Custom format: {}", now.format("%Y-%m-%d %H:%M:%S"));
    
    let duration = time::TimeDuration::from_hours(2);
    let later = now.add(duration);
    println!("2 hours later: {}", later.format_iso8601());
    
    let diff = later.duration_since(&now);
    println!("Duration between: {} hours", diff.total_hours());
    
    println!("Duration examples:");
    let d1 = time::TimeDuration::from_mins(90);
    println!("  90 minutes = {} hours {} minutes", d1.total_hours(), d1.total_mins() % 60);
    
    let d2 = time::TimeDuration::from_days(1);
    let d3 = time::TimeDuration::from_hours(6);
    let total = d2.add(&d3);
    println!("  1 day + 6 hours = {} hours", total.total_hours());
    
    // Stopwatch demo
    println!("Stopwatch demo:");
    let mut stopwatch = time::Stopwatch::new();
    stopwatch.start();
    time::sleep::sleep_millis(50);
    stopwatch.stop();
    println!("  Elapsed: {}ms", stopwatch.elapsed().total_millis());
    
    // Performance measurement
    let (result, timing) = time::measure::time(|| {
        let mut sum = 0;
        for i in 1..=1000 {
            sum += i;
        }
        sum
    });
    println!("  Sum 1-1000 = {} (took {}ms)", result, timing.total_millis());
    
    // Time parsing
    if let Ok(parsed) = time::parse::parse_iso8601("2021-01-01T00:00:00Z") {
        println!("  Parsed 2021-01-01: {}", parsed.format("%Y-%m-%d"));
    }
    println!();
    
    println!("=== Demo Complete ===");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_runs() {
        // Just ensure the demo code doesn't panic
        main();
    }
}