# Recent ANN Research (2024-2025)

## Overview

This document tracks the latest state-of-the-art approximate nearest neighbor search algorithms and research from 2024-2025, beyond the traditional methods (HNSW, Anisotropic VQ + k-means (SCANN), IVF-PQ, DiskANN).

## Key Recent Advances

### 1. Symmetric Neighbor Graph (SNG) - 2025
**Paper**: "Graph-Based Approximate Nearest Neighbor Search Revisited: Theoretical Analysis and Optimization" (Ma et al., 2025)
**ArXiv**: https://arxiv.org/abs/2509.15531

**Key Contributions**:
- Rigorous theoretical analysis of graph construction under realistic random data assumptions
- Optimizes critical **truncation parameter R** through analytical expressions (no manual tuning)
- **2x to 9x acceleration** in index construction across standard benchmarks
- Maintains comparable or improved latency and recall metrics
- Proves GreedySearch converges to approximate nearest neighbors in **O(log n) steps** with high probability

**Why It Matters**:
- Provides theoretical guarantees missing from HNSW
- Automatic parameter optimization (no manual tuning)
- Significant construction speedup

**Implementation Priority**: ⭐⭐⭐⭐⭐ (High - theoretical improvements with practical benefits)

---

### 2. SAQ (Segmented Adaptive Quantization) - 2025
**Paper**: "SAQ: Pushing the Limits of Vector Quantization through Code Adjustment and Dimension Segmentation" (Li et al., 2025)
**ArXiv**: https://arxiv.org/abs/2509.12086

**Key Contributions**:
- Novel vector quantization technique combining code adjustment and dimension segmentation
- Addresses limitations of traditional Product Quantization (PQ)
- Improved accuracy for ANNS in high-dimensional spaces

**Why It Matters**:
- Direct improvement over PQ (used in IVF-PQ)
- Better compression with maintained accuracy

**Implementation Priority**: ⭐⭐⭐⭐ (High - improves existing PQ implementation)

---

### 3. TurboQuant - 2025
**Paper**: "TurboQuant: Online Vector Quantization with Near-optimal Distortion Rate" (Zandieh et al., 2025)
**ArXiv**: https://arxiv.org/abs/2504.19874

**Key Contributions**:
- Online vector quantization (no batch processing required)
- Addresses both **mean-squared error (MSE)** and **inner product distortion**
- Near-optimal distortion rate
- Overcomes limitations of existing quantization methods

**Why It Matters**:
- Online/streaming quantization (important for dynamic datasets)
- Better inner product preservation (critical for MIPS)

**Implementation Priority**: ⭐⭐⭐⭐ (High - online capability is valuable)

---

### 4. Adaptive Graph-Based ANNS - 2025
**Paper**: "Empowering Graph-based Approximate Nearest Neighbor Search with Adaptive Awareness Capabilities" (Ruan et al., 2025)
**ArXiv**: https://arxiv.org/abs/2506.15986

**Key Contributions**:
- Adaptive awareness capabilities for graph-based ANNS
- Addresses limitations of static graph structures
- Improved query performance through adaptive mechanisms

**Why It Matters**:
- Dynamic adaptation to query patterns
- Better performance for diverse query distributions

**Implementation Priority**: ⭐⭐⭐ (Medium - interesting but needs evaluation)

---

### 5. RaBitQ - 2024
**Paper**: "RaBitQ: Quantizing High-Dimensional Vectors with a Theoretical Error Bound for Approximate Nearest Neighbor Search" (Gao & Long, 2024)
**ArXiv**: https://arxiv.org/abs/2405.12497

**Key Contributions**:
- Theoretical error bounds for quantization
- Practical and asymptotically optimal quantization
- Better than existing PQ variants

**Why It Matters**:
- Theoretical guarantees on quantization error
- Proven optimality

**Implementation Priority**: ⭐⭐⭐⭐ (High - theoretical guarantees)

---

### 6. Individualized Non-Uniform Quantization - 2025
**Paper**: "Individualized non-uniform quantization for vector search" (Tepper & Willke, 2025)
**ArXiv**: https://arxiv.org/abs/2509.18471

**Key Contributions**:
- Non-uniform quantization tailored to individual vectors
- Better compression for embedding vectors
- Addresses high-dimensionality problems

**Why It Matters**:
- Personalized quantization (better compression)
- Important for large-scale vector search

**Implementation Priority**: ⭐⭐⭐ (Medium - interesting approach)

---

### 7. Probabilistic Routing for Graph-Based ANNS - 2024
**Paper**: "Probabilistic Routing for Graph-Based Approximate Nearest Neighbor Search" (Lu et al., 2024)
**ArXiv**: https://arxiv.org/abs/2402.11354

**Key Contributions**:
- Probabilistic routing instead of deterministic greedy search
- Improved performance for graph-based methods
- Better handling of diverse query patterns

**Why It Matters**:
- Alternative to greedy search in HNSW
- Could improve recall/performance trade-offs

**Implementation Priority**: ⭐⭐⭐ (Medium - alternative approach to explore)

---

### 8. Learned Indexes for ANNS
**Multiple Papers**: 
- "Learning to Index for Nearest Neighbor Search" (2018)
- "IRLI: Iterative Re-partitioning for Learning to Index" (2021)
- "A Learned Index for Exact Similarity Search in Metric Spaces" (2022)

**Key Contributions**:
- Neural network-based index structures
- End-to-end learning of data structures
- Adaptive to data distribution

**Why It Matters**:
- Paradigm shift: learned vs. hand-designed structures
- Could outperform traditional methods for specific datasets

