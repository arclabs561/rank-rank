# ID Compression Quick Start Guide

Quick reference for implementing lossless ID compression in rank-retrieve.

## Key Concepts

### Problem
- Vector IDs in IVF clusters and HNSW neighbor lists are stored as `Vec<u32>`
- Order of IDs doesn't matter (set semantics, not sequence)
- Current storage: 4 bytes per ID (32 bits)
- Opportunity: Exploit ordering invariance to save `log(n!)` bits per set of size `n`

### Solution
- **ROC (Random Order Coding)**: Compress sets using bits-back coding with ANS
- **REC (Random Edge Coding)**: Compress entire graphs (offline)
- **Wavelet Trees**: Full random access when needed

## Implementation Checklist

### Phase 1: Core Library
- [ ] Add `constriction` or `rans` dependency
- [ ] Create `src/compression/` module structure
- [ ] Implement `IdSetCompressor` trait
- [ ] Implement `RocCompressor` with bits-back coding
- [ ] Add unit tests for compression/decompression
- [ ] Benchmark compression ratios

### Phase 2: IVF-PQ Integration
- [ ] Add `id_compression: Option<IdCompressionMethod>` to `IVFPQParams`
- [ ] Create `ClusterStorage` enum (Uncompressed/Compressed)
- [ ] Modify `Cluster` to support compressed storage
- [ ] Implement `compress_clusters()` in `build()`
- [ ] Implement on-demand decompression in `search()`
- [ ] Add caching to avoid repeated decompression
- [ ] Integration tests

### Phase 3: HNSW Integration
- [ ] Add compression option to `HNSWParams`
- [ ] Create `NeighborStorage` enum
- [ ] Modify `Layer` to support compressed neighbors
- [ ] Implement threshold check (only compress if `m >= 32`)
- [ ] Update graph traversal to decompress on-demand
- [ ] Integration tests

### Phase 4: Persistence
- [ ] Add compression config to persistence format
- [ ] Implement compressed serialization
- [ ] Implement compressed deserialization
- [ ] Version migration support

## Code Snippets

### Basic ROC Usage

```rust
use rank_retrieve::compression::roc::RocCompressor;
use rank_retrieve::compression::traits::IdSetCompressor;

let compressor = RocCompressor::new();
let ids: Vec<u32> = vec![1, 5, 10, 20, 50];
let universe_size = 1000;

// Compress
let compressed = compressor.compress_set(&ids, universe_size)?;

// Decompress
let decompressed = compressor.decompress_set(&compressed, universe_size)?;
```

### IVF-PQ with Compression

```rust
use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
use rank_retrieve::compression::IdCompressionMethod;

let params = IVFPQParams {
    num_clusters: 1024,
    nprobe: 100,
    id_compression: Some(IdCompressionMethod::Roc),  // Enable compression
    ..Default::default()
};

let mut index = IVFPQIndex::new(128, params)?;
// ... add vectors ...
index.build()?;  // Compression happens here
let results = index.search(&query, 10)?;  // Decompression on-demand
```

## Performance Targets

| Metric | Target | Notes |
|--------|--------|-------|
| Compression ratio (IVF) | 5-7x | For clusters with N_k > 1000 |
| Compression ratio (HNSW) | 1.5-2x | For m >= 32 |
| Search slowdown (IVF) | < 20% | Acceptable trade-off |
| Search slowdown (HNSW) | < 30% | Acceptable for memory-constrained |
| Build time increase | < 10% | Compression during build |

## When to Use Compression

### Enable Compression For:
- ✅ Large-scale indexes (million+ vectors)
- ✅ Memory-constrained environments
- ✅ Disk persistence (offline compression)
- ✅ IVF with large clusters (N_k > 1000)
- ✅ HNSW with large neighbor lists (m >= 32)

### Skip Compression For:
- ❌ Small datasets (< 100K vectors)
- ❌ Small clusters/lists (overhead > benefit)
- ❌ Real-time search where latency is critical
- ❌ HNSW with m=16 (limited benefit per paper)

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# ANS entropy coding
constriction = { version = "0.3", optional = true }

[features]
default = []
id-compression = ["constriction"]
```

## Testing

```bash
# Run unit tests
cargo test --features id-compression

# Run integration tests
cargo test --test ivf_compression --features id-compression

# Benchmark
cargo bench --bench id_compression --features id-compression
```

## References

- **Paper**: "Lossless Compression of Vector IDs for Approximate Nearest Neighbor Search" (Severo et al., 2025)
- **Meta Implementation**: https://github.com/facebookresearch/vector_db_id_compression
- **Implementation Plan**: `docs/ID_COMPRESSION_IMPLEMENTATION_PLAN.md`
- **Technical Design**: `docs/ID_COMPRESSION_TECHNICAL_DESIGN.md`

## Common Issues

### Issue: Decompression too slow
**Solution**: Add caching, batch decompression, or increase threshold

### Issue: Compression ratio lower than expected
**Solution**: Check that IDs are sorted, verify ANS implementation, test with larger sets

### Issue: Memory usage increases
**Solution**: Clear decompression cache after search, use interior mutability carefully

### Issue: Search results differ
**Solution**: Ensure IDs are sorted before compression, verify round-trip correctness
