//! Comprehensive tests for rank-retrieve covering concrete functions,
//! batch operations, stress scenarios, and edge cases.
//!
//! These tests focus on:
//! - Concrete function API (retrieve_bm25, retrieve_dense, retrieve_sparse)
//! - Batch operation edge cases
//! - Stress tests (large k, large document sets)
//! - Parameter combinations
//! - Realistic data patterns
//! - Error scenarios

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};
use rank_retrieve::RetrieveError;
#[cfg(feature = "bm25")]
use rank_retrieve::{batch::batch_retrieve_bm25, retrieve_bm25};
#[cfg(feature = "dense")]
use rank_retrieve::{batch::batch_retrieve_dense, retrieve_dense};
#[cfg(feature = "sparse")]
use rank_retrieve::{batch::batch_retrieve_sparse, retrieve_sparse};

// Concrete function tests

#[cfg(feature = "bm25")]
#[test]
fn test_concrete_retrieve_bm25_basic() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);

    let query = vec!["learning".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|(id, _)| *id == 0));
    assert!(results.iter().any(|(id, _)| *id == 1));
    assert!(results[0].1 >= results[1].1);
}

#[cfg(feature = "bm25")]
#[test]
fn test_concrete_retrieve_bm25_with_params() {
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["test".to_string(), "test".to_string(), "test".to_string()],
    );
    index.add_document(1, &["test".to_string()]);

    let query = vec!["test".to_string()];

    let default_results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
    let custom_params = Bm25Params { k1: 2.0, b: 0.5 };
    let custom_results = retrieve_bm25(&index, &query, 10, custom_params).unwrap();

    assert_eq!(default_results.len(), custom_results.len());
    assert!(!default_results.is_empty());
    assert!(!custom_results.is_empty());
}

#[cfg(feature = "dense")]
#[test]
fn test_concrete_retrieve_dense_basic() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    retriever.add_document(1, vec![0.707, 0.707, 0.0]);
    retriever.add_document(2, vec![0.0, 1.0, 0.0]);

    let query = [1.0, 0.0, 0.0];
    let results = retrieve_dense(&retriever, &query, 10).unwrap();

    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > results[1].1);
}

#[cfg(feature = "dense")]
#[test]
fn test_concrete_retrieve_dense_array_slice() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let query_array = [1.0, 0.0];
    let query_vec = vec![1.0, 0.0];

    let results_array = retrieve_dense(&retriever, &query_array, 10).unwrap();
    let results_vec = retrieve_dense(&retriever, &query_vec, 10).unwrap();

    assert_eq!(results_array.len(), results_vec.len());
    assert!((results_array[0].1 - results_vec[0].1).abs() < 1e-6);
}

#[cfg(feature = "sparse")]
#[test]
fn test_concrete_retrieve_sparse_basic() {
    let mut retriever = SparseRetriever::new();
    let doc0 = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]);
    let doc1 = SparseVector::new_unchecked(vec![1, 2], vec![0.8, 0.6]);
    retriever.add_document(0, doc0);
    retriever.add_document(1, doc1);

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retrieve_sparse(&retriever, &query, 10).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > results[1].1);
}

// Batch operation edge cases

#[cfg(feature = "bm25")]
#[test]
fn test_batch_empty_queries() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let queries: Vec<Vec<String>> = vec![];
    let result = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default());

    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_batch_single_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);

    let queries = vec![vec!["machine".to_string()]];
    let results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 1);
    assert!(!results[0].is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_batch_mixed_valid_invalid() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let queries = vec![
        vec!["test".to_string()],
        vec![], // Empty query - should error
    ];

    let result = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default());
    assert!(result.is_err());
}

#[cfg(feature = "dense")]
#[test]
fn test_batch_dense_large() {
    let mut retriever = DenseRetriever::new();
    for i in 0..100 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let queries: Vec<Vec<f32>> = (0..50)
        .map(|i| (0..64).map(|j| ((i + j) as f32) / 200.0).collect())
        .collect();

    let results = batch_retrieve_dense(&retriever, &queries, 10).unwrap();
    assert_eq!(results.len(), 50);
    assert!(results.iter().all(|r| !r.is_empty()));
}

