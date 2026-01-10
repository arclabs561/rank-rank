# Complete ANN Methods Summary

Comprehensive overview of all approximate nearest neighbor search algorithms in `rank-retrieve`, including both modern (2024-2025) and classic methods.

## Quick Reference

### ✅ Implemented (Ready to Use)

1. **HNSW** - Hierarchical Navigable Small World
   - Graph-based, high recall, fast search
   - Best for: General purpose, high-dimensional data

2. **Anisotropic Vector Quantization with k-means Partitioning** (vendor: SCANN)
   - Quantization-based, three-stage search
   - Best for: MIPS, very large datasets

3. **IVF-PQ** - Inverted File Index with Product Quantization
   - Memory-efficient, billion-scale capable
   - Best for: Memory-constrained, very large datasets

4. **DiskANN** - Disk-based ANN
   - Disk-optimized graph structure
   - Best for: Very large datasets that don't fit in memory

5. **K-Means Tree** - Hierarchical Clustering Tree
   - Tree-based, clustering-based partitioning
   - Best for: Medium-dimensional data (20 < d < 200)

### ⏳ To Implement (High Priority)

#### Modern Methods (2024-2025)

5. **OPT-SNG** - Optimized Sparse Neighborhood Graph
   - **5.9× construction speedup** (15.4× peak)
   - Automatic parameter tuning
   - Theoretical guarantees

6. **SAQ** - Segmented Adaptive Quantization
   - **80% quantization error reduction**
   - **80× faster encoding** than Extended RaBitQ
   - Improves PQ implementations

7. **TurboQuant** - Online Vector Quantization
   - **Online/streaming** quantization
   - **Near-optimal distortion** (within 2.7×)
   - **Outperforms PQ in recall**

#### Classic Methods

8. **LSH** - Locality Sensitive Hashing
   - Theoretical guarantees
   - Hash-based indexing
   - Fast construction (O(n))

9. **Random Projection Tree Forest** (vendor: Annoy)
   - Production-proven
   - Simple API
   - Memory-mapped support

10. **KD-Tree** - K-Dimensional Tree
    - Classic method
    - Best for: Low dimensions (d < 20)

11. **Ball Tree** - Hierarchical Ball Partitioning
    - Better than KD-tree for medium dimensions
    - Works with any metric

12. **K-Means Tree** - Hierarchical Clustering Tree
    - Clustering-based partitioning
    - Best for: Medium dimensions (20 < d < 200)

13. **Random Projection Trees**
    - Good baseline
    - Simple implementation

## Method Comparison

| Method | Type | Construction | Search | Recall | Memory | Best For |
|--------|------|--------------|--------|--------|--------|----------|
| **HNSW** | Graph | O(n log n) | O(log n) | Very High | ~2x | General purpose |
| **OPT-SNG** | Graph | **5.9× faster** | O(log n) | Very High | ~2x | Auto-tuning |
| **Anisotropic VQ + k-means (SCANN)** | Quantization | O(n log n) | O(n/k) | High | ~1.5x | MIPS, large datasets |
| **IVF-PQ** | Quantization | O(n log n) | O(n/k + k) | Medium | ~0.1x | Billion-scale (B-scale) |
| **DiskANN** | Graph+Disk | O(n log n) | O(log n) | High | ~0.01x | Very large, disk |
| **SAQ** | Quantization | Similar | Similar | **Better** | Similar | Better PQ |
| **TurboQuant** | Quantization | **Online** | Similar | **Better** | Similar | Streaming |
| **LSH** | Hash | O(n) | O(n^ρ) | Medium | Low | High-d, hash-based |
| **RP-Tree Forest (Annoy)** | Tree | O(n log n) | O(log n) | High | Low | Production, simple |
| **KD-Tree** | Tree | O(n log n) | O(log n) | High (low d) | Low | d < 20 |
| **Ball Tree** | Tree | O(n log n) | O(log n) | High (med d) | Low | 20 < d < 100 |
| **K-Means Tree** | Tree | O(n log n) | O(log n) | High (med d) | Low | 20 < d < 200 |
| **RP-Tree** | Tree | O(n log n) | O(log n) | Medium | Low | Baseline |

## Use Case Guide

### By Dataset Size (Based on 2025 Experimental Evaluation)

**Small to Medium (1M-25GB vectors)**:
- **Best**: HNSW, NSG/SSG (if available)
- **Good**: SPTAG-BKT (for hard datasets)
- **Why**: HNSW provides excellent overall performance; NSG/SSG competitive on easier datasets

**Large (100GB-1B vectors, B-scale)**:
- **Best**: ELPIS (if available), HNSW
- **Good**: Vamana (if available)
- **Why**: II-based methods (HNSW, ELPIS) have best scalability; ELPIS is 2× faster than HNSW on 1B (B-scale) datasets

**Hard Datasets/Workloads** (High LID, Low LRC):
- **Best**: DC-based methods (ELPIS, SPTAG-BKT)
- **Good**: HNSW (baseline)
- **Why**: Divide-and-conquer excels on challenging distributions

**Memory-Constrained**:
- **Best**: ELPIS (40% less memory than HNSW)
- **Good**: IVF-PQ, DiskANN
- **Why**: ELPIS uses smaller max out-degree and beam width