**Implementation Priority**: ⭐⭐ (Low-Medium - requires ML infrastructure, but promising)

---

## Implementation Recommendations

### Phase 1: High Priority (Immediate) ⭐⭐⭐⭐⭐

#### 1. OPT-SNG (Optimized Sparse Neighborhood Graph)
**Why**: Highest impact - 5.9x average speedup (15.4x peak), automatic parameter tuning
**Key Features**:
- Martingale-based theoretical model
- Closed-form rule for truncation parameter R (no manual tuning)
- Maximum out-degree: O(n^{2/3+ε})
- Expected search path: O(log n)
- **5.9× average speedup** in construction, **15.4× peak**

**Implementation Notes**:
- Can be integrated into existing HNSW implementation
- Requires martingale-based pruning model
- Automatic parameter selection eliminates tuning

#### 2. SAQ (Segmented Adaptive Quantization)
**Why**: Massive improvements - 80% quantization error reduction, 80x faster encoding
**Key Features**:
- Dimension segmentation with PCA projection
- Dynamic programming for optimal bit allocation
- Code adjustment with coordinate-descent refinement
- **80% reduction in quantization error**
- **80x faster encoding** than Extended RaBitQ

**Implementation Notes**:
- Improves existing PQ implementation
- Requires PCA projection
- Dynamic programming for segmentation optimization

#### 3. TurboQuant
**Why**: Online capability + near-optimal distortion, outperforms PQ
**Key Features**:
- **Online/streaming quantization** (no batch processing)
- Near-optimal distortion rate (within 2.7× of theoretical bound)
- Addresses both MSE and inner product distortion
- Random rotation + Beta distribution + scalar quantizers
- **Outperforms PQ in recall**, **zero indexing time**

**Implementation Notes**:
- Data-oblivious (suitable for online)
- Random rotation preprocessing
- Two-stage: MSE quantizer + 1-bit QJL transform

### Phase 2: Medium Priority (Next) ⭐⭐⭐⭐

#### 4. RaBitQ
**Why**: Theoretical error bounds, proven optimality
**Key Features**:
- Theoretical error bounds for quantization
- Practical and asymptotically optimal
- Better than existing PQ variants

**Implementation Notes**:
- Can complement SAQ/TurboQuant
- Provides theoretical guarantees

#### 5. GATE (Adaptive Graph with Query Awareness)
**Why**: 1.2-2.0x speedup for graph-based methods
**Key Features**:
- Adaptive entry point selection
- Contrastive learning-based two-tower model
- Hub node extraction
- **1.2-2.0× speedup** in query performance

**Implementation Notes**:
- Requires ML model (two-tower architecture)
- Can be added as enhancement to HNSW/SNG
- Lightweight adaptive module

### Phase 3: Research/Experimental ⭐⭐⭐

#### 6. Probabilistic Routing
**Why**: Alternative to greedy search, better for diverse queries
**Key Features**:
- Probabilistic instead of deterministic routing
- Better handling of local optima

**Implementation Notes**:
- Alternative search strategy
- Can be tested alongside greedy search

#### 7. Learned Indexes
**Why**: Paradigm shift, potentially best for specific datasets
**Key Features**:
- End-to-end learning of data structures
- Neural network-based indexes

**Implementation Notes**:
- Requires ML infrastructure
- More experimental, dataset-specific

## Performance Comparison (Measured/Expected)

| Method | Construction Speed | Search Speed | Recall | Memory | Encoding Speed | Notes |
|--------|-------------------|--------------|--------|--------|----------------|-------|
| HNSW | Baseline | Baseline | High | ~2x | N/A | Current implementation |
| **OPT-SNG** | **5.9× faster** (avg), **15.4× peak** | Similar/Better | Similar/Better | Similar | N/A | Auto parameter tuning, theoretical guarantees |
| **SAQ** | Similar | Similar | **Better** | Similar | **80× faster** | 80% error reduction vs PQ |
| **TurboQuant** | **Online** (zero indexing) | Similar | **Better** | Similar | Fast | Near-optimal distortion, streaming |
| RaBitQ | Similar | Similar | **Better** | Similar | Similar | Theoretical bounds |
| GATE | Similar | **1.2-2.0× faster** | Similar | Similar | N/A | Adaptive entry point |

## Next Steps

1. **Implement SNG** - Highest impact, theoretical improvements
2. **Enhance PQ with SAQ/RaBitQ** - Improve existing IVF-PQ
3. **Add TurboQuant** - Online quantization capability
4. **Research adaptive methods** - Evaluate practical benefits

## References

- Ma et al. (2025): "Graph-Based Approximate Nearest Neighbor Search Revisited" - https://arxiv.org/abs/2509.15531
- Li et al. (2025): "SAQ: Pushing the Limits of Vector Quantization" - https://arxiv.org/abs/2509.12086
- Zandieh et al. (2025): "TurboQuant: Online Vector Quantization" - https://arxiv.org/abs/2504.19874
- Ruan et al. (2025): "Empowering Graph-based ANNS with Adaptive Awareness" - https://arxiv.org/abs/2506.15986
- Gao & Long (2024): "RaBitQ: Quantizing with Theoretical Error Bound" - https://arxiv.org/abs/2405.12497
- Tepper & Willke (2025): "Individualized non-uniform quantization" - https://arxiv.org/abs/2509.18471
- Lu et al. (2024): "Probabilistic Routing for Graph-Based ANNS" - https://arxiv.org/abs/2402.11354
