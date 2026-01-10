# ANN Benchmark Implementation - Following ann-benchmarks

This document describes our implementation of standard ANN benchmarks following the methodology from **ann-benchmarks** (erikbern/ann-benchmarks, 5.5k+ stars).

## ann-benchmarks Structure

### Key Learnings

1. **Standardized Interface**: All algorithms implement same interface
2. **Multiple K Values**: Evaluate at K=1, 10, 100
3. **Percentile Reporting**: p50, p95, p99 query times (not just mean)
4. **Ground Truth**: Always use exact search for fair recall evaluation
5. **Isolation**: Same environment for all algorithms
6. **Visualization**: Recall@K vs Query Time plots

### Standard Metrics

- **Recall@K**: `|retrieved ∩ ground_truth| / |ground_truth|`
- **Query Time**: Milliseconds per query (p50, p95, p99)
- **Build Time**: Seconds to construct index
- **Memory Usage**: Bytes used by index
- **Throughput**: Queries per second (QPS)

### Standard Datasets

- **SIFT-1M**: 1M vectors, 128 dimensions, L2 distance
- **GloVe-100**: 1.2M vectors, 100 dimensions, cosine similarity
- **MNIST**: 60k vectors, 784 dimensions, L2 distance
- **NYTimes**: 290k vectors, 256 dimensions, cosine similarity
- **Random**: Synthetic datasets (various sizes)

## Our Implementation

### Module Structure

```
src/benchmark/
├── mod.rs           # Public API
├── metrics.rs       # Recall@K, statistics
├── datasets.rs      # Dataset generation, ground truth
└── runner.rs        # Benchmark runner
```

### Usage

```rust
use rank_retrieve::benchmark::{BenchmarkRunner, create_benchmark_dataset};
use rank_retrieve::dense::sng::{SNGIndex, SNGParams};

// Create benchmark dataset
let dataset = create_benchmark_dataset(10000, 100, 128, 42);

// Create algorithm
let params = SNGParams::default();
let index = SNGIndex::new(128, params)?;

// Run benchmark
let mut runner = BenchmarkRunner::new();
runner.add_dataset("synthetic_10k".to_string(), dataset);
let results = runner.run_algorithm("sng", index, "synthetic", &dataset)?;

// Results include:
// - recall_mean, recall_p50, recall_p95, recall_p99
// - query_time_mean, query_time_p50, query_time_p95, query_time_p99
// - build_time, memory_usage, throughput
```

### Benchmark Files

1. **`benches/ann_benchmarks.rs`**: Original benchmarks
2. **`benches/ann_benchmarks_standard.rs`**: ann-benchmarks-compatible benchmarks

### Key Functions

- `recall_at_k()`: Compute recall following ann-benchmarks formula
- `generate_synthetic_dataset()`: Create normalized random vectors
- `compute_ground_truth()`: Exact search for fair evaluation
- `BenchmarkRunner`: Run comprehensive benchmarks

## Next Steps

1. ✅ Implement standard metrics (recall, percentiles)
2. ✅ Create synthetic datasets
3. ✅ Implement ground truth computation
4. ⏳ Add real dataset loaders (SIFT, GloVe)
5. ⏳ Create visualization (Recall@K vs Query Time plots)
6. ⏳ Generate HTML reports
7. ⏳ Performance regression detection

## References

- **ann-benchmarks**: https://github.com/erikbern/ann-benchmarks
- **ann-benchmarks.com**: https://ann-benchmarks.com/
- **Erik Bernhardsson's Blog**: https://erikbern.com/2018/06/17/new-approximate-nearest-neighbor-benchmarks.html
