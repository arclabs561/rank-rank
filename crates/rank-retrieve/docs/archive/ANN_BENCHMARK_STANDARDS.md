# ANN Benchmark Standards & Best Practices

This document outlines standard benchmarking practices from well-known ANN benchmark suites, particularly **ann-benchmarks** (erikbern/ann-benchmarks, 5.5k+ stars), and how we apply them to `rank-retrieve`.

## Well-Known ANN Benchmark Suites

### 1. ann-benchmarks (Erik Bernhardsson)

**Repository**: https://github.com/erikbern/ann-benchmarks  
**Stars**: 5,568+  
**Status**: Active, widely-used standard

**Key Features**:
- Standardized evaluation framework
- Multiple datasets (SIFT, GloVe, MNIST, etc.)
- Recall@K vs Query Time plots
- Docker-based isolation
- Results published at ann-benchmarks.com

**Metrics**:
- **Recall@K**: Percentage of true nearest neighbors found
- **Query Time (QPS)**: Queries per second
- **Index Build Time**: Time to construct index
- **Index Size**: Memory footprint

**Datasets**:
- **SIFT**: 1M vectors, 128 dimensions
- **GloVe**: 1.2M vectors, 100 dimensions
- **MNIST**: 60k vectors, 784 dimensions
- **Random**: Synthetic datasets of various sizes
- **NYTimes**: 290k vectors, 256 dimensions

**Structure**:
```python
class BaseANN:
    def fit(self, X):  # Build index
    def query(self, v, k):  # Search for k nearest
    def get_memory_usage(self):  # Memory footprint
```

### 2. Big ANN Benchmarks

**Focus**: Billion-scale datasets  
**Tracks**: 
- Recall vs latency
- Throughput
- Memory efficiency
- Power consumption

### 3. BEAR (if applicable)

**Note**: BEAR may refer to different benchmarks. Research needed.

## Standard Metrics

### Core Metrics (from ann-benchmarks)

1. **Recall@K**
   - Definition: `|retrieved ∩ ground_truth| / |ground_truth|`
   - Standard K values: 1, 10, 100
   - Critical for accuracy evaluation

2. **Query Time (Latency)**
   - Measured in milliseconds per query
   - Often reported as QPS (queries per second)
   - Should measure p50, p95, p99 percentiles

3. **Index Build Time**
   - Time to construct index from vectors
   - Important for production deployment

4. **Index Size (Memory)**
   - Memory footprint of index
   - Often compared to raw vector storage

5. **Throughput**
   - Queries per second under load
   - Batch query performance

### Additional Metrics

6. **Index Quality**
   - Graph connectivity (for graph-based methods)
   - Quantization error (for quantization methods)

7. **Scalability**
   - Performance vs dataset size
   - Memory growth rate

## Standard Datasets

### From ann-benchmarks

| Dataset | Vectors | Dimensions | Distance | Use Case |
|---------|---------|------------|----------|----------|
| **SIFT-1M** | 1M | 128 | L2 | Image descriptors |
| **GloVe-100** | 1.2M | 100 | Cosine | Word embeddings |
| **MNIST** | 60k | 784 | L2 | Image pixels |
| **NYTimes** | 290k | 256 | Cosine | Text embeddings |
| **Random** | Variable | Variable | L2/Cosine | Synthetic testing |

### Standard Dataset Sizes

- **Small**: 1K-10K vectors (testing)
- **Medium**: 100K-1M vectors (standard benchmarks)
- **Large**: 10M+ vectors (scalability)
- **Billion-scale**: 1B+ vectors (Big ANN Benchmarks)

## Benchmark Structure (from ann-benchmarks)

### Algorithm Interface

```python
class BaseANN:
    def __init__(self, metric, **kwargs):
        self.metric = metric  # 'euclidean' or 'angular'
    
    def fit(self, X):
        """Build index from vectors."""
        pass
    
    def query(self, v, k):
        """Return k nearest neighbors."""
        pass
    
    def get_memory_usage(self):
        """Return memory usage in bytes."""
        pass
    
    def __str__(self):
        """String representation for results."""
        pass
```

### Evaluation Loop

```python
def evaluate_algorithm(algorithm, dataset, k_values=[1, 10, 100]):
    # Build index
    start = time.time()
    algorithm.fit(dataset.train)
    build_time = time.time() - start
    
    # Evaluate queries
    results = []
    for k in k_values:
        recalls = []
        query_times = []
        
        for query in dataset.test:
            start = time.time()
            neighbors = algorithm.query(query, k)
            query_time = time.time() - start
            
            # Compute recall
            ground_truth = dataset.get_neighbors(query, k)
            recall = len(set(neighbors) & set(ground_truth)) / len(ground_truth)
            
            recalls.append(recall)
            query_times.append(query_time)
        
        results.append({
            'k': k,
            'recall_mean': np.mean(recalls),
            'recall_std': np.std(recalls),
            'query_time_mean': np.mean(query_times),
            'query_time_p95': np.percentile(query_times, 95),
        })
    
    return {
        'build_time': build_time,
        'memory_usage': algorithm.get_memory_usage(),
        'results': results,
    }
```

