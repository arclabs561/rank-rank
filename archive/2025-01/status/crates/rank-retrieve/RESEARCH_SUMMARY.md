# Research Summary: Latest Findings and Implementation Priorities

## Key Research Insights (2024-2025)

### 1. LTRR: Learning To Rank Retrievers for LLMs
**Source**: SIGIR 2025, arXiv:2506.13743

**Critical Findings**:
- Pairwise XGBoost routers trained on Answer Correctness (AC) metric outperform single-retriever systems
- Query routing should optimize for downstream LLM utility, not traditional retrieval metrics
- Including "no-retrieval" as a routing option improves performance
- Models generalize to unseen query types when trained properly

**Action Items for rank-learn**:
- Prioritize pairwise LTR approaches (LambdaRank, LambdaMART)
- Support utility-aware training metrics (BEM, AC) in addition to NDCG
- Consider query routing framework for multi-retriever selection

### 2. Rankify Toolkit Patterns
**Source**: DataScienceUIBK/Rankify (528 stars)

**Key Patterns**:
- Unified interface across retrieval methods
- Comprehensive benchmark integration (40+ datasets)
- Model-agnostic design
- Clear separation: retrieval → reranking → generation

**Action Items**:
- Ensure rank-retrieve, rank-rerank, rank-fusion have consistent interfaces
- Document integration patterns for full pipeline
- Consider benchmark integration

### 3. HuggingFace Research Trends
**Key Trends**:
- ColBERT late interaction (MaxSim) for improved accuracy
- Focus on RAG optimization (answer correctness, faithfulness)
- Listwise and pairwise LTR approaches showing strong results
- Neural LTR using differentiable ranking operations

**Action Items**:
- Document ColBERT-style late interaction patterns
- Emphasize RAG-optimized metrics in documentation
- Support both listwise and pairwise LTR approaches

## Testing Patterns (From rank-fusion, rank-learn, hop)

### Property-Based Testing Examples

**From rank-fusion**:
```rust
proptest! {
    #[test]
    fn rrf_output_bounded(a in arb_results(50), b in arb_results(50)) {
        let result = rrf(&a, &b);
        prop_assert!(result.len() <= a.len() + b.len());
    }
    
    #[test]
    fn rrf_scores_positive(a in arb_results(50), b in arb_results(50)) {
        let result = rrf(&a, &b);
        for (_, score) in &result {
            prop_assert!(*score > 0.0);
        }
    }
    
    #[test]
    fn rrf_commutative(a in arb_results(20), b in arb_results(20)) {
        let ab = rrf(&a, &b);
        let ba = rrf(&b, &a);
        prop_assert_eq!(ab.len(), ba.len());
        // Scores should match
    }
    
    #[test]
    fn rrf_sorted_descending(a in arb_results(50), b in arb_results(50)) {
        let result = rrf(&a, &b);
        for window in result.windows(2) {
            prop_assert!(window[0].1 >= window[1].1);
        }
    }
}
```

**From rank-learn**:
```rust
proptest! {
    #[test]
    fn ndcg_is_bounded(relevance in prop::collection::vec(0.0f32..10.0, 1..100)) {
        let ndcg = ndcg_at_k(&relevance, None).unwrap();
        prop_assert!(ndcg >= 0.0 && ndcg <= 1.0);
    }
    
    #[test]
    fn lambdarank_gradients_match_scores_length(
        scores in prop::collection::vec(-10.0f32..10.0, 1..100),
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
    ) {
        if scores.len() == relevance.len() && !scores.is_empty() {
            let trainer = LambdaRankTrainer::default();
            let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();
            prop_assert_eq!(lambdas.len(), scores.len());
        }
    }
}
```

### Property Tests Needed for rank-retrieve

1. **BM25**:
   - Scores are positive
   - IDF is monotonic (rare terms have higher IDF)
   - Output length <= input documents
   - Empty query/empty index handled correctly

2. **Dense Retrieval**:
   - Cosine similarity in [-1, 1]
   - Dimension consistency (query matches documents)
   - Normalized vectors produce valid scores
   - Empty embeddings handled

3. **Sparse Retrieval**:
   - Dot product properties (commutative, distributive)
   - Index sorting invariant
   - Value bounds (finite, reasonable ranges)
   - Empty vectors handled

## Python Binding Patterns (Validated)

### Successful Patterns from rank-retrieve Implementation

1. **Helper Functions**:
   ```rust
   fn py_list_to_ranked(py: Python, list: &PyList) -> PyResult<Vec<(u32, f32)>> {
       // Convert Python list of tuples to Rust Vec
   }
   ```

2. **Error Handling**:
   ```rust
   .map_err(|e| PyErr::new::<pyo3::exceptions::PyValueError, _>(e.to_string()))?
   ```

3. **Type Validation**:
   - Check dimensions before Rust calls
   - Validate empty inputs
   - Ensure indices are sorted (for sparse vectors)

4. **PyRef for Inner Access**:
   ```rust
   fn retrieve(&self, params: PyRef<'_, Bm25ParamsPy>) -> PyResult<...> {
       // Access params.inner directly
   }
   ```

## Implementation Priorities

### Immediate (rank-retrieve)
1. ✅ Python bindings implemented
2. ⏳ Comprehensive property tests
3. ⏳ Edge case tests
4. ⏳ Integration tests

### Next (rank-learn)
1. ⏳ Python bindings for LambdaRank
2. ⏳ Python bindings for NDCG
3. ⏳ Python bindings for Neural LTR (when ready)
4. ⏳ Property tests for LTR operations
5. ⏳ Support for utility-aware metrics (BEM, AC)

### Future (Research-Inspired)
1. Query routing framework (LTRR-style)
2. Multi-retriever fusion with routing
3. ColBERT late interaction patterns
4. Comprehensive benchmark integration

## References

1. **LTRR Paper**: [arXiv:2506.13743](https://arxiv.org/html/2506.13743v1) - Learning To Rank Retrievers for LLMs
2. **Rankify**: [DataScienceUIBK/Rankify](https://github.com/DataScienceUIBK/Rankify) - Comprehensive Python Toolkit
3. **rank-fusion proptests**: `crates/rank-fusion/src/proptests.rs`
4. **rank-learn proptests**: `crates/rank-learn/tests/property_tests.rs`
5. **hop proptests**: Example from arclabs561/hop

