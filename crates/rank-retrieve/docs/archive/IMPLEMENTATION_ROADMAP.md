# ANN Implementation Roadmap

Based on recent research (2024-2025), this document outlines the implementation plan for cutting-edge ANN algorithms.

## Current Status

✅ **Completed**:
- HNSW (basic implementation)
- Anisotropic VQ + k-means (SCANN, core implementation)
- IVF-PQ (core implementation)
- DiskANN (framework)

📋 **Research Complete**:
- OPT-SNG, SAQ, TurboQuant (modern 2024-2025 methods)
- LSH, Random Projection Tree Forest (Annoy), KD-tree, Ball tree, Random Projection (classic methods)

## Phase 1: High-Impact Improvements (Immediate Priority)

### 1. OPT-SNG (Optimized Sparse Neighborhood Graph)

**Priority**: ⭐⭐⭐⭐⭐ (Highest)

**Why**:
- **5.9× average speedup** in construction (15.4× peak)
- Automatic parameter tuning (eliminates manual tuning)
- Theoretical guarantees (O(log n) search path)
- Can enhance existing HNSW implementation

**Implementation Plan**:
1. Implement martingale-based pruning model
2. Add closed-form truncation parameter R calculation
3. Integrate with existing HNSW graph structure
4. Replace manual parameter tuning with automatic selection

**Files to Create**:
- `src/dense/sng/mod.rs` - OPT-SNG implementation
- `src/dense/sng/martingale.rs` - Martingale-based pruning model
- `src/dense/sng/optimization.rs` - Parameter optimization

**Estimated Impact**: 5-15× construction speedup, better search performance

---

### 2. SAQ (Segmented Adaptive Quantization)

**Priority**: ⭐⭐⭐⭐⭐ (Highest)

**Why**:
- **80% reduction in quantization error** vs PQ
- **80× faster encoding** than Extended RaBitQ
- Improves existing PQ implementation
- Critical for IVF-PQ and Anisotropic VQ + k-means (SCANN)

**Implementation Plan**:
1. Implement PCA projection for vectors
2. Dimension segmentation with dynamic programming
3. Code adjustment with coordinate-descent refinement
4. Integrate into existing PQ codebase

**Files to Create**:
- `src/dense/quantization/saq.rs` - SAQ implementation
- `src/dense/quantization/pca.rs` - PCA projection
- `src/dense/quantization/segmentation.rs` - Dimension segmentation

**Estimated Impact**: 80% better accuracy, 80× faster encoding

---

### 3. TurboQuant

**Priority**: ⭐⭐⭐⭐⭐ (Highest)

**Why**:
- **Online/streaming quantization** (critical for dynamic datasets)
- **Near-optimal distortion** (within 2.7× of theoretical bound)
- **Outperforms PQ in recall**
- **Zero indexing time** (online processing)

**Implementation Plan**:
1. Random rotation preprocessing
2. Beta distribution coordinate transformation
3. Optimal scalar quantizers per coordinate
4. Two-stage: MSE quantizer + 1-bit QJL transform
5. Online quantization API

**Files to Create**:
- `src/dense/quantization/turboquant.rs` - TurboQuant implementation
- `src/dense/quantization/rotation.rs` - Random rotation
- `src/dense/quantization/qjl.rs` - Quantized JL transform

**Estimated Impact**: Online capability, better recall, near-optimal compression

---

## Phase 2: Enhancements (Next)

### 4. RaBitQ Integration

**Priority**: ⭐⭐⭐⭐

**Why**: Theoretical error bounds, proven optimality

**Implementation**: Enhance existing quantization with RaBitQ principles

---

### 5. GATE (Adaptive Graph Enhancement)

**Priority**: ⭐⭐⭐⭐

**Why**: 1.2-2.0× query speedup for graph methods

**Implementation**: 
- Add as optional enhancement to HNSW/SNG
- Hub node extraction
- Two-tower model for entry point selection
- Navigation graph index

**Note**: Requires ML infrastructure (can be optional feature)

---

## Phase 3: Research/Experimental

### 6. Probabilistic Routing
- Alternative to greedy search
- Better for diverse query patterns

### 7. Learned Indexes
- End-to-end learning
- Requires ML infrastructure

---

## Implementation Order

### Sprint 1: OPT-SNG
- **Goal**: 5-15× construction speedup
- **Time**: 1-2 weeks
- **Impact**: Immediate practical benefit

### Sprint 2: SAQ
- **Goal**: 80% better quantization, 80× faster encoding
- **Time**: 1-2 weeks
- **Impact**: Improves IVF-PQ and Anisotropic VQ + k-means (SCANN) significantly

### Sprint 3: TurboQuant
- **Goal**: Online quantization, better recall
- **Time**: 1-2 weeks
- **Impact**: Enables streaming/dynamic datasets

### Sprint 4: Integration & Testing
- **Goal**: Integrate all improvements
- **Time**: 1 week
- **Impact**: Unified, optimized ANN system

---

## Expected Overall Improvements

After implementing Phase 1:

| Metric | Improvement |
|--------|-------------|
| Construction Speed | **5-15× faster** (OPT-SNG) |
| Quantization Accuracy | **80% better** (SAQ) |
| Encoding Speed | **80× faster** (SAQ) |
| Online Capability | **Yes** (TurboQuant) |
| Search Performance | **1.2-2.0× faster** (GATE, optional) |
| Parameter Tuning | **Automatic** (OPT-SNG) |

---

## Technical Considerations

### Dependencies
- **PCA**: May need linear algebra crate (or implement ourselves)
- **ML Models**: GATE requires neural network infrastructure (optional)
- **Random Rotation**: Can use existing `rand` crate

### Integration Points
- OPT-SNG: Enhance `src/dense/hnsw/construction.rs`
- SAQ: Enhance `src/dense/ivf_pq/pq.rs` and `src/dense/scann/quantization.rs`
- TurboQuant: New module, integrate with IVF-PQ and Anisotropic VQ + k-means (SCANN)

### Testing Strategy
1. Unit tests for each algorithm
2. Benchmark against existing implementations
3. Validate theoretical guarantees
4. Performance regression tests

---

## References

- OPT-SNG: https://arxiv.org/abs/2509.15531
- SAQ: https://arxiv.org/abs/2509.12086
- TurboQuant: https://arxiv.org/abs/2504.19874
- GATE: https://arxiv.org/abs/2506.15986
- RaBitQ: https://arxiv.org/abs/2405.12497
