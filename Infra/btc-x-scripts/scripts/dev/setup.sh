#!/usr/bin/env bash
# Development setup script

set -euo pipefail

# Source common functions
source "$(dirname "${BASH_SOURCE[0]}")/../build/common.sh"

info "Setting up development environment..."

# Install Rust toolchain if not installed
if ! command_exists rustup; then
    info "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
fi

# Install required components
info "Installing Rust components..."
rustup component add rustfmt clippy

# Install cargo-edit for cargo add
if ! command_exists cargo-add; then
    info "Installing cargo-edit..."
    cargo install cargo-edit
fi

# Install additional tools
if ! command_exists upx; then
    info "Note: Install UPX for binary compression:"
    info "  - Ubuntu/Debian: sudo apt install upx"
    info "  - macOS: brew install upx"
fi

success "Development environment is ready!"
echo "Useful commands:"
echo "  make           # Build in debug mode"
echo "  make release   # Build in release mode"
echo "  make test      # Run tests"
echo "  make fmt       # Format code"
echo "  make clippy    # Run clippy"
echo "  make clean     # Clean build artifacts"
