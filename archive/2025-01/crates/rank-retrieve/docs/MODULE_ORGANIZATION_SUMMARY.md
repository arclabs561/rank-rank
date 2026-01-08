# Module Organization Summary

## Final Structure

After research and reorganization, the module structure is now:

```
src/
├── bm25.rs              (269 lines - single file)
├── dense.rs             (156 lines - single file)
├── sparse/              (243 lines total - subdirectory)
│   ├── mod.rs          (sparse retriever)
│   └── vector.rs       (sparse vector data structure)
├── generative/          (1177 lines total - subdirectory)
│   ├── mod.rs          (main retriever)
│   ├── identifier.rs   (identifier generation)
│   ├── scorer.rs       (heuristic scoring)
│   ├── model.rs        (autoregressive model trait)
│   └── ltrgr.rs        (LTR training)
├── routing.rs           (267 lines - single file)
└── error.rs             (45 lines - single file)
```

## Decision Rationale

### Why Subdirectories for `generative/` and `sparse/`?

**Generative** (~1177 lines, 5 components):
- **Size**: 4-9x larger than other modules
- **Complexity**: 5 distinct components with clear boundaries
- **Precedent**: `rank-rerank` uses subdirectories for complex modules (`crossencoder/`)
- **Maintainability**: Easier to navigate and understand
- **Future-proof**: Room for growth (FM-index, advanced training, etc.)

**Sparse** (243 lines, 2 components):
- `sparse_vector.rs` is only used by `sparse.rs`
- Logical grouping: vector operations belong with sparse retrieval
- Cleaner API: `sparse::SparseVector` instead of separate `sparse_vector` module

### Why Single Files for Others?

**Dense** (156 lines):
- Simple, focused retriever
- Single responsibility
- No need for submodules

**BM25** (269 lines):
- Inverted index + scoring in one cohesive unit
- Single responsibility
- Still manageable as one file

**Routing** (267 lines):
- Query routing logic
- Single responsibility
- Manageable size

## Module Size Guidelines

Based on Rust best practices and ecosystem patterns:

- **Single file**: <300 lines, single responsibility, no sub-components
- **Subdirectory**: >300 lines OR multiple distinct components OR logical grouping needed

## API Consistency

All modules maintain consistent public APIs:
- `dense::DenseRetriever`
- `sparse::SparseRetriever` + `sparse::SparseVector`
- `generative::GenerativeRetriever` + related types
- `bm25::InvertedIndex` + `bm25::Bm25Params`

## Migration Notes

- `sparse_vector::SparseVector` → `sparse::SparseVector`
- `sparse_vector::dot_product` → `sparse::dot_product`
- All `generative_*` modules now under `generative/` subdirectory
- Public API unchanged (via re-exports in `prelude`)

