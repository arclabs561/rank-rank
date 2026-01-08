# Deep Review: rank-rank Workspace Analysis

## Executive Summary

After deep analysis of the codebase, documentation, and ranking pipeline, here are the refined recommendations and identified gaps.

## Current State Analysis

### Existing Repositories

1. **rank-retrieve**: First-stage retrieval (BM25, dense ANN, sparse)
   - **Scope**: Stage 1 (10M docs → 1000 candidates)
   - **Status**: 🚧 Created (needs implementation)

2. **rank-fusion**: Combines ranked lists from multiple retrievers (RRF, ISR, CombMNZ, etc.)
   - **Scope**: Post-retrieval fusion (assumes you have results)
   - **Status**:  Complete and well-documented

3. **rank-rerank**: Reranking and late interaction scoring (MaxSim/ColBERT) + cross-encoder trait
   - **Scope**: Stage 2 (1000 → 100 candidates)
   - **Status**:  MaxSim complete, cross-encoder trait exists but disabled
   - **Note**: Renamed from rank-refine
   - **Gap**: Cross-encoder implementation disabled (waiting for ort 2.0)

4. **rank-soft**: Differentiable ranking/sorting for ML training
   - **Scope**: Training-time operations (soft ranking, Spearman loss)
   - **Status**:  Complete
   - **Note**: Renamed from rank-relax

5. **rank-learn**: Learning to Rank frameworks (LambdaRank, XGBoost, etc.)
   - **Scope**: Complete LTR algorithms
   - **Status**: 📋 Planned (see LTR_ANALYSIS.md)

6. **rank-eval**: Evaluation metrics (NDCG, MAP, MRR, etc.)
   - **Scope**: Post-hoc evaluation
   - **Status**:  Complete

7. **rank-sparse**: Sparse vector utilities (dot product, pruning)
   - **Scope**: Utility library for sparse operations
   - **Status**:  Exists but undocumented in rank-rank
   - **Issue**: Not mentioned in README, not included in scripts

### Documented Pipeline (from rank-refine)

```
10M docs → 1000 candidates → 100 candidates → 10 results
    │            │                 │              │
    ▼            ▼                 ▼              ▼
[Dense ANN]  [MaxSim]        [Cross-encoder]  [User]
  (fast)      (precise)        (accurate)
```

**Current Coverage**:
-  Stage 1: rank-retrieve (created, needs implementation)
-  Stage 2: MaxSim reranking (rank-rerank, renamed from rank-refine)
-  Stage 3: Cross-encoder trait exists but disabled (in rank-rerank)

## Critical Gaps

### 1. rank-retrieve: First-Stage Retrieval ( CREATED, needs implementation)

**Problem**: The pipeline assumes you have retrieval results, but no library provides:
- BM25 retrieval (inverted index, term matching)
- Dense ANN search (HNSW, IVF, etc.)
- Sparse retrieval (lexical matching, learned sparse)

**Evidence**:
- `rank-fusion` README: "Combine results from multiple retrievers (BM25, dense, sparse)"
- `rank-refine` docs: Pipeline diagram shows "Dense ANN" as first stage
- `rank-sparse` exists but only provides utilities, not retrieval

**Recommendation**: Create `rank-retrieve` with:
- BM25 implementation (inverted index)
- Dense ANN interface (delegate to existing libraries like `hnsw` or `faiss`)
- Sparse retrieval (lexical matching using rank-sparse utilities)
- Unified API for all three retrieval types

**Why Critical**: Without this, users must bring their own retrieval, making the ecosystem incomplete.

### 2. Cross-Encoder Implementation (MEDIUM PRIORITY)

**Problem**: `rank-rerank` (formerly rank-refine) has a cross-encoder trait but implementation is disabled:
```rust
// #[cfg(feature = "ort")]
// pub mod crossencoder_ort;  // TODO: Enable when ort 2.0 is stable
```

