# Learning to Rank (LTR) Organization Analysis

## Current State

**rank-soft** (formerly rank-relax) has:
- ListNet loss function
- ListMLE loss function
- Differentiable ranking operations (soft ranking, Spearman loss)

**But these are NOT full LTR frameworks** - they're differentiable operations that can be used within LTR training.

## What is LTR?

**Learning to Rank (LTR)** is a category of machine learning algorithms that train models to rank items. Key algorithms:

1. **Pointwise**: Treat ranking as regression/classification
   - Example: XGBoost with ranking objective

2. **Pairwise**: Learn to compare pairs
   - Example: RankNet, LambdaRank

3. **Listwise**: Optimize entire ranking lists
   - Example: ListNet, ListMLE, LambdaMART

## Where Does LTR Fit?

### Option 1: rank-learn (Recommended)

**Create `rank-learn` for full LTR frameworks:**

```
rank-learn/
├── lambdarank/     # LambdaRank, LambdaMART
├── xgboost/        # XGBoost integration for ranking
├── lightgbm/       # LightGBM integration
├── neural/         # Neural LTR models
└── losses/         # LTR loss functions (re-export from rank-soft)
```

**Pros:**
- Clear separation: differentiable ops vs full LTR
- Can integrate with existing libraries (XGBoost, LightGBM)
- Matches user mental model ("I need to train a ranking model")

**Cons:**
- Another crate to maintain
- Some overlap with rank-soft (loss functions)

### Option 2: Keep in rank-soft

**Expand rank-soft to include full LTR:**

```
rank-soft/
├── differentiable/  # Current: soft ranking, differentiable ops
├── losses/          # ListNet, ListMLE (current)
└── ltr/             # NEW: LambdaRank, XGBoost integration
```

**Pros:**
- Everything ranking-related in one place
- Less crate management

**Cons:**
- Mixes concerns (differentiable ops vs full ML frameworks)
- XGBoost/LightGBM are heavy dependencies
- Doesn't match user mental model (LTR is different from differentiable ops)

### Option 3: Out of Scope

**Users bring their own XGBoost/LightGBM**

**Pros:**
- No maintenance burden
- Users can use any LTR library

**Cons:**
- Missing piece of ranking ecosystem
- Users have to figure out integration themselves

## Recommendation: rank-learn

**Why:**
1. **Different concerns**: Differentiable operations (rank-soft) are mathematical primitives. LTR (rank-learn) is complete ML systems.
2. **Different dependencies**: rank-soft is lightweight (just math). rank-learn would need XGBoost/LightGBM bindings (heavy).
3. **Different users**: 
   - rank-soft: People building custom neural ranking models
   - rank-learn: People who want to train ranking models with established algorithms
4. **Industry pattern**: Libraries like `allRank` (Python) separate LTR from differentiable operations

## Structure

```
rank-learn/
├── Cargo.toml
├── README.md
├── rank-learn/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── lambdarank.rs      # LambdaRank, LambdaMART
│   │   ├── xgboost.rs          # XGBoost ranking integration
│   │   ├── lightgbm.rs         # LightGBM ranking integration
│   │   └── neural.rs           # Neural LTR models
│   └── README.md
└── rank-learn-python/
    ├── Cargo.toml
    ├── pyproject.toml
    └── src/
        └── lib.rs
```

**Dependencies:**
- `rank-soft`: For loss functions (ListNet, ListMLE)
- `xgboost` (optional): For XGBoost integration
- `lightgbm` (optional): For LightGBM integration

## Implementation Priority

1. **LambdaRank/LambdaMART** (High)
   - Pure Rust implementation
   - No external dependencies
   - Standard LTR algorithm

2. **XGBoost integration** (Medium)
   - Rust bindings to XGBoost
   - Or Python wrapper that uses XGBoost Python library
   - Most common LTR approach

3. **LightGBM integration** (Low)
   - Similar to XGBoost
   - Less common but still used

4. **Neural LTR** (Low)
   - Could use rank-soft for differentiable operations
   - Build neural ranking models on top

## Summary

**rank-soft**: Differentiable ranking operations (mathematical primitives)
- Soft ranking, differentiable sorting
- Loss functions (ListNet, ListMLE, Spearman)
- Framework-agnostic (PyTorch, JAX, etc.)

**rank-learn**: Learning to Rank frameworks (complete ML systems)
- LambdaRank, LambdaMART
- XGBoost/LightGBM integration
- Neural LTR models
- Uses rank-soft for differentiable operations

**Boundary**: rank-soft provides building blocks, rank-learn provides complete solutions.

