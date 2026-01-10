# Faiss vs rank-retrieve: When to Use Which

This document provides guidance on when to use `rank-retrieve` vs Faiss for approximate nearest neighbor (ANN) search.

## Overview

**Faiss** (Facebook AI Similarity Search) is a production-hardened C++ library with Python bindings, optimized for billion-scale (B-scale) datasets. **rank-retrieve** is a pure Rust implementation focused on zero-dependency, embeddable ANN algorithms with SIMD acceleration.

## Key Differences

| Aspect | Faiss | rank-retrieve |
|--------|-------|---------------|
| **Language** | C++ (Python bindings) | Pure Rust |
| **Dependencies** | C++ toolchain, BLAS | Zero dependencies (by default) |
| **GPU Support** | ✅ Full GPU support | ❌ CPU-only (SIMD-accelerated) |
| **Scale** | Billion+ vectors (B-scale) | Million-scale (optimized for <10M) |
| **Memory Efficiency** | Excellent (PQ, quantization) | Good (pure Rust implementations) |
| **Production Hardening** | ✅ Battle-tested | ⚠️ Research-focused |
| **API Complexity** | Moderate (C++/Python) | Simple (Rust trait-based) |
| **Ecosystem Integration** | Standalone | Integrated with rank-* crates |
| **Index Factory** | ✅ `index_factory()` | ✅ `index_factory()` (inspired by Faiss) |
| **Auto-tuning** | ✅ `ParameterSpace` | ✅ `ParameterTuner` (grid search) |

## When to Use rank-retrieve

### ✅ Use rank-retrieve when:

1. **Pure Rust ecosystem**
   - Building Rust-native RAG pipelines
   - No Python FFI overhead
   - Embedding in Rust applications

2. **Zero-dependency requirements**
   - Minimal binary size
   - No C++ toolchain needed
   - Easy cross-compilation

3. **Medium-scale datasets (<10M vectors)**
   - Million-scale is the sweet spot
   - All data fits in RAM
   - SIMD acceleration provides good performance

4. **Ecosystem integration**
   - Using other `rank-*` crates (rank-fusion, rank-rerank)
   - Unified API across retrieval methods
   - Hybrid search (BM25 + dense + sparse)

5. **Research and experimentation**
   - Easy to modify algorithms
   - Clear, documented implementations
   - Benchmarking and evaluation built-in

6. **Educational purposes**
   - Understanding ANN algorithms
   - Learning from clean Rust implementations
   - Algorithm research

### ❌ Don't use rank-retrieve when:

1. **Billion-scale datasets (B-scale)**
   - Faiss is optimized for 1B+ vectors (B-scale)
   - Better memory efficiency (PQ, quantization)
   - Production-hardened for extreme scale

2. **GPU acceleration needed**
   - Faiss has excellent GPU support
   - rank-retrieve is CPU-only (SIMD)

3. **Maximum performance at scale**
   - Faiss has years of optimization
   - Better for production workloads
   - More mature codebase

4. **Python ecosystem**
   - Faiss has excellent Python bindings
   - Better integration with NumPy, PyTorch
   - More Python examples/documentation

## Performance Comparison

### Expected Performance (Million-Scale)

| Algorithm | rank-retrieve | Faiss | Notes |
|-----------|---------------|-------|-------|
| **HNSW** | ~95% recall@10, ~5ms | ~98% recall@10, ~3ms | Faiss slightly faster (C++ optimizations) |
| **IVF-PQ** | ~90% recall@10, ~2ms | ~92% recall@10, ~1.5ms | Faiss has more PQ optimizations |
| **Brute-force** | SIMD-accelerated | BLAS-accelerated | Comparable for small datasets |

**Note**: These are rough estimates. Actual performance depends on:
- Dataset characteristics (dimension, distribution)
- Hardware (CPU, SIMD support)
- Parameters (nprobe, ef_search, etc.)

### Memory Usage

- **rank-retrieve**: Pure Rust, predictable memory usage
- **Faiss**: More aggressive compression (PQ, quantization), better for billion-scale (B-scale)

## Algorithm Coverage

### rank-retrieve Implementations

