#!/bin/bash
# Generate comprehensive benchmark report with visualizations
#
# This script runs benchmarks and generates HTML reports with plots
# following ann-benchmarks style.

set -e

cd "$(dirname "$0")/.."

echo "=== Generating ANN Benchmark Report ===\n"

OUTPUT_DIR="benchmark_results"
mkdir -p "$OUTPUT_DIR"

# Run benchmark example
echo "1. Running benchmarks..."
cargo run --example benchmark_all_algorithms \
    --features "benchmark,hnsw,nsw,scann,ivf_pq,diskann,sng,lsh,annoy,kdtree,balltree,rptree,serde" \
    --release 2>&1 | tee "$OUTPUT_DIR/benchmark_run.log"

# Move generated files to output directory
if [ -f "benchmark_results.csv" ]; then
    mv benchmark_results.csv "$OUTPUT_DIR/"
    echo "  ✓ CSV results saved"
fi

if [ -f "benchmark_results.json" ]; then
    mv benchmark_results.json "$OUTPUT_DIR/"
    echo "  ✓ JSON results saved"
fi

if [ -f "plot_benchmarks.py" ]; then
    mv plot_benchmarks.py "$OUTPUT_DIR/"
    echo "  ✓ Plot script saved"
fi

# Generate plots if Python is available
if command -v python3 &> /dev/null; then
    echo ""
    echo "2. Generating plots..."
    cd "$OUTPUT_DIR"
    if [ -f "plot_benchmarks.py" ]; then
        python3 plot_benchmarks.py 2>&1 | tee plot_generation.log
        echo "  ✓ Plots generated"
    fi
    cd ..
fi

echo ""
echo "=== Benchmark Report Complete ==="
echo "Results available in: $OUTPUT_DIR/"
echo ""
echo "Files:"
echo "  - benchmark_results.csv: Raw benchmark data"
echo "  - benchmark_results.json: JSON format results"
echo "  - plot_benchmarks.py: Python plotting script"
echo "  - benchmark_run.log: Full benchmark output"
