# Ecosystem Integration Implementation Plan
## Based on RUST_ML_ECOSYSTEM_ALIGNMENT.md Recommendations

**Date:** January 2025  
**Status:** Implementation In Progress

---

## Overview

This document tracks the implementation of ecosystem integration recommendations from `RUST_ML_ECOSYSTEM_ALIGNMENT.md`. Each section includes implementation status, code changes, and next steps.

---

## 1. ONNX Runtime Integration

### Status: ⚠️ Pending ort 2.0 Stable Release

**Current State:**
- Code exists in `crates/rank-rerank/src/crossencoder/ort.rs`
- Feature-gated with `#[cfg(feature = "ort")]`
- Commented out in `Cargo.toml`: `# ort = { version = "2.0.0-rc.10", optional = true }`
- Python bindings commented out in `rank-rerank-python/src/lib.rs`

**Research Finding:**
- `ort` 2.0 is still in release candidate (rc.10) as of January 2025
- Not yet API-stable, but described as production-ready
- ONNX Runtime itself is at 1.22+ (stable)

**Decision:**
- **Option A (Conservative):** Wait for stable 2.0.0 release
- **Option B (Pragmatic):** Use rc.10 with clear documentation about API stability
- **Recommendation:** Option B - rc.10 is production-ready, document API stability risk

**Implementation Steps:**
1. ✅ Research ort 2.0 status (complete)
2. ⏳ Uncomment ort dependency in `Cargo.toml` with rc.10
3. ⏳ Enable ort feature flag
4. ⏳ Test cross-encoder ONNX inference
5. ⏳ Add comprehensive ONNX examples
6. ⏳ Document API stability considerations

---

## 2. Candle Integration Deep Dive

### Status: 🚧 In Progress

**Current State:**
- `rank-soft` has optional `candle-core` dependency
- Example exists but only converts Vec<f64> ↔ Tensor (not true integration)
- No Candle integration in `rank-rerank` for MaxSim operations

**Implementation Plan:**

#### 2.1 Candle Tensor Operations in rank-soft

**Goal:** Direct Candle tensor support for soft_rank, spearman_loss

**Changes Needed:**
1. Add Candle tensor implementations:
   - `soft_rank_candle(tensor: &Tensor, regularization: f64) -> Tensor`
   - `spearman_loss_candle(pred: &Tensor, target: &Tensor, regularization: f64) -> Tensor`
2. Leverage Candle's autograd (if available)
3. Support GPU tensors (Device::Cuda, Device::Metal)

**Files to Modify:**
- `crates/rank-soft/src/lib.rs` - Add Candle module
- `crates/rank-soft/src/candle.rs` - New file for Candle implementations
- `crates/rank-soft/examples/candle_training.rs` - Update to use real Candle tensors

#### 2.2 Candle Integration in rank-rerank

**Goal:** Use Candle tensors for MaxSim operations (optional, for GPU acceleration)

**Approach:**
- Keep existing SIMD implementation (CPU, well-optimized)
- Add optional Candle-based MaxSim for GPU acceleration
- Feature flag: `candle` enables Candle tensor operations

**Files to Create/Modify:**
- `crates/rank-rerank/src/candle.rs` - New module for Candle MaxSim
- `crates/rank-rerank/Cargo.toml` - Add candle-core dependency (optional)
- `crates/rank-rerank/examples/candle_maxsim.rs` - New example

**Benefits:**
- GPU acceleration for large batches
- Unified tensor API
- Leverage Candle's optimizations

---

## 3. Burn Integration

### Status: 📋 Planned

**Current State:**
- Feature flag exists but empty: `burn = []`
- Example exists but placeholder
- No actual Burn dependencies

**Implementation Plan:**

#### 3.1 Add Burn Dependencies

**Dependencies Needed:**
```toml
burn = { version = "0.19", optional = true }
burn-tensor = { version = "0.19", optional = true }
```

#### 3.2 Implement Burn Tensor Operations

**Goal:** Support Burn's generic Backend trait

**Approach:**
- Generic over `B: Backend` trait
- Works with any Burn backend (CUDA, Metal, Vulkan, WebGPU, CPU)
- Leverage Burn's autograd automatically

