# Complete ANN Implementation Roadmap

Comprehensive plan for implementing both modern (2024-2025) and classic ANN algorithms in pure Rust.

## Overview

This roadmap covers:
1. **Modern Methods** (2024-2025): OPT-SNG, SAQ, TurboQuant, GATE
2. **Classic Methods**: LSH, Random Projection Tree Forest (Annoy), KD-tree, Ball tree, Random Projection
3. **Already Implemented**: HNSW, Anisotropic VQ + k-means (SCANN), IVF-PQ, DiskANN

## Implementation Status

### ✅ Completed
- **HNSW**: Full implementation with graph construction and search
- **Anisotropic VQ + k-means (SCANN)**: Core implementation (partitioning, quantization, re-ranking)
- **IVF-PQ**: Core implementation (IVF index, PQ quantization)
- **DiskANN**: Framework (graph structure, cache, disk I/O layer)

### 📋 Research Complete
- **Modern (2024-2025)**: OPT-SNG, SAQ, TurboQuant, GATE, RaBitQ
- **Classic**: LSH, Random Projection Tree Forest (Annoy), KD-tree, Ball tree, Random Projection, FLANN

### ⏳ To Implement

#### Phase 1: Modern High-Impact (Immediate)
1. **OPT-SNG** - 5.9× construction speedup, auto parameter tuning
2. **SAQ** - 80% better quantization, 80× faster encoding
3. **TurboQuant** - Online quantization, near-optimal distortion

#### Phase 2: Graph-Based Methods (Based on 2025 Research)
4. **Vamana** - Competitive with HNSW, better for SSD-based serving (5-10× more points/node)
   - Two-pass construction with RRND + RND
   - **Priority**: High (top 3 performer on large datasets)
