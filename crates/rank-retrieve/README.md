# rank-retrieve

First-stage retrieval for information retrieval pipelines. Provides BM25, dense ANN, and sparse retrieval.

[![CI](https://github.com/arclabs561/rank-rank/actions/workflows/ci.yml/badge.svg)](https://github.com/arclabs561/rank-rank/actions)
[![Crates.io](https://img.shields.io/crates/v/rank-retrieve.svg)](https://crates.io/crates/rank-retrieve)
[![Docs](https://docs.rs/rank-retrieve/badge.svg)](https://docs.rs/rank-retrieve)

```
cargo add rank-retrieve
```

**New to rank-retrieve?** Start with the [Getting Started Guide](docs/GETTING_STARTED.md) for a step-by-step walkthrough.

**Not sure if rank-retrieve is right for you?** See [Use Cases and Decision Guide](docs/USE_CASES.md) to compare with alternatives.

**Want to understand the motivation?** See [Motivation and Competitive Analysis](docs/MOTIVATION.md) for detailed justification and competitive landscape.

**Want to understand the trait-based design?** See [Trait Design Guide](docs/TRAIT_DESIGN.md) for architecture and usage patterns.

**Want a critical analysis?** See [Design Critique](docs/DESIGN_CRITIQUE.md) for honest assessment of trade-offs and alternatives.

**Want research-based recommendations?** See [Research Recommendations](docs/RESEARCH_RECOMMENDATIONS.md) for evidence-based path forward.

**Want to verify integration sufficiency?** See [Integration Sufficiency Analysis](docs/INTEGRATION_SUFFICIENCY.md) for analysis of what other rank-* crates need.

**Want to understand PLAID and late interaction retrieval?** See [PLAID Analysis](docs/PLAID_ANALYSIS.md) for research on ColBERTv2 optimization and where it fits in the rank-* ecosystem.  
See [Late Interaction Guide](docs/LATE_INTERACTION_GUIDE.md) for practical guidance on using rank-retrieve with rank-rerank for ColBERT-style retrieval.

**Real-world examples:**
- [Qdrant Integration](examples/qdrant_real_integration.rs) - RAG pipeline with Qdrant
- [usearch Integration](examples/usearch_integration.rs) - HNSW-based dense retrieval
- [Full Pipeline](examples/full_pipeline.rs) - Complete retrieve → fuse → rerank workflow

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
- **Sparse Retrieval**: Lexical matching using sparse vectors

## Quick Start

### BM25 Retrieval

```rust
use rank_retrieve::{retrieve_bm25};
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};

let mut index = InvertedIndex::new();

// Add documents
index.add_document(0, &["the".to_string(), "quick".to_string(), "brown".to_string()]);
index.add_document(1, &["the".to_string(), "lazy".to_string(), "dog".to_string()]);

// Query using concrete function
let query = vec!["quick".to_string()];
let results = retrieve_bm25(&index, &query, 10, Bm25Params::default())?;

// Results: [(doc_id, score), ...]
for (doc_id, score) in results {
    println!("Doc {}: {:.4}", doc_id, score);
}
```

### Dense Retrieval

```rust
use rank_retrieve::{retrieve_dense};
use rank_retrieve::dense::DenseRetriever;

let mut retriever = DenseRetriever::new();

// Add documents with embeddings (should be L2-normalized)
retriever.add_document(0, vec![1.0, 0.0, 0.0]);
retriever.add_document(1, vec![0.707, 0.707, 0.0]);

// Query embedding using concrete function
let query = [1.0, 0.0, 0.0];
let results = retrieve_dense(&retriever, &query, 10)?;
```

### Sparse Retrieval

```rust
use rank_retrieve::{retrieve_sparse};
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

let mut retriever = SparseRetriever::new();

// Add documents with sparse vectors
let doc0 = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
retriever.add_document(0, doc0);

// Query sparse vector using concrete function
let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
let results = retrieve_sparse(&retriever, &query, 10)?;
```

## Design

This crate focuses on **retrieval** (finding candidates), not scoring. Advanced scoring (MaxSim, cross-encoder) is handled by `rank-rerank`.

### Core Value Proposition

`rank-retrieve` provides **concrete functions** for multiple retrieval methods, matching the pattern used in `rank-fusion` and `rank-rerank` for consistency. All methods return `Vec<(u32, f32)>` for easy integration.

**Key differentiators:**
- **Concrete function API**: Simple, direct functions (`retrieve_bm25()`, `retrieve_dense()`, `retrieve_sparse()`)
- **Consistent output format**: All methods return `Vec<(u32, f32)>` for seamless integration with `rank-fusion`
- **Feature-gated implementations**: Users opt into specific methods (bm25, dense, sparse, generative)
- **Generative retrieval**: Complete LTRGR implementation (unique in Rust ecosystem)
- **Ecosystem integration**: Works seamlessly with `rank-fusion`, `rank-rerank`, `rank-eval`
- **Hybrid search**: Easy combination of multiple methods using `rank-fusion`

**What rank-retrieve does:**
- Indexing and retrieval (BM25, dense, sparse, generative)
- Basic scoring (BM25, cosine similarity, dot product)
- In-memory indexes (fast, simple)
- Unified API for multiple retrieval methods

**What rank-retrieve does NOT do:**

- **Tokenization**: Assumes pre-tokenized input. No stemming, lemmatization, or language-aware processing. Use external tokenizers (e.g., `tantivy`, `whatlang`, or custom).
- **Persistent storage**: In-memory only. No segment merging, write-ahead logging, or ACID guarantees. For persistent indexes, use `tantivy` or integrate with external storage.
- **Query language**: Simple term-based queries only. No boolean queries, phrase queries, or field queries. For complex queries, use a full search engine.
- **Document storage**: Only stores IDs and representations (embeddings, sparse vectors). Does not store document content. This is an index, not a document store.
- **Concurrency**: Single-threaded indexing. No built-in support for concurrent writers or readers. Callers must manage concurrency.
- **Large-scale optimization**: Basic implementations (brute-force dense, simple inverted index). For large scale, integrate with `tantivy` (BM25), `hnsw`/`faiss` (dense), or other specialized libraries.
- **Advanced reranking**: Delegates to `rank-rerank` for MaxSim, cross-encoder, and other advanced scoring.
- **List fusion**: Delegates to `rank-fusion` for combining multiple retrieval results.

**Scale limitations:**
- **BM25**: Suitable for corpora up to ~1M documents. For larger corpora, use `tantivy` with block-max WAND.
- **Dense**: Brute-force cosine similarity is O(n*d) where n is documents and d is dimension. Suitable for <100K documents. For larger, integrate with HNSW/FAISS.
- **Sparse**: Efficient for sparse vectors, but no special optimizations. Suitable for most use cases.
- **Generative**: Requires external autoregressive model. Suitable for research and specialized applications.

**When to use alternatives:**
- **Need persistent storage**: Use `tantivy` or build custom persistence layer
- **Need large-scale BM25**: Use `tantivy` with optimized indexing
- **Need large-scale dense retrieval**: Use `hnsw`, `faiss`, or `qdrant`
- **Need full search engine**: Use `tantivy`, `meilisearch`, or `elasticsearch`
- **Only need BM25**: Consider `bm25` crate for simpler dependency

## Integration with Other rank-* Crates

### Complete Pipeline Example

```rust
use rank_retrieve::prelude::*;
use rank_fusion::rrf_multi;
use rank_rerank::simd;

// 1. Retrieve from multiple methods
use rank_retrieve::{retrieve_bm25, retrieve_dense, retrieve_sparse};

let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 1000, Bm25Params::default())?;
let dense_results = retrieve_dense(&dense_retriever, &query_embedding, 1000)?;
let sparse_results = retrieve_sparse(&sparse_retriever, &query_sparse, 1000)?;

// 2. Fuse results (rank-fusion)
let fused = rrf_multi(&[&bm25_results, &dense_results, &sparse_results], Default::default());

// 3. Rerank top candidates (rank-rerank)
let top_100: Vec<_> = fused.iter().take(100).map(|(id, _)| *id).collect();
let reranked = simd::maxsim_batch(&query_tokens, &doc_tokens_list);

// 4. Evaluate (rank-eval)
// Use rank_eval to compute NDCG, MAP, etc.
```

## API Design

This crate uses **concrete functions** as the primary API, matching the pattern used in `rank-fusion` and `rank-rerank` for consistency:

- `retrieve_bm25()` - BM25 retrieval
- `retrieve_dense()` - Dense retrieval  
- `retrieve_sparse()` - Sparse retrieval

All functions return `Vec<(u32, f32)>` (document ID, score pairs) for easy integration with `rank-fusion`.

**Why concrete functions?**
- Simpler API (no trait complexity)
- Consistent with `rank-*` ecosystem
- Better performance (static dispatch)
- Handles incompatible query types naturally

The `Retriever` trait is available for custom implementations but is deprecated in favor of concrete functions.

## Backend Integration

For large-scale retrieval, implement the `Backend` trait for your chosen backend:

```rust
use rank_retrieve::integration::Backend;
use rank_retrieve::RetrieveError;

struct MyBackend {
    // Your backend (Tantivy, HNSW, FAISS, Qdrant, etc.)
}

impl Backend for MyBackend {
    fn retrieve(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        // Your implementation
        Ok(vec![])
    }

    fn add_document(&mut self, doc_id: u32, embedding: &[f32]) -> Result<(), RetrieveError> {
        // Your implementation
        Ok(())
    }

    fn build(&mut self) -> Result<(), RetrieveError> {
        // Your implementation
        Ok(())
    }
}
```

This keeps rank-retrieve lightweight and avoids maintaining many backend implementations.

## Examples

See the `examples/` directory:
- `basic_retrieval.rs` - Basic usage of all three retrieval methods
- `hybrid_retrieval.rs` - Combining multiple retrieval methods
- **`qdrant_real_integration.rs`** - Real-world Qdrant integration (RAG pipeline)
- `usearch_integration.rs` - HNSW-based dense retrieval with usearch
- `full_pipeline.rs` - Complete retrieve → fuse → rerank workflow

Run examples:
```bash
cargo run --example basic_retrieval
cargo run --example hybrid_retrieval
cargo run --example qdrant_real_integration  # Mock mode (works without Qdrant)
cargo run --example qdrant_real_integration --features qdrant  # Real Qdrant (requires Qdrant running)
```

## Dependencies

- No external dependencies (sparse vectors are built-in)

## Optional Features

**Current features:**
- `serde`: Serialization support for sparse vectors
- `rand`: Required for generative retrieval (LTRGR)
- `unicode`: Unicode normalization
- `qdrant`: Qdrant integration example
- `futures`: Async support for Qdrant

**Note:** Basic implementations are suitable for small-medium corpora. For large scale, implement the `Backend` trait for your chosen backend (Tantivy, HNSW, FAISS, Qdrant, etc.).

**Default features:** This crate uses `default = []` (minimal defaults) to match the workspace pattern. Enable specific features as needed: `rank-retrieve = { path = "...", features = ["bm25", "dense", "sparse"] }`

## Use Cases

**Primary use cases:**
1. **RAG pipelines**: First-stage retrieval before reranking and LLM generation
2. **Research/prototyping**: Rapid experimentation with multiple retrieval methods
3. **Hybrid search systems**: Combining BM25, dense, and sparse retrieval
4. **Embedded applications**: In-memory retrieval for small-medium corpora

**When to use rank-retrieve:**
- Building IR pipelines with multiple retrieval methods
- Need unified API for BM25, dense, and sparse retrieval
- Integrating with `rank-*` ecosystem (fusion, reranking, evaluation)
- Prototyping or research applications
- Small-medium corpora (<1M documents for BM25, <100K for dense)

**When NOT to use rank-retrieve:**
- Need persistent storage (use `tantivy`)
- Need large-scale optimization (use `tantivy` + `hnsw`/`faiss`)
- Only need BM25 (consider `bm25` crate)
- Need full search engine features (use `tantivy`, `meilisearch`)
- Very large corpora (>10M documents)

See [USE_CASES.md](docs/USE_CASES.md) for detailed use case analysis and decision tree.

## Status

**Core functionality implemented and tested**

**Current limitations:**
- In-memory only (no persistence)
- Basic implementations (not optimized for large scale)
- Query routing is experimental (heuristic-based, not trained)
- Generative retrieval requires external model integration

**Scale and capabilities:**
- Suitable for: Prototyping, research, small-medium corpora, RAG pipelines
- Limited for: Very large corpora, large-scale optimization, persistent storage

**For large scale, consider integrating with:**
- **Tantivy** for BM25 (faster indexing, persistent storage, more features)
- **HNSW** or **FAISS** for dense retrieval (approximate nearest neighbor, large-scale)
- **Qdrant** or **Pinecone** for managed vector search
