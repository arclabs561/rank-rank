# rank-* Crate Naming Analysis

## Current Crate Names

| Crate | Purpose | Terminology | Status |
|-------|---------|-------------|--------|
| `rank-retrieve` | First-stage retrieval (BM25, dense ANN, sparse) | Standard IR term | ✅ Correct |
| `rank-fusion` | Rank fusion algorithms (RRF, ISR, CombMNZ) | Standard IR term | ✅ Correct |
| `rank-rerank` | Reranking and late interaction (MaxSim, cross-encoder) | Standard IR term | ✅ Correct |
| `rank-soft` | Differentiable ranking operations | Standard ML term | ✅ Correct |
| `rank-learn` | Learning to Rank frameworks (LambdaRank, XGBoost) | Intuitive (LTR is more precise but less clear) | ✅ Acceptable |
| `rank-eval` | Ranking evaluation metrics (NDCG, MAP, MRR) | Standard term | ✅ Correct |

## Analysis

### Terminology Alignment

All crate names align with standard terminology in their respective domains:

- **IR Terminology**: `retrieve`, `fusion`, `rerank`, `eval` are all standard IR terms
- **ML Terminology**: `soft` (soft ranking, softmax) is standard ML terminology
- **LTR Terminology**: `learn` is intuitive; "Learning to Rank" is the full term, but `rank-learn` is clearer than `rank-ltr` for non-experts

### Historical Renames

- ✅ `rank-refine` → `rank-rerank` (completed, correct)
- ✅ `rank-relax` → `rank-soft` (completed, correct)

Both renames align with standard terminology:
- "rerank" is the standard IR term (used in academic papers, industry docs)
- "soft" is more common than "relax" in ML literature (SoftRank, softmax, soft attention)

## Recommendations

### 1. Keep All Current Names

All current crate names are appropriate and align with standard terminology. No further renames needed.

### 2. Clean Up Old References

Several files still reference old names (`rank-refine`, `rank-relax`):
- README cross-references
- Python code comments
- Documentation files
- Archive files (expected, but noted)

### 3. Consider `rank-ltr` vs `rank-learn`

**Current**: `rank-learn`
**Alternative**: `rank-ltr`

**Analysis**:
- `rank-learn` is more intuitive for non-experts
- `rank-ltr` is more precise (matches "Learning to Rank" acronym)
- Industry uses both terms, but "learn" is more accessible

**Recommendation**: Keep `rank-learn` for better discoverability and clarity.

## Cleanup Tasks

1. Update README cross-references from `rank-relax` → `rank-soft`
2. Update README cross-references from `rank-refine` → `rank-rerank`
3. Update Python code comments referencing old names
4. Update documentation files (except archive, which is historical)

## Summary

✅ **All crate names are correct and align with standard terminology**
✅ **No renames needed**
⚠️ **Cleanup needed**: Remove references to old names in active code/docs

