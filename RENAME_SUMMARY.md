# Rename Summary & LTR Answer

## ✅ Completed in rank-rank

1. **Updated README.md** - New repository names
2. **Updated .cursor/rules/shared-base.mdc** - New names in rules
3. **Created rank-retrieve** - Basic structure with README, Cargo.toml, lib.rs
4. **Created LTR_ANALYSIS.md** - Explains where LTR fits
5. **Updated REVIEW_ANALYSIS.md** - Reflects new structure

## 📋 Still Needed (Git Repo Renames)

Since `rank-refine` and `rank-relax` are separate git repositories, you need to:

### Option A: Rename Directories (Simple)

```bash
cd /Users/arc/Documents/dev/_rank-rank
mv rank-refine rank-rerank
mv rank-relax rank-soft
```

Then update internal references in each repo (see `RENAME_INSTRUCTIONS.md`).

### Option B: Clone with New Names (If you have remotes)

If these are cloned from GitHub, you might want to:
1. Clone with new names
2. Update remotes
3. Push to new locations

## LTR Answer: rank-learn

**Learning to Rank (LambdaRank, XGBoost) should be in `rank-learn`:**

| Crate | Purpose | Example |
|-------|---------|---------|
| **rank-soft** | Differentiable operations | Soft ranking, ListNet loss (mathematical primitives) |
| **rank-learn** | Full LTR frameworks | LambdaRank, XGBoost with ranking objective (complete ML systems) |

**Why separate:**
- `rank-soft` = lightweight math (like NumPy)
- `rank-learn` = complete ML algorithms (like scikit-learn)
- Different dependencies (rank-soft is light, rank-learn needs XGBoost/LightGBM)
- Different users (custom neural models vs standard LTR)

See `LTR_ANALYSIS.md` for full analysis.

## Final Structure

```
rank-retrieve/    # Stage 1: BM25, dense ANN, sparse
rank-fusion/      # Combine lists
rank-rerank/      # Stage 2: MaxSim, cross-encoder (renamed from rank-refine)
rank-soft/        # Differentiable ops (renamed from rank-relax)
rank-learn/       # LTR frameworks (LambdaRank, XGBoost) - TO BE CREATED
rank-eval/        # Evaluation
rank-sparse/      # Utilities
```

## Next Steps

1. **Rename git repos** (see `RENAME_INSTRUCTIONS.md`)
2. **Implement rank-retrieve** (BM25, dense ANN, sparse)
3. **Create rank-learn** (LambdaRank, XGBoost integration)
4. **Document rank-sparse** (add to README or exclude from scripts)

