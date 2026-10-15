# Makefile for btcx_tools

# Default target
all: build

# Build in debug mode
debug:
	@./scripts/build/build.sh

# Build in release mode
release:
	@./scripts/build/build.sh --release

# Clean build artifacts
clean:
	@cargo clean
	@rm -rf dist/*

# Run tests
test:
	@cargo test --workspace

# Format code
fmt:
	@cargo fmt --all

# Check code style
clippy:
	@cargo clippy --workspace -- -D warnings

# Show help
help:
	@echo "Available targets:"
	@echo "  make           - Build in debug mode (same as make debug)"
	@echo "  make debug     - Build in debug mode"
	@echo "  make release   - Build in release mode"
	@echo "  make clean     - Remove build artifacts"
	@echo "  make test      - Run tests"
	@echo "  make fmt       - Format code"
	@echo "  make clippy    - Run clippy"
	@echo "  make help      - Show this help"

.PHONY: all debug release clean test fmt clippy help
