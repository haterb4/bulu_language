use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bulu::lexer::Lexer;

fn lexer_benchmark(c: &mut Criterion) {
    let source = r#"
func fibonacci(n: int32): int32 {
    if n <= 1 {
        return n
    }
    return fibonacci(n - 1) + fibonacci(n - 2)
}

func main() {
    let result = fibonacci(10)
    println("Fibonacci(10) = " + string(result))
}
"#;

    c.bench_function("lexer_tokenize", |b| {
        b.iter(|| {
            let mut lexer = Lexer::new(black_box(source));
            lexer.tokenize().unwrap()
        })
    });
}

criterion_group!(benches, lexer_benchmark);
criterion_main!(benches);