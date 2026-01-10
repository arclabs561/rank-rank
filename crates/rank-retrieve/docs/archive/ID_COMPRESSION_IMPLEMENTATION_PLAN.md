# ID Compression Implementation Plan

Implementation plan for lossless compression of vector IDs in ANN indexes, based on "Lossless Compression of Vector IDs for Approximate Nearest Neighbor Search" (Severo et al., 2025).

## Overview

This document outlines a concrete plan to integrate order-invariant lossless compression for:
- **IVF-PQ clusters**: Compress `vector_indices: Vec<u32>` per cluster using ROC (Random Order Coding)
- **HNSW/NSW neighbor lists**: Compress `neighbors: SmallVec<[u32; 16]>` per node using ROC/REC
- **Persistence layer**: Compress IDs during serialization for disk storage

## Current State Analysis

### Data Structures

**IVF-PQ** (`src/dense/ivf_pq/search.rs`):
```rust
struct Cluster {
    vector_indices: Vec<u32>,  // Uncompressed, order-invariant
}
```

**HNSW** (`src/dense/hnsw/graph.rs`):
```rust
struct Layer {
    neighbors: Vec<SmallVec<[u32; 16]>>,  // Uncompressed, order-invariant per node
}
```

**NSW/SNG/DiskANN**: Similar neighbor list structures.

### Current Compression

- **Persistence**: Delta encoding + bitpacking (Tantivy-style) in `src/persistence/codec.rs`
- **In-memory**: No compression (raw `Vec<u32>`)
- **Limitation**: Current compression doesn't exploit ordering invariance

## Compression Techniques

### 1. Random Order Coding (ROC)

**Use case**: Sets/multisets of IDs where order doesn't matter (IVF clusters, neighbor lists).

**Theory**: 
- A set of `n` elements from universe `[N]` has `C(N, n)` possible sets
- A sequence has `N!/(N-n)!` possible sequences
- Savings: `log(N!/(N-n)!) - log(C(N, n)) = log(n!)` bits ≈ `n log n` bits

**Implementation**: Bits-back coding with ANS, treating permutation as latent variable.

**Rust libraries**:
- `constriction` crate: Provides ANS coder with bits-back support
- Alternative: `rans` crate (simpler, but may need custom bits-back implementation)

### 2. Random Edge Coding (REC)

**Use case**: Entire graph compression (offline setting).

**Theory**: Compress all edges into single ANS state, exploiting edge ordering invariance.

**Implementation**: Extend ROC to graph structures, compress edge sequences.

### 3. Wavelet Trees

**Use case**: Full random access to compressed IDs (when needed).

**Theory**: Index sequence of cluster IDs, enable `select(k, offset)` operations.

**Implementation**: Use existing wavelet tree library or implement basic version.

## Architecture Design

### Phase 1: Core Compression Library

Create `src/compression/` module with:

```
src/compression/
├── mod.rs              # Public API
├── ans.rs              # ANS encoder/decoder wrapper
├── roc.rs              # Random Order Coding for sets
├── rec.rs              # Random Edge Coding for graphs
├── wavelet.rs          # Wavelet tree (optional, Phase 2)
└── traits.rs           # Compression trait definitions
```

**API Design**:

```rust
// Core trait for compressible ID collections
pub trait IdCompressor {
    type Compressed;
    type Decompressed: IntoIterator<Item = u32>;
    
    fn compress(&self, ids: &[u32]) -> Result<Self::Compressed, CompressionError>;
    fn decompress(&self, compressed: &Self::Compressed) -> Result<Self::Decompressed, CompressionError>;
    fn compressed_size(&self, ids: &[u32]) -> usize;  // Estimate without full compression
}

// ROC implementation
pub struct RocCompressor {
    // ANS state, probability model
}

impl IdCompressor for RocCompressor {
    // Compress set of IDs using bits-back coding
}

// Elias-Fano baseline (for comparison)
pub struct EliasFanoCompressor;

impl IdCompressor for EliasFanoCompressor {
    // Delta-encoded monotone sequence
}
```

### Phase 2: Integration with Index Structures

#### 2.1 IVF-PQ Integration

