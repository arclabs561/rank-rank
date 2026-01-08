# Crate Breakdown Verification

## Current Breakdown Analysis

### ✅ Correct Breakdown

| Crate | Purpose | Pipeline Stage | Status | Verification |
|-------|---------|----------------|--------|--------------|
| **rank-retrieve** | First-stage retrieval (BM25, dense ANN, sparse) | Stage 1 (10M → 1000) | ✅ Correct | Standard IR terminology, matches research |
| **rank-fusion** | Combine ranked lists (RRF, CombSum, etc.) | Post-retrieval | ✅ Correct | Standard fusion terminology |
| **rank-rerank** | Reranking and late interaction (MaxSim, cross-encoder) | Stage 2 (1000 → 100) | ✅ Correct | Standard IR term "rerank" |
| **rank-soft** | Differentiable ranking operations | Training-time | ✅ Correct | Standard ML term "soft ranking" |
| **rank-learn** | Learning to Rank frameworks (LambdaRank, XGBoost) | Training-time | ✅ Correct | Standard LTR terminology |
| **rank-eval** | Ranking evaluation metrics | Post-hoc | ✅ Correct | Standard evaluation terminology |

### Boundary Verification

#### ✅ Clear Boundaries

1. **rank-retrieve vs rank-rerank**
   - **rank-retrieve**: Fast, broad retrieval (10M → 1000) - basic scoring (BM25, cosine)
   - **rank-rerank**: Precise, narrow reranking (1000 → 100) - advanced scoring (MaxSim, cross-encoder)
   - **Boundary**: Speed vs precision trade-off, different scale requirements
   - **Status**: ✅ Correct separation

2. **rank-soft vs rank-learn**
   - **rank-soft**: Mathematical primitives (soft ranking, differentiable sorting, loss functions)
   - **rank-learn**: Complete ML systems (LambdaRank, XGBoost, neural LTR models)
   - **Boundary**: Building blocks vs complete solutions
   - **Status**: ✅ Correct separation (rank-learn uses rank-soft internally)

3. **rank-fusion vs rank-rerank**
   - **rank-fusion**: Combines multiple ranked lists (horizontal fusion)
   - **rank-rerank**: Reranks a single list (vertical refinement)
   - **Boundary**: Different operations, can be used together
   - **Status**: ✅ Correct separation

4. **rank-eval vs others**
   - **rank-eval**: Evaluation metrics (post-hoc analysis)
   - **Others**: Production operations (retrieval, fusion, reranking, training)
   - **Boundary**: Evaluation vs operations
   - **Status**: ✅ Correct separation

### Terminology Alignment

#### ✅ Standard IR/ML Terms

- **"retrieve"**: Standard IR term for first-stage retrieval
- **"fusion"**: Standard IR term for combining ranked lists
- **"rerank"**: Standard IR term for second-stage refinement (confirmed by research)
- **"soft ranking"**: Standard ML term for differentiable ranking (confirmed by research)
- **"learning to rank"**: Standard ML term for LTR frameworks
- **"eval"**: Standard term for evaluation

### Pipeline Flow Verification

```
10M docs → 1000 candidates → 100 candidates → 10 results
    │            │                 │              │
    ▼            ▼                 ▼              ▼
[retrieve]   [fusion]          [rerank]        [user]
  (fast)    (combine)         (precise)       (final)
```

**Status**: ✅ Correct flow matches standard IR pipeline

### Research Alignment

All crate names and boundaries align with:
- Standard IR terminology (retrieve, rerank, fusion, eval)
- Standard ML terminology (soft ranking, learning to rank)
- Latest research papers (2024-2025) use these exact terms
- Industry practice (Elasticsearch, OpenSearch, etc.)

## Conclusion

✅ **Breakdown is correct** - All crates have:
- Clear, standard terminology
- Well-defined boundaries
- Correct pipeline placement
- Research-backed naming

No changes needed to the breakdown structure.

