use criterion::{black_box, criterion_group, criterion_main, Criterion};
use bulu::lexer::Lexer;
use bulu::parser::Parser;

fn parser_benchmark(c: &mut Criterion) {
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

    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().unwrap();

    c.bench_function("parser_parse", |b| {
        b.iter(|| {
            let mut parser = Parser::new(black_box(tokens.clone()));
            parser.parse().unwrap()
        })
    });
}

criterion_group!(benches, parser_benchmark);
criterion_main!(benches);