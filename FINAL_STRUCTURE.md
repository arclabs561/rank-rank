# Final rank-* Repository Structure

## Complete Pipeline

```
rank-retrieve/    # Stage 1: 10M docs → 1000 candidates
rank-fusion/      # Combine lists from multiple retrievers  
rank-rerank/      # Stage 2: 1000 → 100 candidates (MaxSim, cross-encoder)
rank-soft/        # Differentiable ranking operations (training-time)
rank-learn/       # Learning to Rank frameworks (LambdaRank, XGBoost, etc.)
rank-eval/        # Evaluation metrics
rank-sparse/      # Sparse vector utilities
```

## Repository Purposes

| Repository | Purpose | Stage | Status |
|------------|---------|-------|--------|
| **rank-retrieve** | First-stage retrieval (BM25, dense ANN, sparse) | Stage 1 | 🚧 Created (needs implementation) |
| **rank-fusion** | Combine ranked lists (RRF, ISR, CombMNZ, etc.) | Post-retrieval | ✅ Complete |
| **rank-rerank** | Reranking (MaxSim/ColBERT, cross-encoder) | Stage 2 | ✅ Complete (renamed from rank-refine) |
| **rank-soft** | Differentiable ranking ops (soft ranking, losses) | Training-time | ✅ Complete (renamed from rank-relax) |
| **rank-learn** | Learning to Rank (LambdaRank, XGBoost, etc.) | Training-time | 📋 Planned |
| **rank-eval** | Evaluation metrics (NDCG, MAP, MRR, etc.) | Post-hoc | ✅ Complete |
| **rank-sparse** | Sparse vector utilities (dot product, pruning) | Utility | ✅ Exists (needs documentation) |

## LTR (Learning to Rank) Placement

### Question: Where does XGBoost/LambdaRank go?

**Answer: `rank-learn`** (separate from `rank-soft`)

### Why Separate?

**rank-soft** (Differentiable Operations):
- Mathematical primitives
- Soft ranking, differentiable sorting
- Loss functions (ListNet, ListMLE, Spearman)
- Framework-agnostic (PyTorch, JAX, Rust ML)
- **Lightweight**: Just math, no heavy dependencies

**rank-learn** (LTR Frameworks):
- Complete ML systems
- LambdaRank, LambdaMART
- XGBoost/LightGBM integration
- Neural LTR models
- **Heavy dependencies**: XGBoost, LightGBM bindings
- Uses `rank-soft` for differentiable operations

### Boundary

- **rank-soft**: Provides building blocks (differentiable operations)
- **rank-learn**: Provides complete solutions (full LTR algorithms)

**Analogy:**
- `rank-soft` = NumPy (mathematical primitives)
- `rank-learn` = scikit-learn (complete ML algorithms)

## Renames Completed

✅ **rank-refine → rank-rerank**
- Standard IR term
- Matches pipeline documentation

✅ **rank-relax → rank-soft**
- More common in papers
- Clearer purpose

## New Repositories

✅ **rank-retrieve** (created)
- Structure created
- Needs implementation

📋 **rank-learn** (planned)
- For LTR frameworks
- See `LTR_ANALYSIS.md` for details

## Next Steps

1. **Rename git repos** (see `RENAME_INSTRUCTIONS.md`)
2. **Implement rank-retrieve** (BM25, dense ANN, sparse)
3. **Create rank-learn** (LambdaRank, XGBoost integration)
4. **Document rank-sparse** (add to README or exclude)