**Evidence**: 
- Trait exists: `rank-rerank/rank-rerank/src/crossencoder.rs` (renamed from rank-refine)
- Decision guide mentions cross-encoders but notes they're disabled
- Pipeline diagram shows cross-encoder as final stage

**Recommendation**: 
- **Option A**: Enable in rank-rerank when ort 2.0 is stable  (chosen)
- **Option B**: Create separate crate for model-based reranking (not needed)
- **Option C**: Keep trait in rank-rerank, provide ONNX/PyTorch bindings separately

**Why Medium**: MaxSim covers most use cases (100-1000 candidates). Cross-encoders are only needed for final top-10 refinement.

### 3. rank-optimize: Score Calibration (LOW PRIORITY)

**Problem**: No library for:
- Score normalization across retrievers (mentioned as fragile in rank-fusion)
- Threshold optimization (when to stop retrieval)
- Score calibration (Platt scaling, isotonic regression)

**Evidence**:
- `rank-fusion` README: "Normalization is fragile and requires tuning" (why RRF exists)
- But some algorithms (combsum, combmnz) still need normalization

**Recommendation**: Create `rank-optimize` for:
- Score normalization methods (min-max, z-score, quantile)
- Threshold optimization (precision/recall trade-offs)
- Calibration (Platt scaling for binary relevance)

**Why Low**: RRF avoids normalization, so this is only needed for score-based fusion.

### 4. rank-learn: Learning to Rank (MEDIUM PRIORITY)

**Problem**: 
- `rank-soft` has ListNet/ListMLE loss functions but not full LTR frameworks
- No LambdaRank, LambdaMART implementations
- No XGBoost/LightGBM integration for ranking

**Evidence**:
- `rank-soft` focuses on differentiable operations (mathematical primitives)
- LTR algorithms (LambdaRank, XGBoost) are complete ML systems, not just operations
- Industry standard: Libraries like `allRank` separate LTR from differentiable ops

**Recommendation**: Create `rank-learn` for:
- LambdaRank, LambdaMART
- XGBoost/LightGBM integration for ranking
- Neural LTR models
- Uses `rank-soft` for differentiable operations

**Why Medium**: Important for users who want to train ranking models, but not blocking for retrieval/reranking pipeline.

### 5. rank-analyze: Explainability & Bias (LOW PRIORITY)

**Problem**: 
- `rank-fusion` mentions explainability but no dedicated analysis tools
- No bias/fairness metrics
- No visualization of ranking decisions

**Evidence**:
- `rank-fusion` has `explain` module mentioned in docs
- User personas mention need for explainability (debugging, support tickets)

**Recommendation**: Create `rank-analyze` for:
- Explainability (why did this doc rank high?)
- Bias metrics (demographic parity, equalized odds)
- Fairness constraints (group fairness in ranking)
- Visualization tools

**Why Low**: Nice to have, not blocking for core functionality.

## Missing from Documentation

### rank-sparse Status

**Current**: Exists but undocumented in rank-rank
- Has sparse vector struct and dot product
- Has Python bindings
- Not mentioned in README.md
- Not included in `inspect_all_rank_readmes.sh`

**Recommendation**: 
1. **If it's a utility for rank-retrieve**: Document it as such, keep it minimal
2. **If it's a standalone library**: Add to README, include in scripts
3. **If it's experimental**: Move to `archive/` or document as experimental

**Action**: Check if rank-sparse is used by other repos or standalone.

## Script Issues

### 1. Hardcoded Repo Lists

**Problem**: `inspect_all_rank_readmes.sh` has hardcoded paths:
```bash
readmes=(
    "$REPO_ROOT/rank-fusion/rank-fusion/README.md"
    "$REPO_ROOT/rank-fusion/README.md"
    # ... hardcoded list
)
```

**Impact**: 
- Breaks when repos are added/removed
- Doesn't discover rank-sparse
- Inconsistent with other scripts that discover dynamically

**Fix**: Discover READMEs dynamically:
```bash
for repo in "$RANK_RANK_DIR"/rank-*; do
    if [ -d "$repo" ] && [ "$(basename "$repo")" != "rank-rank" ]; then
        # Find README.md in repo
        find "$repo" -name "README.md" -type f
    fi
done
```