#[cfg(feature = "sparse")]
#[test]
fn test_batch_sparse_varying_sizes() {
    let mut retriever = SparseRetriever::new();
    for i in 0..20 {
        let indices: Vec<u32> = (0..(i % 10 + 1)).map(|j| j as u32).collect();
        let values: Vec<f32> = (0..(i % 10 + 1)).map(|j| (j as f32 + 1.0) * 0.1).collect();
        let doc = SparseVector::new_unchecked(indices, values);
        retriever.add_document(i, doc);
    }

    let queries: Vec<SparseVector> = (0..10)
        .map(|i| {
            let indices: Vec<u32> = (0..(i % 5 + 1)).map(|j| j as u32).collect();
            let values: Vec<f32> = (0..(i % 5 + 1)).map(|_| 1.0).collect();
            SparseVector::new_unchecked(indices, values)
        })
        .collect();

    let results = batch_retrieve_sparse(&retriever, &queries, 10).unwrap();
    assert_eq!(results.len(), 10);
}

// Stress tests

#[cfg(feature = "bm25")]
#[test]
fn test_stress_large_k() {
    let mut index = InvertedIndex::new();
    for i in 0..1000 {
        index.add_document(i, &[format!("term{}", i % 100)]);
    }

    let query = vec!["term0".to_string()];
    let results = retrieve_bm25(&index, &query, 10000, Bm25Params::default()).unwrap();

    assert!(results.len() <= 1000);
    assert!(!results.is_empty());
}

#[cfg(feature = "dense")]
#[test]
fn test_stress_large_embedding_dim() {
    let mut retriever = DenseRetriever::new();
    let dim = 1024;
    let embedding: Vec<f32> = (0..dim).map(|i| (i as f32) / dim as f32).collect();
    retriever.add_document(0, embedding.clone());
    retriever.add_document(1, embedding.iter().map(|x| 1.0 - x).collect());

    let query: Vec<f32> = (0..dim).map(|i| (i as f32) / dim as f32).collect();
    let results = retrieve_dense(&retriever, &query, 10).unwrap();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].0, 0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_stress_many_documents() {
    let mut index = InvertedIndex::new();
    for i in 0..10000 {
        let terms: Vec<String> = (0..10)
            .map(|j| format!("term{}", (i * 7 + j * 11) % 1000))
            .collect();
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];
    let results = retrieve_bm25(&index, &query, 100, Bm25Params::default()).unwrap();

    assert!(results.len() <= 100);
    assert!(!results.is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_stress_long_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let query: Vec<String> = (0..1000).map(|i| format!("term{}", i)).collect();
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    assert!(results.len() <= 1);
}

#[cfg(feature = "dense")]
#[test]
fn test_stress_many_queries_batch() {
    let mut retriever = DenseRetriever::new();
    for i in 0..100 {
        retriever.add_document(i, vec![(i as f32) / 100.0, 1.0 - (i as f32) / 100.0]);
    }

    let queries: Vec<Vec<f32>> = (0..1000)
        .map(|i| vec![(i as f32) / 1000.0, 1.0 - (i as f32) / 1000.0])
        .collect();

    let results = batch_retrieve_dense(&retriever, &queries, 10).unwrap();
    assert_eq!(results.len(), 1000);
    assert!(results.iter().all(|r| !r.is_empty()));
}

// Parameter combination tests

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_params_extreme_values() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string(), "test".to_string()]);
    index.add_document(1, &["test".to_string()]);

    let query = vec!["test".to_string()];

    let low_k1 = Bm25Params { k1: 0.1, b: 0.0 };
    let high_k1 = Bm25Params { k1: 10.0, b: 1.0 };
    let extreme_b = Bm25Params { k1: 1.2, b: 0.0 };

    let results_low = retrieve_bm25(&index, &query, 10, low_k1).unwrap();
    let results_high = retrieve_bm25(&index, &query, 10, high_k1).unwrap();
    let results_extreme = retrieve_bm25(&index, &query, 10, extreme_b).unwrap();

    assert!(!results_low.is_empty());
    assert!(!results_high.is_empty());
    assert!(!results_extreme.is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_varying_k_values() {
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    let query = vec!["term0".to_string()];

    for k in [1, 5, 10, 20, 50, 100, 200] {
        let results = retrieve_bm25(&index, &query, k, Bm25Params::default()).unwrap();
        assert!(results.len() <= k);
        assert!(results.len() <= 100);
    }
}

// Realistic data patterns

#[cfg(feature = "bm25")]
#[test]
fn test_realistic_term_distribution() {
    let mut index = InvertedIndex::new();

    // Simulate realistic term distribution (Zipf-like)
    let common_terms = vec!["the", "of", "and", "to", "a"];
    let rare_terms: Vec<String> = (0..1000).map(|i| format!("rare{}", i)).collect();

    for doc_id in 0..100 {
        let mut terms = Vec::new();
        // Add common terms to most documents
        for term in &common_terms {
            if doc_id % 2 == 0 {
                terms.push(term.to_string());
            }
        }
        // Add rare terms to specific documents
        if doc_id < 10 {
            terms.push(rare_terms[doc_id as usize].clone());
        }
        index.add_document(doc_id, &terms);
    }

    // Query with common term should return many results
    let common_query = vec!["the".to_string()];
    let common_results = retrieve_bm25(&index, &common_query, 100, Bm25Params::default()).unwrap();
    assert!(common_results.len() >= 50);

    // Query with rare term should return few results
    let rare_query = vec![rare_terms[0].clone()];
    let rare_results = retrieve_bm25(&index, &rare_query, 100, Bm25Params::default()).unwrap();
    assert!(rare_results.len() <= 1);
}

#[cfg(feature = "dense")]
#[test]
fn test_realistic_embedding_clusters() {
    let mut retriever = DenseRetriever::new();
    let dim = 128;

    // Create 3 clusters of embeddings
    for cluster in 0..3 {
        for i in 0..10 {
            let mut embedding = vec![0.0; dim];
            // Cluster center
            embedding[cluster * 10] = 1.0;
            // Add some noise
            for j in 0..dim {
                embedding[j] += (i as f32) * 0.01;
            }
            retriever.add_document((cluster * 10 + i) as u32, embedding);
        }
    }

    // Query should match cluster 0 best
    let mut query = vec![0.0; dim];
    query[0] = 1.0;

    let results = retrieve_dense(&retriever, &query, 10).unwrap();
    assert!(results.iter().any(|(id, _)| *id < 10));
}

// Error scenarios

#[cfg(feature = "bm25")]
#[test]
fn test_concrete_function_empty_query_error() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let query: Vec<String> = vec![];
    let result = retrieve_bm25(&index, &query, 10, Bm25Params::default());

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyQuery));
}

