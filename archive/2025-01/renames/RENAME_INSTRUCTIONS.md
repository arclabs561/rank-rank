# Rename Instructions

## Git Repository Renames

Since `rank-refine` and `rank-relax` are separate git repositories, renaming requires git operations:

### 1. Rename rank-refine → rank-rerank

```bash
cd /Users/arc/Documents/dev/_rank-rank
mv rank-refine rank-rerank
cd rank-rerank

# Update git remote if it exists
git remote set-url origin <new-url>  # If you have a remote

# Update all internal references
find . -type f -name "*.rs" -o -name "*.toml" -o -name "*.md" | xargs sed -i '' 's/rank-refine/rank-rerank/g'
find . -type f -name "*.rs" -o -name "*.toml" -o -name "*.md" | xargs sed -i '' 's/rank_refine/rank_rerank/g'
find . -type f -name "*.py" | xargs sed -i '' 's/rank_refine/rank_rerank/g'
```

### 2. Rename rank-relax → rank-soft

```bash
cd /Users/arc/Documents/dev/_rank-rank
mv rank-relax rank-soft
cd rank-soft

# Update git remote if it exists
git remote set-url origin <new-url>  # If you have a remote

# Update all internal references
find . -type f -name "*.rs" -o -name "*.toml" -o -name "*.md" | xargs sed -i '' 's/rank-relax/rank-soft/g'
find . -type f -name "*.rs" -o -name "*.toml" -o -name "*.md" | xargs sed -i '' 's/rank_relax/rank_soft/g'
find . -type f -name "*.py" | xargs sed -i '' 's/rank_relax/rank_soft/g'
```

## What's Been Updated in rank-rank

✅ Updated `README.md` with new names
✅ Updated `.cursor/rules/shared-base.mdc` with new names
✅ Created `rank-retrieve` structure
✅ Created `LTR_ANALYSIS.md` explaining LTR placement

## Next Steps

1. **Rename git repos** (see commands above)
2. **Update Cargo.toml files** in other repos that depend on rank-refine/rank-relax
3. **Update documentation** in renamed repos
4. **Update CI/CD** if you have GitHub Actions or similar
5. **Update crates.io** if published (requires new crate names)

## LTR Answer

**Learning to Rank (LambdaRank, XGBoost) should be in `rank-learn`:**

- **rank-soft**: Differentiable ranking operations (mathematical primitives)
- **rank-learn**: Full LTR frameworks (LambdaRank, XGBoost integration, etc.)

See `LTR_ANALYSIS.md` for detailed analysis.

