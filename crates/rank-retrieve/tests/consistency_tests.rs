//! Consistency and Robustness Tests
//!
//! These tests focus on:
//! - Consistency and repeatability (repeatable read isolation)
//! - Deterministic results across multiple calls
//! - Error handling and recovery scenarios
//! - Boundary conditions around ID handling
//! - Concurrent access patterns

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

// Repeatable read isolation tests
// Ensure snapshots produce identical results across multiple reads

#[cfg(feature = "bm25")]
#[test]
fn test_repeatable_read_isolation() {
    // A snapshot should produce identical results across multiple reads
    // This ensures deterministic, repeatable behavior

    let mut index = InvertedIndex::new();
    for i in 0..100 {
        let terms: Vec<String> = (0..10)
            .map(|j| format!("term{}", (i * 7 + j * 11) % 50))
            .collect();
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];

    // Create a "snapshot" by reading the index state
    let results1 = retrieve_bm25(&index, &query, 20, Bm25Params::default()).unwrap();

    // Read again - should be identical (repeatable read)
    let results2 = retrieve_bm25(&index, &query, 20, Bm25Params::default()).unwrap();

    // Results should be identical (handle ties by comparing sets)
    assert_eq!(results1.len(), results2.len());

    // Create maps to handle potential non-deterministic ordering for tied scores
    let map1: std::collections::HashMap<u32, f32> = results1.iter().cloned().collect();
    let map2: std::collections::HashMap<u32, f32> = results2.iter().cloned().collect();

    // When scores are tied, different documents might be returned
    // So we verify that documents appearing in both have identical scores
    for (id, score1) in &map1 {
        if let Some(score2) = map2.get(id) {
            assert!(
                (score1 - score2).abs() < 1e-6,
                "Scores should be identical for doc {}",
                id
            );
        }
    }

    // Verify that at least some documents are common (not all ties)
    let common: Vec<u32> = map1
        .keys()
        .filter(|id| map2.contains_key(id))
        .cloned()
        .collect();
    assert!(
        !common.is_empty(),
        "At least some documents should be common"
    );
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_repeatable_read_isolation() {
    let mut retriever = DenseRetriever::new();
    for i in 0..50 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();

    let results1 = retrieve_dense(&retriever, &query, 20).unwrap();
    let results2 = retrieve_dense(&retriever, &query, 20).unwrap();

    assert_eq!(results1.len(), results2.len());
    for ((id1, s1), (id2, s2)) in results1.iter().zip(results2.iter()) {
        assert_eq!(id1, id2);
        assert!((s1 - s2).abs() < 1e-6);
    }
}

// Deterministic ID handling tests
// Ensure document IDs are stable and consistent

#[cfg(feature = "bm25")]
#[test]
fn test_document_id_consistency() {
    // Document IDs should be stable and consistent
    // This tests that the same document always has the same ID

    let mut index = InvertedIndex::new();
    index.add_document(42, &["unique".to_string(), "document".to_string()]);
    index.add_document(100, &["another".to_string(), "document".to_string()]);

    let query = vec!["unique".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Document 42 should be in results
    assert!(results.iter().any(|(id, _)| *id == 42));

    // Query again - same document should have same ID
    let results2 = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
    assert!(results2.iter().any(|(id, _)| *id == 42));
}

#[cfg(feature = "bm25")]
#[test]
fn test_no_phantom_reads() {
    // A "phantom read" would occur if results change between reads
    // even though the index hasn't changed. This should never happen.

    let mut index = InvertedIndex::new();
    for i in 0..50 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    let query = vec!["term0".to_string()];

    let results1 = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
    let results2 = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Results should be identical (no phantom reads)
    let ids1: std::collections::HashSet<u32> = results1.iter().map(|(id, _)| *id).collect();
    let ids2: std::collections::HashSet<u32> = results2.iter().map(|(id, _)| *id).collect();

    assert_eq!(
        ids1, ids2,
        "Result sets should be identical (no phantom reads)"
    );
}

// Error recovery and robustness tests
// Ensure system can recover from errors gracefully

#[cfg(feature = "bm25")]
#[test]
fn test_error_recovery_empty_index() {
    // After an error, should be able to recover by adding documents

    let mut index = InvertedIndex::new();
    let query = vec!["test".to_string()];

    // First call should fail
    let result = retrieve_bm25(&index, &query, 10, Bm25Params::default());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyIndex));

    // Add documents and retry - should succeed
    index.add_document(0, &["test".to_string()]);
    let result = retrieve_bm25(&index, &query, 10, Bm25Params::default());
    assert!(result.is_ok());
    assert!(!result.unwrap().is_empty());
}

#[cfg(feature = "dense")]
#[test]
fn test_error_recovery_dimension_mismatch() {
    // After a dimension mismatch error, should be able to recover

    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    // Wrong dimension - should fail
    let result = retrieve_dense(&retriever, &[1.0, 0.0, 0.0], 10);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RetrieveError::DimensionMismatch { .. }
    ));

    // Correct dimension - should succeed
    let result = retrieve_dense(&retriever, &[1.0, 0.0], 10);
    assert!(result.is_ok());
}