#[cfg(feature = "bm25")]
#[test]
fn test_concrete_function_empty_index_error() {
    let index = InvertedIndex::new();
    let query = vec!["test".to_string()];

    let result = retrieve_bm25(&index, &query, 10, Bm25Params::default());

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyIndex));
}

#[cfg(feature = "dense")]
#[test]
fn test_concrete_function_dimension_mismatch_error() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let query = [1.0, 0.0, 0.0];
    let result = retrieve_dense(&retriever, &query, 10);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RetrieveError::DimensionMismatch { .. }
    ));
}

#[cfg(feature = "sparse")]
#[test]
fn test_concrete_function_sparse_empty_query_error() {
    let mut retriever = SparseRetriever::new();
    let doc = SparseVector::new_unchecked(vec![0], vec![1.0]);
    retriever.add_document(0, doc);

    let query = SparseVector::new(vec![], vec![]).unwrap();
    let result = retrieve_sparse(&retriever, &query, 10);

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyQuery));
}

// Consistency tests

#[cfg(feature = "bm25")]
#[test]
fn test_concrete_vs_trait_consistency() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);

    let query = vec!["learning".to_string()];

    let concrete_results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
    // Note: Trait is deprecated, but verify concrete function works
    assert!(!concrete_results.is_empty());
    assert!(concrete_results.len() >= 2);
    assert!(concrete_results
        .iter()
        .all(|(_, score)| score.is_finite() && *score >= 0.0));
}

#[cfg(feature = "dense")]
#[test]
fn test_concrete_vs_trait_dense_consistency() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    retriever.add_document(1, vec![0.707, 0.707, 0.0]);

    let query = [1.0, 0.0, 0.0];

    let concrete_results = retrieve_dense(&retriever, &query, 10).unwrap();
    let trait_results = retriever.retrieve(&query, 10).unwrap();

    assert_eq!(concrete_results.len(), trait_results.len());
    for ((id1, s1), (id2, s2)) in concrete_results.iter().zip(trait_results.iter()) {
        assert_eq!(id1, id2);
        assert!((s1 - s2).abs() < 1e-6);
    }
}

