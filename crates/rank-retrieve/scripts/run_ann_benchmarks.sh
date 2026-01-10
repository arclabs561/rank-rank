#!/bin/bash
# Run comprehensive ANN benchmarks following ann-benchmarks methodology
#
# This script runs benchmarks on all implemented ANN algorithms and generates
# reports with visualizations.

set -e

cd "$(dirname "$0")/.."

echo "=== Running ANN Benchmarks ==="
echo ""

# Check if benchmark feature is enabled
if ! cargo check --features benchmark 2>/dev/null; then
    echo "Error: benchmark feature not available"
    exit 1
fi

# Create output directory
OUTPUT_DIR="benchmark_results"
mkdir -p "$OUTPUT_DIR"

echo "1. Running criterion benchmarks..."
cargo bench --bench ann_benchmarks_standard --features "benchmark,hnsw,nsw,scann,ivf_pq,diskann,sng,lsh,annoy,kdtree,balltree,rptree" 2>&1 | tee "$OUTPUT_DIR/benchmark_output.txt"

echo ""
echo "2. Generating benchmark reports..."
# The benchmark runner will generate CSV and JSON outputs
# Visualization scripts can be run separately

echo ""
echo "=== Benchmarks Complete ==="
echo "Results saved to: $OUTPUT_DIR/"
echo ""
echo "Next steps:"
echo "- Review benchmark_output.txt for detailed results"
echo "- Use visualization utilities to generate plots"
echo "- Compare algorithms using generated CSV/JSON files"