**Files to Create/Modify:**
- `crates/rank-soft/src/burn.rs` - New file for Burn implementations
- `crates/rank-soft/Cargo.toml` - Add Burn dependencies
- `crates/rank-soft/examples/burn_training.rs` - Update with real Burn code

**Benefits:**
- Multi-backend support (future-proof)
- Training + inference support
- Native quantization support

---

## 4. Vector Database Integration Examples

### Status: 📋 Planned

**Goal:** Provide production-ready examples integrating rank-rank with vector databases

#### 4.1 Qdrant Integration Example

**Files to Create:**
- `crates/rank-rerank/examples/qdrant_integration.rs`
- `docs/ECOSYSTEM_INTEGRATION.md` - Add Qdrant section

**Example Workflow:**
1. Store embeddings in Qdrant
2. Retrieve top-K candidates (dense retrieval)
3. Rerank with rank-rerank (MaxSim or cross-encoder)
4. Return final results

#### 4.2 usearch Integration Example

**Files to Create:**
- `crates/rank-retrieve/examples/usearch_ann.rs`
- Document HNSW integration for dense retrieval

**Example Workflow:**
1. Build HNSW index with usearch
2. Approximate nearest neighbor search
3. Rerank results with rank-rerank

#### 4.3 End-to-End RAG Pipeline

**Files to Create:**
- `examples/rag_pipeline_complete.rs`
- `docs/ECOSYSTEM_INTEGRATION.md` - Complete RAG workflow

**Components:**
- Vector DB (Qdrant/usearch) for dense retrieval
- rank-retrieve for BM25/sparse retrieval
- rank-fusion for combining results
- rank-rerank for final reranking
- rank-eval for evaluation

---

## 5. GPU Acceleration

### Status: 📋 Planned

**Current State:**
- Plan exists in `docs/GPU_ACCELERATION_PLAN.md`
- No implementation yet

**Implementation Strategy:**
- **Phase 1:** Candle GPU support (CUDA, Metal)
- **Phase 2:** Burn GPU support (multi-backend)
- **Phase 3:** Benchmark and optimize

**Priority:** Medium (CPU SIMD is already fast, GPU helps with large batches)

---

## 6. Quantization Support

### Status: 📋 Planned

**Current State:**
- ONNX export supports quantization (Python-side)
- No native Rust quantization

**Implementation Plan:**
1. Add INT8 quantization for embeddings
2. Add FP16 support
3. Integrate with Candle's quantization (if applicable)
4. Document quantization workflows

**Priority:** Low (ONNX quantization covers most use cases)

---

## 7. Documentation & Examples

### Status: 🚧 In Progress

**Files to Create/Update:**
- `docs/ECOSYSTEM_INTEGRATION.md` - Comprehensive integration guide
- `docs/CANDLE_INTEGRATION.md` - Candle-specific examples
- `docs/BURN_INTEGRATION.md` - Burn-specific examples
- `docs/VECTOR_DB_INTEGRATION.md` - Vector database examples
- Update READMEs with ecosystem integration sections

---

## Implementation Priority

### High Priority (Next 1-2 Weeks)
1. ✅ Research ort 2.0 status
2. ⏳ Enable ort rc.10 integration (with documentation)
3. ⏳ Candle tensor operations in rank-soft
4. ⏳ Candle MaxSim example in rank-rerank
5. ⏳ Ecosystem integration documentation

### Medium Priority (Next Month)
1. Burn integration in rank-soft
2. Qdrant integration example
3. usearch integration example
4. End-to-end RAG pipeline example

### Low Priority (Future)
1. GPU acceleration (Candle/Burn)
2. Native quantization
3. WASM completion
4. no_std support

---

## Next Steps

1. **Immediate:** Start with Candle integration (highest impact, already partially there)
2. **Short-term:** Enable ort rc.10, add vector DB examples
3. **Medium-term:** Complete Burn integration, GPU acceleration
4. **Long-term:** Quantization, edge deployment

---

**Last Updated:** January 2025  
**Next Review:** After completing high-priority items