**Location**: `src/dense/ivf_pq/compression.rs`

**Changes**:
1. Add compression option to `IVFPQParams`:
```rust
pub struct IVFPQParams {
    // ... existing fields ...
    pub id_compression: Option<IdCompressionMethod>,
}

pub enum IdCompressionMethod {
    None,           // Uncompressed (current)
    EliasFano,      // Baseline
    Roc,            // Random Order Coding
    WaveletTree,    // Full random access
}
```

2. Modify `Cluster` to support compressed storage:
```rust
enum ClusterStorage {
    Uncompressed(Vec<u32>),
    Compressed(CompressedCluster),
}

struct CompressedCluster {
    data: Vec<u8>,           // Compressed bitstream
    compressor: CompressorId, // Which compressor was used
    num_ids: usize,          // For decompression
}
```

3. Update search to decompress on-demand:
```rust
impl IVFPQIndex {
    fn get_cluster_ids(&self, cluster_idx: usize) -> Vec<u32> {
        match &self.clusters[cluster_idx].storage {
            ClusterStorage::Uncompressed(ids) => ids.clone(),
            ClusterStorage::Compressed(compressed) => {
                self.decompress_cluster(compressed)
            }
        }
    }
}
```

**Performance consideration**: Cache decompressed clusters during search to avoid repeated decompression.

#### 2.2 HNSW Integration

**Location**: `src/dense/hnsw/compression.rs`

**Changes**:
1. Add compression to `HNSWParams`:
```rust
pub struct HNSWParams {
    // ... existing fields ...
    pub compress_neighbors: bool,  // Enable neighbor list compression
    pub compression_method: IdCompressionMethod,
}
```

2. Modify `Layer` to support compressed neighbor lists:
```rust
enum NeighborStorage {
    Uncompressed(Vec<SmallVec<[u32; 16]>>),
    Compressed(Vec<CompressedNeighbors>),
}

struct CompressedNeighbors {
    data: Vec<u8>,
    num_neighbors: usize,
}
```

3. Decompress neighbors during graph traversal:
```rust
impl HNSWIndex {
    fn get_neighbors(&self, layer: usize, node: u32) -> SmallVec<[u32; 16]> {
        match &self.layers[layer].storage {
            NeighborStorage::Uncompressed(neighbors) => {
                neighbors[node as usize].clone()
            }
            NeighborStorage::Compressed(compressed) => {
                self.decompress_neighbors(&compressed[node as usize])
            }
        }
    }
}
```

**Performance consideration**: For small neighbor lists (m=16), compression overhead may exceed benefits. Add threshold check.

### Phase 3: Persistence Integration

**Location**: `src/persistence/compression.rs`

**Changes**:
1. Add compression option to persistence format:
```rust
pub struct PersistenceConfig {
    // ... existing fields ...
    pub compress_ids: bool,
    pub id_compression_method: IdCompressionMethod,
}
```

2. Compress during serialization:
```rust
impl DenseSegmentWriter {
    fn write_ivf_clusters(&mut self, clusters: &[Cluster]) -> Result<()> {
        if self.config.compress_ids {
            for cluster in clusters {
                let compressed = self.compressor.compress(&cluster.vector_indices)?;
                self.write_compressed_cluster(compressed)?;
            }
        } else {
            // Current delta+bitpacking
            self.write_clusters_uncompressed(clusters)?;
        }
        Ok(())
    }
}
```

3. Decompress during deserialization:
```rust
impl DenseSegmentReader {
    fn read_ivf_clusters(&mut self) -> Result<Vec<Cluster>> {
        if self.config.compress_ids {
            // Read compressed, decompress on-demand
            self.read_compressed_clusters()
        } else {
            // Current format
            self.read_clusters_uncompressed()
        }
    }
}
```

## Implementation Phases

### Phase 1: Foundation (Week 1-2)

**Goal**: Core compression library with ROC implementation.

**Tasks**:
1. ✅ Research Rust ANS libraries (`constriction`, `rans`)
2. ✅ Design compression trait API
3. ✅ Implement basic ANS wrapper
4. ✅ Implement ROC compressor for sets
5. ✅ Unit tests with small examples
6. ✅ Benchmark compression ratio vs. baseline

