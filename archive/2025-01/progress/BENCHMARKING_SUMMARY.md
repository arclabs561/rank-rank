# Benchmarking Summary

## Overview

Comprehensive benchmarking infrastructure has been created for all `rank-*` crates, with comparison capabilities against similar tools in Rust and other languages.

## Infrastructure Created

### Directory Structure

```
benchmarks/
├── README.md                    # Main benchmarking documentation
├── COMPARISON_TOOLS.md          # Documentation of similar tools
├── BENCHMARK_REPORT_TEMPLATE.md # Template for benchmark reports
├── RUN_BENCHMARKS.md            # Guide for running benchmarks
├── BENCHMARKING_SUMMARY.md      # This file
├── run_all_benchmarks.sh        # Script to run all benchmarks
├── comparisons/                 # Cross-tool comparison benchmarks
│   ├── Cargo.toml
│   └── benches/
│       ├── bm25_comparison.rs
│       └── dense_comparison.rs
└── results/                     # Benchmark results (gitignored)
```

### Crate Benchmarks

#### rank-retrieve
- `benches/bm25.rs` - BM25 indexing, retrieval, and scoring benchmarks
- `benches/dense.rs` - Dense retrieval (cosine similarity) benchmarks

#### rank-fusion
- `benches/fusion.rs` - Rank fusion algorithm benchmarks (already existed)

#### rank-rerank
- `benches/comprehensive.rs` - MaxSim, cosine similarity, MMR benchmarks (already existed)

#### rank-learn
- `benches/lambdarank.rs` - LambdaRank computation, NDCG, batch processing benchmarks

#### rank-soft
- `benches/ranking_benchmark.rs` - Soft ranking method benchmarks (already existed)

## Comparison Tools Documented

### Rust Tools
- **BM25**: `bm25` crate, `tantivy`, `anda_db_tfs`
- **Dense Retrieval**: `hnsw`, `usearch`
- **Learning to Rank**: No dedicated Rust crates (rank-learn is pioneering)

### Python Tools
- **BM25**: `rank-bm25`
- **Dense Retrieval**: `sentence-transformers`
- **Reranking**: Various reranking libraries
- **Learning to Rank**: XGBoost, LightGBM

## Benchmark Metrics

All benchmarks measure:
1. **Throughput**: Operations per second
2. **Latency**: Time per operation (p50, p95, p99)
3. **Memory**: Peak memory usage (via profiling)
4. **Accuracy**: Correctness of results (where applicable)

## Test Scenarios

Benchmarks cover:
- **Small scale**: 1K documents, 100 queries
- **Medium scale**: 10K documents, 1K queries
- **Large scale**: 100K documents, 10K queries
- **Very large scale**: 1M documents, 100K queries

## Running Benchmarks

### Individual Crates

```bash
# rank-retrieve
cd crates/rank-retrieve/rank-retrieve
cargo bench --bench bm25
cargo bench --bench dense

# rank-fusion
cd crates/rank-fusion/rank-fusion
cargo bench --bench fusion

# rank-rerank
cd crates/rank-rerank/rank-rerank-core
cargo bench --bench comprehensive

# rank-learn
cd crates/rank-learn/rank-learn
cargo bench --bench lambdarank

# rank-soft
cd crates/rank-soft
cargo bench
```

### All Crates

```bash
./benchmarks/run_all_benchmarks.sh
```

### Comparison Benchmarks

```bash
cd benchmarks/comparisons
cargo bench
```

## Viewing Results

Criterion generates HTML reports in `target/criterion/`. View with:

```bash
open target/criterion/*/report/index.html
```

## Next Steps

1. **Run Initial Benchmarks**: Establish baseline performance
2. **Add Comparison Libraries**: Integrate `bm25` crate, `tantivy`, etc. for direct comparison
3. **Python Comparisons**: Create Python benchmark scripts for cross-language comparison
4. **CI/CD Integration**: Add benchmark runs to CI/CD pipeline
5. **Performance Regression Testing**: Set up baseline comparison for PRs
6. **Document Results**: Fill in `BENCHMARK_REPORT_TEMPLATE.md` with actual results

## Status

- [x] Benchmarking infrastructure created
- [x] Individual crate benchmarks implemented
- [x] Comparison framework created
- [x] Documentation written
- [ ] Initial benchmarks run (pending execution)
- [ ] Comparison libraries integrated (pending)
- [ ] Results documented (pending)

## Notes

- All benchmarks use `criterion` for statistical rigor
- Benchmarks are designed to be reproducible with fixed seeds where applicable
- Results are stored in `target/criterion/` (gitignored)
- Comparison benchmarks require external dependencies to be added to `Cargo.toml`

