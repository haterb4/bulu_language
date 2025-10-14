# Makefile for Bulu Programming Language

.PHONY: all build release test clean fmt lint doc bench install uninstall help

# Default target
all: build

# Build in debug mode
build:
	cargo build

# Build in release mode (optimized)
release:
	cargo build --release

# Run all tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/
	rm -rf build/
	rm -rf dist/

# Format code
fmt:
	cargo fmt

# Check formatting
fmt-check:
	cargo fmt --check

# Run linter
lint:
	cargo clippy -- -D warnings

# Generate documentation
doc:
	cargo doc --open

# Run benchmarks
bench:
	cargo bench

# Install binaries to system
install: release
	cargo install --path .

# Uninstall binaries from system
uninstall:
	cargo uninstall bulu

# Development setup
dev-setup:
	rustup component add rustfmt clippy
	cargo install cargo-watch

# Watch for changes and rebuild
watch:
	cargo watch -x build

# Watch for changes and run tests
watch-test:
	cargo watch -x test

# Create example project
example:
	./target/release/lang new example-project || ./target/debug/lang new example-project

# Run integration tests
integration-test: build
	@echo "Running integration tests..."
	@mkdir -p test-output
	@echo 'func main() { println("Hello, Bulu!") }' > test-output/hello.bu
	./target/debug/langc test-output/hello.bu --emit tokens
	@rm -rf test-output

# Check code coverage
coverage:
	cargo tarpaulin --out Html --output-dir coverage/

# Security audit
audit:
	cargo audit

# Update dependencies
update:
	cargo update

# Build for multiple targets
cross-build:
	cargo build --target x86_64-unknown-linux-gnu
	cargo build --target x86_64-pc-windows-gnu
	cargo build --target x86_64-apple-darwin

# Package for distribution
package: release
	@mkdir -p dist
	@cp target/release/langc dist/
	@cp target/release/lang dist/
	@cp README.md dist/
	@cp LICENSE dist/
	@tar -czf dist/bulu-$(shell cargo pkgid | cut -d# -f2).tar.gz -C dist .

# Help target
help:
	@echo "Bulu Programming Language Build System"
	@echo ""
	@echo "Available targets:"
	@echo "  build          Build in debug mode"
	@echo "  release        Build in release mode (optimized)"
	@echo "  test           Run all tests"
	@echo "  test-verbose   Run tests with output"
	@echo "  clean          Clean build artifacts"
	@echo "  fmt            Format code"
	@echo "  fmt-check      Check code formatting"
	@echo "  lint           Run linter (clippy)"
	@echo "  doc            Generate and open documentation"
	@echo "  bench          Run benchmarks"
	@echo "  install        Install binaries to system"
	@echo "  uninstall      Uninstall binaries from system"
	@echo "  dev-setup      Set up development environment"
	@echo "  watch          Watch for changes and rebuild"
	@echo "  watch-test     Watch for changes and run tests"
	@echo "  example        Create an example project"
	@echo "  integration-test Run integration tests"
	@echo "  coverage       Generate code coverage report"
	@echo "  audit          Run security audit"
	@echo "  update         Update dependencies"
	@echo "  cross-build    Build for multiple targets"
	@echo "  package        Package for distribution"
	@echo "  help           Show this help message"