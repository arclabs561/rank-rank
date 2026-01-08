# Module Organization Analysis: rank-retrieve

## Current State

### Single-File Modules
- `error.rs`: 45 lines - Simple error types ✓
- `dense.rs`: 156 lines - Simple retriever ✓
- `sparse.rs`: 128 lines - Simple retriever ✓
- `sparse_vector.rs`: 115 lines - Data structure (used by sparse)
- `bm25.rs`: 269 lines - Inverted index + scoring
- `routing.rs`: 267 lines - Query routing

### Flattened Multi-File Module
- `generative.rs`: 234 lines (main)
- `generative_identifier.rs`: 291 lines
- `generative_scorer.rs`: 268 lines
- `generative_model.rs`: 115 lines
- `generative_ltrgr.rs`: 269 lines
- **Total: ~1177 lines across 5 files**

## The Problem

**Inconsistency**: We flattened `generative` to match the single-file pattern, but:
1. It's **4-9x larger** than other modules
2. It has **5 distinct components** that are logically separate
3. Other modules are simple single-responsibility files

**Sparse Organization Issue**: `sparse.rs` and `sparse_vector.rs` are separate but related:
- `sparse_vector.rs` is only used by `sparse.rs`
- They could be organized as `sparse/mod.rs` + `sparse/vector.rs`

## Rust Best Practices Research

### When to Use Subdirectories

Based on Rust community practices:
- **Subdirectories**: When module has multiple related components that benefit from separation
- **Single files**: When module is small, focused, and has single responsibility
- **Threshold**: No hard rule, but ~300+ lines with multiple components suggests subdirectory

### Examples from Ecosystem

**rank-rerank** (similar crate):
- Uses subdirectories for complex modules: `crossencoder/` (with `mod.rs` + `ort.rs`)
- Single files for focused modules: `colbert.rs`, `diversity.rs`, `scoring.rs`

**Pattern**: Complex modules get subdirectories, simple ones stay as files.

## Recommended Structure

### Option A: Restore Generative Subdirectory (Recommended)

```
src/
├── bm25.rs              (269 lines - single file OK)
├── dense.rs             (156 lines - single file OK)
├── sparse/
│   ├── mod.rs          (sparse retriever)
│   └── vector.rs        (sparse vector data structure)
├── generative/          (1177 lines - SUBDIRECTORY)
│   ├── mod.rs          (main retriever)
│   ├── identifier.rs   (identifier generation)
│   ├── scorer.rs       (heuristic scoring)
│   ├── model.rs        (autoregressive model trait)
│   └── ltrgr.rs        (LTR training)
├── routing.rs           (267 lines - single file OK)
└── error.rs             (45 lines - single file OK)
```

**Rationale**:
- Generative is **4x larger** than largest single-file module
- Has **5 distinct components** with clear boundaries
- Matches pattern from `rank-rerank` (`crossencoder/` subdirectory)
- Better organization for future growth

### Option B: Keep Flattened but Organize Sparse

```
src/
├── bm25.rs
├── dense.rs
├── sparse/
│   ├── mod.rs          (sparse retriever)
│   └── vector.rs       (sparse vector)
├── generative.rs        (main)
├── generative_identifier.rs
├── generative_scorer.rs
├── generative_model.rs
├── generative_ltrgr.rs
└── ...
```

**Rationale**:
- Consistent flat structure
- But generative is still awkwardly large

## Recommendation

**Use Option A**: Restore `generative/` subdirectory because:
1. **Size**: 1177 lines is significantly larger than other modules
2. **Complexity**: 5 distinct components with clear boundaries
3. **Precedent**: `rank-rerank` uses subdirectories for complex modules
4. **Maintainability**: Easier to navigate and understand
5. **Future-proof**: Room for growth (FM-index, advanced training, etc.)

**Also organize sparse**: Move `sparse_vector.rs` into `sparse/vector.rs` since it's only used by sparse retrieval.

## Comparison

| Module | Lines | Components | Current | Recommended |
|--------|-------|------------|---------|-------------|
| `dense` | 156 | 1 | Single file ✓ | Single file ✓ |
| `sparse` | 128 + 115 | 2 | Two files | Subdirectory |
| `generative` | 1177 | 5 | Flattened | **Subdirectory** |
| `bm25` | 269 | 1 | Single file ✓ | Single file ✓ |
| `routing` | 267 | 1 | Single file ✓ | Single file ✓ |

## Conclusion

The flattening of `generative` was a mistake. It should be a subdirectory because:
- It's much larger and more complex than other modules
- It has multiple distinct components
- It matches patterns from similar crates (`rank-rerank`)

**Action**: Restore `generative/` subdirectory and organize `sparse/` subdirectory.