**Deliverables**:
- `src/compression/roc.rs` with working ROC implementation
- Tests showing compression ratios matching paper (7x for large sets)
- Benchmarks showing compression/decompression speed

**Dependencies**:
- Add `constriction` or `rans` to `Cargo.toml`
- Or implement minimal ANS from paper

### Phase 2: IVF-PQ Integration (Week 3)

**Goal**: Compress IVF cluster IDs with minimal API changes.

**Tasks**:
1. Add `IdCompressionMethod` enum to `IVFPQParams`
2. Implement `CompressedCluster` storage type
3. Modify `Cluster` to support both compressed/uncompressed
4. Update `search()` to decompress clusters on-demand
5. Add caching to avoid repeated decompression
6. Integration tests with real datasets

**Deliverables**:
- IVF-PQ with optional ID compression
- Benchmarks showing memory reduction and search speed impact
- Documentation on when to enable compression

**Performance targets**:
- Compression ratio: 5-7x for large clusters (N_k > 1000)
- Search slowdown: < 20% (matching paper)
- Memory reduction: 30% for billion-scale (when IDs are significant portion)

### Phase 3: HNSW Integration (Week 4)

**Goal**: Compress HNSW neighbor lists.

**Tasks**:
1. Add compression option to `HNSWParams`
2. Implement compressed neighbor storage
3. Update graph traversal to decompress on-demand
4. Add threshold check (skip compression for small lists)
5. Integration tests

**Deliverables**:
- HNSW with optional neighbor compression
- Benchmarks showing trade-offs
- Documentation on optimal `m` values for compression

**Performance targets**:
- Compression ratio: 1.5-2x for m=32-256 (paper shows limited benefit for m=16)
- Search slowdown: < 30% (acceptable for memory-constrained scenarios)

### Phase 4: Persistence (Week 5)

**Goal**: Compress IDs during disk serialization.

**Tasks**:
1. Add compression config to persistence format
2. Implement compressed serialization for IVF
3. Implement compressed serialization for HNSW
4. Update deserialization to handle compressed format
5. Version migration support

**Deliverables**:
- Persistence format with compressed IDs
- Migration tools for existing indexes
- Documentation on format compatibility

### Phase 5: Optimization & Polish (Week 6)

**Goal**: Performance optimization and production readiness.

**Tasks**:
1. Profile and optimize hot paths
2. Add SIMD optimizations where applicable
3. Implement caching strategies
4. Add comprehensive benchmarks
5. Documentation and examples
6. Consider wavelet trees for full random access (if needed)

**Deliverables**:
- Optimized implementation
- Comprehensive benchmark suite
- Production-ready API
- Full documentation

## Technical Decisions

### ANS Library Choice

**Option 1: `constriction` crate**
- Pros: High-level API, bits-back support, well-maintained
- Cons: Larger dependency, may be overkill for simple use case
- Decision: **Use `constriction`** for Phase 1, can switch to lighter implementation later

**Option 2: `rans` crate**
- Pros: Lightweight, simple API
- Cons: May need custom bits-back implementation
- Decision: Fallback if `constriction` doesn't work well

**Option 3: Custom ANS implementation**
- Pros: Full control, minimal dependencies
- Cons: Significant implementation effort, potential bugs
- Decision: Only if libraries don't meet needs

### Compression Thresholds

**IVF clusters**:
- Enable compression if `cluster_size > 100` (empirically determined)
- Small clusters: overhead exceeds benefits

**HNSW neighbors**:
- Enable compression if `m >= 32` (from paper: m=16 shows limited benefit)
- Small neighbor lists: skip compression

### Caching Strategy

**During search**:
- Cache decompressed clusters/neighbors for duration of search
- Use `HashMap<usize, Vec<u32>>` for decompressed cache
- Clear cache after search completes

**Memory management**:
- Limit cache size to prevent unbounded growth
- Use LRU eviction if needed

## API Design

### Public API

