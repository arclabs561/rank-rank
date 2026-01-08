# Running Benchmarks

This guide explains how to run benchmarks for all `rank-*` crates.

## Prerequisites

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Ensure criterion is available (should be in workspace dependencies)
cargo --version
```

## Running Individual Crate Benchmarks

### rank-retrieve

```bash
cd crates/rank-retrieve/rank-retrieve
cargo bench --bench bm25
cargo bench --bench dense
```

### rank-fusion

```bash
cd crates/rank-fusion/rank-fusion
cargo bench --bench fusion
```

### rank-rerank

```bash
cd crates/rank-rerank/rank-rerank-core
cargo bench --bench comprehensive
```

### rank-learn

```bash
cd crates/rank-learn/rank-learn
cargo bench --bench lambdarank
```

### rank-soft

```bash
cd crates/rank-soft
cargo bench
```

## Running Comparison Benchmarks

```bash
cd benchmarks/comparisons
cargo bench
```

## Viewing Results

Criterion generates HTML reports in `target/criterion/`. View them with:

```bash
# Open the report in your browser
open target/criterion/*/report/index.html
```

Or use a simple HTTP server:

```bash
cd target/criterion
python3 -m http.server 8000
# Then visit http://localhost:8000
```

## Benchmarking All Crates

```bash
# From the root directory
./benchmarks/run_all_benchmarks.sh
```

## Continuous Benchmarking

For CI/CD, use:

```bash
# Run benchmarks and save results
cargo bench -- --output-format json > benchmarks/results/$(date +%Y%m%d).json

# Compare against baseline
cargo bench -- --baseline baseline
```

## Performance Regression Testing

```bash
# Establish baseline
cargo bench -- --save-baseline baseline

# Compare against baseline
cargo bench -- --baseline baseline
```

## Memory Profiling

For memory profiling, use `valgrind` or `heaptrack`:

```bash
# Using valgrind
valgrind --tool=massif cargo bench --bench bm25

# Using heaptrack (Linux)
heaptrack cargo bench --bench bm25
```

## Tips

1. **Warm up**: First run may be slower due to CPU frequency scaling
2. **Multiple runs**: Run benchmarks multiple times for consistency
3. **Isolated environment**: Run on dedicated hardware for consistent results
4. **Document environment**: Note CPU, RAM, OS for reproducibility