✅ **Implemented**:
- HNSW (Hierarchical Navigable Small World)
- NSW (Flat Navigable Small World)
- IVF-PQ (Inverted File Index with Product Quantization)
- SCANN (Anisotropic Vector Quantization with k-means)
- DiskANN (disk-based ANN)
- OPT-SNG (Optimized Sparse Neighborhood Graph)
- Classic methods (LSH, KD-Tree, Ball Tree, K-Means Tree, etc.)

### Faiss Implementations

✅ **Comprehensive**:
- All major ANN algorithms
- GPU-accelerated variants
- Advanced quantization (OPQ, PCA preprocessing)
- Billion-scale optimizations

## API Comparison

### Faiss (Python)

```python
import faiss

# Index factory
index = faiss.index_factory(128, "IVF1024,PQ8")
index.train(vectors)
index.add(vectors)

# Search
distances, indices = index.search(queries, k=10)

# Auto-tune
autotuner = faiss.ParameterSpace()
autotuner.initialize(index)
# ... tune parameters
```

### rank-retrieve (Rust)

```rust
use rank_retrieve::dense::ann::factory::index_factory;

// Index factory
let mut index = index_factory(128, "IVF1024,PQ8")?;
for (i, vec) in vectors.iter().enumerate() {
    index.add(i as u32, vec.clone())?;
}
index.build()?;

// Search
let results = index.search(&query, 10)?;

// Auto-tune
let tuner = ParameterTuner::new()
    .criterion(Criterion::RecallAtK { k: 10, target: 0.95 });
let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 1024, &[1, 2, 4, 8, 16, 32])?;
```

## Migration Guide

### From Faiss to rank-retrieve

1. **Replace index creation**:
   ```python
   # Faiss
   index = faiss.index_factory(d, "HNSW32")
   ```
   ```rust
   // rank-retrieve
   let mut index = index_factory(d, "HNSW32")?;
   ```

2. **Replace search**:
   ```python
   # Faiss
   D, I = index.search(queries, k)
   ```
   ```rust
   // rank-retrieve
   let results = index.search(&query, k)?;
   // Returns Vec<(u32, f32)> (doc_id, score)
   ```

3. **Replace auto-tune**:
   ```python
   # Faiss
   autotuner = faiss.ParameterSpace()
   ```
   ```rust
   // rank-retrieve
   let tuner = ParameterTuner::new()
       .criterion(Criterion::RecallAtK { k: 10, target: 0.95 });
   ```

### From rank-retrieve to Faiss

Use Faiss when:
- Dataset exceeds 10M vectors
- GPU acceleration needed
- Maximum performance required
- Python ecosystem preferred

## Recommendations

### For New Projects

1. **Start with rank-retrieve** if:
   - Building in Rust
   - Dataset < 10M vectors
   - Want zero dependencies
   - Need ecosystem integration

2. **Start with Faiss** if:
   - Dataset > 10M vectors
   - Need GPU acceleration
   - Python ecosystem
   - Maximum performance critical

### Hybrid Approach

You can use both:
- **rank-retrieve** for development, experimentation, medium-scale
- **Faiss** for production, billion-scale, GPU workloads

The `Backend` trait in `rank-retrieve` allows integrating Faiss as an external backend:

```rust
// Use rank-retrieve API with Faiss backend
impl Backend for FaissBackend {
    fn retrieve(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        // Delegate to Faiss
    }
}
```

## Conclusion

**rank-retrieve** is ideal for:
- Rust-native applications
- Medium-scale datasets (<10M)
- Zero-dependency requirements
- Ecosystem integration
- Research and education

**Faiss** is ideal for:
- Billion-scale datasets
- GPU acceleration
- Maximum performance
- Production workloads
- Python ecosystem

Both libraries learn from each other:
- rank-retrieve borrows patterns (index factory, auto-tune) from Faiss
- Faiss provides reference implementations for correctness validation
- Both contribute to ANN algorithm research

## References

- [Faiss Documentation](https://github.com/facebookresearch/faiss/wiki)
- [rank-retrieve Documentation](../README.md)
- [ANN Benchmark Standards](./ANN_BENCHMARK_STANDARDS.md)
- [Critical Perspectives](./CRITICAL_PERSPECTIVES_AND_LIMITATIONS.md)