### By Dimensionality

### High-Dimensional Embeddings (d > 100)
**Recommended**: HNSW, OPT-SNG, Anisotropic VQ + k-means (SCANN), IVF-PQ
**Also Good**: LSH, Random Projection Tree Forest (Annoy)
**Avoid**: KD-Tree, Ball Tree

### Low-Dimensional Data (d < 20)
**Recommended**: KD-Tree (exact), Ball Tree
**Also Good**: HNSW, Random Projection
**Avoid**: LSH (overkill)

### Medium-Dimensional Data (20 < d < 100)
**Recommended**: HNSW, Ball Tree, Random Projection Tree Forest (Annoy)
**Also Good**: Anisotropic VQ + k-means (SCANN), Random Projection
**Avoid**: KD-Tree

### Very Large Datasets (Billion-scale, B-scale)
**Recommended**: IVF-PQ, DiskANN, Anisotropic VQ + k-means (SCANN), ELPIS (if available)
**Also Good**: HNSW, OPT-SNG
**Avoid**: Tree methods (memory), NP-based methods (KGraph, EFANNA, NSG, SSG)

### Streaming/Online Data
**Recommended**: TurboQuant, LSH
**Also Good**: Random Projection Tree Forest (Annoy, with updates)
**Avoid**: Methods requiring full rebuild

### Production Systems
**Recommended**: Random Projection Tree Forest (Annoy, battle-tested), HNSW
**Also Good**: OPT-SNG (auto-tuning), Anisotropic VQ + k-means (SCANN)
**Consider**: LSH (hash-based systems)

### Memory-Constrained
**Recommended**: IVF-PQ, LSH
**Also Good**: Random Projection Tree Forest (Annoy), Random Projection
**Avoid**: HNSW, DiskANN (higher memory)

## Implementation Status

### ✅ Phase 1: Foundation (Complete)
- HNSW (full implementation)
- Anisotropic VQ + k-means (SCANN, core implementation)
- IVF-PQ (core implementation)
- DiskANN (framework)

### ⏳ Phase 2: Modern High-Impact (Next)
- OPT-SNG (5.9× speedup)
- SAQ (80% better quantization)
- TurboQuant (online quantization)

### ⏳ Phase 3: Classic Methods (After Modern)
- LSH (hash-based)
- Random Projection Tree Forest (Annoy, production-proven)
- KD-Tree (low dimensions)
- Ball Tree (medium dimensions)
- Random Projection Trees (baseline)

## Feature Flags

```toml
# Modern methods
hnsw = ["dense", "dep:smallvec", "dep:rand"]
scann = ["dense", "dep:rand"]
ivf_pq = ["dense", "dep:rand"]
diskann = ["dense", "dep:smallvec", "dep:rand"]
sng = ["dense", "dep:smallvec", "dep:rand"]      # OPT-SNG
saq = ["dense", "dep:rand"]                      # SAQ
turboquant = ["dense", "dep:rand"]               # TurboQuant

# Classic methods
lsh = ["dense", "dep:rand"]                      # LSH
annoy = ["dense", "dep:rand"]                    # Random Projection Tree Forest (vendor: Annoy)
kdtree = ["dense"]                               # KD-Tree
balltree = ["dense"]                             # Ball Tree
rptree = ["dense", "dep:rand"]                   # Random Projection

# Convenience
ann_modern = ["hnsw", "scann", "ivf_pq", "diskann", "sng", "saq", "turboquant"]
ann_classic = ["lsh", "annoy", "kdtree", "balltree", "rptree"]
ann_all = ["ann_modern", "ann_classic"]
```

## Unified API

All methods implement the same `ANNIndex` trait:

```rust
use rank_retrieve::dense::ann::ANNIndex;

// Works with any algorithm
fn search_ann<I: ANNIndex>(index: &I, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
    index.search(query, k)
}
```

## Performance Targets

After implementing all methods:

| Metric | Target |
|--------|--------|
| Construction Speed | **5-15× faster** (OPT-SNG) |
| Quantization Accuracy | **80% better** (SAQ) |
| Encoding Speed | **80× faster** (SAQ) |
| Online Capability | **Yes** (TurboQuant) |
| Search Performance | **1.2-2.0× faster** (GATE, optional) |
| Parameter Tuning | **Automatic** (OPT-SNG) |

## Documentation

- **Modern Methods**: `RECENT_ANN_RESEARCH.md`
- **Classic Methods**: `CLASSIC_ANN_METHODS.md`
- **Implementation Plan**: `COMPLETE_ANN_ROADMAP.md`
- **Status**: `ANN_IMPLEMENTATION_COMPLETE.md`

## Summary

This creates the **most comprehensive pure Rust ANN library** with:
- ✅ **4 implemented methods** (HNSW, SCANN, IVF-PQ, DiskANN)
- ⏳ **3 cutting-edge methods** (OPT-SNG, SAQ, TurboQuant)
- ⏳ **5 classic methods** (LSH, Random Projection Tree Forest (Annoy), KD-tree, Ball tree, RP-trees)

**Total: 12 ANN algorithms** - all pure Rust, SIMD-accelerated, minimal dependencies.
