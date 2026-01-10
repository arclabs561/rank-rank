# Index Factory and Auto-Tuning Guide

Complete guide to using the Faiss-inspired index factory and parameter auto-tuning in `rank-retrieve`.

## Table of Contents

1. [Index Factory](#index-factory)
2. [Auto-Tuning](#auto-tuning)
3. [Best Practices](#best-practices)
4. [Examples](#examples)
5. [Troubleshooting](#troubleshooting)

## Index Factory

### Overview

The index factory provides a string-based API for creating ANN indexes, inspired by Faiss's `index_factory` pattern. This simplifies experimentation and makes it easy to try different index types.

### Supported Index Types

#### HNSW (Hierarchical Navigable Small World)

```rust
use rank_retrieve::dense::ann::factory::index_factory;

// Basic: HNSW with m=32 connections
let mut index = index_factory(128, "HNSW32")?;

// Advanced: HNSW with m=16, m_max=32
let mut index = index_factory(128, "HNSW16,32")?;
```

**Format**: `"HNSW{m}"` or `"HNSW{m},{m_max}"`
- `m`: Maximum connections per node (typically 16-64)
- `m_max`: Maximum connections for new nodes (defaults to `m`)

#### NSW (Flat Navigable Small World)

```rust
// Flat NSW (single layer, lower memory)
let mut index = index_factory(128, "NSW32")?;
```

**Format**: `"NSW{m}"`
- `m`: Maximum connections per node

#### IVF-PQ (Inverted File Index with Product Quantization)

```rust
// Basic: 1024 clusters, 8 codebooks
let mut index = index_factory(128, "IVF1024,PQ8")?;

// Advanced: 1024 clusters, 8 codebooks, 8 bits per codebook
let mut index = index_factory(128, "IVF1024,PQ8x8")?;
```

**Format**: `"IVF{n},PQ{m}"` or `"IVF{n},PQ{m}x{b}"`
- `n`: Number of clusters (typically 100-4096)
- `m`: Number of codebooks (dimension must be divisible by m)
- `b`: Bits per codebook (defaults to 8, meaning 256 codebook size)

**Important**: Dimension must be divisible by number of codebooks.

#### SCANN (Anisotropic Vector Quantization)

```rust
// SCANN with 256 partitions
let mut index = index_factory(128, "SCANN256")?;
```

**Format**: `"SCANN{n}"`
- `n`: Number of partitions (typically 64-1024)

#### Tree Methods (Not Supported)

Tree-based methods (KD-Tree, Ball Tree, K-Means Tree, Random Projection Tree) are **not supported** via the factory pattern because they have complex parameter structures that don't map well to simple strings. Create them directly:

```rust
use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};

let params = KMeansTreeParams {
    num_clusters: 16,
    max_leaf_size: 50,
    max_depth: 10,
    max_iterations: 10,
};
let mut index = KMeansTreeIndex::new(128, params)?;
```

### Usage Pattern

```rust
use rank_retrieve::dense::ann::factory::index_factory;

// 1. Create index
let mut index = index_factory(128, "HNSW32")?;

// 2. Add vectors
for (i, vec) in vectors.iter().enumerate() {
    index.add(i as u32, vec.clone())?;
}

// 3. Build index (required before search)
index.build()?;

// 4. Search
let results = index.search(&query, 10)?;
```

### Error Handling

The factory provides clear error messages:

```rust
// Invalid format
let result = index_factory(128, "INVALID");
assert!(result.is_err());
// Error: "Unsupported index factory string: 'INVALID'. Supported: ..."

// Missing feature
#[cfg(not(feature = "hnsw"))]
let result = index_factory(128, "HNSW32");
// Error: "HNSW feature not enabled. Add 'hnsw' feature to Cargo.toml"

// Invalid parameters
let result = index_factory(128, "HNSW0");
// Error: "HNSW m parameter must be greater than 0"

// Dimension mismatch
let result = index_factory(100, "IVF1024,PQ8");  // 100 % 8 != 0
// Error: "Dimension (100) must be divisible by num_codebooks (8) for PQ"
```

## Auto-Tuning

### Overview

Auto-tuning automatically finds optimal parameters for ANN indexes using grid search. This is especially useful for parameters like `nprobe` (IVF-PQ) and `ef_search` (HNSW).

### Performance Criteria

#### RecallAtK

Maximize recall above a target threshold:

```rust
use rank_retrieve::dense::ann::autotune::{ParameterTuner, Criterion};

let tuner = ParameterTuner::new()
    .criterion(Criterion::RecallAtK { 
        k: 10, 
        target: 0.95  // Minimum acceptable recall
    });
```

#### LatencyWithRecall

Minimize latency while maintaining minimum recall:

```rust
let tuner = ParameterTuner::new()
    .criterion(Criterion::LatencyWithRecall {
        k: 10,
        min_recall: 0.90,
        max_latency_ms: 10.0,  // Maximum acceptable latency
    });
```

#### Balanced

Weighted combination of recall and latency:

```rust
let tuner = ParameterTuner::new()
    .criterion(Criterion::Balanced {
        k: 10,
        recall_weight: 0.7,   // 70% weight on recall
        latency_weight: 0.3,  // 30% weight on latency
    });
```

### Tuning IVF-PQ nprobe

```rust
use rank_retrieve::dense::ann::autotune::{ParameterTuner, Criterion};
use rank_retrieve::benchmark::datasets::create_benchmark_dataset;

// Create dataset
let dataset = create_benchmark_dataset(10000, 1000, 128, 42);

// Create tuner
let tuner = ParameterTuner::new()
    .criterion(Criterion::RecallAtK { k: 10, target: 0.95 })
    .num_test_queries(100)  // Use 100 queries for evaluation
    .time_budget(std::time::Duration::from_secs(60));  // 60 second limit

// Tune nprobe
let result = tuner.tune_ivf_pq_nprobe(
    &dataset,
    128,      // dimension
    1024,     // num_clusters
    &[1, 2, 4, 8, 16, 32, 64],  // nprobe values to try
)?;

println!("Best nprobe: {}", result.best_value);
println!("Recall@10: {:.4}", result.recall);
println!("Latency: {:.2}ms", result.latency_ms);
println!("Criterion met: {}", result.criterion_met);

// Use optimal parameter
use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
let params = IVFPQParams {
    num_clusters: 1024,
    nprobe: result.best_value,  // Use tuned value
    num_codebooks: 8,
    codebook_size: 256,
};
let mut index = IVFPQIndex::new(128, params)?;
```

### Tuning HNSW ef_search

```rust
#[cfg(feature = "hnsw")]
{
    let tuner = ParameterTuner::new()
        .criterion(Criterion::Balanced {
            k: 10,
            recall_weight: 0.7,
            latency_weight: 0.3,
        })
        .num_test_queries(100);

    let result = tuner.tune_hnsw_ef_search(
        &dataset,
        128,      // dimension
        32,       // m parameter
        &[10, 20, 50, 100, 200],  // ef_search values to try
    )?;

    // Use optimal ef_search in search calls
    let results = index.search(&query, 10, result.best_value)?;
}
```

### Understanding Results

```rust
let result = tuner.tune_ivf_pq_nprobe(...)?;

// Best parameter found
println!("Best nprobe: {}", result.best_value);

// Performance with best parameter
println!("Recall: {:.4}", result.recall);
println!("Latency: {:.2}ms", result.latency_ms);

// Whether criterion was met
println!("Criterion met: {}", result.criterion_met);

// All parameter values tried (for analysis)
for (nprobe, recall, latency, score) in &result.all_results {
    println!("nprobe={}: recall={:.4}, latency={:.2}ms, score={:.4}",
             nprobe, recall, latency, score);
}
```

## Best Practices

### Choosing Index Type

1. **HNSW**: General-purpose, high recall, fast search
   - Use for: Most applications, when memory is available
   - Factory: `"HNSW32"` or `"HNSW16,32"`

2. **NSW**: Lower memory, comparable performance to HNSW
   - Use for: High-dimensional data (d > 32), memory-constrained
   - Factory: `"NSW32"`

3. **IVF-PQ**: Memory-efficient, billion-scale capable
   - Use for: Very large datasets, memory-constrained
   - Factory: `"IVF1024,PQ8"` or `"IVF4096,PQ16"`

4. **SCANN**: Optimized for MIPS (Maximum Inner Product Search)
   - Use for: Inner product similarity, large datasets
   - Factory: `"SCANN256"`

### Parameter Selection

**Before Auto-Tuning:**
- Start with defaults from factory
- Use recommended values from documentation
- Consider your dataset size and requirements

**With Auto-Tuning:**
- Use representative dataset (subset of production data)
- Set realistic criteria (don't aim for 100% recall)
- Use time budget for practical tuning
- Review all results, not just best value

**Parameter Ranges:**
- **nprobe**: Start with `[1, 2, 4, 8, 16, 32, 64]` for IVF-PQ
- **ef_search**: Start with `[10, 20, 50, 100, 200]` for HNSW
- Adjust based on num_clusters (nprobe) or dataset size (ef_search)

### Performance Optimization

1. **Pre-compute Ground Truth**: Auto-tune does this automatically
2. **Limit Test Queries**: Use `num_test_queries()` for faster tuning
3. **Time Budget**: Set realistic limits to prevent long runs
4. **Parallel Tuning**: Run multiple tunings in parallel for different parameters

### Error Handling

Always handle errors gracefully:

```rust
match index_factory(128, "HNSW32") {
    Ok(mut index) => {
        // Use index
    }
    Err(e) => {
        eprintln!("Failed to create index: {}", e);
        // Fallback or error handling
    }
}
```

## Examples

### Complete Workflow

```rust
use rank_retrieve::dense::ann::factory::index_factory;
use rank_retrieve::dense::ann::autotune::{ParameterTuner, Criterion};
use rank_retrieve::benchmark::datasets::create_benchmark_dataset;

// 1. Create dataset
let dataset = create_benchmark_dataset(10000, 1000, 128, 42);

// 2. Auto-tune parameters
let tuner = ParameterTuner::new()
    .criterion(Criterion::RecallAtK { k: 10, target: 0.95 })
    .num_test_queries(100);

let tuning_result = tuner.tune_ivf_pq_nprobe(
    &dataset,
    128,
    1024,
    &[1, 2, 4, 8, 16, 32, 64],
)?;

// 3. Create index with tuned parameters
// (Note: Factory doesn't support custom nprobe yet, use direct creation)
use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
let params = IVFPQParams {
    num_clusters: 1024,
    nprobe: tuning_result.best_value,
    num_codebooks: 8,
    codebook_size: 256,
};
let mut index = IVFPQIndex::new(128, params)?;

// 4. Add vectors and build
for (i, vec) in dataset.train.iter().enumerate() {
    index.add(i as u32, vec.clone())?;
}
index.build()?;

// 5. Search
let results = index.search(&dataset.test[0], 10)?;
```

### Comparing Index Types

```rust
let dimension = 128;
let vectors = generate_vectors(1000, dimension);

// Try different index types
let index_types = vec!["HNSW32", "NSW32", "IVF64,PQ8", "SCANN64"];

for index_type in index_types {
    let mut index = index_factory(dimension, index_type)?;
    
    // Add vectors
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    index.build()?;
    
    // Measure performance
    let start = std::time::Instant::now();
    let _results = index.search(&vectors[0], 10)?;
    let elapsed = start.elapsed();
    
    println!("{}: {:.2}ms, {} bytes", 
             index_type, elapsed.as_secs_f32() * 1000.0, index.size_bytes());
}
```

## Troubleshooting

### Common Issues

1. **"Feature not enabled"**
   - Solution: Add feature to `Cargo.toml`: `rank-retrieve = { features = ["hnsw"] }`

2. **"Dimension must be divisible by num_codebooks"**
   - Solution: Choose dimension divisible by codebooks, or adjust codebooks
   - Example: For dimension 100, use `PQ4` (100 % 4 = 0) instead of `PQ8`

3. **"nprobe cannot exceed num_clusters"**
   - Solution: Ensure all nprobe values ≤ num_clusters
   - Example: For num_clusters=64, use `&[1, 2, 4, 8, 16, 32, 64]`

4. **Auto-tune takes too long**
   - Solution: Reduce `num_test_queries()` or set `time_budget()`
   - Example: `.num_test_queries(50).time_budget(Duration::from_secs(30))`

5. **Low recall in auto-tune results**
   - Solution: Try larger parameter values or different index type
   - Example: Increase nprobe range or try HNSW instead of IVF-PQ

### Performance Tips

1. **Factory Overhead**: Factory has negligible overhead (< 1μs parsing)
2. **Auto-Tune Speed**: Pre-computes ground truth once, reuses for all parameters
3. **Memory**: Factory-created indexes use same memory as direct creation
4. **Parallelization**: Can run multiple auto-tunes in parallel threads

## References

- [Faiss Comparison](./FAISS_COMPARISON.md) - When to use rank-retrieve vs Faiss
- [Faiss Learnings](./FAISS_LEARNINGS_VALIDATION.md) - Research and validation
- [Review Summary](./REVIEW_VALIDATION_SUMMARY.md) - Complete review findings
- [ANN Benchmark Standards](./ANN_BENCHMARK_STANDARDS.md) - Evaluation methodology