5. **NSG/SSG** - Good performance on 1M-25GB datasets
   - NSG: RND + NP paradigm
   - SSG: MOND-based diversification
   - **Priority**: Medium (doesn't scale beyond 25GB)
6. **ELPIS** - Best overall performer (DC + II + ND)
   - Superior on large and hard datasets
   - **Priority**: Medium-High (complex implementation, best performance)

#### Phase 3: Classic Methods (Next)
7. **LSH** - Theoretical guarantees, hash-based indexing
8. **Random Projection Tree Forest (vendor: Annoy)** - Production-proven, simple API
9. **KD-Tree** - Classic method, good for low dimensions
10. **Ball Tree** - Better than KD-tree for medium dimensions
11. **Random Projection Trees** - Good baseline

#### Phase 4: Enhancements (Later)
12. **GATE** - Adaptive graph enhancement (requires ML)
13. **RaBitQ** - Theoretical error bounds
14. **FLANN Methods** - Auto-tuning framework
15. **SPTAG** - DC-based with K-D Trees or BKT (high indexing time, good search)

## Detailed Implementation Plan

### Modern Methods (2024-2025)

#### 1. OPT-SNG ⭐⭐⭐⭐⭐
**Priority**: Highest
**Impact**: 5.9× construction speedup, automatic parameter tuning
**Time**: 1-2 weeks

**Key Components**:
- Martingale-based pruning model
- Closed-form truncation parameter R calculation
- Integration with existing HNSW structure

**Files**:
- `src/dense/sng/mod.rs`
- `src/dense/sng/martingale.rs`
- `src/dense/sng/optimization.rs`

---

#### 2. SAQ ⭐⭐⭐⭐⭐
**Priority**: Highest
**Impact**: 80% quantization error reduction, 80× faster encoding
**Time**: 1-2 weeks

**Key Components**:
- PCA projection
- Dimension segmentation with dynamic programming
- Code adjustment with coordinate-descent

**Files**:
- `src/dense/quantization/saq.rs`
- `src/dense/quantization/pca.rs`
- `src/dense/quantization/segmentation.rs`

---

#### 3. TurboQuant ⭐⭐⭐⭐⭐
**Priority**: Highest
**Impact**: Online quantization, better recall than PQ
**Time**: 1-2 weeks

**Key Components**:
- Random rotation preprocessing
- Beta distribution transformation
- Optimal scalar quantizers
- Two-stage: MSE + 1-bit QJL transform

**Files**:
- `src/dense/quantization/turboquant.rs`
- `src/dense/quantization/rotation.rs`
- `src/dense/quantization/qjl.rs`

---

### Classic Methods

#### 4. LSH (Locality Sensitive Hashing) ⭐⭐⭐⭐
**Priority**: High
**Impact**: Theoretical guarantees, hash-based indexing
**Time**: 1 week

**Key Components**:
- Random projection LSH (cosine similarity)
- Hash table indexing
- Candidate generation and verification
- Multiple hash functions (hash family)

**Files**:
- `src/dense/classic/lsh/mod.rs`
- `src/dense/classic/lsh/random_projection.rs`
- `src/dense/classic/lsh/hash_table.rs`

**Variants to Implement**:
- Random Projection LSH (cosine)
- E2LSH (L2 distance) - optional
- MinHash LSH (Jaccard) - optional
- LSH Forest (adaptive) - optional

---

#### 5. Random Projection Tree Forest (vendor: Annoy) ⭐⭐⭐⭐
**Priority**: High
**Impact**: Production-proven, simple API
**Time**: 1 week

**Key Components**:
- Random projection tree forest
- Memory-mapped index support (optional)
- Thread-safe search
- SIMD distance computation

**Files**:
- `src/dense/classic/trees/annoy.rs`
- `src/dense/classic/trees/rp_tree.rs`

---

#### 6. KD-Tree ⭐⭐⭐
**Priority**: Medium
**Impact**: Classic method, good for low dimensions
**Time**: 3-5 days

**Key Components**:
- Recursive space partitioning
- Alternating dimension splits
- Nearest neighbor search with backtracking
- Approximate variants (early termination)

**Files**:
- `src/dense/classic/trees/kdtree.rs`

**Best For**: d < 20 dimensions

---

#### 7. Ball Tree ⭐⭐⭐
**Priority**: Medium
**Impact**: Better than KD-tree for medium dimensions
**Time**: 3-5 days

**Key Components**:
- Hierarchical ball partitioning
- Metric-based (any metric)
- Recursive construction
- Approximate search variants

**Files**:
- `src/dense/classic/trees/balltree.rs`

**Best For**: 20 < d < 100 dimensions

---

#### 8. Random Projection Trees ⭐⭐⭐
**Priority**: Medium
**Impact**: Good baseline, simple implementation
**Time**: 2-3 days

**Key Components**:
- Random hyperplane splits
- Binary tree structure
- Forest of trees for better recall

**Files**:
- `src/dense/classic/trees/random_projection.rs`

---

## Module Structure

```
crates/rank-retrieve/src/dense/
├── hnsw/              # ✅ HNSW (implemented)
├── scann/             # ✅ Anisotropic VQ + k-means (SCANN, implemented)
├── ivf_pq/            # ✅ IVF-PQ (implemented)
├── diskann/           # ✅ DiskANN (framework)
├── sng/               # ⏳ OPT-SNG (to implement)
│   ├── mod.rs
│   ├── martingale.rs
│   └── optimization.rs
├── quantization/      # ⏳ Enhanced quantization
│   ├── saq.rs
│   ├── turboquant.rs
│   ├── pca.rs
│   └── segmentation.rs
└── classic/           # ⏳ Classic methods
    ├── mod.rs
    ├── lsh/
    │   ├── mod.rs
    │   ├── random_projection.rs
    │   └── hash_table.rs
    └── trees/
        ├── mod.rs
        ├── annoy.rs
        ├── kdtree.rs
        ├── balltree.rs
        └── random_projection.rs
```

## Feature Flags

```toml
[features]
# Modern methods
hnsw = ["dense", "dep:smallvec", "dep:rand"]
scann = ["dense", "dep:rand"]
ivf_pq = ["dense", "dep:rand"]
diskann = ["dense", "dep:smallvec", "dep:rand"]
sng = ["dense", "dep:smallvec", "dep:rand"]  # OPT-SNG
saq = ["dense", "dep:rand"]                  # SAQ quantization
turboquant = ["dense", "dep:rand"]           # TurboQuant

# Classic methods
lsh = ["dense", "dep:rand"]                  # LSH
annoy = ["dense", "dep:rand"]               # Random Projection Tree Forest (vendor: Annoy)
kdtree = ["dense"]                          # KD-Tree
balltree = ["dense"]                        # Ball Tree
rptree = ["dense", "dep:rand"]              # Random Projection Trees

# Convenience features
ann_modern = ["hnsw", "scann", "ivf_pq", "diskann", "sng", "saq", "turboquant"]
ann_classic = ["lsh", "annoy", "kdtree", "balltree", "rptree"]
ann_all = ["ann_modern", "ann_classic"]
```

## Implementation Timeline

### Month 1: Modern High-Impact
- Week 1-2: OPT-SNG
- Week 3-4: SAQ
- Week 5-6: TurboQuant

### Month 2: Classic Methods
- Week 1: LSH
- Week 2: Random Projection Tree Forest (Annoy)
- Week 3: KD-Tree + Ball Tree
- Week 4: Random Projection Trees

### Month 3: Integration & Polish
- Week 1-2: Unified API, benchmarks
- Week 3-4: Documentation, optimization

## Expected Performance

| Method | Construction | Search | Recall | Memory | Best For |
|--------|--------------|--------|--------|--------|----------|
| **HNSW** | Baseline | Baseline | Very High | ~2x | General purpose |
| **OPT-SNG** | **5.9× faster** | Similar/Better | Similar/Better | Similar | Auto-tuning |
| **SAQ** | Similar | Similar | **Better** | Similar | Quantization |
| **TurboQuant** | **Online** | Similar | **Better** | Similar | Streaming |
| **LSH** | Fast (O(n)) | Medium | Medium | Low | High-d, hash-based |
| **RP-Tree Forest (Annoy)** | Fast | Fast | High | Low | Production, simple |
| **KD-Tree** | Fast | Fast (low d) | High (low d) | Low | d < 20 |
| **Ball Tree** | Fast | Fast (med d) | High (med d) | Low | 20 < d < 100 |

## Use Case Guide

### High-Dimensional Embeddings (d > 100)
- **Best**: HNSW, OPT-SNG, Anisotropic VQ + k-means (SCANN), IVF-PQ
- **Good**: LSH, Random Projection Tree Forest (Annoy)
- **Avoid**: KD-Tree, Ball Tree

### Low-Dimensional Data (d < 20)
- **Best**: KD-Tree (exact), Ball Tree
- **Good**: HNSW, Random Projection
- **Avoid**: LSH (overkill)

### Medium-Dimensional Data (20 < d < 100)
- **Best**: HNSW, Ball Tree, Random Projection Tree Forest (Annoy)
- **Good**: Anisotropic VQ + k-means (SCANN), Random Projection
- **Avoid**: KD-Tree

### Very Large Datasets (Billion-scale)
- **Best**: IVF-PQ, DiskANN, Anisotropic VQ + k-means (SCANN)
- **Good**: HNSW, OPT-SNG
- **Avoid**: Tree methods (memory)

### Streaming/Online Data
- **Best**: TurboQuant, LSH
- **Good**: Annoy (with updates)
- **Avoid**: Methods requiring full rebuild

### Production Systems
- **Best**: Random Projection Tree Forest (Annoy, battle-tested), HNSW
- **Good**: OPT-SNG (auto-tuning), Anisotropic VQ + k-means (SCANN)
- **Consider**: LSH (hash-based systems)

## Dependencies

**Minimal dependencies** (all methods):
- `smallvec` (for HNSW, DiskANN, OPT-SNG)
- `rand` (for all probabilistic methods)
- No additional dependencies needed

## References

### Modern Methods
- OPT-SNG: https://arxiv.org/abs/2509.15531
- SAQ: https://arxiv.org/abs/2509.12086
- TurboQuant: https://arxiv.org/abs/2504.19874
- GATE: https://arxiv.org/abs/2506.15986

### Classic Methods
- LSH: Indyk & Motwani (1998)
- Random Projection Tree Forest (Annoy): Spotify Engineering Blog
- KD-Tree: Bentley (1975)
- Ball Tree: Omohundro (1989)
- Random Projection: Dasgupta & Freund (2008)

## Summary

This roadmap provides a **complete ANN library** covering:
- ✅ **4 modern methods** (HNSW, Anisotropic VQ + k-means (SCANN), IVF-PQ, DiskANN) - implemented
- ⏳ **3 cutting-edge methods** (OPT-SNG, SAQ, TurboQuant) - to implement
- ⏳ **5 classic methods** (LSH, Random Projection Tree Forest (Annoy), KD-tree, Ball tree, RP-trees) - to implement

All methods will be:
- **Pure Rust** (no FFI)
- **SIMD-accelerated** (using existing infrastructure)
- **Minimal dependencies** (only `smallvec` and `rand`)
- **Unified API** (same `ANNIndex` trait)

This creates the **most comprehensive pure Rust ANN library** available.
