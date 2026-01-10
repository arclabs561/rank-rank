# Comprehensive ANN Algorithms Implementation Plan

Pure Rust implementations of state-of-the-art approximate nearest neighbor search algorithms, optimized with SIMD and minimal dependencies.

## Algorithms to Implement

### 1. HNSW (Hierarchical Navigable Small World)
**Status**: Foundation complete, implementing full algorithm
**Paper**: Malkov & Yashunin (2016)
**Best for**: General-purpose, high recall, fast search
**Complexity**: O(log n) search, O(n log n) construction

### 2. Anisotropic Vector Quantization with k-means Partitioning (vendor: SCANN)
**Status**: To implement
**Papers**: 
- Guo et al. (2020): "Accelerating Large-Scale Inference with Anisotropic Vector Quantization"
- Sun et al. (2023): "SOAR: Improved Indexing for Approximate Nearest Neighbor Search"
**Best for**: Maximum Inner Product Search (MIPS), very large datasets
**Key components**:
- **Partitioning**: k-means clustering to divide dataset
- **Anisotropic Quantization**: Preserves parallel components for accurate inner products
- **Re-ranking**: Fine-tune top-k results with exact distances
**Complexity**: O(n/k) search (k = partitions), O(n log n) construction

### 3. IVF-PQ (Inverted File Index with Product Quantization)
**Status**: To implement
**Paper**: Jégou et al. (2011): "Product Quantization for Nearest Neighbor Search"
**Best for**: Memory-efficient, billion-scale datasets
**Key components**:
- **IVF**: Inverted file index (clustering-based)
- **PQ**: Product quantization (vector compression)
**Complexity**: O(n/k + k) search, O(n log n) construction

### 4. DiskANN
**Status**: To implement
**Paper**: Jayaram Subramanya et al. (2019): "DiskANN: Fast Accurate Billion-point Nearest Neighbor Search on a Single Node"
**Best for**: Very large datasets that don't fit in memory
**Key components**:
- **Graph-based index**: Similar to HNSW but disk-optimized
- **Cached working set**: Keep hot vectors in memory
- **Disk I/O optimization**: Sequential access patterns
**Complexity**: O(log n) search with disk I/O

### 5. NSG (Navigating Spreading-out Graph)
**Status**: To implement (optional)
**Paper**: Fu et al. (2019): "Fast Approximate Nearest Neighbor Search with the Navigating Spreading-out Graph"
**Best for**: High-dimensional spaces, good graph connectivity
**Complexity**: O(log n) search

## Unified Architecture

### Module Structure

```
crates/rank-retrieve/src/dense/ann/
├── mod.rs              # Unified ANN API
├── traits.rs           # Common traits for all ANN algorithms
├── hnsw/               # HNSW implementation
│   ├── mod.rs
│   ├── graph.rs
│   ├── search.rs
│   ├── construction.rs
│   └── distance.rs
├── scann/              # Anisotropic VQ + k-means (SCANN) implementation
│   ├── mod.rs
│   ├── partitioning.rs # k-means clustering
│   ├── quantization.rs # Anisotropic vector quantization
│   ├── reranking.rs    # Re-ranking stage
│   └── search.rs
├── ivf_pq/             # IVF-PQ implementation
│   ├── mod.rs
│   ├── ivf.rs          # Inverted file index
│   ├── pq.rs           # Product quantization
│   └── search.rs
├── diskann/            # DiskANN implementation
│   ├── mod.rs
│   ├── graph.rs
│   ├── disk_io.rs      # Disk I/O optimization
│   └── cache.rs        # Working set cache
└── shared/             # Shared utilities
    ├── clustering.rs   # k-means (used by Anisotropic VQ + k-means (SCANN), IVF)
    ├── quantization.rs # Product quantization (used by IVF-PQ, Anisotropic VQ + k-means (SCANN))
    └── distance.rs     # Distance computation (SIMD-accelerated)
```

### Unified API

```rust
pub trait ANNIndex {
    /// Add vector to index
    fn add(&mut self, doc_id: u32, vector: Vec<f32>) -> Result<(), RetrieveError>;
    
    /// Build index (required before search)
    fn build(&mut self) -> Result<(), RetrieveError>;
    
    /// Search for k nearest neighbors
    fn search(
        &self,
        query: &[f32],
        k: usize,
    ) -> Result<Vec<(u32, f32)>, RetrieveError>;
    
    /// Get index size in bytes
    fn size_bytes(&self) -> usize;
    
    /// Get index statistics
    fn stats(&self) -> ANNStats;
}

pub enum ANNAlgorithm {
    HNSW(HNSWParams),
    SCANN(SCANNParams),
    IVFPQ(IVFPQParams),
    DiskANN(DiskANNParams),
}
```

