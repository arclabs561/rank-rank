# Use Cases and Decision Guide

This document provides specific use cases for `rank-retrieve` and helps you decide when to use it versus alternatives.

## Table of Contents

1. [Primary Use Cases](#primary-use-cases)
2. [Decision Tree](#decision-tree)
3. [Comparison with Alternatives](#comparison-with-alternatives)
4. [Specific Application Examples](#specific-application-examples)

---

## Primary Use Cases

### 1. RAG (Retrieval-Augmented Generation) Pipelines

**Scenario**: Building a RAG system that retrieves context for LLM generation.

**Why rank-retrieve:**
- Unified API for multiple retrieval methods (BM25 + dense + sparse)
- Easy integration with `rank-fusion` for hybrid search
- Designed for first-stage retrieval (10M → 1000 candidates)
- Works well with `rank-rerank` for final ranking

**Example workflow:**
```rust
// 1. Retrieve candidates (rank-retrieve)
let bm25_results = bm25_index.retrieve(&query_terms, 1000, Bm25Params::default())?;
let dense_results = dense_retriever.retrieve(&query_embedding, 1000)?;

// 2. Fuse results (rank-fusion)
let fused = rrf_multi(&[&bm25_results, &dense_results], Default::default());

// 3. Rerank (rank-rerank)
let reranked = maxsim_rerank(&query_tokens, &doc_tokens_list, 100)?;

// 4. Use top-k for LLM context
let context = &reranked[..10];
```

**When to use alternatives:**
- Need persistent storage: Use `tantivy` for BM25, `qdrant` for dense
- Very large corpus (>10M docs): Use production backends

### 2. Research and Prototyping

**Scenario**: Experimenting with different retrieval methods, comparing algorithms, or building research prototypes.

**Why rank-retrieve:**
- Multiple retrieval methods in one library
- Simple, understandable implementations
- Easy to modify and experiment with
- Good for A/B testing different methods

**Example:**
```rust
// Compare BM25 vs dense vs sparse
let bm25_results = bm25_index.retrieve(&query, 100, Bm25Params::default())?;
let dense_results = dense_retriever.retrieve(&query_embedding, 100)?;
let sparse_results = sparse_retriever.retrieve(&query_sparse, 100)?;

// Evaluate with rank-eval
let bm25_ndcg = ndcg_at_k(&bm25_results, &ground_truth, 10)?;
let dense_ndcg = ndcg_at_k(&dense_results, &ground_truth, 10)?;
```

**When to use alternatives:**
- Need production performance: Use optimized libraries
- Need to publish research: May need to compare with production systems

### 3. Hybrid Search Systems

**Scenario**: Building a search system that combines lexical (BM25) and semantic (dense) retrieval.

**Why rank-retrieve:**
- Unified API makes it easy to combine methods
- Designed for integration with `rank-fusion`
- Consistent interface across retrieval methods

**Example:**
```rust
// Retrieve from multiple methods
let bm25_results = bm25_index.retrieve(&query_terms, 1000, Bm25Params::default())?;
let dense_results = dense_retriever.retrieve(&query_embedding, 1000)?;
let sparse_results = sparse_retriever.retrieve(&query_sparse, 1000)?;

// Fuse using RRF
let fused = rrf_multi(&[&bm25_results, &dense_results, &sparse_results], Default::default());
```

**When to use alternatives:**
- Need production-scale hybrid search: Use `tantivy` + `qdrant` or managed services
- Need real-time updates: Use systems with persistent storage

### 4. Embedded Applications

**Scenario**: Building an application that needs in-memory retrieval for small-medium corpora.

**Why rank-retrieve:**
- Zero external dependencies (by default)
- In-memory, fast for small-medium corpora
- Simple API, easy to integrate
- No persistence overhead

**Example:**
```rust
// Build index at startup
let mut index = InvertedIndex::new();
for doc in documents {
    index.add_document(doc.id, &doc.terms);
}

// Fast retrieval at runtime
let results = index.retrieve(&query_terms, 10, Bm25Params::default())?;
```

**When to use alternatives:**
- Need persistence: Use `tantivy` or custom persistence
- Very large corpus: Use production backends
- Need concurrent access: Use systems with built-in concurrency

---

## Decision Tree

### Do you need persistent storage?

**Yes** → Use `tantivy` (BM25) or `qdrant`/`pinecone` (dense)

**No** → Continue

### What is your corpus size?

**< 100K documents (dense) or < 1M documents (BM25)** → `rank-retrieve` is suitable

**> 100K documents (dense) or > 1M documents (BM25)** → Use production backends:
- BM25: `tantivy` with optimized indexing
- Dense: `hnsw`, `faiss`, `qdrant`

### Do you need multiple retrieval methods?

**Yes** → `rank-retrieve` provides unified API

**No, only BM25** → Consider `bm25` crate (simpler) or `tantivy` (production)

**No, only dense** → Consider `hnsw`, `faiss`, or `qdrant`

### Are you building a research prototype?

**Yes** → `rank-retrieve` is good for experimentation

**No, production system** → Consider production backends

### Do you need integration with `rank-*` ecosystem?

**Yes** → `rank-retrieve` is designed for this

**No** → Consider alternatives based on other criteria

---

## Comparison with Alternatives

### vs. `tantivy`

| Feature | rank-retrieve | tantivy |
|---------|---------------|---------|
| **Scope** | First-stage retrieval component | Full search engine |
| **Storage** | In-memory only | Persistent with segments |
| **BM25** | Basic inverted index | Optimized with block-max WAND |
| **Dense retrieval** | Included | Not included |
| **Sparse retrieval** | Included | Not included |
| **Generative retrieval** | Included | Not included |
| **Query language** | Term-based only | Boolean, phrase, field queries |
| **Concurrency** | Single-threaded | Multi-threaded indexing |
| **Use case** | IR pipelines, prototyping | Production search engines |

**When to use tantivy:**
- Need persistent storage
- Need production-scale BM25
- Need complex queries
- Building a full search engine

**When to use rank-retrieve:**
- Need multiple retrieval methods
- Building IR pipelines
- Prototyping/research
- In-memory is sufficient

### vs. `bm25` crate

| Feature | rank-retrieve | bm25 crate |
|---------|---------------|------------|
| **BM25** | Basic implementation | More features |
| **Dense retrieval** | Included | Not included |
| **Sparse retrieval** | Included | Not included |
| **Generative retrieval** | Included | Not included |
| **Dependencies** | Zero (by default) | Some dependencies |
| **Use case** | Multi-method retrieval | BM25 only |

**When to use bm25 crate:**
- Only need BM25
- Want simpler dependency
- Don't need other retrieval methods

**When to use rank-retrieve:**
- Need multiple retrieval methods
- Building hybrid search
- Integrating with `rank-*` ecosystem

### vs. `hnsw` / `faiss` / `qdrant`

| Feature | rank-retrieve | Production ANN |
|---------|---------------|----------------|
| **Dense retrieval** | Brute-force | Approximate (HNSW/IVF) |
| **Scale** | < 100K documents | Millions+ documents |
| **Performance** | O(n*d) | O(log n) approximate |
| **Storage** | In-memory | Persistent options |
| **BM25** | Included | Not included |
| **Sparse retrieval** | Included | Not included |

**When to use production ANN:**
- Very large corpus (>100K documents)
- Need production-scale performance
- Need persistent storage
- Only need dense retrieval

**When to use rank-retrieve:**
- Small-medium corpus
- Need multiple retrieval methods
- Prototyping/research
- In-memory is sufficient

---

## Specific Application Examples

### Example 1: Academic Paper Search

**Requirements:**
- Search over 10K-100K papers
- Combine keyword (BM25) and semantic (dense) search
- In-memory index (rebuild on startup)
- Fast retrieval for web interface

**Solution: `rank-retrieve`**
- Suitable corpus size
- Need multiple methods (BM25 + dense)
- In-memory is acceptable
- Unified API simplifies integration

### Example 2: E-commerce Product Search

**Requirements:**
- Search over millions of products
- Persistent index
- Real-time updates
- Production-scale performance

**Solution: `tantivy` + `qdrant`**
- Too large for `rank-retrieve`
- Need persistence
- Need production optimization
- `rank-retrieve` not suitable

### Example 3: RAG Chatbot

**Requirements:**
- Retrieve context from knowledge base (10K-100K documents)
- Combine BM25 and dense retrieval
- Fast retrieval (<100ms)
- In-memory index (loaded at startup)

**Solution: `rank-retrieve`**
- Suitable corpus size
- Need hybrid search
- In-memory acceptable
- Designed for RAG pipelines

### Example 4: Research Prototype

**Requirements:**
- Experiment with different retrieval methods
- Compare BM25, dense, sparse, generative
- Easy to modify and test
- No production requirements

**Solution: `rank-retrieve`**
- Perfect for research
- Multiple methods in one library
- Simple implementations
- Easy to experiment

---

## Summary

**Use `rank-retrieve` when:**
- Building IR pipelines with multiple retrieval methods
- Corpus size is small-medium (<1M for BM25, <100K for dense)
- In-memory storage is acceptable
- Need integration with `rank-*` ecosystem
- Prototyping or research applications

**Use alternatives when:**
- Need persistent storage → `tantivy`, `qdrant`
- Very large corpus → Production backends
- Only need BM25 → `bm25` crate or `tantivy`
- Only need dense → `hnsw`, `faiss`, `qdrant`
- Need full search engine → `tantivy`, `meilisearch`

