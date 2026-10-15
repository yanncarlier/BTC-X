#!/usr/bin/env bash
# Common build functions and variables

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DIST_DIR="$PROJECT_ROOT/dist"
TARGET_DIR="$PROJECT_ROOT/target"

# Ensure directories exist
mkdir -p "$DIST_DIR"

# Print error message and exit
error() {
    echo -e "${RED}Error: $1${NC}" >&2
    exit 1
}

# Print success message
success() {
    echo -e "${GREEN}$1${NC}"
}

# Print info message
info() {
    echo -e "${YELLOW}$1${NC}"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Build a specific crate
build_crate() {
    local crate_dir="$1"
    local target="$2"
    local release="$3"
    
    (
        cd "$PROJECT_ROOT/$crate_dir" || error "Failed to enter directory: $crate_dir"
        
        local cmd="cargo build"
        [[ "$release" == "release" ]] && cmd+=" --release"
        [[ -n "$target" ]] && cmd+=" --target $target"
        
        info "Building $crate_dir ($target, $release)..."
        $cmd || error "Failed to build $crate_dir"
    )
}

# Copy binary to dist directory
copy_binary() {
    local src="$1"
    local dest="$2"
    
    if [[ -f "$src" ]]; then
        info "Copying $(basename "$src") to $dest"
        cp -v "$src" "$dest" || error "Failed to copy $src to $dest"
    else
        error "Binary not found: $src"
    fi
}

# Post-process binaries (strip and compress)
post_process_binary() {
    local binary_path="$1"
    
    if [[ ! -f "$binary_path" ]]; then
        error "Binary not found: $binary_path"
    fi
    
    # Strip debug symbols
    if command_exists strip; then
        info "Stripping $binary_path..."
        strip "$binary_path" 2>/dev/null || true
    fi
    
    # Compress with UPX if available
    if command_exists upx; then
        info "Compressing $binary_path with UPX..."
        upx --best "$binary_path" 2>/dev/null || true
    fi
}
