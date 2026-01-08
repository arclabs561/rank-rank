#!/usr/bin/env bash
# Run all benchmarks across all rank-* crates

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "Running all benchmarks for rank-* crates..."
echo ""

# rank-retrieve
if [ -d "$ROOT_DIR/crates/rank-retrieve/rank-retrieve" ]; then
    echo "=== rank-retrieve ==="
    cd "$ROOT_DIR/crates/rank-retrieve/rank-retrieve"
    cargo bench --bench bm25 2>&1 | head -20
    cargo bench --bench dense 2>&1 | head -20
    echo ""
fi

# rank-fusion
if [ -d "$ROOT_DIR/crates/rank-fusion/rank-fusion" ]; then
    echo "=== rank-fusion ==="
    cd "$ROOT_DIR/crates/rank-fusion/rank-fusion"
    cargo bench --bench fusion 2>&1 | head -20
    echo ""
fi

# rank-rerank
if [ -d "$ROOT_DIR/crates/rank-rerank/rank-rerank-core" ]; then
    echo "=== rank-rerank ==="
    cd "$ROOT_DIR/crates/rank-rerank/rank-rerank-core"
    cargo bench --bench comprehensive 2>&1 | head -20
    echo ""
fi

# rank-learn
if [ -d "$ROOT_DIR/crates/rank-learn/rank-learn" ]; then
    echo "=== rank-learn ==="
    cd "$ROOT_DIR/crates/rank-learn/rank-learn"
    cargo bench --bench lambdarank 2>&1 | head -20
    echo ""
fi

# rank-soft
if [ -d "$ROOT_DIR/crates/rank-soft" ]; then
    echo "=== rank-soft ==="
    cd "$ROOT_DIR/crates/rank-soft"
    cargo bench 2>&1 | head -20
    echo ""
fi

echo "All benchmarks completed!"
echo ""
echo "View results in: target/criterion/"

