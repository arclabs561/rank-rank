# ANN Benchmark Research

## Current State

The `rank-retrieve` dense retriever currently uses **brute-force cosine similarity** for all documents. This is O(n*d) where n is the number of documents and d is the dimension.

## ANN Integration Plan

The code comments indicate readiness for HNSW/FAISS integration, but it's not yet implemented. When ANN is added, benchmarks should follow these patterns:

## Benchmark Patterns from Research

### 1. HNSW Benchmarks (from `rusty-hnsw`, `vecstore`, etc.)

**Common Metrics:**
- **Indexing Time**: Time to build HNSW graph
- **Query Latency**: p50, p95, p99 percentiles
- **Recall@K**: Accuracy vs brute force (should be >95% for production)
- **Memory Usage**: Index size vs brute force storage
- **Throughput**: Queries per second

**Typical Configurations:**
- **M** (connections per layer): 16, 32, 64
- **ef_construction** (search width during build): 200, 400
- **ef_search** (search width during query): 50, 100, 200
- **Dimensions**: 128, 256, 384, 768, 1536
- **Dataset sizes**: 1K, 10K, 100K, 1M, 10M vectors

**Benchmark Structure:**
```rust
// Indexing benchmarks
for (n_docs, dim) in [(1_000, 128), (10_000, 256), (100_000, 384)].iter() {
    // Measure time to build index
}

// Query benchmarks
for (n_docs, dim, k, ef) in [
    (10_000, 128, 10, 50),
    (100_000, 256, 20, 100),
].iter() {
    // Measure query latency and recall
}
```

### 2. Comparison Benchmarks

**Baseline Comparisons:**
- **Brute Force**: Current implementation (exact, slow)
- **HNSW**: Approximate, fast (when integrated)
- **FAISS**: If integrated (highly optimized)
- **usearch**: Lightweight alternative

**Key Comparisons:**
- Speedup vs brute force (should be 10-100x for large datasets)
- Recall degradation (should be <5% for ef_search=100)
- Memory overhead (HNSW typically 2-3x brute force)

### 3. Real-World Workloads

**Typical RAG Pipeline:**
- **Corpus size**: 1M-10M documents
- **Query rate**: 10-1000 QPS
- **Dimension**: 384 (sentence-transformers) or 768 (BERT)
- **Top-K**: 10-100 candidates

**Benchmark Scenarios:**
1. **Cold Start**: First query after index build
2. **Warm Cache**: Repeated queries
3. **Batch Queries**: Multiple queries in parallel
4. **Incremental Updates**: Adding documents to existing index

## Implementation Recommendations

### Phase 1: Brute Force Baseline (Current)
✅ Already benchmarked in `benches/dense.rs`

### Phase 2: HNSW Integration
When HNSW is added:
1. Add `benches/ann_hnsw.rs` with:
   - Indexing benchmarks (various M, ef_construction)
   - Query benchmarks (various ef_search, k)
   - Recall@K measurements
   - Memory profiling

2. Comparison benchmarks:
   - HNSW vs brute force (speed/accuracy tradeoff)
   - Different HNSW parameters (M, ef_construction, ef_search)

### Phase 3: Production Benchmarks
- Large-scale datasets (1M+ vectors)
- Concurrent query handling
- Index persistence and loading
- Incremental updates

## References

From GitHub research:
- `rusty-hnsw`: HNSW implementation in Rust
- `vecstore`: Embeddable vector database with HNSW
- `usearch`: Fast vector search library
- `FAISS`: Facebook AI Similarity Search (C++/Python)

## Current Benchmark Status

✅ **Brute Force Dense Retrieval**: Fully benchmarked
- Indexing: 100-100K docs, 128-768 dims
- Retrieval: Various k values
- Scoring: Individual document scoring

⏳ **ANN Benchmarks**: Pending HNSW/FAISS integration
- Will be added when ANN is implemented
- Should follow patterns above

