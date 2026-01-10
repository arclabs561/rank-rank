# Low-Level Implementation Insights

This document compiles low-level implementation details, optimizations, and wisdom from existing well-regarded implementations (Tantivy, SPLADE, ColBERT, BM25S) and applies them to improve `rank-retrieve`.

## Table of Contents

1. [Tantivy BM25 Optimizations](#tantivy-bm25-optimizations)
2. [SPLADE Learned Sparse Retrieval](#splade-learned-sparse-retrieval)
3. [ColBERT MaxSim Implementation](#colbert-maxsim-implementation)
4. [BM25S Eager Scoring](#bm25s-eager-scoring)
5. [Sparse Vector Optimizations](#sparse-vector-optimizations)
6. [Three-Way Retrieval Patterns](#three-way-retrieval-patterns)

---

## Tantivy BM25 Optimizations

### Code-Level Insights

**Source**: `quickwit-oss/tantivy/src/postings/mod.rs`, `src/indexer/segment_serializer.rs`

#### Segment-Based Indexing

**Pattern**: Tantivy uses segment-based indexing with memory-mapped files.

```rust
// Tantivy pattern (conceptual)
struct Segment {
    postings: MemoryMappedPostings,
    doc_lengths: MemoryMappedArray,
    field_norms: MemoryMappedArray,
}
```

**Key Insights**:
- **Memory mapping**: Enables efficient disk-based indexes without full RAM loading
- **Segment merging**: Allows incremental updates without full reindexing
- **Field-specific normalization**: Different fields can have different normalization strategies

**Application to rank-retrieve**:
- Current: Simple HashMap-based inverted index (in-memory)
- **Improvement**: Add optional segment-based structure for persistence (planned feature)
- **Keep**: Simple in-memory structure for lightweight usage

#### Skip Lists for Posting Lists

**Pattern**: Tantivy uses skip lists for efficient posting list traversal.

**Key Insights**:
- Skip lists enable O(log n) random access within posting lists
- Useful for early termination when processing top-k candidates
- Reduces memory overhead compared to full arrays

**Application to rank-retrieve**:
- Current: `Vec<u32>` for posting lists (simple, fast for in-memory)
- **Consideration**: Skip lists add complexity; only beneficial for very large posting lists (>10K entries)
- **Decision**: Keep simple Vec for now; add skip lists as optional optimization for large-scale

#### Block Compression

**Pattern**: Tantivy compresses posting lists in blocks.

**Key Insights**:
- Variable-byte encoding (VBE) for document IDs
- Delta encoding for term frequencies
- Reduces memory usage by 50-70% for large indexes

**Application to rank-retrieve**:
- Current: Uncompressed Vec storage
- **Improvement**: Add optional compression for persistence layer
- **Trade-off**: Compression adds CPU overhead; only beneficial for disk-based indexes

### Optimizations Applied

1. ✅ **Precomputed IDF**: Already implemented (lazy computation)
2. ✅ **Early termination**: Already implemented (top-k heap)
3. ✅ **Efficient candidate collection**: Already implemented (HashSet deduplication)
4. 🔄 **Eager scoring option**: To be added (BM25S-style)
5. ⏳ **Skip lists**: Future optimization for very large posting lists
6. ⏳ **Block compression**: Future optimization for persistence

---

## SPLADE Learned Sparse Retrieval

### Code-Level Insights

**Source**: `naver/splade`, `castorini/pyserini/docs/experiments-spladev2.md`

#### Architecture

**Pattern**: SPLADE uses BERT's MLM head to generate sparse vectors.

```python
# SPLADE pattern (conceptual)
class SPLADE:
    def __init__(self, model_name="naver/splade-cocondenser-ensembledistil"):
        self.model = AutoModelForMaskedLM.from_pretrained(model_name)
        self.tokenizer = AutoTokenizer.from_pretrained(model_name)
    
    def encode(self, text: str) -> SparseVector:
        # Tokenize
        tokens = self.tokenizer(text, return_tensors="pt")
        
        # Forward pass through BERT MLM head
        outputs = self.model(**tokens)
        logits = outputs.logits  # [batch, seq_len, vocab_size]
        
        # Apply ReLU and sum over sequence
        sparse_weights = F.relu(logits).sum(dim=1)  # [batch, vocab_size]
        
        # Prune low weights
        sparse_weights = sparse_weights * (sparse_weights > threshold)
        
        # Convert to sparse vector
        indices = torch.nonzero(sparse_weights).squeeze()
        values = sparse_weights[indices]
        
        return SparseVector(indices, values)
```

**Key Insights**:
- **30K-dimensional vectors**: Uses full BERT vocabulary (30,522 tokens)
- **Automatic expansion**: Learns which terms to expand (e.g., "car" → "vehicle", "automobile")
- **Stopword removal**: Automatically removes stopwords via learned weights
- **Outperforms BM25**: On many IR benchmarks (MS MARCO, BEIR)

**Application to rank-retrieve**:
- Current: Basic sparse vectors (manual construction)
- **Improvement**: Add SPLADE support for learned sparse retrieval
- **Implementation**: 
  - Feature-gated module `splade`
  - Integration with HuggingFace transformers (via PyO3 or ONNX)
  - Generate 30K-dimensional sparse vectors
  - Integrate with existing `SparseRetriever`

#### Performance Characteristics

**From research**:
- **Index size**: ~2-3x larger than BM25 (30K dimensions vs. document-specific)
- **Retrieval speed**: Comparable to BM25 (sparse dot product)
- **Effectiveness**: Outperforms BM25 on semantic queries, comparable on keyword queries

**Optimizations**:
- **Pruning**: Remove weights below threshold (typically 0.1)
- **Top-k selection**: Keep only top-k terms per document (typically 200-500)
- **Quantization**: Optional 8-bit quantization for storage

---

## ColBERT MaxSim Implementation

### Code-Level Insights

**Source**: `stanford-futuredata/ColBERT`, `joe32140/maxsim-web/src/js/maxsim-baseline.js`

#### MaxSim Algorithm

**Pattern**: For each query token, find best-matching document token.

```javascript
// MaxSim pattern (from maxsim-web)
function maxsim(queryTokens, docTokens) {
    let score = 0.0;
    
    for (let q = 0; q < queryTokens.length; q++) {
        let maxSim = -Infinity;
        
        for (let d = 0; d < docTokens.length; d++) {
            const sim = dotProduct(queryTokens[q], docTokens[d]);
            maxSim = Math.max(maxSim, sim);
        }
        
        score += maxSim;
    }
    
    return score;
}
```

**Key Insights**:
- **Complexity**: O(|Q| × |D| × d) where d is embedding dimension
- **SIMD acceleration**: Dot products can use SIMD (already in rank-retrieve)
- **Early termination**: Can skip document tokens below threshold
- **Token limits**: Typically 32 query tokens, 128-512 document tokens

**Application to rank-retrieve**:
- **Note**: MaxSim is implemented in `rank-rerank`, not `rank-retrieve` (correct architecture)
- **Integration**: `rank-retrieve` provides first-stage retrieval, `rank-rerank` provides MaxSim reranking
- **Improvement**: Ensure efficient integration between crates

#### Optimizations

**From research and implementations**:
1. **Token pooling**: Reduce document tokens by 50% with <1% quality loss
2. **Pruning**: Skip document tokens with low max similarity
3. **Batch processing**: Process multiple documents in parallel
4. **SIMD**: Use SIMD for dot products (already implemented in `rank-retrieve`)

---

## BM25S Eager Scoring

### Code-Level Insights

**Source**: BM25S paper (2024), eager sparse scoring implementations

#### Eager Scoring Pattern

**Pattern**: Precompute BM25 scores during indexing, store in sparse matrix.

```rust
// BM25S pattern (conceptual)
struct EagerBm25Index {
    // Precomputed scores: doc_id -> term_id -> score
    scores: HashMap<u32, SparseVector>,  // Sparse matrix representation
}

impl EagerBm25Index {
    fn add_document(&mut self, doc_id: u32, terms: &[String]) {
        // Precompute BM25 scores for all terms in document
        let mut sparse_scores = SparseVector::new();
        
        for term in terms {
            let term_id = self.vocab.get_or_insert(term);
            let score = self.compute_bm25_score(doc_id, term);
            sparse_scores.add(term_id, score);
        }
        
        self.scores.insert(doc_id, sparse_scores);
    }
    
    fn retrieve(&self, query: &[String], k: usize) -> Vec<(u32, f32)> {
        // Fast retrieval: just sum precomputed scores
        let mut doc_scores: HashMap<u32, f32> = HashMap::new();
        
        for term in query {
            if let Some(term_id) = self.vocab.get(term) {
                for (doc_id, sparse_scores) in &self.scores {
                    if let Some(score) = sparse_scores.get(term_id) {
                        *doc_scores.entry(*doc_id).or_insert(0.0) += score;
                    }
                }
            }
        }
        
        // Sort and return top-k
        // ...
    }
}
```

**Key Insights**:
- **500x speedup**: For repeated queries (scores precomputed)
- **Memory trade-off**: Larger index size (stores all scores)
- **Query speed**: O(|Q| × |D| × sparsity) vs. O(|Q| × |D|) for lazy scoring
- **Best for**: Systems with many repeated queries

**Application to rank-retrieve**:
- Current: Lazy scoring (compute on-demand)
- **Improvement**: Add optional eager scoring mode
- **Implementation**:
  - Feature-gated `eager` mode
  - Store precomputed scores in sparse matrix
  - Fast retrieval via sparse dot product
  - Trade memory for speed

---

## Sparse Vector Optimizations

### Current Implementation

**Source**: `rank-retrieve/src/sparse/vector.rs`

#### Current Optimizations

1. ✅ **Sorted indices**: Enables efficient merge-based dot product
2. ✅ **SIMD acceleration**: Uses SIMD for index comparison (via `simd::sparse_dot`)
3. ✅ **Pruning**: `prune()` method for threshold-based filtering

#### Improvements from Research

**From SPLADE and BM25S implementations**:

1. **Block-based processing**: Process indices in blocks to reduce branch mispredictions
   ```rust
   // Improved pattern
   const BLOCK_SIZE: usize = 8;
   
   for block in indices.chunks(BLOCK_SIZE) {
       // Process block with SIMD
   }
   ```

2. **Top-k pruning**: Keep only top-k terms per vector (reduces memory)
   ```rust
   fn top_k(&self, k: usize) -> Self {
       // Keep only top-k terms by absolute value
   }
   ```

3. **Quantization**: Optional 8-bit quantization for storage
   ```rust
   struct QuantizedSparseVector {
       indices: Vec<u32>,
       values: Vec<u8>,  // Quantized to [0, 255]
       scale: f32,
   }
   ```

4. **Normalization**: L2 normalization for sparse vectors
   ```rust
   fn normalize(&self) -> Self {
       let norm = self.dot(self).sqrt();
       // Scale values by 1/norm
   }
   ```

---

## Graph-Based Vector Search Insights (2025 Research)

### Seed Selection Strategies

**Key Finding**: Seed selection significantly impacts both indexing and query performance.

**Stacked NSW (SN)** - Used by HNSW:
- Hierarchical multi-resolution graphs
- Logarithmic adaptation to dataset growth
- **Best for**: Billion-scale datasets
- **Trade-off**: Higher indexing overhead (182M-22.3B more distance calculations)

**K-Sampled Random Seeds (KS)**:
- K random nodes per query
- Lower indexing overhead
- **Best for**: Small to medium datasets (1M-25GB)
- **Trade-off**: Requires more samples on large datasets

**Recommendation**: Use SN for billion-scale, KS for medium-scale.

### Neighborhood Diversification

**Key Finding**: ND is crucial for query performance, especially at scale.

**RND (Relative Neighborhood Diversification)**:
- Highest pruning ratios (20-25%)
- Best overall performance
- Smaller graph sizes
- **Used by**: HNSW, NSG, SPTAG, ELPIS

**MOND (Maximum-Oriented ND)**:
- Angle-based diversification (θ ≥ 60°)
- Second-best performance
- Moderate pruning (2-4%)
- **Used by**: DPG, SSG, NSSG

**RRND (Relaxed RND)**:
- Relaxation factor α ≥ 1.5
- Less effective than RND
- Lowest pruning (0.6-0.7%)
- **Used by**: Vamana

**Recommendation**: Always use ND (RND preferred, MOND as alternative).

### Incremental Insertion vs Neighborhood Propagation

**Key Finding**: II-based methods have best scalability.

**Incremental Insertion (II)**:
- Builds graph one vertex at a time
- **Methods**: NSW, HNSW, Vamana
- **Performance**: Lowest indexing time, best scalability
- **Scales to**: 1B+ vectors

**Neighborhood Propagation (NP)**:
- Refines existing graph using NNDescent
- **Methods**: KGraph, EFANNA, NSG, SSG
- **Performance**: High indexing time, high memory
- **Scales to**: 25GB-100GB max (EFANNA needs 1.4TB for 100GB)

**Recommendation**: Prefer II-based methods for large-scale.

### Divide-and-Conquer for Hard Datasets

**Key Finding**: DC-based methods excel on hard datasets (high LID, low LRC).

**DC Methods**: SPTAG, HCNNG, ELPIS

**Performance**:
- Superior on challenging datasets (Seismic, RandPow0, RandPow50)
- ELPIS: Best overall on large and hard datasets
- **Trade-off**: Higher indexing time (SPTAG), but excellent search performance

**Recommendation**: Use DC-based methods for hard datasets/workloads.

## Three-Way Retrieval Patterns

### Architecture Pattern

**From research**: BM25 + dense + sparse is optimal for RAG.

**Pattern**:
```rust
// Three-way retrieval pattern
fn retrieve_three_way(
    bm25_index: &InvertedIndex,
    dense_index: &DenseRetriever,
    sparse_index: &SparseRetriever,
    query: &Query,
    k: usize,
) -> Vec<(u32, f32)> {
    // 1. Retrieve from each method
    let bm25_results = retrieve_bm25(bm25_index, &query.terms, k * 3, params)?;
    let dense_results = retrieve_dense(dense_index, &query.embedding, k * 3)?;
    let sparse_results = retrieve_sparse(sparse_index, &query.sparse_vector, k * 3)?;
    
    // 2. Fuse results (RRF or weighted)
    let fused = fuse_results(
        &[bm25_results, dense_results, sparse_results],
        FusionMethod::RRF { k: 60 },
    )?;
    
    // 3. Return top-k
    fused.into_iter().take(k).collect()
}
```

**Key Insights**:
- **Full-text (BM25)**: Handles keywords not in pre-trained vocabularies
- **Sparse (SPLADE)**: Better precision for pre-trained data scenarios
- **Dense**: Semantic understanding, meaning-based retrieval
- **Fusion**: RRF is common but weighted fusion can be better

**Application to rank-retrieve**:
- **Improvement**: Add helper functions for three-way retrieval
- **Integration**: Work seamlessly with `rank-fusion` for score fusion
- **Documentation**: Add examples and best practices

---

## Implementation Plan

### Phase 1: Core Optimizations (Current)

1. ✅ **BM25 eager scoring option**: Add `EagerBm25Index` with precomputed scores
2. ✅ **Sparse vector improvements**: Add top-k pruning, normalization, better SIMD
3. ✅ **Three-way retrieval helpers**: Add convenience functions for hybrid search

### Phase 2: Learned Sparse Retrieval

1. ⏳ **SPLADE support**: Add feature-gated module for learned sparse retrieval
2. ⏳ **Model integration**: Support HuggingFace models (via ONNX or PyO3)
3. ⏳ **Optimizations**: Pruning, quantization, top-k selection

### Phase 3: Advanced Features

1. ⏳ **Skip lists**: For very large posting lists
2. ⏳ **Block compression**: For persistence layer
3. ⏳ **Better score fusion**: Beyond RRF (weighted, learned)

---

## References

1. **Tantivy**: `quickwit-oss/tantivy` - Segment-based indexing, skip lists, block compression
2. **SPLADE**: `naver/splade` - Learned sparse retrieval, 30K-dimensional vectors
3. **ColBERT**: `stanford-futuredata/ColBERT` - MaxSim late interaction
4. **BM25S**: Eager sparse scoring paper (2024) - Precomputed scores, 500x speedup
5. **MaxSim**: `joe32140/maxsim-web` - JavaScript implementation with optimizations

---

## Conclusion

These low-level insights from existing implementations provide a roadmap for improving `rank-retrieve` while maintaining its core value: **simplicity and unified API**. The improvements focus on:

1. **Performance**: Eager scoring, better SIMD, optimizations
2. **Effectiveness**: SPLADE support, three-way retrieval
3. **Integration**: Better ecosystem integration, helper functions

All improvements are feature-gated to maintain lightweight usage for simple use cases.
