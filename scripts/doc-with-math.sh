#!/usr/bin/env bash
# Build rustdoc documentation with math rendering enabled

set -e

# Get the crate name from argument or current directory
if [ -n "$1" ]; then
    CRATE_DIR="crates/$1"
else
    CRATE_DIR=$(pwd)
fi

# Find the workspace root (where rustdoc-header.html is)
WORKSPACE_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || echo "$(cd "$(dirname "$0")/.." && pwd)")

if [ ! -f "$WORKSPACE_ROOT/rustdoc-header.html" ]; then
    echo "Error: rustdoc-header.html not found at $WORKSPACE_ROOT/rustdoc-header.html"
    exit 1
fi

# Calculate relative path from crate to header
cd "$CRATE_DIR"
REL_PATH=$(realpath --relative-to . "$WORKSPACE_ROOT/rustdoc-header.html")

echo "Building documentation for $(basename "$CRATE_DIR") with math rendering..."
RUSTDOCFLAGS="--html-in-header $REL_PATH" cargo doc --open