### Visualization

**Standard Plot**: Recall@K vs Query Time
- X-axis: Query time (ms) or QPS
- Y-axis: Recall@K
- Multiple curves for different K values
- Multiple algorithms on same plot

## Our Implementation Strategy

### 1. Adopt ann-benchmarks Structure

```rust
pub trait ANNBenchmark {
    fn fit(&mut self, vectors: &[f32], num_vectors: usize) -> Result<(), RetrieveError>;
    fn query(&self, query: &[f32], k: usize) -> Result<Vec<u32>, RetrieveError>;
    fn get_memory_usage(&self) -> usize;
    fn get_build_time(&self) -> Duration;
}
```

### 2. Standard Datasets

Create dataset loaders for:
- SIFT-1M (or generate synthetic equivalent)
- GloVe-100 (or synthetic word embeddings)
- Random datasets (various sizes/dimensions)

### 3. Standard Metrics

Implement:
- `recall_at_k(ground_truth, retrieved, k) -> f32`
- `query_time_percentiles(times) -> (p50, p95, p99)`
- `throughput(queries, time) -> f32` (QPS)

### 4. Benchmark Harness

```rust
pub struct BenchmarkRunner {
    datasets: Vec<Dataset>,
    algorithms: Vec<Box<dyn ANNBenchmark>>,
    k_values: Vec<usize>,
}

impl BenchmarkRunner {
    pub fn run(&self) -> BenchmarkResults {
        // For each dataset
        //   For each algorithm
        //     Build index, measure time
        //     For each query
        //       Measure query time
        //       Compute recall@k
        //   Generate plots
    }
}
```

### 5. Results Format

```rust
pub struct BenchmarkResult {
    pub algorithm: String,
    pub dataset: String,
    pub k: usize,
    pub recall_mean: f32,
    pub recall_std: f32,
    pub query_time_mean: f32,  // ms
    pub query_time_p50: f32,
    pub query_time_p95: f32,
    pub query_time_p99: f32,
    pub build_time: f32,  // seconds
    pub memory_usage: usize,  // bytes
    pub throughput: f32,  // QPS
}
```

## Key Learnings from ann-benchmarks

### 1. Standardized Interface

All algorithms implement the same interface, making comparison fair and easy.

### 2. Multiple K Values

Evaluate at multiple K values (1, 10, 100) to understand behavior across use cases.

### 3. Percentile Reporting

Report p50, p95, p99 query times, not just mean (tail latency matters).

### 4. Ground Truth

Always compute ground truth using exact search for fair recall evaluation.

### 5. Isolation

Use Docker/containers to ensure fair comparison (same environment).

### 6. Visualization

Standard plots (Recall@K vs Query Time) make results easy to compare.

### 7. Reproducibility

Fixed random seeds, documented parameters, versioned datasets.

## Implementation Plan

### Phase 1: Core Infrastructure

1. ✅ Create `ANNBenchmark` trait (similar to `ANNIndex`)
2. ✅ Implement standard metrics (recall, query time, memory)
3. ⏳ Create dataset loaders (SIFT, GloVe, Random)
4. ⏳ Implement benchmark harness

### Phase 2: Standard Datasets

1. ⏳ SIFT-1M equivalent (synthetic or real)
2. ⏳ GloVe-100 equivalent
3. ⏳ Random datasets (various sizes)
4. ⏳ Ground truth computation (exact search)

### Phase 3: Comprehensive Benchmarks

1. ⏳ All algorithms on all datasets
2. ⏳ Multiple K values (1, 10, 100)
3. ⏳ Percentile reporting (p50, p95, p99)
4. ⏳ Memory usage tracking

### Phase 4: Visualization & Reporting

1. ⏳ Recall@K vs Query Time plots
2. ⏳ HTML report generation
3. ⏳ Comparison tables
4. ⏳ Performance regression detection

## References

- **ann-benchmarks**: https://github.com/erikbern/ann-benchmarks
- **ann-benchmarks.com**: https://ann-benchmarks.com/
- **Big ANN Benchmarks**: https://big-ann-benchmarks.com/
- **Erik Bernhardsson's Blog**: https://erikbern.com/2018/06/17/new-approximate-nearest-neighbor-benchmarks.html
