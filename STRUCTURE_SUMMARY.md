# rank-* Repository Structure Summary

## Complete Structure

| Repository | Purpose | Pipeline Stage | Status | Notes |
|------------|---------|----------------|--------|-------|
| **rank-retrieve** | First-stage retrieval (BM25, dense ANN, sparse) | Stage 1 (10M → 1000) | 🚧 Created | Needs implementation |
| **rank-fusion** | Combine ranked lists (RRF, ISR, etc.) | Post-retrieval | ✅ Complete | Well-documented |
| **rank-rerank** | Reranking (MaxSim, cross-encoder) | Stage 2 (1000 → 100) | ✅ Complete | Renamed from rank-refine |
| **rank-soft** | Differentiable ranking ops | Training-time | ✅ Complete | Renamed from rank-relax |
| **rank-learn** | Learning to Rank (LambdaRank, XGBoost) | Training-time | 📋 Planned | See LTR_ANALYSIS.md |
| **rank-eval** | Evaluation metrics | Post-hoc | ✅ Complete | NDCG, MAP, MRR, etc. |
| **rank-sparse** | Sparse vector utilities | Utility | ⚠️ Exists | Needs documentation |

## Pipeline Flow

```
10M docs → 1000 → 100 → 10 results
    │        │      │      │
    ▼        ▼      ▼      ▼
[retrieve] [rerank] [cross] [user]
           [fusion]  [encoder]
```

## LTR (Learning to Rank) Placement

### Question: Where does XGBoost/LambdaRank go?

**Answer: `rank-learn`** (separate crate)

### Separation Logic

| Crate | What It Does | Dependencies | Users |
|-------|--------------|--------------|-------|
| **rank-soft** | Differentiable operations (soft ranking, losses) | Lightweight (just math) | People building custom neural ranking models |
| **rank-learn** | Full LTR frameworks (LambdaRank, XGBoost) | Heavy (XGBoost, LightGBM bindings) | People who want standard LTR algorithms |

**Analogy:**
- `rank-soft` = NumPy (mathematical primitives)
- `rank-learn` = scikit-learn (complete ML algorithms)

**Boundary:**
- `rank-soft` provides building blocks (ListNet/ListMLE losses are differentiable operations)
- `rank-learn` provides complete solutions (LambdaRank uses rank-soft internally)

## Renames Completed

✅ **rank-refine → rank-rerank**
- Standard IR term
- Matches pipeline docs

✅ **rank-relax → rank-soft**
- More common in papers
- Clearer purpose

## New Repositories

✅ **rank-retrieve** (created)
- Structure: README, Cargo.toml, lib.rs
- Status: Needs implementation (BM25, dense ANN, sparse)

📋 **rank-learn** (planned)
- For LambdaRank, XGBoost, LightGBM integration
- See `LTR_ANALYSIS.md` for detailed plan

## What's Been Updated

✅ `README.md` - New names
✅ `.cursor/rules/shared-base.mdc` - New names
✅ `rank-retrieve/` - Created structure
✅ Documentation - Updated references

## What Still Needs Doing

1. **Rename git repos** (rank-refine → rank-rerank, rank-relax → rank-soft)
   - See `RENAME_INSTRUCTIONS.md` for commands
2. **Update internal references** in renamed repos
3. **Implement rank-retrieve** (BM25, dense ANN, sparse)
4. **Create rank-learn** (LambdaRank, XGBoost integration)
5. **Document rank-sparse** (add to README or exclude from scripts)

