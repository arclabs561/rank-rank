//! Property-based tests for eager BM25 scoring.
//!
//! Verifies that eager BM25 produces same results as lazy BM25.

use proptest::prelude::*;
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::eager::EagerBm25Index;

#[cfg(feature = "bm25")]
proptest! {
    #[test]
    fn test_eager_vs_lazy_bm25_equivalence(
        num_docs in 5u32..50,
        query_terms in prop::collection::vec("[a-z]{3,8}", 1..5)
    ) {
        // Property: Eager BM25 should produce same scores as lazy BM25
        let mut standard_index = InvertedIndex::new();
        
        // Add documents with varying term frequencies
        for i in 0..num_docs {
            let terms: Vec<String> = (0..10)
                .map(|j| format!("term{}", (i + j) % 20))
                .collect();
            standard_index.add_document(i, &terms);
        }
        
        // Convert to eager index
        let eager_index = EagerBm25Index::from_bm25_index(&standard_index, Bm25Params::default());
        
        // Retrieve with both
        let standard_results = standard_index.retrieve(&query_terms, 10, Bm25Params::default()).unwrap();
        let eager_results = eager_index.retrieve(&query_terms, 10).unwrap();
        
        // Results should match (same documents, same scores)
        prop_assert_eq!(
            standard_results.len(),
            eager_results.len(),
            "Result lengths should match: standard={}, eager={}",
            standard_results.len(),
            eager_results.len()
        );
        
        for ((id1, score1), (id2, score2)) in standard_results.iter().zip(eager_results.iter()) {
            prop_assert_eq!(
                id1,
                id2,
                "Document IDs should match: standard={}, eager={}",
                id1,
                id2
            );
            prop_assert!(
                (score1 - score2).abs() < 1e-4,
                "Scores should match: doc {}: standard={}, eager={}, diff={}",
                id1,
                score1,
                score2,
                (score1 - score2).abs()
            );
        }
    }

    #[test]
    fn test_eager_bm25_retrieval_properties(
        num_docs in 10u32..100,
        query_terms in prop::collection::vec("[a-z]{3,8}", 1..5),
        k in 1usize..50
    ) {
        // Property: Eager BM25 should have same properties as lazy BM25
        let mut standard_index = InvertedIndex::new();
        
        for i in 0..num_docs {
            let terms: Vec<String> = (0..10)
                .map(|j| format!("term{}", (i + j) % 20))
                .collect();
            standard_index.add_document(i, &terms);
        }
        
        let eager_index = EagerBm25Index::from_bm25_index(&standard_index, Bm25Params::default());
        let k = k.min(num_docs as usize);
        let results = eager_index.retrieve(&query_terms, k).unwrap();
        
        // Property 1: Should return at most k results
        prop_assert!(
            results.len() <= k,
            "Should return at most k results: got {}, expected <= {}",
            results.len(),
            k
        );
        
        // Property 2: Results should be sorted descending
        for i in 1..results.len() {
            prop_assert!(
                results[i - 1].1 >= results[i].1,
                "Results should be sorted descending: position {} score {} should be >= position {} score {}",
                i - 1,
                results[i - 1].1,
                i,
                results[i].1
            );
        }
        
        // Property 3: All scores should be finite and non-negative
        for (doc_id, score) in &results {
            prop_assert!(
                score.is_finite(),
                "Score should be finite for doc {}: {}",
                doc_id,
                score
            );
            prop_assert!(
                *score >= 0.0,
                "Score should be non-negative for doc {}: {}",
                doc_id,
                score
            );
        }
        
        // Property 4: No duplicate document IDs
        let mut seen = std::collections::HashSet::new();
        for (doc_id, _) in &results {
            prop_assert!(
                seen.insert(*doc_id),
                "Duplicate document ID in results: {}",
                doc_id
            );
        }
    }
}