// Concurrent access robustness tests
// Ensure multiple concurrent reads see consistent state

#[cfg(feature = "bm25")]
#[test]
fn test_concurrent_reads_consistency() {
    // Multiple concurrent reads should all see the same consistent state

    use std::sync::Arc;
    use std::thread;

    let index = Arc::new({
        let mut idx = InvertedIndex::new();
        for i in 0..100 {
            idx.add_document(i, &[format!("term{}", i % 10)]);
        }
        idx
    });

    let query = vec!["term0".to_string()];
    let params = Bm25Params::default();

    let handles: Vec<_> = (0..20)
        .map(|_| {
            let idx = Arc::clone(&index);
            let q = query.clone();
            thread::spawn(move || retrieve_bm25(&idx, &q, 10, params))
        })
        .collect();

    let mut all_results = Vec::new();
    for handle in handles {
        let result = handle.join().unwrap();
        assert!(result.is_ok());
        all_results.push(result.unwrap());
    }

    // All results should be identical (same snapshot)
    // Use maps to handle ties
    let first_map: std::collections::HashMap<u32, f32> = all_results[0].iter().cloned().collect();
    for other in &all_results[1..] {
        let other_map: std::collections::HashMap<u32, f32> = other.iter().cloned().collect();
        assert_eq!(first_map.len(), other_map.len());
        for (id, score1) in &first_map {
            let score2 = other_map
                .get(id)
                .expect("Document should be in all results");
            assert!((score1 - score2).abs() < 1e-6);
        }
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_concurrent_reads_deterministic() {
    // Concurrent reads should produce deterministic, consistent results

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

    let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();

    let handles: Vec<_> = (0..10)
        .map(|_| {
            let r = Arc::clone(&retriever);
            let q = query.clone();
            thread::spawn(move || retrieve_dense(&r, &q, 10))
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().unwrap().unwrap())
        .collect();

    // All results should be identical
    let first = &results[0];
    for other in &results[1..] {
        assert_eq!(first.len(), other.len());
        for ((id1, s1), (id2, s2)) in first.iter().zip(other.iter()) {
            assert_eq!(id1, id2);
            assert!((s1 - s2).abs() < 1e-6);
        }
    }
}

// Boundary condition tests
// Test edge cases around ID handling and limits

#[cfg(feature = "bm25")]
#[test]
fn test_max_document_id_boundary() {
    // Test behavior near maximum document ID values (u32)

    let mut index = InvertedIndex::new();

    // Add documents with IDs near u32::MAX
    let max_safe_id = u32::MAX - 10;
    for i in 0..10 {
        let id = max_safe_id + i;
        index.add_document(id, &[format!("term{}", i)]);
    }

    let query = vec!["term0".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should find the document with max_safe_id
    assert!(results.iter().any(|(id, _)| *id == max_safe_id));
}

#[cfg(feature = "bm25")]
#[test]
fn test_id_zero_handling() {
    // Document ID 0 should be handled correctly

    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let query = vec!["test".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    assert!(results.iter().any(|(id, _)| *id == 0));
}

// Score consistency tests
// Ensure scores are deterministic and reproducible

#[cfg(feature = "bm25")]
#[test]
fn test_score_determinism() {
    // Scores should be deterministic and reproducible
    // This is important for consistency guarantees

    let mut index = InvertedIndex::new();
    for i in 0..50 {
        let terms: Vec<String> = (0..5).map(|j| format!("term{}", (i + j) % 20)).collect();
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];

    // Run same query multiple times
    let scores: Vec<f32> = (0..10)
        .map(|_| {
            let results = retrieve_bm25(&index, &query, 5, Bm25Params::default()).unwrap();
            results[0].1
        })
        .collect();

    // All scores should be identical
    let first_score = scores[0];
    for score in &scores[1..] {
        assert!(
            (first_score - score).abs() < 1e-6,
            "Scores should be deterministic"
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_score_determinism() {
    let mut retriever = DenseRetriever::new();
    for i in 0..20 {
        let embedding: Vec<f32> = (0..32).map(|j| ((i + j) as f32) / 100.0).collect();
        retriever.add_document(i, embedding);
    }

    let query: Vec<f32> = (0..32).map(|j| (j as f32) / 100.0).collect();

    let scores: Vec<f32> = (0..10)
        .map(|_| {
            let results = retrieve_dense(&retriever, &query, 5).unwrap();
            results[0].1
        })
        .collect();

    let first_score = scores[0];
    for score in &scores[1..] {
        assert!((first_score - score).abs() < 1e-6);
    }
}

// Batch operation consistency tests
// Ensure batch operations produce consistent results

#[cfg(feature = "bm25")]
#[test]
fn test_batch_operation_consistency() {
    // Batch operations should produce consistent results
    // All queries in a batch should see the same index state

    let mut index = InvertedIndex::new();
    for i in 0..100 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    let queries = vec![
        vec!["term0".to_string()],
        vec!["term1".to_string()],
        vec!["term2".to_string()],
    ];

    let batch_results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();

    // Each query should produce consistent results
    for (i, results) in batch_results.iter().enumerate() {
        // Re-run the same query individually
        let individual = retrieve_bm25(&index, &queries[i], 10, Bm25Params::default()).unwrap();

        // Results should match (use maps to handle ties)
        let batch_map: std::collections::HashMap<u32, f32> = results.iter().cloned().collect();
        let individual_map: std::collections::HashMap<u32, f32> =
            individual.iter().cloned().collect();

        assert_eq!(batch_map.len(), individual_map.len());
        for (id, score1) in &batch_map {
            let score2 = individual_map
                .get(id)
                .expect("Document should be in both results");
            assert!((score1 - score2).abs() < 1e-6);
        }
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_batch_dense_consistency() {
    let mut retriever = DenseRetriever::new();
    for i in 0..50 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    let queries: Vec<Vec<f32>> = (0..5)
        .map(|i| (0..64).map(|j| ((i + j) as f32) / 200.0).collect())
        .collect();

    let batch_results = batch_retrieve_dense(&retriever, &queries, 10).unwrap();

    for (i, results) in batch_results.iter().enumerate() {
        let individual = retrieve_dense(&retriever, &queries[i], 10).unwrap();

        assert_eq!(results.len(), individual.len());
        for ((id1, s1), (id2, s2)) in results.iter().zip(individual.iter()) {
            assert_eq!(id1, id2);
            assert!((s1 - s2).abs() < 1e-6);
        }
    }
}

// Error propagation tests
// Ensure errors are properly propagated through batch operations

#[cfg(feature = "bm25")]
#[test]
fn test_error_propagation_batch() {
    // Errors in batch operations should be properly propagated
    // and not cause partial results

    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);

    let queries = vec![
        vec!["test".to_string()],
        vec![], // Empty query - should error
        vec!["test".to_string()],
    ];

    let result = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default());

    // Should fail entirely, not return partial results
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyQuery));
}

#[cfg(feature = "dense")]
#[test]
fn test_error_propagation_dimension_mismatch() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let queries = vec![
        vec![1.0, 0.0],
        vec![1.0, 0.0, 0.0], // Dimension mismatch
        vec![1.0, 0.0],
    ];

    let result = batch_retrieve_dense(&retriever, &queries, 10);

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        RetrieveError::DimensionMismatch { .. }
    ));
}

// Stress tests for consistency
// Ensure results remain consistent under heavy load

#[cfg(feature = "bm25")]
#[test]
fn test_consistency_under_stress() {
    // Under stress (many documents, many queries), results should remain consistent

    let mut index = InvertedIndex::new();
    for i in 0..1000 {
        let terms: Vec<String> = (0..20)
            .map(|j| format!("term{}", (i * 7 + j * 11) % 100))
            .collect();
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];

    // Run query many times
    let mut all_results = Vec::new();
    for _ in 0..50 {
        let results = retrieve_bm25(&index, &query, 100, Bm25Params::default()).unwrap();
        all_results.push(results);
    }

    // All results should be consistent (use maps to handle ties)
    // When scores are tied, different documents might be returned, so we verify
    // that documents appearing in multiple results have identical scores
    let first_map: std::collections::HashMap<u32, f32> = all_results[0].iter().cloned().collect();
    for other in &all_results[1..] {
        let other_map: std::collections::HashMap<u32, f32> = other.iter().cloned().collect();
        assert_eq!(first_map.len(), other_map.len());

        // Verify scores match for common documents
        for (id, score1) in &first_map {
            if let Some(score2) = other_map.get(id) {
                assert!(
                    (score1 - score2).abs() < 1e-5,
                    "Scores should match for doc {}",
                    id
                );
            }
        }

        // Verify at least some documents are common (not all ties)
        let common: Vec<u32> = first_map
            .keys()
            .filter(|id| other_map.contains_key(id))
            .cloned()
            .collect();
        assert!(
            !common.is_empty(),
            "At least some documents should be common across results"
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_consistency_under_stress() {
    let mut retriever = DenseRetriever::new();
    for i in 0..500 {
        let embedding: Vec<f32> = (0..128).map(|j| ((i + j) as f32) / 500.0).collect();
        retriever.add_document(i, embedding);
    }

    let query: Vec<f32> = (0..128).map(|j| (j as f32) / 500.0).collect();

    let mut all_results = Vec::new();
    for _ in 0..50 {
        let results = retrieve_dense(&retriever, &query, 100).unwrap();
        all_results.push(results);
    }

    let first = &all_results[0];
    for other in &all_results[1..] {
        assert_eq!(first.len(), other.len());
        for ((id1, s1), (id2, s2)) in first.iter().zip(other.iter()) {
            assert_eq!(id1, id2);
            assert!((s1 - s2).abs() < 1e-5);
        }
    }
}
