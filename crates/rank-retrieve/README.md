# rank-retrieve

First-stage retrieval for information retrieval pipelines. Provides BM25, dense ANN, and sparse retrieval.

[![CI](https://github.com/arclabs561/rank-retrieve/actions/workflows/ci.yml/badge.svg)](https://github.com/arclabs561/rank-retrieve/actions)
[![Crates.io](https://img.shields.io/crates/v/rank-retrieve.svg)](https://crates.io/crates/rank-retrieve)
[![Docs](https://docs.rs/rank-retrieve/badge.svg)](https://docs.rs/rank-retrieve)

```
cargo add rank-retrieve
```

## Purpose

This crate provides the first stage of the ranking pipeline: retrieving candidate documents from a large corpus (typically 10M+ documents) down to a manageable set (1000 candidates) for reranking.

## Pipeline Stage

```
10M docs → 1000 candidates → 100 candidates → 10 results
    │            │                 │              │
    ▼            ▼                 ▼              ▼
[retrieve]   [rerank]         [cross-encoder]   [User]
  (fast)      (precise)        (accurate)
```

## Features

- **BM25 Retrieval**: Inverted index with Okapi BM25 scoring
- **Dense ANN**: Cosine similarity-based retrieval (ready for HNSW/FAISS integration)
- **Sparse Retrieval**: Lexical matching using sparse vectors (uses rank-sparse)

## Quick Start

### BM25 Retrieval

```rust
use rank_retrieve::prelude::*;

let mut index = InvertedIndex::new();

// Add documents
index.add_document(0, &["the".to_string(), "quick".to_string(), "brown".to_string()]);
index.add_document(1, &["the".to_string(), "lazy".to_string(), "dog".to_string()]);

// Query
let query = vec!["quick".to_string()];
let results = index.retrieve(&query, 10, Bm25Params::default());

// Results: [(doc_id, score), ...]
for (doc_id, score) in results {
    println!("Doc {}: {:.4}", doc_id, score);
}
```

### Dense Retrieval

```rust
use rank_retrieve::prelude::*;

let mut retriever = DenseRetriever::new();

// Add documents with embeddings (should be L2-normalized)
retriever.add_document(0, vec![1.0, 0.0, 0.0]);
retriever.add_document(1, vec![0.707, 0.707, 0.0]);

// Query embedding
let query = vec![1.0, 0.0, 0.0];
let results = retriever.retrieve(&query, 10);
```

### Sparse Retrieval

```rust
use rank_retrieve::prelude::*;
use rank_sparse::SparseVector;

let mut retriever = SparseRetriever::new();

// Add documents with sparse vectors
let doc0 = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
retriever.add_document(0, doc0);

// Query sparse vector
let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
let results = retriever.retrieve(&query, 10);
```

## Design

This crate focuses on **retrieval** (finding candidates), not scoring. Advanced scoring (MaxSim, cross-encoder) is handled by `rank-rerank`.

**Boundaries:**
- ✅ Indexing and retrieval
- ✅ Basic scoring (BM25, cosine similarity)
- ❌ Advanced reranking (delegates to rank-rerank)
- ❌ List fusion (delegates to rank-fusion)

## Integration with Other rank-* Crates

### Complete Pipeline Example

```rust
use rank_retrieve::prelude::*;
use rank_fusion::rrf_multi;
use rank_rerank::simd;

// 1. Retrieve from multiple methods
let bm25_results = bm25_index.retrieve(&query_terms, 1000, Bm25Params::default());
let dense_results = dense_retriever.retrieve(&query_embedding, 1000);
let sparse_results = sparse_retriever.retrieve(&query_sparse, 1000);

// 2. Fuse results (rank-fusion)
let fused = rrf_multi(&[&bm25_results, &dense_results, &sparse_results], Default::default());

// 3. Rerank top candidates (rank-rerank)
let top_100: Vec<_> = fused.iter().take(100).map(|(id, _)| *id).collect();
let reranked = simd::maxsim_batch(&query_tokens, &doc_tokens_list);

// 4. Evaluate (rank-eval)
// Use rank_eval to compute NDCG, MAP, etc.
```

## Examples

See the `examples/` directory:
- `basic_retrieval.rs` - Basic usage of all three retrieval methods
- `hybrid_retrieval.rs` - Combining multiple retrieval methods

Run examples:
```bash
cargo run --example basic_retrieval
cargo run --example hybrid_retrieval
```

## Dependencies

- `rank-sparse`: For sparse vector operations

## Optional Features

Future features (not yet implemented):
- `tantivy`: For production BM25 indexing
- `hnsw`: For production dense ANN search
- `faiss`: Alternative dense ANN backend

## Status

✅ **Core functionality implemented and tested**

Ready for use in retrieval pipelines. For production scale, consider integrating with:
- **Tantivy** for BM25 (faster indexing, more features)
- **HNSW** or **FAISS** for dense retrieval (approximate nearest neighbor)
