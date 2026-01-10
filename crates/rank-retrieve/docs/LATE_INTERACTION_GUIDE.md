# Late Interaction Retrieval Guide

## Overview

This guide explains how to use `rank-retrieve` with `rank-rerank` for late interaction retrieval (ColBERT/ColPali-style token-level matching). It synthesizes the latest research (2024-2025) to provide evidence-based guidance. Supports both text (ColBERT) and multimodal (ColPali) late interaction.

**All code examples in this guide are validated by tests** in `tests/integration_doc_tests.rs` and `tests/executable_docs_tests.rs`.

## What is Late Interaction?

Late interaction models (like ColBERT) keep **one vector per token** instead of compressing entire documents into single vectors. This enables fine-grained token-level matching while maintaining efficient retrieval.

**Key difference:**
- **Dense retrieval**: "the quick brown fox" → `[0.1, 0.2, ...]` (1 vector)
- **Late interaction**: "the quick brown fox" → `[[...], [...], [...], [...]]` (4 vectors)

## Recommended Pipeline

### Standard Approach (Most Use Cases)

Research shows that **BM25 + ColBERT/ColPali reranking** often matches PLAID's efficiency-effectiveness trade-off (MacAvaney & Tonellotto, SIGIR 2024). This pipeline works for both text-only (ColBERT) and multimodal (ColPali) retrieval.

```rust
use rank_retrieve::{retrieve_bm25, bm25::{Bm25Params, InvertedIndex}};
use rank_rerank::colbert;

// 1. First-stage retrieval: BM25 (rank-retrieve)
let mut index = InvertedIndex::new();
// ... add documents to index ...
let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default())?;

// 2. Rerank with MaxSim (rank-rerank)
// Convert document IDs to token embeddings (you need to store these)
let doc_tokens: Vec<(&str, Vec<Vec<f32>>)> = candidates.iter()
    .map(|(id, _)| (*id, get_document_tokens(*id))) // Your function to get token embeddings
    .collect();

let query_tokens = encode_query(&query_text)?; // Your ColBERT encoder
let reranked = colbert::rank(&query_tokens, &doc_tokens);

// 3. Optional: Apply token pooling for storage optimization
// Pool documents at index time (50% reduction, <1% quality loss)
let pooled_docs: Vec<_> = doc_tokens.iter()
    .map(|(id, tokens)| (*id, colbert::pool_tokens(tokens, 2).unwrap()))
    .collect();
```

### Why This Works

**Research finding**: The BM25 + ColBERT/ColPali reranking pipeline provides excellent efficiency-effectiveness trade-offs because:

1. **BM25 provides good recall**: Lexical matching retrieves most relevant documents
2. **MaxSim reranking improves precision**: Token-level matching refines the ranking
3. **Simple and fast**: No complex indexing infrastructure needed

