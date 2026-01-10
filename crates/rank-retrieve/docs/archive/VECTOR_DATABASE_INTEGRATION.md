# Vector Database Integration Guide

This guide explains how to integrate `rank-retrieve` with production vector databases for large-scale dense retrieval.

## Overview

`rank-retrieve` provides efficient in-memory implementations suitable for any scale of corpora. For production systems requiring persistent storage, horizontal scaling, or distributed search, integrate with external vector databases.

## When to Use Vector Databases

**Use vector databases when:**
- Need persistent storage (survive restarts)
- Require horizontal scaling (distributed search)
- Need metadata filtering (date ranges, categories, etc.)
- Multi-tenant systems requiring isolation
- Production deployments with high availability requirements
- Want managed infrastructure

**Use in-memory `rank-retrieve` when:**
- Any scale of corpora (from small to very large)
- Prototyping or research
- Embedded applications
- Latency-critical scenarios (no network overhead)
- Production systems that can fit data in memory

## Integration Patterns

### Pattern 1: Qdrant Integration

**What is Qdrant?**
Qdrant is a production-ready vector database with persistent storage, horizontal scaling, and advanced filtering.

**Integration Steps:**

1. **Start Qdrant** (Docker):
   ```bash
   docker run -p 6333:6333 qdrant/qdrant
   ```

2. **Add dependency** to `Cargo.toml`:
   ```toml
   [dependencies]
   qdrant-client = { version = "1.7", optional = true }
   futures = { version = "0.3", optional = true }
   ```

3. **Enable feature**:
   ```toml
   [features]
   qdrant = ["dep:qdrant-client", "dep:futures"]
   ```

4. **Use in code**:
   ```rust
   use qdrant_client::{prelude::*, qdrant::*};
   
   // Create client
   let client = QdrantClient::from_url("http://localhost:6333").build()?;
   
   // Index documents (offline)
   for (doc_id, embedding) in documents {
       client.upsert_points(collection_name, vec![PointStruct::new(
           doc_id,
           embedding,
           payload: HashMap::new(),
       )]).await?;
   }
   
   // Retrieve (online)
   let results = client.search_points(&SearchPoints {
       collection_name: "documents".to_string(),
       vector: Some(Vector { vector: Some(Vectors::Dense(query_embedding)) }),
       limit: 1000,
       ..Default::default()
   }).await?;
   
   // Convert to rank-retrieve format
   let candidates: Vec<(u32, f32)> = results.result
       .into_iter()
       .map(|point| (point.id.num as u32, point.score as f32))
       .collect();
   
   // Rerank with rank-rerank
   let reranked = rank_rerank::simd::maxsim_batch(&query_tokens, &doc_tokens_list);
   ```

**See example:** `examples/qdrant_real_integration.rs`

**Performance:**
- Indexing: ~100-1000 docs/sec (depends on embedding dimension)
- Retrieval: ~5-20ms for 10M docs → 1000 candidates (depends on HNSW config)
- Supports metadata filtering, replication, backups

### Pattern 2: Usearch Integration

**What is Usearch?**
Usearch is a fast, header-only library for approximate nearest neighbor search using HNSW graphs.

**Integration Steps:**

1. **Add dependency** to `Cargo.toml`:
   ```toml
   [dependencies]
   usearch = "2.11"  # Optional, not a feature flag
   ```

2. **Use in code**:
   ```rust
   use usearch::{Index, IndexOptions, MetricKind, ScalarKind};
   
   // Create index
   let options = IndexOptions {
       dimensions: 128,
       metric: MetricKind::InnerProduct,
       quantization: ScalarKind::F32,
       ..Default::default()
   };
   let mut index = Index::new(&options)?;
   
   // Index documents
   for (doc_id, embedding) in documents {
       index.add(doc_id, &embedding)?;
   }
   
   // Build index
   index.reserve(documents.len())?;
   
   // Retrieve
   let results = index.search(&query_embedding, 1000)?;
   
   // Convert to rank-retrieve format
   let candidates: Vec<(u32, f32)> = results
       .into_iter()
       .map(|(id, distance)| (id, 1.0 - distance)) // Convert distance to similarity
       .collect();
   
   // Rerank with rank-rerank
   let reranked = rank_rerank::simd::maxsim_batch(&query_tokens, &doc_tokens_list);
   ```

**See example:** `examples/usearch_integration.rs`

**Performance:**
- Indexing: ~1000-10000 docs/sec (in-memory, very fast)
- Retrieval: ~1-5ms for 10M docs → 1000 candidates
- No persistence (in-memory only)

### Pattern 3: Custom Backend Integration

For other vector databases (Pinecone, Weaviate, Milvus, etc.), implement the `Backend` trait:

```rust
use rank_retrieve::integration::Backend;
use rank_retrieve::RetrieveError;

struct MyVectorDB {
    // Your vector database client
}

impl Backend for MyVectorDB {
    fn retrieve(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        // Your implementation
        // 1. Query your vector database
        // 2. Convert results to Vec<(u32, f32)>
        Ok(vec![])
    }
    
    fn add_document(&mut self, doc_id: u32, embedding: &[f32]) -> Result<(), RetrieveError> {
        // Your implementation
        Ok(())
    }
    
    fn build(&mut self) -> Result<(), RetrieveError> {
        // Your implementation (if needed)
        Ok(())
    }
}
```

## Hybrid Retrieval with Vector Databases

For production systems, combine vector database (dense) with `rank-retrieve` (BM25/sparse):

```rust
// 1. Dense retrieval from vector database
let dense_results = qdrant_client.search(&query_embedding, 1000).await?;

// 2. Sparse retrieval with rank-retrieve (BM25)
let sparse_results = rank_retrieve::retrieve_bm25(&bm25_index, &query_terms, 1000, params)?;

// 3. Fuse results
let fused = rank_fusion::rrf(&dense_results, &sparse_results);

// 4. Rerank top candidates
let reranked = rank_rerank::simd::maxsim_batch(&query_tokens, &top_candidates);
```

## Performance Comparison

| Method | Corpus Size | Indexing | Retrieval | Persistence | Scaling |
|--------|------------|----------|-----------|-------------|---------|
| `rank-retrieve` (in-memory) | <100K docs | Fast | ~1-5ms | ❌ | ❌ |
| Usearch (in-memory) | <10M docs | Very fast | ~1-5ms | ❌ | ❌ |
| Qdrant | 100M+ docs | Medium | ~5-20ms | ✅ | ✅ |
| Pinecone | 100M+ docs | Medium | ~10-30ms | ✅ | ✅ |

## Best Practices

1. **Use vector databases for production**: In-memory is fine for prototyping, but production needs persistence and scaling.

2. **Combine dense + sparse**: Use vector database for dense retrieval, `rank-retrieve` for BM25/sparse, then fuse with `rank-fusion`.

3. **Rerank after retrieval**: Always rerank top candidates (1000 → 100) with `rank-rerank` for final precision.

4. **Monitor performance**: Track indexing throughput, retrieval latency, and recall@k metrics.

5. **Use metadata filtering**: Vector databases support filtering by metadata (date, category, etc.) - use this for domain-specific retrieval.

## See Also

- [Qdrant Integration Example](examples/qdrant_real_integration.rs)
- [Usearch Integration Example](examples/usearch_integration.rs)
- [Full Pipeline Example](examples/full_pipeline.rs)
- [Late Interaction Guide](LATE_INTERACTION_GUIDE.md) - BM25 + ColBERT/ColPali reranking
- [Use Cases](USE_CASES.md) - When to use in-memory vs. vector databases
