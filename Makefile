# NEXUS Makefile - Build Automation

.PHONY: help build test check lint clean install

help:
	@echo "NEXUS - Build System"
	@echo ""
	@echo "Targets:"
	@echo "  build       - Build release binary"
	@echo "  test        - Run all tests"
	@echo "  check       - Format + lint + test"
	@echo "  lint        - Run clippy"
	@echo "  format      - Format code"
	@echo "  clean       - Clean build artifacts"
	@echo "  install     - Install to /usr/local/bin"
	@echo "  bench       - Run benchmarks"

build:
	@echo "Building NEXUS..."
	cargo build --release
	@echo "✓ Build complete: target/release/nexus"

build-debug:
	cargo build

test:
	@echo "Running tests..."
	cargo test --workspace --all-features
	@echo "✓ All tests passed"

check: format lint test
	@echo "✓ All checks passed"

format:
	@echo "Formatting code..."
	cargo fmt --all
	@echo "✓ Code formatted"

lint:
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings
	@echo "✓ No lint errors"

clean:
	@echo "Cleaning..."
	cargo clean
	@echo "✓ Clean complete"

install: build
	@echo "Installing NEXUS..."
	sudo cp target/release/nexus /usr/local/bin/
	sudo chmod +x /usr/local/bin/nexus
	@echo "✓ Installed to /usr/local/bin/nexus"

bench:
	@echo "Running benchmarks..."
	cargo bench --workspace
	@echo "✓ Benchmarks complete"

docs:
	@echo "Generating documentation..."
	cargo doc --workspace --no-deps --open
	@echo "✓ Documentation generated"

init:
	@echo "Initializing NEXUS deployment..."
	./target/release/nexus init --genesis --data-dir ./nexus-data
	@echo "✓ Initialization complete"

start: build
	@echo "Starting NEXUS node..."
	./target/release/nexus start --node-id 1 --bind 0.0.0.0:8080

stats:
	@echo "=== NEXUS Statistics ==="
	@echo ""
	@echo "Lines of Code:"
	@find . -name "*.rs" -not -path "./target/*" | xargs wc -l | tail -1
	@echo ""
	@echo "Test Count:"
	@grep -r "#\[test\]" --include="*.rs" | wc -l
