# Similar Tools for Comparison

This document lists similar tools in Rust and other languages that we benchmark against.

## Rust Tools

### BM25 / Retrieval

- **`bm25` crate** (v2.3.2, 34K downloads)
  - Features: Embedder, scorer, in-memory search engine
  - Use case: In-memory search, vector DB integration
  - Repository: https://docs.rs/bm25
  - Benchmark: Compare BM25 scoring performance

- **`tantivy`** (Lucene-inspired search engine)
  - Features: Full-text search, BM25 scoring, inverted index
  - Use case: Production search engines
  - Repository: https://github.com/quickwit-oss/tantivy
  - Benchmark: Compare indexing and retrieval speed

- **`anda_db_tfs`**
  - Features: Segment indexing, top-k queries, CBOR serialization
  - Use case: Persistent full-text search
  - Repository: https://docs.rs/anda_db_tfs
  - Benchmark: Compare persistent storage performance

### Dense Retrieval / ANN

- **`hnsw`** (Hierarchical Navigable Small World)
  - Features: Approximate nearest neighbor search
  - Use case: Vector similarity search
  - Repository: https://github.com/guillaume-be/rusty-hnsw
  - Benchmark: Compare ANN search performance

- **`usearch`**
  - Features: Fast vector search and clustering
  - Use case: Similarity search
  - Repository: https://github.com/unum-cloud/usearch
  - Benchmark: Compare vector search speed

### Learning to Rank

- **No dedicated Rust LTR crates found**
  - Our `rank-learn` is pioneering in Rust
  - Compare against Python implementations (XGBoost, LightGBM)

### Rank Fusion

- **No dedicated Rust fusion crates found**
  - Our `rank-fusion` is pioneering in Rust
  - Compare against Python implementations

## Python Tools

### BM25 / Retrieval

- **`rank-bm25`**
  - Features: BM25 implementation for Python
  - Use case: Document retrieval
  - Repository: https://github.com/dorianbrown/rank_bm25
  - Benchmark: Compare BM25 scoring accuracy and speed

- **`sentence-transformers`**
  - Features: Dense embeddings for semantic search
  - Use case: Semantic retrieval
  - Repository: https://github.com/UKPLab/sentence-transformers
  - Benchmark: Compare embedding-based retrieval

### Reranking

- **`reranking` libraries** (various)
  - Features: Cross-encoder reranking
  - Use case: Post-retrieval refinement
  - Benchmark: Compare reranking accuracy and latency

### Learning to Rank

- **XGBoost** (with ranking objectives)
  - Features: Gradient boosting for ranking
  - Use case: LTR training
  - Benchmark: Compare LambdaRank/LambdaMART performance

- **LightGBM** (with ranking objectives)
  - Features: Fast gradient boosting for ranking
  - Use case: LTR training
  - Benchmark: Compare training speed and accuracy

## Comparison Strategy

### Performance Metrics

1. **Throughput**: Operations per second
2. **Latency**: Time per operation (p50, p95, p99)
3. **Memory**: Peak memory usage
4. **Accuracy**: Correctness of results (NDCG, MAP, etc.)

### Benchmark Scenarios

1. **Small scale**: 1K documents, 100 queries
2. **Medium scale**: 10K documents, 1K queries
3. **Large scale**: 100K documents, 10K queries
4. **Very large scale**: 1M documents, 100K queries

### Test Data

- Use standard IR datasets (MS MARCO, BEIR, etc.)
- Synthetic data for stress testing
- Real-world query patterns

## Benchmark Results

Results are documented in:
- `benchmarks/results/` (generated reports)
- Individual crate `PERFORMANCE.md` files
- This document (summary)

