# Dense Retrieval Persistence Design for rank-retrieve

This document extends the main persistence design (`PERSISTENCE_DESIGN.md`) with specific details for dense vector retrieval, ANN indexes, and hybrid retrieval systems.

## Overview

Dense retrieval persistence must handle:
- **Vector storage**: Dense embeddings (typically 128-1536 dimensions, f32)
- **ANN indexes**: HNSW, IVF-PQ, DiskANN, and other graph/quantization structures
- **Hybrid systems**: Combined sparse + dense retrieval with unified persistence
- **Incremental updates**: Adding vectors without full rebuild
- **Memory vs. disk trade-offs**: Balancing query latency vs. storage cost

## Design Principles

### 1. Dual Storage Strategy

**Vector data on disk, indexes configurable:**
- Vector embeddings stored directly on disk (like Qdrant's `on_disk: true`)
- ANN indexes (HNSW graphs, IVF centroids) can be memory-mapped or fully in-memory
- Configuration allows independent control of vector storage vs. index storage

### 2. Format Compatibility

**Reuse segment architecture from sparse retrieval:**
- Same directory structure (`segments/segment_{id}/`)
- Same transaction log for coordination
- Separate segment types for dense vs. sparse (can coexist)

### 3. Incremental Indexing

**Support both batch and streaming:**
- Batch mode: Disable indexing during bulk load (`m=0` for HNSW), rebuild after
- Streaming mode: Incremental updates with periodic optimization
- Hybrid: Batch initial load, then streaming updates

## Vector Storage Format

### Segment Structure for Dense Retrieval

```
segment_{segmentID}/
├── vectors.bin              # Raw vector data (Structure of Arrays)
├── vector_metadata.bin       # Per-vector metadata (doc_id, norm, etc.)
├── [hnsw_index.bin]?        # HNSW graph (if HNSW enabled)
├── [ivf_index.bin]?         # IVF centroids + assignments (if IVF-PQ enabled)
├── [diskann_index/]?        # DiskANN graph + data (if DiskANN enabled)
├── docid_to_userid.fst      # FST mapping docID → userID
├── userid_to_docid.fst      # FST mapping userID → docID range
├── tombstones.bin           # Deleted document bitset
└── footer.bin               # Segment footer (offsets, metadata)
```

### Vector Data Format

**Structure of Arrays (SoA) Layout:**
```
vectors.bin:
[v0[0], v1[0], ..., vn[0], v0[1], v1[1], ..., vn[1], ..., v0[d-1], v1[d-1], ..., vn[d-1]]
```

**Rationale**: SoA layout enables:
- SIMD-friendly batch operations (process multiple vectors at once)
- Better cache locality for distance computations
- Efficient memory mapping (can map entire file)

**Alternative: Array of Structures (AoS)**
```
vectors.bin:
[v0[0..d], v1[0..d], ..., vn[0..d]]
```

**When to use AoS**: When accessing individual vectors frequently (less common in ANN search).

**Default: SoA** (better for batch operations, SIMD).

### Vector Metadata Format

```rust
#[repr(C)]
struct VectorMetadata {
    doc_id: u32,               // Document ID
    norm: f32,                 // L2 norm (for cosine similarity)
    flags: u8,                  // Bit flags (normalized, quantized, etc.)
    padding: [u8; 3],          // Padding to 8-byte alignment
}
```

**Metadata Array**: `[VectorMetadata; num_vectors]` stored contiguously.

### HNSW Index Format

**Graph Structure:**
```
hnsw_index.bin:
[header][layer_0][layer_1]...[layer_max][layer_assignments]
```

**Header:**
```rust
#[repr(C)]
struct HNSWHeader {
    magic: [u8; 4],              // b"HNSW"
    format_version: u32,
    dimension: u32,
    num_vectors: u32,
    num_layers: u8,
    m: u16,                      // Max connections per node
    m_max: u16,                  // Max connections for new nodes
    m_l: f64,                    // Layer assignment parameter
    ef_construction: u32,
    ef_search: u32,
}
```

**Layer Format:**
```rust
struct Layer {
    // For each vector: list of neighbor IDs (u32)
    // Stored as: [num_neighbors_0: u16][neighbor_0, neighbor_1, ...][num_neighbors_1: u16][...]
    neighbors: Vec<u8>,           // Variable-length encoding
}
```

**Layer Assignments:**
```rust
// Array of u8: max layer where each vector appears
// [layer_0, layer_1, ..., layer_n]
layer_assignments: Vec<u8>,
```

### IVF-PQ Index Format

**IVF-PQ Structure:**
```
ivf_index.bin:
[header][centroids][assignments][pq_codebooks][pq_codes]
```

**Header:**
```rust
#[repr(C)]
struct IVFHeader {
    magic: [u8; 4],              // b"IVFP"
    format_version: u32,
    dimension: u32,
    num_vectors: u32,
    num_clusters: u32,           // Number of Voronoi cells
    pq_m: u8,                    // Number of PQ subquantizers
    pq_bits: u8,                 // Bits per PQ code (typically 8)
}
```

**Centroids:**
```rust
// Array of cluster centroids: [c0[0..d], c1[0..d], ..., ck[0..d]]
centroids: Vec<f32>,             // num_clusters * dimension floats
```

**Assignments:**
```rust
// For each vector: cluster ID (u32)
assignments: Vec<u32>,
```

**PQ Codebooks:**
```rust
// For each subquantizer: codebook of 2^pq_bits centroids
// Layout: [codebook_0[0..d/m], codebook_1[0..d/m], ..., codebook_m[0..d/m]]
pq_codebooks: Vec<f32>,          // m * (2^pq_bits) * (d/m) floats
```

**PQ Codes:**
```rust
// For each vector: m PQ codes (u8 each)
// Layout: [v0_code_0, v0_code_1, ..., v0_code_m, v1_code_0, ...]
pq_codes: Vec<u8>,                // num_vectors * m bytes
```

### DiskANN Index Format

**DiskANN Structure:**
```
diskann_index/
├── graph.bin                   # Graph structure
├── data.bin                    # Vector data (compressed)
├── cache_metadata.bin          # Hot vector cache metadata
└── footer.bin                  # Index footer
```

**Graph Format:**
```rust
#[repr(C)]
struct DiskANNGraphHeader {
    magic: [u8; 4],              // b"DSKN"
    format_version: u32,
    dimension: u32,
    num_vectors: u32,
    max_degree: u16,
    // Graph stored as adjacency lists (similar to HNSW)
}
```

**Data Format:**
- Vectors stored with compression (quantization, delta encoding)
- Hot vectors cached in memory
- Cold vectors loaded on-demand from disk

## Hybrid Retrieval Persistence

### Unified Index Structure

**Combined Segment:**
```
segment_{segmentID}/
├── sparse/                     # Sparse retrieval data
│   ├── term_dict.fst
│   ├── postings.bin
│   └── ...
├── dense/                      # Dense retrieval data
│   ├── vectors.bin
│   ├── hnsw_index.bin
│   └── ...
├── unified_metadata.bin        # Shared metadata (doc_id mappings, etc.)
└── footer.bin                  # Combined footer
```

**Alternative: Separate Segments**
- Sparse segments: `segment_sparse_{id}/`
- Dense segments: `segment_dense_{id}/`
- Unified transaction log coordinates both

**Recommendation: Separate segments** (cleaner separation, independent optimization).

### Transaction Log Extensions

**New Entry Types:**
```rust
enum WalEntry {
    // ... existing entries ...
    
    AddDenseSegment {
        entry_id: u64,
        segment_id: u64,
        num_vectors: u32,
        dimension: u32,
        index_type: IndexType,  // HNSW, IVF-PQ, DiskANN, etc.
    },
    
    AddHybridSegment {
        entry_id: u64,
        segment_id: u64,
        sparse_doc_count: u32,
        dense_vector_count: u32,
    },
    
    UpdateVector {
        entry_id: u64,
        segment_id: u64,
        doc_id: u32,
        vector: Vec<f32>,
    },
}
```

## Incremental Updates

### HNSW Incremental Construction

**Strategy:**
1. **During bulk load**: Disable HNSW (`m=0`), store vectors only
2. **After bulk load**: Build HNSW graph in batches (`indexing_threshold=10000`)
3. **Streaming updates**: Insert into existing graph (may require periodic optimization)

**Optimization:**
- Periodically rebuild upper layers (expensive but improves quality)
- Use `ef_construction` parameter to balance build time vs. quality

### IVF-PQ Incremental Updates

**Strategy:**
1. **Initial build**: Compute centroids via k-means on sample
2. **Assign vectors**: Assign all vectors to nearest centroid
3. **Train PQ**: Train codebooks on assigned vectors
4. **Incremental**: Assign new vectors to existing centroids (no retraining)

**Limitation**: Centroids may become stale. Periodic retraining recommended.

### DiskANN Incremental Updates

**Strategy:**
1. **Initial build**: Build graph on disk
2. **Incremental**: Insert into graph, update cache
3. **Optimization**: Periodic graph rebalancing

## Memory vs. Disk Trade-offs

### Configuration Options

```rust
pub struct DensePersistenceConfig {
    /// Store vectors on disk immediately (like Qdrant on_disk: true)
    pub vectors_on_disk: bool,
    
    /// Store ANN index on disk (memory-mapped)
    pub index_on_disk: bool,
    
    /// Disable indexing during bulk load (m=0 for HNSW)
    pub disable_indexing_during_load: bool,
    
    /// Indexing threshold (batch size for building index)
    pub indexing_threshold: usize,
    
    /// Compression: None, ScalarQuantization (int8), Binary
    pub compression: CompressionType,
}
```

### Performance Characteristics

**Memory mode (vectors in RAM, index in RAM):**
- Latency: <10ms
- Cost: High (all data in RAM)
- Use case: Real-time queries, small-medium datasets

**Hybrid mode (vectors on disk, index in RAM):**
- Latency: 10-50ms (disk I/O for vectors)
- Cost: Medium (index in RAM, vectors on disk)
- Use case: Large datasets, moderate latency acceptable

**Disk mode (vectors on disk, index memory-mapped):**
- Latency: 50-200ms (disk I/O for both)
- Cost: Low (minimal RAM usage)
- Use case: Very large datasets, cost-sensitive

## Compression Strategies

### Scalar Quantization (int8)

**Format:**
```rust
struct QuantizedVector {
    scale: f32,                  // Per-vector scale factor
    offset: f32,                // Per-vector offset (optional)
    data: Vec<i8>,              // Quantized values
}
```

**Compression ratio**: 4x (f32 → i8)
**Quality loss**: <5% for most use cases
**Use case**: Reduce RAM usage while maintaining quality

### Binary Quantization

**Format:**
```rust
struct BinaryVector {
    data: Vec<u8>,              // Bits packed (dimension/8 bytes)
}
```

**Compression ratio**: 32x (f32 → 1 bit)
**Quality loss**: 10-20% (significant)
**Use case**: Very large datasets, approximate search only

### Product Quantization (PQ)

**Format**: See IVF-PQ section above
**Compression ratio**: 16-64x (depending on m and bits)
**Quality loss**: 5-15% (depends on configuration)
**Use case**: Billion-scale datasets

## Implementation Phases

### Phase 1: Basic Vector Persistence
1. Vector storage format (SoA layout)
2. Vector metadata storage
3. Basic save/load cycle
4. Memory mapping support

### Phase 2: HNSW Persistence
1. HNSW graph serialization
2. Layer storage format
3. Incremental construction support
4. Memory-mapped graph access

### Phase 3: IVF-PQ Persistence
1. Centroid storage
2. PQ codebook storage
3. Assignment storage
4. Incremental updates

### Phase 4: DiskANN Persistence
1. Graph persistence
2. Compressed vector storage
3. Cache management
4. On-demand loading

### Phase 5: Hybrid Persistence
1. Unified transaction log
2. Coordinated sparse/dense segments
3. Combined query execution
4. Unified recovery

## References

1. **Qdrant On-Disk Storage**: https://qdrant.tech/articles/indexing-optimization/
2. **OpenSearch Disk Mode**: https://aws.amazon.com/blogs/big-data/opensearch-vector-engine-is-now-disk-optimized-for-low-cost-accurate-vector-search/
3. **HNSW Paper**: Malkov & Yashunin, "Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs"
4. **IVF-PQ Paper**: Jégou et al., "Product quantization for nearest neighbor search"
5. **DiskANN Paper**: Subramanya et al., "DiskANN: Fast Accurate Billion-point Nearest Neighbor Search on a Single Node"
