#!/bin/bash
# Profiling script for rank-retrieve
# Uses cargo-flamegraph for flamegraph generation

set -e

cd "$(dirname "$0")/.."

echo "Building release profile..."
cargo build --release --features "bm25,sparse,dense" --quiet

echo "Running benchmarks for profiling..."
cargo bench --bench bm25 --features "bm25" --quiet 2>&1 | head -20
cargo bench --bench sparse --features "sparse" --quiet 2>&1 | head -20
cargo bench --bench dense --features "dense" --quiet 2>&1 | head -20

echo "Generating flamegraph for BM25..."
cargo flamegraph --bench bm25 --features "bm25" -- --bench 2>&1 | tail -10 || echo "Flamegraph generation failed (may need sudo on Linux)"

echo "Profiling complete. Check flamegraph.svg in current directory."