```rust
// Enable compression in IVF-PQ
let params = IVFPQParams {
    num_clusters: 1024,
    nprobe: 100,
    id_compression: Some(IdCompressionMethod::Roc),
    ..Default::default()
};

let mut index = IVFPQIndex::new(128, params)?;
// ... add vectors, build ...
// Compression happens automatically during build()

// Search works transparently (decompression on-demand)
let results = index.search(&query, 10)?;
```

### Feature Flags

```toml
[dependencies]
# Compression support (optional)
constriction = { version = "0.3", optional = true }

[features]
default = []
id-compression = ["constriction"]  # Enable ID compression
```

## Testing Strategy

### Unit Tests

1. **Compression correctness**:
   - Compress/decompress round-trip
   - Verify all IDs preserved
   - Test edge cases (empty set, single element, large sets)

2. **Compression ratio**:
   - Verify ratios match paper (7x for large sets)
   - Test with various cluster sizes

3. **Performance**:
   - Benchmark compression speed
   - Benchmark decompression speed
   - Compare with uncompressed baseline

### Integration Tests

1. **IVF-PQ with compression**:
   - Build index with compression enabled
   - Verify search results identical to uncompressed
   - Measure memory usage reduction
   - Measure search speed impact

2. **HNSW with compression**:
   - Build graph with compressed neighbors
   - Verify search results identical
   - Measure memory and speed trade-offs

3. **Persistence**:
   - Serialize compressed index
   - Deserialize and verify correctness
   - Test format version compatibility

### Benchmark Suite

Create `benches/id_compression.rs`:

```rust
#[bench]
fn bench_roc_compress_large_set(b: &mut Bencher) {
    let ids: Vec<u32> = (0..10000).collect();
    let compressor = RocCompressor::new();
    b.iter(|| compressor.compress(&ids));
}

#[bench]
fn bench_roc_decompress_large_set(b: &mut Bencher) {
    // Pre-compress, then benchmark decompression
}

#[bench]
fn bench_ivf_search_with_compression(b: &mut Bencher) {
    // Full search benchmark with compression enabled
}
```

## Success Metrics

### Compression Ratio

- **IVF clusters**: 5-7x compression for large clusters (N_k > 1000)
- **HNSW neighbors**: 1.5-2x for m >= 32
- **Overall index**: 20-30% size reduction for billion-scale

### Performance Impact

- **IVF search**: < 20% slowdown (acceptable trade-off)
- **HNSW search**: < 30% slowdown (acceptable for memory-constrained)
- **Build time**: < 10% increase (compression during build)

### Code Quality

- Zero unsafe code (use safe Rust)
- Comprehensive tests (> 90% coverage)
- Clear documentation
- Backward compatible API

## Risks & Mitigations

### Risk 1: ANS Library Complexity

**Risk**: `constriction` may be too complex or have performance issues.

**Mitigation**: 
- Start with simple prototype using `rans`
- Benchmark early to identify issues
- Can implement minimal ANS if needed

### Risk 2: Decompression Overhead

**Risk**: Decompression during search may be too slow.

**Mitigation**:
- Implement aggressive caching
- Profile and optimize hot paths
- Consider SIMD optimizations
- Make compression optional (default: off)

### Risk 3: Memory vs. Speed Trade-off

**Risk**: Compression saves memory but hurts speed too much.

**Mitigation**:
- Make compression opt-in (not default)
- Provide clear guidance on when to use
- Benchmark on real workloads
- Allow per-index configuration

## Future Enhancements

1. **Wavelet Trees**: Full random access compression (if needed)
2. **REC for graphs**: Offline graph compression
3. **Adaptive compression**: Choose method based on data characteristics
4. **Quantized code compression**: Extend to compress PQ codes (paper Section 5.2)
5. **Parallel compression**: Compress multiple clusters in parallel during build

## References

- Paper: "Lossless Compression of Vector IDs for Approximate Nearest Neighbor Search" (Severo et al., 2025)
- Meta implementation: https://github.com/facebookresearch/vector_db_id_compression
- ANS theory: Duda (2009), "Asymmetric numeral systems"
- Bits-back coding: Hinton & Van Camp (1993), "Keeping neural networks simple"
- ROC paper: Severo et al. (2022), "Compressing multisets with large alphabets"
- REC paper: Severo et al. (2023), "Random edge coding"
