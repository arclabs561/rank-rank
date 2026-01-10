# Testing Gaps Analysis

## Missing Property Tests

### 1. Heap vs Sort Correctness Properties

**Missing**: Properties verifying that heap-based early termination produces identical results to full sort.

```rust
// Property: Heap-based top-k should match full sort top-k
proptest! {
    #[test]
    fn test_heap_vs_sort_equivalence(
        scores in prop::collection::vec(prop::num::f32::ANY.filter(|&x| x.is_finite()), 10..1000),
        k in 1usize..100
    ) {
        let k = k.min(scores.len());
        
        // Full sort approach
        let mut full: Vec<(usize, f32)> = scores.iter().enumerate().map(|(i, &s)| (i, s)).collect();
        full.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
        let full_top_k: Vec<f32> = full.iter().take(k).map(|(_, s)| *s).collect();
        
        // Heap approach (simulate our implementation)
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;
        
        #[derive(PartialEq, PartialOrd)]
        struct FloatOrd(f32);
        impl Eq for FloatOrd {}
        impl Ord for FloatOrd {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
            }
        }
        
        let mut heap: BinaryHeap<Reverse<(FloatOrd, usize)>> = BinaryHeap::with_capacity(k + 1);
        for (id, &score) in scores.iter().enumerate() {
            if heap.len() < k {
                heap.push(Reverse((FloatOrd(score), id)));
            } else if let Some(&Reverse((FloatOrd(min_score), _))) = heap.peek() {
                if score > min_score {
                    heap.pop();
                    heap.push(Reverse((FloatOrd(score), id)));
                }
            }
        }
        
        let mut heap_top_k: Vec<f32> = heap.into_iter().map(|Reverse((FloatOrd(s), _))| s).collect();
        heap_top_k.sort_unstable_by(|a, b| b.total_cmp(a));
        
        // Should match
        prop_assert_eq!(full_top_k.len(), heap_top_k.len());
        for i in 0..full_top_k.len() {
            prop_assert!((full_top_k[i] - heap_top_k[i]).abs() < 1e-5);
        }
    }
}
```

### 2. Threshold Selection Properties

**Missing**: Properties verifying that heap vs sort threshold selection is correct.

```rust
// Property: Heap should be used when k < sqrt(num_docs) or k < 100
// Full sort should be used otherwise
proptest! {
    #[test]
    fn test_threshold_selection_correctness(
        num_docs in 10usize..10000,
        k in 1usize..500
    ) {
        let threshold = ((num_docs as f64).sqrt() as usize).min(100);
        let should_use_heap = k < threshold;
        
        // Verify that our implementation uses the correct approach
        // (This would require exposing internal implementation details or using benchmarks)
    }
}
```

### 3. BM25 Score Monotonicity Properties

**Missing**: Properties verifying BM25 score behavior with document/query variations.

```rust
// Property: Adding more matching terms should increase score (monotonicity)
proptest! {
    #[test]
    fn test_bm25_score_monotonicity(
        base_terms in prop::collection::vec("[a-z]{3,8}", 5..20),
        additional_terms in prop::collection::vec("[a-z]{3,8}", 1..10)
    ) {
        let mut index = InvertedIndex::new();
        let doc_id = 0;
        
        // Add document with base terms
        index.add_document(doc_id, &base_terms);
        
        // Query with base terms
        let query_base = base_terms.clone();
        let score_base = index.score(doc_id, &query_base, Bm25Params::default());
        
        // Query with base + additional terms
        let mut query_extended = base_terms.clone();
        query_extended.extend(additional_terms.clone());
        let score_extended = index.score(doc_id, &query_extended, Bm25Params::default());
        
        // Extended query should have higher or equal score
        prop_assert!(score_extended >= score_base);
    }
}
```

### 4. IDF Monotonicity Properties

**Missing**: Properties verifying IDF decreases as document frequency increases.

```rust
// Property: IDF should decrease as document frequency increases
proptest! {
    #[test]
    fn test_idf_monotonicity(
        num_docs in 10u32..1000
    ) {
        let mut index = InvertedIndex::new();
        
        // Add documents with varying term frequencies
        for i in 0..num_docs {
            let terms = if i < num_docs / 2 {
                vec!["common".to_string(), format!("doc{}", i)]
            } else {
                vec!["rare".to_string(), format!("doc{}", i)]
            };
            index.add_document(i, &terms);
        }
        
        let idf_common = index.idf("common");
        let idf_rare = index.idf("rare");
        
        // Common term (appears in more docs) should have lower IDF
        prop_assert!(idf_common < idf_rare);
    }
}
```

### 5. Sparse Vector Operations Properties

**Missing**: Properties for sparse vector operations beyond basic dot product.

