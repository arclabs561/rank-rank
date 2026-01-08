# Getting Started with rank-retrieve

This guide walks you through using `rank-retrieve` for first-stage retrieval in information retrieval pipelines.

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Core Concepts](#core-concepts)
4. [Common Use Cases](#common-use-cases)
5. [Next Steps](#next-steps)

---

## Overview

`rank-retrieve` provides **first-stage retrieval** methods for information retrieval pipelines. It implements BM25, dense ANN, sparse retrieval, and hybrid approaches.

### Key Features

- **Multiple retrieval methods**: BM25, dense, sparse, hybrid
- **Efficient**: Optimized for large-scale retrieval
- **Flexible**: Works with various backends
- **Integration**: Designed to work with `rank-rerank` and `rank-fusion`

### When to Use

**Use rank-retrieve when**:
- Building first-stage retrieval (before reranking)
- Need BM25, dense, or sparse retrieval
- Building hybrid search systems
- Need efficient large-scale retrieval

**Don't use rank-retrieve when**:
- Need reranking (use `rank-rerank`)
- Need learning-to-rank (use `rank-learn`)
- Only need evaluation (use `rank-eval`)

---

## Quick Start

Get started in 5 minutes:

### Installation

**Rust:**
```bash
cargo add rank-retrieve
```

**Python:**
```bash
pip install rank-retrieve
# Or for development:
cd rank-retrieve-python && maturin develop
```

### Basic Example: BM25

```rust
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};

// Build index
let mut index = InvertedIndex::new();
index.add_document(0, &["the", "quick", "brown", "fox"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
index.add_document(1, &["the", "lazy", "dog"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
index.add_document(2, &["fox", "and", "dog"].iter().map(|s| s.to_string()).collect::<Vec<_>>());

// Search
let query_terms: Vec<String> = vec!["brown", "fox"].iter().map(|s| s.to_string()).collect();
let results = index.retrieve(&query_terms, 10, Bm25Params::default())?;
// Returns top-10 documents with BM25 scores: Vec<(u32, f32)>
```

### Dense Retrieval

```rust
use rank_retrieve::dense::DenseRetriever;

// Build index from embeddings
let mut retriever = DenseRetriever::new();
retriever.add_document(0, vec![0.1, 0.2, 0.3]);  // doc1 embedding
retriever.add_document(1, vec![0.4, 0.5, 0.6]);  // doc2 embedding
retriever.add_document(2, vec![0.7, 0.8, 0.9]);  // doc3 embedding

// Search with query embedding
let query_embedding = vec![0.15, 0.25, 0.35];
let results = retriever.retrieve(&query_embedding, 10)?;
// Returns top-10 documents with cosine similarity scores: Vec<(u32, f32)>
```

### Hybrid Retrieval

```rust
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_fusion::rrf;  // Use rank-fusion for hybrid

// BM25 index
let mut bm25_index = InvertedIndex::new();
// ... add documents to bm25_index ...

// Dense retriever
let mut dense_retriever = DenseRetriever::new();
// ... add embeddings to dense_retriever ...

// Retrieve from both
let query_terms: Vec<String> = vec!["query", "terms"].iter().map(|s| s.to_string()).collect();
let bm25_results = bm25_index.retrieve(&query_terms, 100, Bm25Params::default())?;
let dense_results = dense_retriever.retrieve(&query_embedding, 100)?;

// Fuse using Reciprocal Rank Fusion (from rank-fusion crate)
let fused = rrf(&[&bm25_results, &dense_results], 60);
// Returns fused results: Vec<(u32, f32)>
```

---

## Core Concepts

### First-Stage vs Reranking

**First-stage retrieval** (this crate):
- Fast, approximate retrieval
- Returns many candidates (100-1000)
- Uses simple scoring (BM25, cosine)
- Goal: Recall (find all relevant docs)

**Reranking** (`rank-rerank`):
- Slower, accurate scoring
- Works on small candidate set (10-100)
- Uses complex scoring (MaxSim, cross-encoder)
- Goal: Precision (rank relevant docs highly)

**Typical pipeline**:
1. First-stage: `rank-retrieve` → 1000 candidates
2. Fusion: `rank-fusion` → combine multiple retrievers
3. Rerank: `rank-rerank` → top 100 candidates
4. Final: Top 10 results

### BM25 Algorithm

BM25 (Best Matching 25) is a probabilistic ranking function:
- Term frequency (TF): How often query terms appear
- Inverse document frequency (IDF): How rare terms are
- Document length normalization: Prevents long doc bias

**Formula**:
$$\text{BM25}(q, d) = \sum_{t \in q} \text{IDF}(t) \cdot \frac{\text{TF}(t, d) \cdot (k_1 + 1)}{\text{TF}(t, d) + k_1 \cdot (1 - b + b \cdot |d| / \text{avgdl})}$$

where $k_1$ and $b$ are tuning parameters.

### Dense vs Sparse Retrieval

**Dense retrieval**:
- Uses learned embeddings (BERT, etc.)
- Semantic matching
- Requires embedding model
- Good for semantic queries

**Sparse retrieval**:
- Uses term-based matching (BM25, TF-IDF)
- Lexical matching
- No model needed
- Good for keyword queries

**Hybrid**: Combine both for best results.

---

## Common Use Cases

### Simple Search Pipeline

```rust
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};

// 1. Build index
let mut index = InvertedIndex::new();
// ... add documents with index.add_document(doc_id, &terms) ...

// 2. Search
let query_terms: Vec<String> = vec!["query", "terms"].iter().map(|s| s.to_string()).collect();
let results = index.retrieve(&query_terms, 100, Bm25Params::default())?;

// 3. Use results (e.g., for reranking)
for (doc_id, score) in results {
    // Process document
}
```

### RAG Pipeline Integration

```rust
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_fusion::rrf;
use rank_rerank::simd;

// 1. First-stage: Multiple retrievers
let query_terms: Vec<String> = vec!["query", "terms"].iter().map(|s| s.to_string()).collect();
let bm25_results = bm25_index.retrieve(&query_terms, 100, Bm25Params::default())?;
let dense_results = dense_retriever.retrieve(&query_embedding, 100)?;

// 2. Fuse results
let fused = rrf(&[&bm25_results, &dense_results], 60);

// 3. Rerank with late interaction (MaxSim)
// ... use rank-rerank for reranking ...

// 4. Use top-k for LLM context
let context = &fused[..5];
```

### Building a Search Index

```rust
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};

// Build index from documents
let mut index = InvertedIndex::new();
for (doc_id, document) in documents.iter().enumerate() {
    let terms: Vec<String> = tokenize(document);  // Your tokenization function
    index.add_document(doc_id as u32, &terms);
}

// Index is ready to use
let results = index.retrieve(&query_terms, 10, Bm25Params::default())?;

// Note: Serialization not yet implemented - index is in-memory only
```

---

## Next Steps

1. **Read the [README](../README.md)** - Complete API reference
2. **See [USE_CASES.md](USE_CASES.md)** - Use cases and decision guide (when to use rank-retrieve vs alternatives)
3. **Check [examples/](../examples/)** - More code examples
4. **See [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)** - Integration patterns

---

## Python Usage

```bash
pip install rank-retrieve
```

```python
import rank_retrieve

# BM25
index = rank_retrieve.InvertedIndex()
index.add_document(0, ["machine", "learning"])
results = index.retrieve(["machine"], 10)

# Dense
retriever = rank_retrieve.DenseRetriever()
retriever.add_document(0, [1.0, 0.0, 0.0])
results = retriever.retrieve([1.0, 0.0, 0.0], 10)

# Sparse
retriever = rank_retrieve.SparseRetriever()
vector = rank_retrieve.SparseVector([0, 1], [1.0, 0.5])
retriever.add_document(0, vector)
query = rank_retrieve.SparseVector([0, 1], [1.0, 1.0])
results = retriever.retrieve(query, 10)
```

See [Python bindings README](../rank-retrieve-python/README.md) for full API.

---

## Performance Tips

1. **Build index once**: Reuse index across queries
2. **Limit candidate count**: Retrieve only what you need for reranking
3. **Use approximate methods**: For very large corpora, use approximate nearest neighbor
4. **Batch queries**: Process multiple queries together when possible

---

## Troubleshooting

**Q: Should I use BM25 or dense retrieval?**
A: Use both (hybrid). BM25 for keyword matching, dense for semantic matching.

**Q: How many candidates should I retrieve?**
A: Typically 100-1000 for first-stage. More = better recall but slower.

**Q: Can I use external retrieval systems?**
A: Yes, `rank-retrieve` is designed to work with external systems. Use it for fusion and reranking.

**Q: How do I tune BM25 parameters?**
A: Default parameters work well. Tune $k_1$ and $b$ based on your corpus characteristics.