## Implementation Strategy

### Phase 1: Complete HNSW (Current Priority)
1. ✅ Graph structure
2. ✅ SIMD distance computation
3. ⏳ Graph construction algorithm
4. ⏳ Multi-layer search
5. ⏳ Testing & benchmarking

### Phase 2: Shared Infrastructure
1. **k-means clustering** (used by Anisotropic VQ + k-means (SCANN), IVF)
   - SIMD-accelerated distance computation
   - Efficient initialization (k-means++)
   - Iterative refinement

2. **Product Quantization** (used by IVF-PQ, Anisotropic VQ + k-means (SCANN))
   - Vector decomposition into subvectors
   - Codebook generation
   - Fast distance computation via lookup tables

3. **Anisotropic Quantization** (Anisotropic VQ + k-means (SCANN)-specific)
   - Preserve parallel components
   - Optimize for inner product accuracy

### Phase 3: Anisotropic VQ + k-means (SCANN) Implementation
1. Partitioning stage (k-means)
2. Quantization stage (anisotropic PQ)
3. Re-ranking stage
4. Integration & optimization

### Phase 4: IVF-PQ Implementation
1. IVF index construction
2. PQ codebook training
3. Search algorithm
4. Memory optimization

### Phase 5: DiskANN Implementation
1. Graph construction (similar to HNSW)
2. Disk I/O layer
3. Working set cache
4. Sequential access optimization

## Optimization Principles

### 1. SIMD Acceleration
- Use existing `simd` module for all distance computations
- Batch operations where possible
- AVX-512/AVX2/NEON support

### 2. Memory Layout
- Structure of Arrays (SoA) for vectors
- Cache-friendly data structures
- Compact neighbor lists (SmallVec)

### 3. Minimal Dependencies
- Only essential crates: `smallvec`, `rand`
- No FFI dependencies
- Pure Rust implementation

### 4. Feature Gating
- Each algorithm as separate feature
- Shared utilities always available
- Users enable only what they need

## Performance Targets

| Algorithm | Search Time (1M vectors) | Memory Overhead | Best For |
|-----------|---------------------------|-----------------|----------|
| HNSW      | <1ms (k=10, ef=50)        | ~2x             | General purpose |
| Anisotropic VQ + k-means (SCANN) | <2ms (k=10)               | ~1.5x           | MIPS, large datasets |
| IVF-PQ    | <5ms (k=10, nprobe=100)    | ~0.1x           | Billion-scale, memory-constrained |
| DiskANN   | <10ms (k=10, with disk I/O)| ~0.01x          | Very large, disk-based |

## Dependencies

```toml
[dependencies]
# HNSW
smallvec = { version = "1.11", optional = true }
rand = { version = "0.8", optional = true }  # Already exists

# No additional dependencies needed - all algorithms use existing SIMD infrastructure
```

## Feature Flags

```toml
[features]
default = []
dense = []  # Basic brute-force (existing)
hnsw = ["dense", "dep:smallvec", "dep:rand"]
scann = ["dense", "dep:rand"]  # Anisotropic VQ + k-means (vendor: SCANN), uses shared clustering
ivf_pq = ["dense", "dep:rand"]  # Uses shared clustering, PQ
diskann = ["dense", "dep:smallvec", "dep:rand"]
ann_all = ["hnsw", "scann", "ivf_pq", "diskann"]
```

## Next Steps

1. **Complete HNSW** (immediate)
   - Implement graph construction
   - Implement multi-layer search
   - Add tests

2. **Shared Infrastructure** (next)
   - k-means clustering
   - Product quantization
   - Anisotropic quantization

3. **Anisotropic VQ + k-means (SCANN)** (after shared infra)
   - Partitioning
   - Quantization
   - Re-ranking

4. **IVF-PQ** (after Anisotropic VQ + k-means (SCANN))
   - IVF index
   - PQ integration
   - Search optimization

5. **DiskANN** (final)
   - Graph construction
   - Disk I/O layer
   - Cache management