```rust
// Property: top_k preserves relative ordering of top elements
proptest! {
    #[test]
    fn test_sparse_vector_top_k_preserves_ordering(
        indices in prop::collection::vec(0u32..1000, 10..100),
        values in prop::collection::vec(prop::num::f32::ANY.filter(|&x| x.is_finite()), 10..100),
        k in 1usize..50
    ) {
        // Ensure indices and values have same length
        let len = indices.len().min(values.len());
        let indices = indices[..len].to_vec();
        let values = values[..len].to_vec();
        
        // Create sparse vector
        let mut pairs: Vec<(u32, f32)> = indices.into_iter().zip(values.into_iter()).collect();
        pairs.sort_unstable_by_key(|(idx, _)| *idx);
        let (indices, values) = pairs.into_iter().unzip();
        let vector = SparseVector::new_unchecked(indices, values);
        
        let k = k.min(vector.nnz());
        let top_k = vector.top_k(k);
        
        // Verify top_k has k elements (or fewer if original had fewer)
        prop_assert!(top_k.nnz() <= k);
        prop_assert!(top_k.nnz() <= vector.nnz());
        
        // Verify all top_k values are in original vector
        // Verify top_k contains the k largest absolute values
    }
}
```

### 6. Eager BM25 Properties

**Missing**: Properties comparing eager vs lazy BM25 scoring.

```rust
// Property: Eager BM25 should produce same scores as lazy BM25
proptest! {
    #[test]
    fn test_eager_vs_lazy_bm25_equivalence(
        num_docs in 10u32..100,
        query_terms in prop::collection::vec("[a-z]{3,8}", 1..5)
    ) {
        // Create standard index
        let mut standard_index = InvertedIndex::new();
        // ... add documents ...
        
        // Convert to eager index
        let eager_index = EagerBm25Index::from_bm25_index(&standard_index, Bm25Params::default());
        
        // Retrieve with both
        let standard_results = standard_index.retrieve(&query_terms, 10, Bm25Params::default()).unwrap();
        let eager_results = eager_index.retrieve(&query_terms, 10).unwrap();
        
        // Results should match
        prop_assert_eq!(standard_results.len(), eager_results.len());
        for ((id1, score1), (id2, score2)) in standard_results.iter().zip(eager_results.iter()) {
            prop_assert_eq!(id1, id2);
            prop_assert!((score1 - score2).abs() < 1e-5);
        }
    }
}
```

### 7. Numerical Stability Properties

**Missing**: Properties for extreme values, subnormal numbers, very large/small scores.

```rust
// Property: Handling of extreme values
proptest! {
    #[test]
    fn test_extreme_value_handling(
        values in prop::collection::vec(
            prop::num::f32::ANY.filter(|&x| x.is_finite() && x.abs() < 1e20),
            10..100
        )
    ) {
        // Test that operations don't overflow/underflow
        // Test that results remain finite
    }
}

// Property: Subnormal number handling
proptest! {
    #[test]
    fn test_subnormal_handling() {
        // Test with values near f32::MIN_POSITIVE
        // Verify SIMD doesn't cause 100x slowdowns
    }
}
```

### 8. Cross-Method Consistency Properties

**Missing**: Properties verifying consistency across retrieval methods.

```rust
// Property: BM25, dense, and sparse should all return sorted, finite results
proptest! {
    #[test]
    fn test_cross_method_consistency(
        // Generate same document set for all methods
    ) {
        // Verify all methods:
        // - Return sorted results
        // - Have finite scores
        // - Have no duplicates
        // - Respect k parameter
    }
}
```

## Unrealistic Tests and Examples

### 1. Scale Issues

**Current State**:
- Examples use 3-5 documents
- Tests use 10-100 documents
- No tests with 1K+, 10K+, 100K+ documents

**Missing Realistic Scenarios**:
- **Small-scale production**: 1K-10K documents (small business knowledge base)
- **Medium-scale production**: 100K-1M documents (enterprise search)
- **Large-scale production**: 10M+ documents (web search scale)

**Recommendation**: Add benchmarks/tests with realistic document counts:
```rust
// tests/scale_tests.rs
#[test]
fn test_bm25_retrieval_10k_docs() {
    let mut index = InvertedIndex::new();
    // Load 10K realistic documents (e.g., from a sample dataset)
    // Test retrieval performance and correctness
}

#[test]
fn test_dense_retrieval_100k_docs() {
    // Test with 100K documents
    // Verify early termination works correctly
    // Measure performance
}
```

### 2. Query Realism

**Current State**:
- Queries are single words or 2-3 terms
- No realistic user queries
- No long queries, no conversational queries

**Missing Realistic Queries**:
- **E-commerce**: "wireless bluetooth headphones with noise cancellation under $200"
- **Technical docs**: "how to handle async errors in rust tokio with proper error propagation"
- **RAG/QA**: "what are the main differences between transformer attention mechanisms and RNNs?"
- **Long-tail**: Very specific, multi-part queries

**Recommendation**: Use real query datasets:
```rust
// tests/realistic_queries.rs
// Load queries from:
// - MS MARCO (real web queries)
// - BEIR (diverse domains)
// - Natural Questions (real Google queries)
```

### 3. Document Realism

**Current State**:
- Documents are single sentences or short phrases
- No realistic document lengths (50-500 words typical)
- No realistic vocabulary distributions
- No domain-specific content