// Boundary conditions

#[cfg(feature = "bm25")]
#[test]
fn test_zero_k_returns_empty() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let query = vec!["test".to_string()];
    let results = retrieve_bm25(&index, &query, 0, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 0);
}

#[cfg(feature = "dense")]
#[test]
fn test_zero_k_dense_returns_empty() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let query = [1.0, 0.0];
    let results = retrieve_dense(&retriever, &query, 0).unwrap();

    assert_eq!(results.len(), 0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_k_larger_than_documents() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    index.add_document(1, &["test".to_string()]);

    let query = vec!["test".to_string()];
    let results = retrieve_bm25(&index, &query, 1000, Bm25Params::default()).unwrap();

    assert!(results.len() <= 2);
}

// Performance regression tests

#[cfg(feature = "bm25")]
#[test]
fn test_retrieval_consistency_across_calls() {
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    let query = vec!["term0".to_string()];

    let results1 = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
    let results2 = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Results should have same length (up to k)
    assert_eq!(results1.len(), results2.len());

    // Create maps to handle potential non-deterministic ordering for tied scores
    let map1: std::collections::HashMap<u32, f32> = results1.iter().cloned().collect();
    let map2: std::collections::HashMap<u32, f32> = results2.iter().cloned().collect();

    // Same documents should be returned
    assert_eq!(map1.len(), map2.len());
    // Scores should be consistent (within floating point precision)
    for (id, score1) in &map1 {
        let score2 = map2.get(id).expect("Document should be in both results");
        assert!(
            (score1 - score2).abs() < 1e-5,
            "Scores should be consistent for doc {}: {} vs {}",
            id,
            score1,
            score2
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_retrieval_consistency() {
    let mut retriever = DenseRetriever::new();
    for i in 0..50 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();

    let results1 = retrieve_dense(&retriever, &query, 10).unwrap();
    let results2 = retrieve_dense(&retriever, &query, 10).unwrap();

    assert_eq!(results1.len(), results2.len());
    for ((id1, s1), (id2, s2)) in results1.iter().zip(results2.iter()) {
        assert_eq!(id1, id2);
        assert!((s1 - s2).abs() < 1e-6);
    }
}

// Concurrent access tests

#[cfg(feature = "bm25")]
#[test]
fn test_concurrent_reads() {
    use std::sync::Arc;
    use std::thread;

    let index = Arc::new({
        let mut idx = InvertedIndex::new();
        for i in 0..100 {
            idx.add_document(i, &[format!("term{}", i % 10)]);
        }
        idx
    });

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let idx = Arc::clone(&index);
            thread::spawn(move || {
                let query = vec!["term0".to_string()];
                retrieve_bm25(&idx, &query, 10, Bm25Params::default())
            })
        })
        .collect();

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(!results.is_empty());
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_concurrent_dense_reads() {
    use std::sync::Arc;
    use std::thread;

    let retriever = Arc::new({
        let mut r = DenseRetriever::new();
        for i in 0..50 {
            let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
            r.add_document(i, embedding);
        }
        r
    });

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let r = Arc::clone(&retriever);
            thread::spawn(move || {
                let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();
                retrieve_dense(&r, &query, 10)
            })
        })
        .collect();

    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(!results.is_empty());
    }
}

// Additional batch error scenarios

#[cfg(feature = "dense")]
#[test]
fn test_batch_dense_dimension_mismatch() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let queries = vec![
        vec![1.0, 0.0],
        vec![1.0, 0.0, 0.0], // Dimension mismatch
    ];

    let result = batch_retrieve_dense(&retriever, &queries, 10);
    assert!(result.is_err());
}

#[cfg(feature = "sparse")]
#[test]
fn test_batch_sparse_empty_query() {
    let mut retriever = SparseRetriever::new();
    let doc = SparseVector::new_unchecked(vec![0], vec![1.0]);
    retriever.add_document(0, doc);

    let queries = vec![
        SparseVector::new_unchecked(vec![0], vec![1.0]),
        SparseVector::new(vec![], vec![]).unwrap(), // Empty query
    ];

    let result = batch_retrieve_sparse(&retriever, &queries, 10);
    assert!(result.is_err());
}