**When PLAID becomes necessary:**
- Very high recall requirements (beyond BM25's capabilities)
- Very large collections (millions+ documents)
- Strict latency requirements (<100ms) with high recall needs

## Integration with rank-* Ecosystem

### Complete Pipeline

```
10M docs → 1000 candidates → 100 candidates → 10 results
    │            │                 │              │
    ▼            ▼                 ▼              ▼
[rank-retrieve] [rank-rerank]  [cross-encoder]  [User]
  (BM25/dense)   (MaxSim)       (optional)
```

### Example: Full Pipeline

```rust
use rank_retrieve::{retrieve_bm25, bm25::{Bm25Params, InvertedIndex}};
use rank_rerank::colbert;
use rank_fusion::rrf;
use rank_eval::binary::ndcg_at_k;

// 1. Retrieve (rank-retrieve)
let bm25_results = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default())?;

// 2. Rerank (rank-rerank)
let query_tokens = encode_query(&query_text)?;
let doc_tokens = get_document_tokens(&bm25_results);
let reranked = colbert::rank(&query_tokens, &doc_tokens);

// 3. Optional: Fuse multiple retrievers (rank-fusion)
let dense_results = retrieve_dense(&dense_index, &query_embedding, 1000)?;
let fused = rrf(&bm25_results, &dense_results);

// 4. Evaluate (rank-eval)
let ranked_ids: Vec<String> = reranked.iter().map(|(id, _)| id.to_string()).collect();
let relevant: HashSet<String> = get_relevant_docs(&query_id);
let ndcg = ndcg_at_k(&ranked_ids, &relevant, 10);
```

## Token Pooling Optimization

Token pooling is a research-backed optimization that reduces storage by 50-66% with <1% quality loss (Clavie et al., 2024).

### When to Use

- **Storage-constrained**: Large document collections
- **Index time**: Pool documents when building your index
- **Query time**: Keep queries at full resolution

### Implementation

```rust
use rank_rerank::colbert;

// Pool documents at index time (factor 2 = 50% reduction, ~0% quality loss)
let pooled_doc = colbert::pool_tokens(&doc_tokens, 2)?;

// Queries stay at full resolution for best quality
let score = colbert::maxsim(&query_tokens, &pooled_doc);
```

### Research-Backed Settings

| Factor | Storage Saved | Quality Loss | Use Case |
|--------|---------------|--------------|----------|
| 2 | 50% | ~0% | **Default** - near-free compression |
| 3 | 66% | ~1% | **Good tradeoff** - minimal impact |
| 4+ | 75%+ | 3-5% | Storage-critical (use `hierarchical` feature) |

## Comparison with Alternatives

### BM25 + MaxSim (Current Recommendation)

**Pros:**
- Simple to implement
- Excellent efficiency-effectiveness trade-off
- Well-tested and reliable
- No specialized infrastructure

**Cons:**
- Limited by BM25's recall
- May miss semantically relevant documents

**When to use:** Most use cases (default choice)

### PLAID (Future Enhancement)

**Pros:**
- Higher recall than BM25
- Optimized for large collections
- 45x CPU speedup over naive ColBERT

**Cons:**
- Complex indexing infrastructure
- Research shows BM25+rerank often matches its trade-offs
- Higher implementation complexity

**When to use:** High-recall scenarios, very large collections, strict latency requirements

### SPLATE (Future Consideration)

**Pros:**
- Simpler than PLAID
- Uses standard inverted indexes
- Comparable effectiveness

**Cons:**
- Not yet implemented
- May have limitations for complex queries

**When to use:** Alternative to PLAID when simpler approach is preferred

### Multimodal Retrieval (ColPali)

The same pipeline works for multimodal retrieval (text-to-image):

```rust
use rank_retrieve::{retrieve_bm25, bm25::{Bm25Params, InvertedIndex}};
use rank_rerank::colbert;

// 1. First-stage retrieval: BM25 on document text (if available) or metadata
let mut index = InvertedIndex::new();
// ... add document text/metadata to index ...
let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default())?;

// 2. Rerank with ColPali (text query tokens vs image patch embeddings)
// In ColPali, document images are split into patches (e.g., 32×32 grid = 1024 patches)
// Each patch becomes a "token" embedding
let doc_image_patches: Vec<(&str, Vec<Vec<f32>>)> = candidates.iter()
    .map(|(id, _)| (*id, get_image_patch_embeddings(*id))) // Your function to get patch embeddings
    .collect();

let query_tokens = encode_query_text(&query_text)?; // Text query tokens
let reranked = colbert::rank(&query_tokens, &doc_image_patches);

// 3. Visual snippet extraction: identify which image patches match query tokens
let alignments = colbert::alignments(&query_tokens, &doc_image_patches[0].1);
// Use alignments to extract visual regions from document images
```

**ColPali vs ColBERT:**
- **ColBERT**: Text-to-text late interaction (query text tokens vs document text tokens)
- **ColPali**: Text-to-image late interaction (query text tokens vs image patch embeddings)
- **Same MaxSim algorithm**: Both use the same token-level matching, just different input modalities

**When to use ColPali:**
- Document image retrieval (PDFs, scanned documents, screenshots)
- Visual search where text metadata is limited
- Multimodal RAG pipelines combining text and images

See `rank-rerank`'s [multimodal documentation](../rank-rerank/rank-rerank-core/README.md#multimodal-support-colpali) for details.

## Best Practices

### 1. Use BM25 for First-Stage Retrieval

BM25 provides excellent recall for most queries and is fast:

```rust
let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default())?;
```

### 2. Apply Token Pooling at Index Time

Pool documents when building your index, not at query time:

```rust
// At index time
let pooled_doc = colbert::pool_tokens(&doc_tokens, 2)?;
index.add_document(id, pooled_doc);

// At query time - queries stay full resolution
let score = colbert::maxsim(&query_tokens, &pooled_doc);
```

### 3. Leverage rank-rerank's SIMD Acceleration

`rank-rerank` automatically uses SIMD for fast similarity computation:

```rust
use rank_rerank::simd;
let score = simd::maxsim_vecs(&query_tokens, &doc_tokens);
```

### 4. Consider Hybrid Retrieval

Combine BM25 and dense retrieval for better coverage:

```rust
let bm25_results = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default())?;
let dense_results = retrieve_dense(&dense_index, &query_embedding, 1000)?;
let fused = rrf(&bm25_results, &dense_results);
```

## Research References

1. **Reproducibility Study**: MacAvaney & Tonellotto (2024). "A Reproducibility Study of PLAID". SIGIR 2024. Shows BM25+rerank often matches PLAID's trade-offs.

2. **Token Pooling**: Clavie et al. (2024). "Token Pooling in Multi-Vector Retrieval". [arXiv:2409.14683](https://arxiv.org/abs/2409.14683). 50-66% reduction with <1% loss.

3. **PLAID**: Santhanam et al. (2022). "PLAID: An Efficient Engine for Late Interaction Retrieval". [arXiv:2205.09707](https://arxiv.org/abs/2205.09707).

4. **SPLATE**: "SPLATE: Sparse Late Interaction Retrieval" (SIGIR 2024). Simpler alternative to PLAID.

5. **ColPali**: "ColPali: Efficient Document Retrieval with Vision Language Models" (ICLR 2025). [arXiv:2407.01449](https://arxiv.org/abs/2407.01449). Multimodal late interaction for document images.

## See Also

- `rank-rerank`'s [PLAID and Optimization Guide](../rank-rerank/docs/PLAID_AND_OPTIMIZATION.md) for detailed research analysis
- `rank-retrieve`'s [PLAID Analysis](PLAID_ANALYSIS.md) for comprehensive PLAID overview
- `rank-rerank`'s [ColBERT Documentation](../rank-rerank/src/colbert.rs) for MaxSim implementation details
- `rank-rerank`'s [Multimodal Support](../rank-rerank/rank-rerank-core/README.md#multimodal-support-colpali) for ColPali usage
- `rank-retrieve`'s [Refinement Techniques](REFINEMENT_TECHNIQUES.md) for Matryoshka, ColBERT, and cross-encoder refinement
- `rank-retrieve`'s [ColPali Multimodal Example](../examples/colpali_multimodal_pipeline.rs) for complete ColPali pipeline

