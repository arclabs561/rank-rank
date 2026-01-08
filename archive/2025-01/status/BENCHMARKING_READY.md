# Benchmarking Infrastructure - Ready to Use

## Status: ✅ Complete and Verified

All benchmarking infrastructure has been created and verified to compile successfully.

## Quick Start

### Run All Benchmarks

```bash
./benchmarks/run_all_benchmarks.sh
```

### Individual Crate Benchmarks

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

### Comparison Benchmarks

```bash
cd benchmarks/comparisons
cargo bench
```

## What's Included

### ✅ Benchmark Suites Created

1. **rank-retrieve**
   - BM25 indexing, retrieval, and scoring
   - Dense retrieval (cosine similarity)

2. **rank-fusion**
   - Already had comprehensive benchmarks

3. **rank-rerank**
   - Already had comprehensive benchmarks

4. **rank-learn**
   - LambdaRank computation
   - NDCG computation
   - Batch processing

5. **rank-soft**
   - Already had comprehensive benchmarks

### ✅ Comparison Framework

- BM25 comparison benchmarks (ready for `bm25` crate integration)
- Dense retrieval comparison benchmarks (ready for `hnsw` integration)

### ✅ Documentation

- `benchmarks/README.md` - Main guide
- `benchmarks/COMPARISON_TOOLS.md` - Similar tools documented
- `benchmarks/BENCHMARK_REPORT_TEMPLATE.md` - Results template
- `benchmarks/RUN_BENCHMARKS.md` - Running guide
- `benchmarks/BENCHMARKING_SUMMARY.md` - Infrastructure summary

## Verification

All benchmarks compile successfully:
- ✅ rank-retrieve benchmarks compile
- ✅ rank-learn benchmarks compile
- ✅ Comparison benchmarks compile
- ✅ All warnings resolved

## Next Steps

1. **Run Initial Benchmarks**: Establish baseline performance
   ```bash
   ./benchmarks/run_all_benchmarks.sh
   ```

2. **Add Comparison Libraries**: When ready, add to `benchmarks/comparisons/Cargo.toml`:
   ```toml
   [dev-dependencies]
   bm25 = "2.3"  # For BM25 comparison
   hnsw = "0.10" # For dense retrieval comparison
   ```

3. **View Results**: HTML reports in `target/criterion/`
   ```bash
   open target/criterion/*/report/index.html
   ```

4. **Document Results**: Fill in `benchmarks/BENCHMARK_REPORT_TEMPLATE.md`

## Performance Metrics Tracked

- **Throughput**: Operations per second
- **Latency**: Time per operation (p50, p95, p99)
- **Memory**: Peak memory usage
- **Accuracy**: Correctness of results

## Test Scenarios

- Small scale: 1K documents, 100 queries
- Medium scale: 10K documents, 1K queries
- Large scale: 100K documents, 10K queries
- Very large scale: 1M documents, 100K queries

## Similar Tools Documented

See `benchmarks/COMPARISON_TOOLS.md` for full list, including:
- Rust: `bm25` crate, `tantivy`, `hnsw`, `usearch`
- Python: `rank-bm25`, `sentence-transformers`, XGBoost, LightGBM

## Ready to Use! 🚀

All infrastructure is in place and ready for benchmarking runs.