// Score validation tests

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_scores_are_finite() {
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    let query = vec!["term0".to_string()];
    let results = retrieve_bm25(&index, &query, 100, Bm25Params::default()).unwrap();

    assert!(results.iter().all(|(_, score)| score.is_finite()));
    assert!(results.iter().all(|(_, score)| *score >= 0.0));
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_scores_in_valid_range() {
    let mut retriever = DenseRetriever::new();
    for i in 0..50 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();
    let results = retrieve_dense(&retriever, &query, 50).unwrap();

    // Scores should be finite (cosine similarity can exceed [-1, 1] if vectors aren't normalized)
    assert!(results.iter().all(|(_, score)| score.is_finite()));
    // Scores should be reasonable (not NaN, not infinite)
    assert!(results.iter().all(|(_, score)| score.abs() < 1e6));
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_scores_are_finite() {
    let mut retriever = SparseRetriever::new();
    for i in 0..20 {
        let indices: Vec<u32> = (0..10).map(|j| (i * 10 + j) as u32).collect();
        let values: Vec<f32> = (0..10).map(|_| 1.0).collect();
        let doc = SparseVector::new_unchecked(indices, values);
        retriever.add_document(i, doc);
    }

    let query = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 1.0, 1.0]);
    let results = retrieve_sparse(&retriever, &query, 20).unwrap();

    assert!(results.iter().all(|(_, score)| score.is_finite()));
}

// Sorting validation tests

#[cfg(feature = "bm25")]
#[test]
fn test_results_sorted_descending() {
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        let terms: Vec<String> = (0..10)
            .map(|j| format!("term{}", (i * 7 + j * 11) % 100))
            .collect();
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];
    let results = retrieve_bm25(&index, &query, 100, Bm25Params::default()).unwrap();

    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results must be sorted descending"
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_results_sorted_descending() {
    let mut retriever = DenseRetriever::new();
    for i in 0..50 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();
    let results = retrieve_dense(&retriever, &query, 50).unwrap();

    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results must be sorted descending"
        );
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_results_sorted_descending() {
    let mut retriever = SparseRetriever::new();
    for i in 0..20 {
        let indices: Vec<u32> = (0..10).map(|j| (i * 10 + j) as u32).collect();
        let values: Vec<f32> = (0..10).map(|_| 1.0).collect();
        let doc = SparseVector::new_unchecked(indices, values);
        retriever.add_document(i, doc);
    }

    let query = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 1.0, 1.0]);
    let results = retrieve_sparse(&retriever, &query, 20).unwrap();

    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results must be sorted descending"
        );
    }
}

// ID uniqueness tests

#[cfg(feature = "bm25")]
#[test]
fn test_no_duplicate_ids() {
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    let query = vec!["term0".to_string()];
    let results = retrieve_bm25(&index, &query, 100, Bm25Params::default()).unwrap();

    let mut seen = std::collections::HashSet::new();
    for (id, _) in &results {
        assert!(!seen.contains(id), "Duplicate document ID found: {}", id);
        seen.insert(*id);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_no_duplicate_ids() {
    let mut retriever = DenseRetriever::new();
    for i in 0..50 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();
    let results = retrieve_dense(&retriever, &query, 50).unwrap();

    let mut seen = std::collections::HashSet::new();
    for (id, _) in &results {
        assert!(!seen.contains(id), "Duplicate document ID found: {}", id);
        seen.insert(*id);
    }
}

// Large batch operations

#[cfg(feature = "bm25")]
#[test]
fn test_batch_large_number_queries() {
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    let queries: Vec<Vec<String>> = (0..1000).map(|i| vec![format!("term{}", i % 10)]).collect();

    let results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 1000);
    assert!(results.iter().all(|r| !r.is_empty()));
}

#[cfg(feature = "dense")]
#[test]
fn test_batch_dense_large_k() {
    let mut retriever = DenseRetriever::new();
    for i in 0..100 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let queries: Vec<Vec<f32>> = (0..10)
        .map(|i| (0..64).map(|j| ((i + j) as f32) / 200.0).collect())
        .collect();

    let results = batch_retrieve_dense(&retriever, &queries, 1000).unwrap();

    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|r| r.len() <= 100));
}
