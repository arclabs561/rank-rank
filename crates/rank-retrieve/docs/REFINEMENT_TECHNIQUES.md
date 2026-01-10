# Refinement Techniques in Information Retrieval

This document describes refinement techniques available in the rank-* ecosystem and how to use them effectively.

## What is Refinement?

**Refinement** in information retrieval refers to improving the quality, relevance, and precision of search results through sequential processing stages of increasing complexity. The fundamental principle is to use fast, approximate methods to create a shortlist of candidates, then apply more expensive but accurate methods to rank that shortlist.

The classic two-stage approach:
1. **Stage 1 (Coarse)**: Fast retrieval to narrow from large corpus (10M+ docs) to candidates (100-1000)
2. **Stage 2 (Refine)**: Expensive reranking to improve precision of the candidate set

## Refinement Techniques in rank-* Ecosystem

### 1. Matryoshka Refinement (rank-rerank)

**Matryoshka Representation Learning (MRL)** enables two-stage retrieval using embeddings that contain nested valid representations at multiple dimensional levels.

**How it works:**
- Embeddings are split into "head" (first k dims) and "tail" (remaining dims)
- Stage 1: Vector DB searches using head dimensions only (fast, coarse)
- Stage 2: Re-score candidates using tail dimensions (refined, precise)

**Example:**
```rust
use rank_retrieve::retrieve_dense;
use rank_rerank::matryoshka;

// Stage 1: Coarse search using head dimensions (e.g., first 512 dims)
let candidates = retrieve_dense(&retriever, &query_head, 1000)?;

// Stage 2: Refine using tail dimensions
let refined = matryoshka::refine(
    &candidates,
    &query_full,  // Full query embedding
    &doc_full,    // Full document embeddings
    512,          // head_dims: first 512 are head, rest are tail
);
```

**When to use:**
- You have Matryoshka embeddings (e.g., OpenAI text-embedding-3-large)
- Need to balance speed and accuracy
- Storage-constrained (can use head dims for indexing, tail for refinement)

**Benefits:**
- 50-80% storage reduction using head dimensions for indexing
- Near-optimal accuracy when refining with tail dimensions
- No quality loss compared to full-dimensional search

