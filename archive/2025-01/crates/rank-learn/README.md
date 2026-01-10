# rank-learn (DEPRECATED)

⚠️ **This crate has been merged into `rank-soft`** (January 2025)

All Learning to Rank functionality (LambdaRank, Ranking SVM, Neural LTR) is now available in [`rank-soft`](../rank-soft/README.md).

## Migration Guide

### Rust

```rust
// Old (rank-learn)
use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};

// New (rank-soft)
use rank_soft::{LambdaRankTrainer, ndcg_at_k};
```

### Python

```python
# Old (rank-learn)
import rank_learn

# New (rank-soft)
import rank_soft

# All APIs are identical - just change the import
trainer = rank_soft.LambdaRankTrainer()
```

## What Changed

- **LambdaRank** → `rank_soft::LambdaRankTrainer`
- **Ranking SVM** → `rank_soft::RankingSVMTrainer`
- **Neural LTR** → `rank_soft::NeuralLTRModel`
- **NDCG computation** → `rank_soft::ndcg_at_k`

All APIs are identical - only the import path changed.

## Why the Merge?

After research and analysis, we determined that:
1. LambdaRank and Ranking SVM are gradient computation algorithms (like other gradients in `rank-soft`)
2. No heavy dependencies existed (XGBoost/LightGBM bindings were never implemented)
3. Matches industry patterns (LightGBM/XGBoost integrate ranking objectives)
4. Simplifies the API - one crate for all ranking training needs

See [`rank-soft`](../rank-soft/README.md) for complete documentation.

---

**Note:** This directory is kept for historical reference. All new code should use `rank-soft`.
