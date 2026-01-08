# Rename Plan: rank-refine → rank-rerank, rank-relax → rank-soft

## Scope

These are separate git repositories, so renaming requires:
1. Git repository rename (if they're separate repos)
2. Update all references in documentation
3. Update Cargo.toml files
4. Update scripts that reference them

## Steps

### 1. Rename rank-refine → rank-rerank

**Files to update:**
- README.md (rank-rank)
- All documentation references
- Cursor rules
- Scripts (inspect_all_rank_readmes.sh, etc.)
- Cargo.toml files (if they reference each other)

**Git operations:**
- If separate repo: `git mv` or clone with new name
- Update remote URLs if needed

### 2. Rename rank-relax → rank-soft

**Same as above**

### 3. Create rank-retrieve

**Structure:**
```
rank-retrieve/
├── Cargo.toml
├── README.md
├── rank-retrieve/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── bm25.rs
│   │   ├── dense.rs
│   │   └── sparse.rs
│   └── README.md
└── rank-retrieve-python/
    ├── Cargo.toml
    ├── pyproject.toml
    └── src/
        └── lib.rs
```

## LTR (Learning to Rank) Question

**Current state:**
- `rank-relax` (soon `rank-soft`) has ListNet/ListMLE loss functions
- But these are differentiable ranking operations, not full LTR frameworks

**LTR includes:**
- LambdaRank, LambdaMART
- XGBoost with ranking objectives
- LightGBM with ranking objectives
- Neural LTR models

**Options:**
1. **rank-learn**: Full LTR framework (LambdaRank, XGBoost integration, etc.)
2. **Keep in rank-soft**: Only differentiable operations (ListNet/ListMLE losses)
3. **Out of scope**: Users bring their own XGBoost/LightGBM

**Recommendation**: **rank-learn** for full LTR, keep differentiable operations in rank-soft

