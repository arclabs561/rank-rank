# Crate Organization & Naming Analysis

## Current Structure

| Crate | Purpose | Input | Output | Stage |
|-------|---------|-------|--------|-------|
| **rank-retrieve** | ❌ Missing | Query | Ranked list | Stage 1 (10M → 1000) |
| **rank-fusion** | Combine lists | Multiple ranked lists | Single ranked list | Post-retrieval |
| **rank-refine** | Rerank/scoring | Embeddings + candidates | Re-ranked list | Stage 2 (1000 → 100) |
| **rank-relax** | Differentiable ops | Scores | Soft ranks/gradients | Training-time |
| **rank-eval** | Evaluation | Ranked list + qrels | Metrics | Post-hoc |
| **rank-sparse** | Sparse utilities | Sparse vectors | Dot product, etc. | Utility |

## Industry Patterns

### Pattern 1: Monolithic (Tantivy, Lucene)
**Structure**: Single crate with modules
```
tantivy/
├── core/        # Index, segments, searchers
├── directory/   # Storage abstraction
├── query/       # Query parsing
└── schema/      # Document schema
```

**Pros**: 
- Simple dependency management
- Cohesive API
- Easier to optimize across boundaries

**Cons**:
- Can't use parts independently
- Larger binary size
- Harder to version components separately

### Pattern 2: Pipeline Stages (Your Current Approach)
**Structure**: Separate crates per pipeline stage
```
rank-retrieve → rank-fusion → rank-refine → rank-eval
```

**Pros**:
- Clear boundaries
- Can use independently
- Independent versioning
- Matches user mental model (pipeline stages)

**Cons**:
- More dependencies to manage
- Potential API friction between stages
- Some duplication (e.g., scoring in both retrieve and refine)

### Pattern 3: Functional Separation (Alternative)
**Structure**: Separate by function, not stage
```
rank-core/        # Shared types, traits
rank-scoring/     # All scoring (dense, sparse, MaxSim, cross-encoder)
rank-fusion/      # List combination
rank-indexing/    # Indexing and retrieval
rank-eval/        # Evaluation
```

**Pros**:
- Groups related functionality
- Less duplication
- Clearer for "I need scoring" use cases

**Cons**:
- Doesn't match pipeline mental model
- Harder to understand "what do I need for stage 2?"

## Naming Analysis

### Current Names

| Name | Clarity | Industry Standard | Issues |
|------|---------|-------------------|--------|
| **rank-fusion** | ✅ Clear | ✅ Standard term | None |
| **rank-refine** | ⚠️ Ambiguous | ❌ Not standard | "Refine" could mean many things |
| **rank-relax** | ⚠️ Unclear | ❌ Not standard | Requires explanation |
| **rank-eval** | ✅ Clear | ✅ Standard term | None |
| **rank-retrieve** | ✅ Clear | ✅ Standard term | None |
| **rank-sparse** | ✅ Clear | ✅ Standard term | None |

### "Refine" vs "Rerank"

**Current**: `rank-refine` (reranking/scoring)

**Industry Terms**:
- **Reranking**: Standard IR term (used in papers, libraries)
- **Refining**: Less common, more ambiguous
- **Scoring**: Too generic (could be retrieval scoring too)

**Evidence from codebase**:
- `rank-refine` docs say "rerank search candidates"
- Pipeline diagram says "rerank"
- But crate name is "refine"

**Recommendation**: Consider `rank-rerank` instead of `rank-refine`

### "Relax" vs Alternatives

**Current**: `rank-relax` (differentiable ranking)

**Industry Terms**:
- **Soft ranking**: Common in papers
- **Differentiable ranking**: More descriptive
- **Relax**: Requires explanation

**Alternatives**:
- `rank-soft` (shorter, clearer)
- `rank-diff` (too cryptic)
- `rank-differentiable` (too long)

**Recommendation**: `rank-relax` is fine if documented, but `rank-soft` might be clearer

## Boundary Analysis

### Overlap Issues

**1. Scoring in Multiple Places**
- `rank-retrieve` (if created): Would need scoring for BM25, dense, sparse
- `rank-refine`: Has scoring (dense, MaxSim, cross-encoder)
- **Overlap**: Dense scoring appears in both

**2. Sparse Operations**
- `rank-sparse`: Sparse vector utilities
- `rank-retrieve` (if created): Would need sparse retrieval
- **Question**: Should sparse retrieval use rank-sparse or have its own?

**3. Cross-Encoder Location**
- Currently: Trait in `rank-refine`, implementation disabled
- **Question**: Should cross-encoders be in `rank-refine` or separate `rank-rerank`?

### Current Boundaries (Good)

✅ **rank-fusion**: Clear boundary - takes lists, outputs list
✅ **rank-eval**: Clear boundary - takes results, outputs metrics
✅ **rank-relax**: Clear boundary - training-time only

### Current Boundaries (Questionable)

