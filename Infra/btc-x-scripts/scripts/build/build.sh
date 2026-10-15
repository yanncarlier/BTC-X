#!/usr/bin/env bash
set -euo pipefail

# Source common functions
source "$(dirname "${BASH_SOURCE[0]}")/common.sh"

# Default values
BUILD_TYPE="debug"
TARGET=""
CLEAN=false

# Print usage information
print_usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Build all btcx_tools workspace members and copy binaries to dist/ directory."
    echo
    echo "Options:"
    echo "  --release           Build in release mode (default: debug)"
    echo "  --target TARGET     Build for specific target triple"
    echo "  --clean             Clean build directories before building"
    echo "  -h, --help          Show this help message"
    echo
    echo "Examples:"
    echo "  $0                  # Build debug binaries"
    echo "  $0 --release        # Build release binaries"
    echo "  $0 --clean --release # Clean and build release"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case "$1" in
        --release)
            BUILD_TYPE="release"
            shift
            ;;
        --target)
            TARGET="$2"
            shift 2
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        -h|--help)
            print_usage
            exit 0
            ;;
        *)
            error "Unknown option: $1\nUse --help for usage information."
            ;;
    esac
done

# Clean if requested
if [[ "$CLEAN" == true ]]; then
    info "Cleaning build directories..."
    cargo clean
    rm -rf "$DIST_DIR"/*
fi

# Build all workspace members
info "Building all workspace members ($BUILD_TYPE)..."
cargo build --workspace $([[ "$BUILD_TYPE" == "release" ]] && echo "--release") $([[ -n "$TARGET" ]] && echo "--target $TARGET")

# Create list of crates to process
CRATES=(
    "net/blockstream_info:blockstream_balance_loop"
    "net/blockstream_tx:blockstream_tx"
    "net/broadcast_tx:broadcast_tx"
    "tx/create_tx:create_tx"
    "tx/estimate_fee:estimate_fee"
    "tx/fetch_utxos:fetch_utxos"
    "tx/sign_tx:sign_tx"
    "gen/all_address_types:all_address_types"
    "gen/brain_wallet:brain_wallet"
    "gen/generate_mnemonic:generate_mnemonic"
    "gen/m_0_i:m_0_i"
    "gen/m_0h_i:m_0h_i"
    "gen/m_0h_0_0h:m_0h_0_0h"
    "gen/m_0h_0_i:m_0h_0_i"
    "gen/m_0h_0_ih:m_0h_0_ih"
    "gen/m_0h_0h_0h:m_0h_0h_0h"
    "gen/m_0h_0h_i:m_0h_0h_i"
    "gen/m_0h_0h_ih:m_0h_0h_ih"
    "gen/m_44h_0h_0h_0:m_44h_0h_0h_0"
    "gen/m_44h_0h_0h_0_i:m_44h_0h_0h_0_i"
    "gen/m_44h_0h_0h_0_ih:m_44h_0h_0h_0_ih"
    "gen/m_44h_0h_0h_i:m_44h_0h_0h_i"
    "gen/m_44h_0h_0h_ih:m_44h_0h_0h_ih"
    "gen/m_49h_0h_0h_0_i:m_49h_0h_0h_0_i"
    "gen/m_49h_0h_0h_0_ih:m_49h_0h_0h_0_ih"
    "gen/m_84h_0h_0h_0_i:m_84h_0h_0h_0_i"
    "gen/m_84h_0h_0h_0_ih:m_84h_0h_0h_0_ih"
    "gen/m_86h_0h_0h_i:m_86h_0h_0h_i"
)

# Process each crate
for crate_spec in "${CRATES[@]}"; do
    IFS=':' read -r crate_dir bin_name <<< "$crate_spec"
    
    # Determine source path based on target and build type
    target_dir=""
    if [[ -n "$TARGET" ]]; then
        target_dir="$TARGET/$([[ "$BUILD_TYPE" == "release" ]] && echo "release" || echo "debug")"
    else
        target_dir="$([[ "$BUILD_TYPE" == "release" ]] && echo "release" || echo "debug")"
    fi
    
    src_path="$PROJECT_ROOT/target/$target_dir/$bin_name"
    dest_path="$DIST_DIR/$bin_name"
    
    # Copy binary
    if [[ -f "$src_path" ]]; then
        copy_binary "$src_path" "$dest_path"
        post_process_binary "$dest_path"
    else
        error "Binary not found: $src_path"
    fi
done

# Copy library files if they exist
for lib_ext in so dll dylib; do
    lib_path="$PROJECT_ROOT/target/$([[ "$BUILD_TYPE" == "release" ]] && echo "release" || echo "debug")/libbtcx_lib.$lib_ext"
    if [[ -f "$lib_path" ]]; then
        copy_binary "$lib_path" "$DIST_DIR/libbtcx_lib.$lib_ext"
        post_process_binary "$DIST_DIR/libbtcx_lib.$lib_ext"
    fi
done

success "\nBuild completed successfully!"
echo "Binaries are available in: $DIST_DIR"
ls -lh "$DIST_DIR"
