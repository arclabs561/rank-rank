//! Cross-method consistency property tests.
//!
//! Verifies that BM25, dense, and sparse retrieval all follow the same
//! invariants: sorted results, finite scores, no duplicates, respect k.

use proptest::prelude::*;
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

proptest! {
    #[test]
    fn test_all_methods_sorted_results(
        num_docs in 10u32..100,
        k in 1usize..50
    ) {
        // Property: All retrieval methods should return sorted results
        let k = k.min(num_docs as usize);
        
        #[cfg(feature = "bm25")]
        {
            let mut index = InvertedIndex::new();
            for i in 0..num_docs {
                let terms: Vec<String> = (0..10)
                    .map(|j| format!("term{}", (i + j) % 20))
                    .collect();
                index.add_document(i, &terms);
            }
            
            let query = vec!["term0".to_string(), "term1".to_string()];
            let results = index.retrieve(&query, k, Bm25Params::default()).unwrap();
            
            // Verify sorted
            for i in 1..results.len() {
                prop_assert!(
                    results[i - 1].1 >= results[i].1,
                    "BM25 results should be sorted: position {} score {} should be >= position {} score {}",
                    i - 1,
                    results[i - 1].1,
                    i,
                    results[i].1
                );
            }
        }
        
        #[cfg(feature = "dense")]
        {
            let mut retriever = DenseRetriever::new();
            for i in 0..num_docs {
                let embedding: Vec<f32> = (0..128)
                    .map(|j| ((i + j) as f32 * 0.01).sin())
                    .collect();
                retriever.add_document(i, embedding);
            }
            
            let query: Vec<f32> = (0..128).map(|j| (j as f32 * 0.01).sin()).collect();
            let results = retriever.retrieve(&query, k).unwrap();
            
            // Verify sorted
            for i in 1..results.len() {
                prop_assert!(
                    results[i - 1].1 >= results[i].1,
                    "Dense results should be sorted"
                );
            }
        }
        
        #[cfg(feature = "sparse")]
        {
            let mut retriever = SparseRetriever::new();
            for i in 0..num_docs {
                let indices: Vec<u32> = (0..50).map(|j| ((i + j) % 100) as u32).collect();
                let values: Vec<f32> = (0..50).map(|j| ((i + j) as f32 * 0.01).sin().abs()).collect();
                let vector = SparseVector::new_unchecked(indices, values);
                retriever.add_document(i, vector);
            }
            
            let query_indices: Vec<u32> = (0..20).map(|i| (i * 5) as u32).collect();
            let query_values: Vec<f32> = vec![1.0; 20];
            let query = SparseVector::new_unchecked(query_indices, query_values);
            let results = retriever.retrieve(&query, k).unwrap();
            
            // Verify sorted
            for i in 1..results.len() {
                prop_assert!(
                    results[i - 1].1 >= results[i].1,
                    "Sparse results should be sorted"
                );
            }
        }
    }

    #[test]
    fn test_all_methods_finite_scores(
        num_docs in 10u32..100,
        k in 1usize..50
    ) {
        // Property: All retrieval methods should return finite scores
        let k = k.min(num_docs as usize);
        
        #[cfg(feature = "bm25")]
        {
            let mut index = InvertedIndex::new();
            for i in 0..num_docs {
                let terms: Vec<String> = (0..10)
                    .map(|j| format!("term{}", (i + j) % 20))
                    .collect();
                index.add_document(i, &terms);
            }
            
            let query = vec!["term0".to_string()];
            let results = index.retrieve(&query, k, Bm25Params::default()).unwrap();
            
            for (doc_id, score) in &results {
                prop_assert!(
                    score.is_finite(),
                    "BM25 score should be finite for doc {}: {}",
                    doc_id,
                    score
                );
            }
        }
        
        #[cfg(feature = "dense")]
        {
            let mut retriever = DenseRetriever::new();
            for i in 0..num_docs {
                let embedding: Vec<f32> = (0..128)
                    .map(|j| ((i + j) as f32 * 0.01).sin())
                    .collect();
                retriever.add_document(i, embedding);
            }
            
            let query: Vec<f32> = (0..128).map(|j| (j as f32 * 0.01).sin()).collect();
            let results = retriever.retrieve(&query, k).unwrap();
            
            for (doc_id, score) in &results {
                prop_assert!(
                    score.is_finite(),
                    "Dense score should be finite for doc {}: {}",
                    doc_id,
                    score
                );
            }
        }
        
        #[cfg(feature = "sparse")]
        {
            let mut retriever = SparseRetriever::new();
            for i in 0..num_docs {
                let indices: Vec<u32> = (0..50).map(|j| ((i + j) % 100) as u32).collect();
                let values: Vec<f32> = (0..50).map(|j| ((i + j) as f32 * 0.01).sin().abs()).collect();
                let vector = SparseVector::new_unchecked(indices, values);
                retriever.add_document(i, vector);
            }
            
            let query_indices: Vec<u32> = (0..20).map(|i| (i * 5) as u32).collect();
            let query_values: Vec<f32> = vec![1.0; 20];
            let query = SparseVector::new_unchecked(query_indices, query_values);
            let results = retriever.retrieve(&query, k).unwrap();
            
            for (doc_id, score) in &results {
                prop_assert!(
                    score.is_finite(),
                    "Sparse score should be finite for doc {}: {}",
                    doc_id,
                    score
                );
            }
        }
    }

    #[test]
    fn test_all_methods_no_duplicates(
        num_docs in 10u32..100,
        k in 1usize..50
    ) {
        // Property: All retrieval methods should return no duplicate document IDs
        let k = k.min(num_docs as usize);
        
        #[cfg(feature = "bm25")]
        {
            let mut index = InvertedIndex::new();
            for i in 0..num_docs {
                let terms: Vec<String> = (0..10)
                    .map(|j| format!("term{}", (i + j) % 20))
                    .collect();
                index.add_document(i, &terms);
            }
            
            let query = vec!["term0".to_string()];
            let results = index.retrieve(&query, k, Bm25Params::default()).unwrap();
            
            let mut seen = std::collections::HashSet::new();
            for (doc_id, _) in &results {
                prop_assert!(
                    seen.insert(*doc_id),
                    "BM25 should not return duplicate doc IDs: {}",
                    doc_id
                );
            }
        }
        
        #[cfg(feature = "dense")]
        {
            let mut retriever = DenseRetriever::new();
            for i in 0..num_docs {
                let embedding: Vec<f32> = (0..128)
                    .map(|j| ((i + j) as f32 * 0.01).sin())
                    .collect();
                retriever.add_document(i, embedding);
            }
            
            let query: Vec<f32> = (0..128).map(|j| (j as f32 * 0.01).sin()).collect();
            let results = retriever.retrieve(&query, k).unwrap();
            
            let mut seen = std::collections::HashSet::new();
            for (doc_id, _) in &results {
                prop_assert!(
                    seen.insert(*doc_id),
                    "Dense should not return duplicate doc IDs: {}",
                    doc_id
                );
            }
        }
        
        #[cfg(feature = "sparse")]
        {
            let mut retriever = SparseRetriever::new();
            for i in 0..num_docs {
                let indices: Vec<u32> = (0..50).map(|j| ((i + j) % 100) as u32).collect();
                let values: Vec<f32> = (0..50).map(|j| ((i + j) as f32 * 0.01).sin().abs()).collect();
                let vector = SparseVector::new_unchecked(indices, values);
                retriever.add_document(i, vector);
            }
            
            let query_indices: Vec<u32> = (0..20).map(|i| (i * 5) as u32).collect();
            let query_values: Vec<f32> = vec![1.0; 20];
            let query = SparseVector::new_unchecked(query_indices, query_values);
            let results = retriever.retrieve(&query, k).unwrap();
            
            let mut seen = std::collections::HashSet::new();
            for (doc_id, _) in &results {
                prop_assert!(
                    seen.insert(*doc_id),
                    "Sparse should not return duplicate doc IDs: {}",
                    doc_id
                );
            }
        }
    }

    #[test]
    fn test_all_methods_respect_k(
        num_docs in 10u32..100,
        k in 1usize..50
    ) {
        // Property: All retrieval methods should respect k parameter
        let k = k.min(num_docs as usize);
        
        #[cfg(feature = "bm25")]
        {
            let mut index = InvertedIndex::new();
            for i in 0..num_docs {
                let terms: Vec<String> = (0..10)
                    .map(|j| format!("term{}", (i + j) % 20))
                    .collect();
                index.add_document(i, &terms);
            }
            
            let query = vec!["term0".to_string()];
            let results = index.retrieve(&query, k, Bm25Params::default()).unwrap();
            
            prop_assert!(
                results.len() <= k,
                "BM25 should return at most k results: got {}, expected <= {}",
                results.len(),
                k
            );
        }
        
        #[cfg(feature = "dense")]
        {
            let mut retriever = DenseRetriever::new();
            for i in 0..num_docs {
                let embedding: Vec<f32> = (0..128)
                    .map(|j| ((i + j) as f32 * 0.01).sin())
                    .collect();
                retriever.add_document(i, embedding);
            }
            
            let query: Vec<f32> = (0..128).map(|j| (j as f32 * 0.01).sin()).collect();
            let results = retriever.retrieve(&query, k).unwrap();
            
            prop_assert!(
                results.len() <= k,
                "Dense should return at most k results: got {}, expected <= {}",
                results.len(),
                k
            );
        }
        
        #[cfg(feature = "sparse")]
        {
            let mut retriever = SparseRetriever::new();
            for i in 0..num_docs {
                let indices: Vec<u32> = (0..50).map(|j| ((i + j) % 100) as u32).collect();
                let values: Vec<f32> = (0..50).map(|j| ((i + j) as f32 * 0.01).sin().abs()).collect();
                let vector = SparseVector::new_unchecked(indices, values);
                retriever.add_document(i, vector);
            }
            
            let query_indices: Vec<u32> = (0..20).map(|i| (i * 5) as u32).collect();
            let query_values: Vec<f32> = vec![1.0; 20];
            let query = SparseVector::new_unchecked(query_indices, query_values);
            let results = retriever.retrieve(&query, k).unwrap();
            
            prop_assert!(
                results.len() <= k,
                "Sparse should return at most k results: got {}, expected <= {}",
                results.len(),
                k
            );
        }
    }
}