**Reference:** [Matryoshka Representation Learning](https://arxiv.org/abs/2205.13147) (Kusupati et al., 2022)

### 2. ColBERT/ColPali Refinement (rank-rerank)

**Late interaction refinement** blends initial retrieval scores with MaxSim (token-level matching) scores.

**How it works:**
- Stage 1: Initial retrieval (BM25, dense, or sparse) produces candidates with scores
- Stage 2: MaxSim reranking computes token-level similarity
- Blend: `final_score = alpha × original_score + (1 - alpha) × maxsim_score`

**Example:**
```rust
use rank_retrieve::retrieve_bm25;
use rank_rerank::colbert;

// Stage 1: BM25 retrieval
let candidates = retrieve_bm25(&index, &query_terms, 1000, params)?;

// Stage 2: Refine with MaxSim
let query_tokens = encode_query(&query_text)?;
let doc_tokens = get_document_tokens(&candidates);
let refined = colbert::refine(
    &candidates,      // Initial (id, score) pairs
    &query_tokens,    // Query token embeddings
    &doc_tokens,      // Document token embeddings
    0.5,              // alpha: 50% original, 50% MaxSim
);
```

**Alpha parameter:**
- `alpha = 1.0`: Use original scores only (no refinement)
- `alpha = 0.5`: Equal blend (default, good balance)
- `alpha = 0.0`: Use MaxSim only (ignores original scores)

**When to use:**
- You have token-level embeddings (ColBERT, ColPali)
- Initial retrieval scores are useful but not perfect
- Want to combine lexical (BM25) with semantic (MaxSim) signals

**Benefits:**
- Combines strengths of multiple retrieval methods
- Token-level matching improves precision
- Works for both text (ColBERT) and multimodal (ColPali)

### 3. Cross-Encoder Refinement (rank-rerank)

**Cross-encoder refinement** uses transformer models that process query-document pairs jointly.

**How it works:**
- Stage 1: Fast bi-encoder retrieval (dense embeddings)
- Stage 2: Cross-encoder reranking (joint query-document processing)

**Example:**
```rust
use rank_retrieve::retrieve_dense;
use rank_rerank::crossencoder;

// Stage 1: Dense retrieval
let candidates = retrieve_dense(&retriever, &query_emb, 100)?;

// Stage 2: Cross-encoder refinement
let refined = crossencoder::refine(
    &model,           // Cross-encoder model
    &query_text,      // Query text
    &candidates,      // Candidate documents
    0.5,              // alpha: blend original and cross-encoder scores
);
```

**When to use:**
- Need highest accuracy for top results
- Can afford computational cost (slower than MaxSim)
- Have access to cross-encoder models (e.g., bge-reranker-v2-m3)

**Benefits:**
- Highest accuracy (better than bi-encoders)
- Models query-document interactions explicitly
- Best for final top-10 refinement

**Trade-offs:**
- Slower than MaxSim (requires full transformer forward pass per candidate)
- More expensive computationally
- Typically used only for top 10-100 candidates

### 4. Multi-Stage Refinement Pipeline

**Complete pipeline** combining multiple refinement techniques:

```rust
use rank_retrieve::{retrieve_bm25, retrieve_dense};
use rank_fusion::rrf;
use rank_rerank::{colbert, matryoshka};

// Stage 1: Hybrid retrieval (BM25 + dense)
let bm25_results = retrieve_bm25(&index, &query_terms, 1000, params)?;
let dense_results = retrieve_dense(&retriever, &query_emb, 1000)?;
let fused = rrf(&bm25_results, &dense_results);

// Stage 2: MaxSim refinement (top 100)
let top_100: Vec<_> = fused.iter().take(100).collect();
let query_tokens = encode_query(&query_text)?;
let doc_tokens = get_document_tokens(&top_100);
let maxsim_refined = colbert::refine(&top_100, &query_tokens, &doc_tokens, 0.5);

// Stage 3: Matryoshka refinement (top 10, if using Matryoshka embeddings)
let top_10: Vec<_> = maxsim_refined.iter().take(10).collect();
let matryoshka_refined = matryoshka::refine(
    &top_10,
    &query_full,
    &doc_full,
    512,  // head_dims
);

// Final: Cross-encoder refinement (top 5, highest accuracy)
let top_5: Vec<_> = matryoshka_refined.iter().take(5).collect();
let final_refined = crossencoder::refine(&model, &query_text, &top_5, 0.3);
```

## Choosing the Right Refinement Strategy

### Decision Tree

1. **Do you have Matryoshka embeddings?**
   - Yes → Use Matryoshka refinement for storage efficiency
   - No → Continue to next question

2. **Do you have token-level embeddings (ColBERT/ColPali)?**
   - Yes → Use ColBERT/ColPali refinement for token-level precision
   - No → Continue to next question

3. **Can you afford cross-encoder computation?**
   - Yes → Use cross-encoder for highest accuracy
   - No → Use simpler refinement or skip refinement

4. **What's your latency budget?**
   - <100ms → Single-stage retrieval only
   - 100-500ms → Two-stage (retrieve + one refinement)
   - >500ms → Multi-stage refinement pipeline

### Performance Characteristics

| Technique | Latency | Accuracy | Storage | Use Case |
|-----------|---------|----------|---------|----------|
| **No refinement** | Fastest | Baseline | Standard | Simple queries, low latency |
| **Matryoshka** | Fast | High | 50-80% reduction | Storage-constrained, Matryoshka embeddings |
| **ColBERT/ColPali** | Medium | Very High | Standard | Token-level precision needed |
| **Cross-encoder** | Slow | Highest | Standard | Top-10 final refinement |
| **Multi-stage** | Slowest | Highest | Varies | Complex queries, high accuracy needs |

## Best Practices

### 1. Start Simple, Add Refinement When Needed

Begin with single-stage retrieval. Add refinement only when:
- Accuracy is insufficient
- You have the computational budget
- Latency requirements allow

### 2. Use Appropriate Candidate Set Sizes

- **Matryoshka refinement**: 100-1000 candidates
- **ColBERT/ColPali refinement**: 50-200 candidates
- **Cross-encoder refinement**: 10-50 candidates

### 3. Balance Alpha Parameters

- Start with `alpha = 0.5` (equal blend)
- Tune based on your data:
  - If original scores are reliable → increase alpha (0.7-0.9)
  - If refinement scores are more accurate → decrease alpha (0.1-0.3)

### 4. Measure Impact

Always measure refinement impact:
- Accuracy improvement (NDCG, MAP, MRR)
- Latency increase
- Cost increase (if using paid APIs)

### 5. Consider Hybrid Approaches

Combine multiple refinement techniques:
- Matryoshka for storage efficiency
- ColBERT/ColPali for token-level precision
- Cross-encoder for final top results

## Integration with rank-retrieve

All refinement techniques work seamlessly with `rank-retrieve`:

1. **First-stage retrieval** (`rank-retrieve`):
   - BM25, dense, sparse, or hybrid retrieval
   - Returns `Vec<(u32, f32)>` candidates

2. **Refinement** (`rank-rerank`):
   - Matryoshka, ColBERT/ColPali, or cross-encoder
   - Takes candidates from `rank-retrieve`
   - Returns refined `Vec<(u32, f32)>`

3. **Evaluation** (`rank-eval`):
   - Measure improvement from refinement
   - Compare different refinement strategies

## Research References

1. **Matryoshka Representation Learning**: Kusupati et al. (2022). [arXiv:2205.13147](https://arxiv.org/abs/2205.13147)
2. **ColBERT**: Khattab & Zaharia (2020). "ColBERT: Efficient and Effective Passage Search via Contextualized Late Interaction over BERT". [arXiv:2004.12832](https://arxiv.org/abs/2004.12832)
3. **ColPali**: Biten et al. (2024). "ColPali: Efficient Document Retrieval with Vision Language Models". [arXiv:2407.01449](https://arxiv.org/abs/2407.01449)
4. **Cross-Encoders**: Nogueira & Cho (2019). "Passage Re-ranking with BERT". [arXiv:1901.04085](https://arxiv.org/abs/1901.04085)

## See Also

- `rank-rerank`'s [Matryoshka documentation](../rank-rerank/src/matryoshka.rs)
- `rank-rerank`'s [ColBERT documentation](../rank-rerank/src/colbert.rs)
- `rank-retrieve`'s [Late Interaction Guide](LATE_INTERACTION_GUIDE.md)
- `rank-retrieve`'s [PLAID Analysis](PLAID_ANALYSIS.md)
