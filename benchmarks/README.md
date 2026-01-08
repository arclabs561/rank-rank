# Benchmarking Infrastructure

This directory contains comprehensive benchmarks comparing `rank-*` crates against similar tools in Rust and other languages.

## Structure

```
benchmarks/
├── README.md                    # This file
├── COMPARISON_TOOLS.md          # Documentation of similar tools
├── rank-retrieve/              # BM25, dense, sparse retrieval benchmarks
├── rank-fusion/                # Rank fusion algorithm benchmarks
├── rank-rerank/                # Reranking and MaxSim benchmarks
├── rank-learn/                 # Learning to Rank benchmarks
├── rank-soft/                  # Soft ranking benchmarks (already exists)
├── comparisons/                # Cross-tool comparisons
└── results/                    # Benchmark results (gitignored)
```

## Running Benchmarks

### Individual Crate Benchmarks

```bash
# rank-retrieve
cd crates/rank-retrieve/rank-retrieve
cargo bench

# rank-fusion
cd crates/rank-fusion/rank-fusion
cargo bench

# rank-rerank
cd crates/rank-rerank/rank-rerank-core
cargo bench

# rank-learn
cd crates/rank-learn/rank-learn
cargo bench

# rank-soft
cd crates/rank-soft
cargo bench
```

### Comparison Benchmarks

```bash
cd benchmarks/comparisons
cargo bench
```

## Benchmark Results

Results are stored in `benchmarks/results/` (gitignored) and can be viewed with:

```bash
# View HTML reports
open benchmarks/results/*/report/index.html
```

## Similar Tools

See `COMPARISON_TOOLS.md` for a comprehensive list of similar tools in:
- Rust: `bm25` crate, `tantivy`, `hnsw`
- Python: `rank-bm25`, `sentence-transformers`, `reranking` libraries
- Other languages: Go, C++, etc.

## Performance Goals

Our benchmarks measure:
- **Throughput**: Operations per second
- **Latency**: Time per operation
- **Memory**: Peak memory usage
- **Accuracy**: Correctness of results (where applicable)

## Continuous Benchmarking

Benchmarks run in CI/CD to track performance regressions.

