# Integration Guide: Retrieval → Fusion → Rerank Pipeline

This guide demonstrates how to use the `rank-*` crates together in a complete retrieval pipeline.

## Pipeline Overview

```
Query
  ↓
rank-retrieve (10M → 1000 docs)
  ↓
rank-fusion (combine multiple retrieval results)
  ↓
rank-rerank (1000 → 100 docs)
  ↓
rank-learn (train ranking models)
  ↓
rank-eval (evaluate results)
```

## Python Example: Complete Pipeline

```python
import rank_retrieve
import rank_fusion
import rank_rerank
import rank_learn
import rank_eval

# Step 1: Retrieve candidates using multiple methods
# BM25 retrieval
bm25_index = rank_retrieve.InvertedIndex()
bm25_index.add_document(0, ["machine", "learning", "tutorial"])
bm25_index.add_document(1, ["deep", "learning", "neural", "networks"])
bm25_results = bm25_index.retrieve(["machine", "learning"], k=1000)

# Dense retrieval
dense_retriever = rank_retrieve.DenseRetriever()
dense_retriever.add_document(0, [0.8, 0.2, 0.1, ...])  # normalized embedding
dense_retriever.add_document(1, [0.7, 0.3, 0.2, ...])
query_embedding = [0.8, 0.2, 0.1, ...]
dense_results = dense_retriever.retrieve(query_embedding, k=1000)

# Step 2: Fuse multiple retrieval results
fused_results = rank_fusion.rrf(
    bm25_results,
    dense_results,
    k=60,
    top_k=1000
)

# Step 3: Rerank using cross-encoder or MaxSim
reranker = rank_rerank.CrossEncoderReranker()
reranked = reranker.rerank(
    query="machine learning",
    documents=[doc1, doc2, ...],
    candidates=fused_results[:1000],
    top_k=100
)

# Step 4: Train ranking model (optional)
trainer = rank_learn.LambdaRankTrainer()
scores = [0.8, 0.7, 0.6, ...]  # Model scores
relevance = [3.0, 2.0, 1.0, ...]  # Ground truth
gradients = trainer.compute_gradients(scores, relevance)

# Step 5: Evaluate results
ndcg = rank_eval.ndcg_at_k(relevance, k=10)
map_score = rank_eval.map(relevance)
```

## Rust Example: Complete Pipeline

```rust
use rank_retrieve::prelude::*;
use rank_fusion::rrf;
use rank_rerank::simd::maxsim_rerank;
use rank_learn::lambdarank::LambdaRankTrainer;
use rank_eval::ndcg_at_k;

// Step 1: Retrieve
let mut bm25_index = InvertedIndex::new();
bm25_index.add_document(0, &["machine", "learning"]);
let bm25_results = bm25_index.retrieve(&["machine", "learning"], 1000, Bm25Params::default())?;

let mut dense_retriever = DenseRetriever::new();
dense_retriever.add_document(0, vec![0.8, 0.2, 0.1]);
let dense_results = dense_retriever.retrieve(&[0.8, 0.2, 0.1], 1000)?;

// Step 2: Fuse
let fused = rrf(&bm25_results, &dense_results);

// Step 3: Rerank
let reranked = maxsim_rerank(&query_embeddings, &doc_embeddings, 100)?;

// Step 4: Train
let trainer = LambdaRankTrainer::default();
let gradients = trainer.compute_gradients(&scores, &relevance, None)?;

// Step 5: Evaluate
let ndcg = ndcg_at_k(&relevance, Some(10))?;
```

## Research-Inspired Patterns

### Query Routing (LTRR-style)

Based on the LTRR paper (SIGIR 2025), you can implement query routing to select the best retriever:

```python
import rank_learn

# Train a router to select between BM25 and dense retrieval
trainer = rank_learn.LambdaRankTrainer()

# For each query, try both retrievers
bm25_results = bm25_index.retrieve(query_terms, k=100)
dense_results = dense_retriever.retrieve(query_embedding, k=100)

# Score each retriever's results
bm25_utility = compute_utility(bm25_results, ground_truth)
dense_utility = compute_utility(dense_results, ground_truth)

# Train router to predict which retriever is better
# (Implementation would use LTRR framework)
```

### Multi-Retriever Fusion

Combine results from multiple specialized retrievers:

```python
# Sparse retrieval (BM25)
bm25_results = bm25_index.retrieve(query_terms, k=1000)

# Dense retrieval (embeddings)
dense_results = dense_retriever.retrieve(query_embedding, k=1000)

# Sparse vector retrieval (lexical matching)
sparse_results = sparse_retriever.retrieve(query_sparse_vector, k=1000)

# Fuse all three
fused = rank_fusion.rrf_multi([
    bm25_results,
    dense_results,
    sparse_results
], k=60, top_k=1000)
```

## Performance Considerations

1. **Retrieval Stage**: Use fast approximate methods (BM25, dense ANN)
2. **Fusion Stage**: RRF is O(n log n) - efficient for combining results
3. **Reranking Stage**: More expensive (cross-encoder), so limit to top 100-1000
4. **Training Stage**: LambdaRank gradients computed in O(n²) - batch efficiently

## Best Practices

1. **Retrieve broadly**: Get 1000+ candidates from first stage
2. **Fuse intelligently**: Use RRF or weighted fusion based on query type
3. **Rerank precisely**: Apply expensive reranking to top candidates only
4. **Evaluate consistently**: Use same metrics (NDCG@10) across experiments
5. **Train on utility**: Use downstream metrics (BEM, AC) not just retrieval metrics

## References

- **LTRR Paper**: Learning To Rank Retrievers for LLMs (SIGIR 2025)
- **Rankify**: Comprehensive Python toolkit patterns
- **rank-fusion**: RRF and weighted fusion algorithms
- **rank-rerank**: Cross-encoder and MaxSim reranking
- **rank-learn**: LambdaRank and Neural LTR training