**Missing Realistic Documents**:
- **Real document lengths**: 50-500 words (typical for passages)
- **Real vocabulary**: Zipfian distribution, domain-specific terms
- **Real content**: Actual Wikipedia articles, news articles, technical docs
- **Domain diversity**: Medical, legal, technical, general web content

**Recommendation**: Use real document datasets:
```rust
// tests/realistic_documents.rs
// Load documents from:
// - MS MARCO passages (real web content)
// - BEIR corpora (domain-specific)
// - Wikipedia (general knowledge)
```

### 4. Evaluation Realism

**Current State**:
- No ground truth relevance judgments
- No standard IR metrics (nDCG, MRR, Precision@k)
- No comparison with baseline methods
- No statistical significance testing

**Missing Realistic Evaluations**:
- **Ground truth**: Use qrels from MS MARCO, BEIR, TREC
- **Standard metrics**: nDCG@10, MRR, Precision@k, Recall@k
- **Baseline comparison**: Compare against BM25 baseline, dense baseline
- **Statistical testing**: Confidence intervals, significance tests

**Recommendation**: Integrate with real evaluation datasets:
```rust
// tests/realistic_evaluation.rs
use rank_eval::trec::load_qrels;
use rank_eval::binary::ndcg_at_k;

#[test]
fn test_bm25_on_msmarco_sample() {
    // Load MS MARCO sample (queries, corpus, qrels)
    // Run BM25 retrieval
    // Evaluate with nDCG@10
    // Compare against published baselines
}
```

### 5. Performance Realism

**Current State**:
- Benchmarks use synthetic data
- No realistic latency targets
- No throughput testing
- No memory usage testing

**Missing Realistic Performance Tests**:
- **Latency targets**: <10ms for 10M docs → 1K candidates
- **Throughput**: Queries per second under load
- **Memory usage**: Memory footprint for large indices
- **Scalability**: Performance degradation with scale

**Recommendation**: Add realistic performance tests:
```rust
// benches/realistic_performance.rs
#[bench]
fn bench_bm25_10m_docs_1k_candidates(b: &mut Bencher) {
    // Load 10M realistic documents
    // Measure retrieval latency
    // Target: <10ms
}
```

### 6. Integration Realism

**Current State**:
- Examples show isolated usage
- No end-to-end pipeline testing
- No integration with other rank-* crates in realistic scenarios

**Missing Realistic Integration**:
- **Full pipeline**: Retrieve → Fusion → Rerank → Eval with real data
- **Cross-crate integration**: Test with rank-fusion, rank-rerank, rank-eval
- **Production patterns**: Batch processing, caching, error handling

**Recommendation**: Add realistic integration tests:
```rust
// tests/realistic_integration.rs
#[test]
fn test_full_pipeline_with_real_data() {
    // Load real dataset (MS MARCO sample)
    // Run full pipeline: retrieve → fuse → rerank → eval
    // Verify end-to-end correctness and performance
}
```

## Recommendations

### High Priority

1. **Add Real-World Dataset Integration**
   - Integrate MS MARCO sample (1000 queries, 10K documents)
   - Integrate BEIR datasets (at least 2-3 domains)
   - Use real qrels for evaluation

2. **Add Scale Tests**
   - Test with 1K, 10K, 100K, 1M documents
   - Verify correctness and performance at scale
   - Test early termination effectiveness

3. **Add Property Tests for Heap Operations**
   - Heap vs sort equivalence
   - Threshold selection correctness
   - Numerical stability

### Medium Priority

4. **Add Realistic Query/Document Sets**
   - Use real queries from MS MARCO, BEIR
   - Use real documents (passages, articles)
   - Test with domain-specific content

5. **Add Standard IR Evaluation**
   - nDCG@k, MRR, Precision@k, Recall@k
   - Compare against published baselines
   - Statistical significance testing

6. **Add Performance Targets**
   - Latency targets for different scales
   - Throughput testing
   - Memory usage profiling

### Low Priority

7. **Add Advanced Property Tests**
   - BM25 monotonicity
   - IDF properties
   - Sparse vector operations
   - Cross-method consistency

8. **Add Integration Tests**
   - Full pipeline with real data
   - Cross-crate integration
   - Production patterns

## Implementation Plan

### Phase 1: Real-World Datasets (Week 1-2)
- [ ] Integrate MS MARCO sample loader
- [ ] Integrate BEIR dataset loader
- [ ] Add tests using real datasets
- [ ] Add evaluation with real qrels

### Phase 2: Scale Tests (Week 2-3)
- [ ] Add tests for 1K, 10K, 100K documents
- [ ] Verify early termination at scale
- [ ] Measure performance at scale

### Phase 3: Property Tests (Week 3-4)
- [ ] Add heap vs sort equivalence tests
- [ ] Add threshold selection tests
- [ ] Add numerical stability tests

### Phase 4: Evaluation Integration (Week 4-5)
- [ ] Add standard IR metrics
- [ ] Compare against baselines
- [ ] Add statistical testing
