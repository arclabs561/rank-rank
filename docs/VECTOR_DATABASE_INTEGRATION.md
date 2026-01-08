# Vector Database Integration Guide

This guide demonstrates how to integrate `rank-rank` crates with production vector databases for complete RAG pipelines.

## Overview

Vector databases are essential for dense retrieval in RAG systems. This guide covers:
- **Qdrant**: Production-ready vector database with Rust client
- **usearch**: Fast HNSW-based approximate nearest neighbor search
- **Integration patterns**: How to combine vector DB retrieval with `rank-rank` crates

---

## Qdrant Integration

### Setup

Add to `Cargo.toml`:

```toml
[dependencies]
qdrant-client = { version = "1.7", optional = true }
rank-retrieve = { path = "../rank-retrieve" }
rank-rerank = { path = "../rank-rerank" }
rank-fusion = { path = "../rank-fusion" }

[features]
qdrant = ["dep:qdrant-client"]
```

### Basic Integration Pattern

```rust
use qdrant_client::{
    prelude::*,
    qdrant::{SearchPoints, PointStruct, Vector},
};
use rank_retrieve::Bm25Index;
use rank_rerank::simd::maxsim_vecs;

// 1. Initialize Qdrant client
let client = QdrantClient::from_url("http://localhost:6333").build()?;

// 2. Dense retrieval from Qdrant
let query_embedding = vec![0.1f32; 128]; // Your query embedding
let dense_results = client
    .search_points(&SearchPoints {
        collection_name: "documents".to_string(),
        vector: query_embedding,
        limit: 100,
        with_payload: Some(true.into()),
        ..Default::default()
    })
    .await?;

// 3. Sparse retrieval with BM25 (rank-retrieve)
let mut bm25 = Bm25Index::new();
// ... add documents to BM25 index ...
let sparse_results = bm25.retrieve(&query_terms, 100, Default::default())?;

// 4. Rank fusion (combine dense + sparse)
use rank_fusion::rrf;
let fused = rrf(&dense_results, &sparse_results, 60)?;

// 5. Rerank top-K with MaxSim (rank-rerank)
let top_candidates = fused.iter().take(20);
// ... rerank with MaxSim or cross-encoder ...
```

### Complete Example

See `crates/rank-retrieve/examples/qdrant_integration.rs` for a complete example.

**Key Integration Points:**
1. **Dense retrieval**: Use Qdrant's `search_points` for vector similarity search
2. **Sparse retrieval**: Use `rank-retrieve` BM25 for keyword-based search
3. **Fusion**: Use `rank-fusion` to combine results (RRF, ISR, CombMNZ)
4. **Reranking**: Use `rank-rerank` for final reranking (MaxSim, cross-encoder)

---

## usearch Integration

### Setup

Add to `Cargo.toml`:

```toml
[dependencies]
usearch = { version = "2.11", optional = true }
rank-retrieve = { path = "../rank-retrieve" }
rank-rerank = { path = "../rank-rerank" }

[features]
usearch = ["dep:usearch"]
```

### Basic Integration Pattern

```rust
use usearch::{Index, Metric, ScalarKind};
use rank_retrieve::Bm25Index;
use rank_rerank::simd::maxsim_vecs;

// 1. Build HNSW index with usearch
let dimension = 128;
let index = Index::new(
    dimension,
    Metric::Cosine,
    ScalarKind::F32,
    None,
    None,
)?;

// 2. Add vectors to index
for (i, embedding) in document_embeddings.iter().enumerate() {
    index.add(i as u64, embedding)?;
}

// 3. Approximate nearest neighbor search
let query_embedding = vec![0.1f32; dimension];
let (labels, distances) = index.search(&query_embedding, 100)?;

// 4. Rerank results with MaxSim
let top_candidates = labels.iter().take(20);
// ... rerank with MaxSim or cross-encoder ...
```

### Complete Example

See `crates/rank-retrieve/examples/usearch_integration.rs` for a complete example.

**Key Integration Points:**
1. **HNSW indexing**: Use usearch for fast approximate nearest neighbor search
2. **Sparse retrieval**: Use `rank-retrieve` BM25 for keyword-based search
3. **Reranking**: Use `rank-rerank` for final reranking (MaxSim, cross-encoder)

---

## End-to-End RAG Pipeline

### Complete Workflow

```rust
use rank_retrieve::Bm25Index;
use rank_fusion::rrf;
use rank_rerank::{simd::maxsim_vecs, crossencoder::CrossEncoderModel};
use rank_eval::ndcg_at_k;

// 1. First-stage retrieval (sparse)
let mut bm25 = Bm25Index::new();
// ... add documents ...
let sparse_results = bm25.retrieve(&query_terms, 100, Default::default())?;

// 2. First-stage retrieval (dense) - from vector DB
let dense_results = vector_db.search(&query_embedding, 100).await?;

// 3. Rank fusion (combine dense + sparse)
let fused = rrf(&dense_results, &sparse_results, 60)?;

// 4. Rerank top-K with MaxSim or cross-encoder
let top_candidates = fused.iter().take(20);
let reranked = rerank_with_maxsim(&query_tokens, &top_candidates)?;

// 5. Evaluate
let ndcg = ndcg_at_k(&reranked, &ground_truth, 10)?;
```

### Complete Example

See `examples/rag_pipeline_complete.rs` for a complete end-to-end example.

**Pipeline Components:**
1. **Vector DB**: Qdrant/usearch for dense retrieval
2. **rank-retrieve**: BM25/sparse retrieval
3. **rank-fusion**: Combining results (RRF, ISR, CombMNZ)
4. **rank-rerank**: Final reranking (MaxSim, cross-encoder)
5. **rank-eval**: Evaluation metrics (NDCG, MAP, MRR)

---

## Performance Considerations

### Dense Retrieval
- **Qdrant**: Optimized for production workloads, supports filtering
- **usearch**: Fast HNSW search, good for high-throughput scenarios
- **Trade-offs**: Qdrant offers more features, usearch is faster for pure ANN

### Sparse Retrieval
- **rank-retrieve BM25**: Fast, memory-efficient, good for keyword matching
- **Integration**: Can run in parallel with dense retrieval

### Reranking
- **MaxSim**: Fast, good for late interaction (ColBERT-style)
- **Cross-encoder**: Slower but more accurate, use for top-K reranking

---

## Best Practices

1. **Hybrid Retrieval**: Always combine dense + sparse for best results
2. **Rank Fusion**: Use RRF (Reciprocal Rank Fusion) for combining results
3. **Reranking Budget**: Rerank top 20-100 candidates, not all results
4. **Evaluation**: Use `rank-eval` to measure pipeline performance

---

## References

- **Qdrant**: https://qdrant.tech/documentation/
- **usearch**: https://github.com/unum-cloud/usearch
- **rank-rank**: See individual crate READMEs for detailed API documentation