### 2. Path Examples in README

**Problem**: README shows introspection examples with wrong paths:
```bash
python3 scripts/introspect_rank_eval.py ../rank-eval
```

**Reality**: Repos are siblings, so should be:
```bash
python3 scripts/introspect_rank_eval.py rank-eval
# or
python3 scripts/introspect_rank_eval.py ./rank-eval
```

**Fix**: Update all path examples in README.md

## Refined Recommendations

### Immediate (Fix Scripts & Docs)

1.  **Fixed**: Path resolution in sync/verify scripts
2. **Fix**: Make `inspect_all_rank_readmes.sh` discover READMEs dynamically
3. **Fix**: Update README path examples to use correct relative paths
4. **Decide**: Document rank-sparse or exclude from scripts

### Short-term (Complete Pipeline)

1. **Create rank-retrieve**: First-stage retrieval (BM25, dense ANN, sparse)
   - **Priority**: HIGH (completes the pipeline)
   - **Scope**: Inverted index for BM25, ANN interface for dense, lexical for sparse
   - **Dependencies**: Can use existing crates (tantivy for BM25, hnsw for ANN)

2. **Enable cross-encoder**: When ort 2.0 is stable
   - **Priority**: MEDIUM (MaxSim covers most cases)
   - **Location**: rank-refine (trait already exists)

### Long-term (Nice to Have)

1. **rank-optimize**: Score calibration and normalization
   - **Priority**: LOW (RRF avoids need)
   - **Use case**: Score-based fusion algorithms

2. **rank-analyze**: Explainability and bias metrics
   - **Priority**: LOW (not blocking)
   - **Use case**: Debugging, fairness audits

## Architecture Questions

### Should rank-retrieve include indexing?

**Option A**: Full indexing (inverted index, HNSW index)
- **Pros**: Complete solution, users don't need other libraries
- **Cons**: Large scope, duplicates existing libraries (tantivy, hnsw)

**Option B**: Interface + delegation
- **Pros**: Smaller scope, leverages existing libraries
- **Cons**: Adds dependencies, less control

**Recommendation**: **Option B** - Provide unified API that delegates to:
- `tantivy` for BM25 (or vendored minimal version)
- `hnsw` or `faiss` for dense ANN
- Custom sparse retrieval using rank-sparse

### Should cross-encoders be in rank-refine or separate?

**Current**: Trait in rank-refine, implementation disabled

**Option A**: Keep in rank-refine
- **Pros**: All reranking in one place
- **Cons**: Adds heavy dependencies (ONNX, model loading)

**Option B**: Separate `rank-rerank` crate
- **Pros**: Keeps rank-refine lightweight, clear separation
- **Cons**: Another repo to maintain

**Recommendation**: **Option A** - Keep trait in rank-refine, enable implementation when ready. Cross-encoders are conceptually part of reranking.

## Summary: Missing Repos by Priority

### Must Have

1. **rank-retrieve**: First-stage retrieval (BM25, dense ANN, sparse)
   - **Why**: Completes the pipeline, currently users must bring their own
   - **Complexity**: Medium (can delegate to existing libraries)

### Should Have

2. **Cross-encoder implementation**: Enable in rank-refine
   - **Why**: Final stage of documented pipeline
   - **Complexity**: Low (trait exists, just need implementation)

### Nice to Have

3. **rank-optimize**: Score calibration
   - **Why**: Needed for score-based fusion (combsum, combmnz)
   - **Complexity**: Low (statistical methods)

4. **rank-analyze**: Explainability and bias
   - **Why**: User personas mention need for debugging
   - **Complexity**: Medium (requires research into bias metrics)

## Next Steps

1. **Immediate**: Fix script issues (dynamic discovery, path examples)
2. **Short-term**: Decide on rank-sparse status, create rank-retrieve
3. **Long-term**: Enable cross-encoders, consider rank-optimize/rank-analyze