⚠️ **rank-refine**: Does reranking (MaxSim) AND has cross-encoder trait
- **Issue**: Cross-encoders are conceptually different (model-based vs embedding-based)
- **Question**: Should cross-encoders be separate?

## Alternative Organizations

### Option A: Keep Current + Fix Names
```
rank-retrieve/    # Stage 1: BM25, dense ANN, sparse
rank-fusion/      # Combine lists (keep name)
rank-rerank/      # Stage 2: MaxSim, cross-encoder (rename from refine)
rank-soft/        # Differentiable ranking (rename from relax)
rank-eval/        # Evaluation (keep name)
rank-sparse/      # Utilities (keep name)
```

**Pros**: 
- Clearer names
- Matches industry terms
- Minimal restructuring

**Cons**:
- Breaking change (rename)
- Still have scoring overlap

### Option B: Functional Grouping
```
rank-core/         # Shared types, traits
rank-scoring/      # All scoring: dense, sparse, MaxSim, cross-encoder
rank-fusion/       # List combination
rank-indexing/     # Indexing + retrieval (BM25, ANN)
rank-eval/         # Evaluation
rank-soft/         # Differentiable ranking
```

**Pros**:
- Less duplication
- Clearer for "I need scoring"
- Groups related functionality

**Cons**:
- Doesn't match pipeline mental model
- Harder to understand "what do I need?"

### Option C: Hybrid (Recommended)
```
rank-retrieve/     # Stage 1: Indexing + retrieval
  ├── bm25/        # BM25 retrieval
  ├── dense/       # Dense ANN (delegates to hnsw/faiss)
  └── sparse/      # Sparse retrieval (uses rank-sparse)

rank-fusion/       # Combine lists (keep)

rank-rerank/       # Stage 2: All reranking
  ├── maxsim/      # Late interaction
  ├── crossencoder/ # Cross-encoders
  └── scoring/     # Shared scoring traits

rank-soft/         # Differentiable ranking (rename)

rank-eval/         # Evaluation (keep)

rank-sparse/       # Utilities (keep, used by retrieve)
```

**Pros**:
- Clear pipeline stages
- Groups related functionality within stages
- Minimal breaking changes (just rename refine→rerank, relax→soft)

**Cons**:
- Still some overlap (scoring traits)

## Recommendations

### Immediate (Low Risk)

1. **Rename `rank-refine` → `rank-rerank`**
   - **Why**: "Rerank" is standard IR term, "refine" is ambiguous
   - **Impact**: Breaking change, but clearer
   - **Priority**: Medium (can wait for major version)

2. **Consider `rank-relax` → `rank-soft`**
   - **Why**: "Soft ranking" is more common in papers
   - **Impact**: Breaking change
   - **Priority**: Low (current name is fine if documented)

### Structural (Higher Risk)

3. **Create `rank-retrieve` with clear boundaries**
   - **Scope**: Indexing + retrieval only
   - **Scoring**: Basic scoring (BM25, cosine) for retrieval
   - **Advanced scoring**: Delegate to `rank-rerank` for reranking

4. **Keep cross-encoders in `rank-rerank`**
   - **Why**: They're conceptually part of reranking (stage 2)
   - **Alternative**: Could be separate `rank-crossencoder` but adds complexity

5. **Keep `rank-sparse` as utility**
   - **Used by**: `rank-retrieve` (sparse retrieval)
   - **Keep separate**: Utilities are fine as separate crate

### Naming Summary

| Current | Recommended | Reason |
|---------|-------------|--------|
| rank-fusion | ✅ Keep | Standard term, clear |
| rank-refine | → rank-rerank | Standard IR term |
| rank-relax | → rank-soft (optional) | More common in papers |
| rank-eval | ✅ Keep | Standard term, clear |
| rank-retrieve | ✅ Use | Standard term, clear |
| rank-sparse | ✅ Keep | Standard term, clear |

## Conclusion

**Current structure is mostly good**, but:

1. **Naming**: `rank-refine` should be `rank-rerank` (standard term) ✅ RENAMED
2. **Missing**: `rank-retrieve` is critical gap ✅ TO BE CREATED
3. **LTR**: Should be separate `rank-learn` crate (different from differentiable ops)
4. **Boundaries**: Mostly clear, some scoring overlap is acceptable
5. **Organization**: Pipeline-stage organization matches user mental model

**Recommendation**: Keep pipeline-stage organization, fix naming, add `rank-retrieve`, add `rank-learn` for LTR.

## Updated Structure

```
rank-retrieve/    # Stage 1: BM25, dense ANN, sparse retrieval
rank-fusion/      # Combine lists from multiple retrievers
rank-rerank/      # Stage 2: MaxSim, cross-encoder reranking
rank-soft/        # Differentiable ranking operations (training-time)
rank-learn/       # Learning to Rank frameworks (LambdaRank, XGBoost, etc.)
rank-eval/        # Evaluation metrics
rank-sparse/      # Sparse vector utilities
```

