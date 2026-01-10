//! Comprehensive tests for BM25 eager scoring implementation.
//!
//! Tests cover:
//! - Basic functionality
//! - Edge cases (empty, NaN, zero scores)
//! - Conversion from lazy BM25 index
//! - Performance characteristics
//! - Correctness vs lazy scoring

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::eager::EagerBm25Index;
#[cfg(feature = "bm25")]
use rank_retrieve::RetrieveError;

// ─────────────────────────────────────────────────────────────────────────────
// Basic Functionality Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_basic_retrieval() {
    let mut index = EagerBm25Index::new();
    
    // Add documents with precomputed scores
    let mut doc0_scores = std::collections::HashMap::new();
    doc0_scores.insert("hello".to_string(), 2.5);
    doc0_scores.insert("world".to_string(), 1.2);
    index.add_document_with_scores(0, doc0_scores);
    
    let mut doc1_scores = std::collections::HashMap::new();
    doc1_scores.insert("hello".to_string(), 1.8);
    doc1_scores.insert("rust".to_string(), 2.0);
    index.add_document_with_scores(1, doc1_scores);
    
    // Query for "hello"
    let query = vec!["hello".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    assert!(!results.is_empty());
    assert!(results[0].0 == 0 || results[0].0 == 1); // Either doc should match
    assert!(results[0].1 > 0.0); // Score should be positive
    // Document 0 should score higher (2.5 > 1.8)
    assert_eq!(results[0].0, 0);
    assert!((results[0].1 - 2.5).abs() < 1e-6);
}

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_empty_query() {
    let mut index = EagerBm25Index::new();
    
    let mut doc_scores = std::collections::HashMap::new();
    doc_scores.insert("test".to_string(), 1.0);
    index.add_document_with_scores(0, doc_scores);
    
    let query: Vec<String> = vec![];
    let result = index.retrieve(&query, 10);
    
    // Empty query should return error
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyQuery));
}

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_no_matching_terms() {
    let mut index = EagerBm25Index::new();
    
    let mut doc_scores = std::collections::HashMap::new();
    doc_scores.insert("test".to_string(), 1.0);
    index.add_document_with_scores(0, doc_scores);
    
    // Query with term not in vocabulary
    let query = vec!["nonexistent".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    assert!(results.is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_top_k_limiting() {
    let mut index = EagerBm25Index::new();
    
    // Add 20 documents
    for i in 0..20 {
        let mut doc_scores = std::collections::HashMap::new();
        doc_scores.insert("test".to_string(), 1.0 + (i as f32) * 0.1); // Varying scores
        index.add_document_with_scores(i, doc_scores);
    }
    
    let query = vec!["test".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    assert_eq!(results.len(), 10);
    // Results should be sorted descending
    for i in 1..results.len() {
        assert!(results[i-1].1 >= results[i].1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge Cases: NaN, Infinity, Zero Scores
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_filters_zero_scores() {
    let mut index = EagerBm25Index::new();
    
    // Create a document with a term that won't match the query
    let mut doc_scores = std::collections::HashMap::new();
    doc_scores.insert("other".to_string(), 1.0);
    index.add_document_with_scores(0, doc_scores);
    
    // Query for term not in document
    let query = vec!["query".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    // Should return empty (no matching terms, zero dot product)
    assert!(results.is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_handles_large_k() {
    let mut index = EagerBm25Index::new();
    
    // Add 5 documents
    for i in 0..5 {
        let mut doc_scores = std::collections::HashMap::new();
        doc_scores.insert("test".to_string(), 1.0 + (i as f32) * 0.1);
        index.add_document_with_scores(i, doc_scores);
    }
    
    let query = vec!["test".to_string()];
    
    // k larger than num_docs should use full sort path
    let results = index.retrieve(&query, 10).unwrap();
    
    assert_eq!(results.len(), 5); // All documents match
    assert!(results.iter().all(|(_, score)| score.is_finite() && *score > 0.0));
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversion from Lazy BM25 Index
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_from_lazy_bm25_index() {
    // Create a lazy BM25 index
    let mut lazy_index = InvertedIndex::new();
    lazy_index.add_document(0, &["hello".to_string(), "world".to_string()]);
    lazy_index.add_document(1, &["hello".to_string(), "rust".to_string(), "rust".to_string()]);
    
    let params = Bm25Params::default();
    
    // Convert to eager
    let eager_index = EagerBm25Index::from_bm25_index(&lazy_index, params);
    
    assert_eq!(eager_index.num_docs(), 2);
    assert!(eager_index.vocabulary_size() > 0);
    
    // Query should work
    let query = vec!["hello".to_string()];
    let results = eager_index.retrieve(&query, 10).unwrap();
    
    assert!(!results.is_empty());
    assert_eq!(results.len(), 2); // Both documents match
}

#[cfg(feature = "bm25")]
#[test]
fn test_eager_vs_lazy_same_results() {
    // Create documents
    let docs = vec![
        vec!["hello".to_string(), "world".to_string(), "hello".to_string()],
        vec!["hello".to_string(), "rust".to_string()],
        vec!["world".to_string(), "world".to_string()],
    ];
    
    // Build lazy index
    let mut lazy_index = InvertedIndex::new();
    for (i, doc) in docs.iter().enumerate() {
        lazy_index.add_document(i as u32, doc);
    }
    
    // Build eager index
    let params = Bm25Params::default();
    let eager_index = EagerBm25Index::from_bm25_index(&lazy_index, params);
    
    // Query both
    let query = vec!["hello".to_string()];
    let lazy_results = lazy_index.retrieve(&query, 10, params).unwrap();
    let eager_results = eager_index.retrieve(&query, 10).unwrap();
    
    // Results should match (allowing for floating point precision)
    assert_eq!(lazy_results.len(), eager_results.len());
    
    // Check that same documents are returned (scores may differ slightly due to precision)
    let lazy_doc_ids: std::collections::HashSet<u32> = lazy_results.iter().map(|(id, _)| *id).collect();
    let eager_doc_ids: std::collections::HashSet<u32> = eager_results.iter().map(|(id, _)| *id).collect();
    
    assert_eq!(lazy_doc_ids, eager_doc_ids);
    
    // Scores should be approximately equal (within floating point precision)
    for (lazy_result, eager_result) in lazy_results.iter().zip(eager_results.iter()) {
        if lazy_result.0 == eager_result.0 {
            let diff = (lazy_result.1 - eager_result.1).abs();
            assert!(
                diff < 1e-5,
                "Score mismatch for doc {}: lazy={}, eager={}, diff={}",
                lazy_result.0,
                lazy_result.1,
                eager_result.1,
                diff
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Vocabulary Management Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_vocabulary_management() {
    let mut index = EagerBm25Index::new();
    
    let mut doc0_scores = std::collections::HashMap::new();
    doc0_scores.insert("term1".to_string(), 1.0);
    doc0_scores.insert("term2".to_string(), 2.0);
    index.add_document_with_scores(0, doc0_scores);
    
    let mut doc1_scores = std::collections::HashMap::new();
    doc1_scores.insert("term2".to_string(), 1.0);
    doc1_scores.insert("term3".to_string(), 1.5);
    index.add_document_with_scores(1, doc1_scores);
    
    // Vocabulary should contain all unique terms
    assert_eq!(index.vocabulary_size(), 3);
    
    // Query with all terms should work
    let query = vec!["term1".to_string(), "term2".to_string(), "term3".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    assert!(!results.is_empty());
    assert_eq!(results.len(), 2); // Both documents match
}

// ─────────────────────────────────────────────────────────────────────────────
// Performance Characteristics Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_heap_vs_sort_paths() {
    let mut index = EagerBm25Index::new();
    
    // Add 100 documents
    for i in 0..100 {
        let mut doc_scores = std::collections::HashMap::new();
        doc_scores.insert("test".to_string(), 1.0 + (i as f32) * 0.01);
        index.add_document_with_scores(i, doc_scores);
    }
    
    let query = vec!["test".to_string()];
    
    // Small k should use heap path (k < num_docs / 2 = 50)
    let small_k_results = index.retrieve(&query, 10).unwrap();
    assert_eq!(small_k_results.len(), 10);
    
    // Large k should use sort path (k >= num_docs / 2)
    let large_k_results = index.retrieve(&query, 60).unwrap();
    assert_eq!(large_k_results.len(), 60);
    
    // Both should return valid results
    assert!(small_k_results.iter().all(|(_, score)| score.is_finite() && *score > 0.0));
    assert!(large_k_results.iter().all(|(_, score)| score.is_finite() && *score > 0.0));
    
    // Both should be sorted descending
    for i in 1..small_k_results.len() {
        assert!(small_k_results[i-1].1 >= small_k_results[i].1);
    }
    for i in 1..large_k_results.len() {
        assert!(large_k_results[i-1].1 >= large_k_results[i].1);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sort Stability Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_sort_stability() {
    let mut index = EagerBm25Index::new();
    
    // Add documents with same scores (to test sort_unstable behavior)
    for i in 0..5 {
        let mut doc_scores = std::collections::HashMap::new();
        doc_scores.insert("test".to_string(), 1.0); // Same score for all
        index.add_document_with_scores(i, doc_scores);
    }
    
    let query = vec!["test".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    // Results should be sorted descending by score
    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results not sorted: {} >= {}",
            results[i - 1].1,
            results[i].1
        );
    }
    
    // All should have same score (1.0)
    assert!(results.iter().all(|(_, score)| (*score - 1.0).abs() < 1e-6));
}

// ─────────────────────────────────────────────────────────────────────────────
// NaN and Infinity Handling Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_filters_nan_scores() {
    let mut index = EagerBm25Index::new();
    
    // Add document with normal scores
    let mut doc0_scores = std::collections::HashMap::new();
    doc0_scores.insert("test".to_string(), 1.0);
    index.add_document_with_scores(0, doc0_scores);
    
    // Add document with NaN score (simulated by creating a vector that could produce NaN)
    // Note: In practice, BM25 scores shouldn't produce NaN, but we test the filter anyway
    let mut doc1_scores = std::collections::HashMap::new();
    doc1_scores.insert("other".to_string(), f32::NAN);
    index.add_document_with_scores(1, doc1_scores);
    
    let query = vec!["test".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    // Should only return document 0 (document 1 has NaN and doesn't match query anyway)
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1.is_finite());
}

#[cfg(feature = "bm25")]
#[test]
fn test_eager_bm25_filters_infinity_scores() {
    let mut index = EagerBm25Index::new();
    
    let mut doc0_scores = std::collections::HashMap::new();
    doc0_scores.insert("test".to_string(), 1.0);
    index.add_document_with_scores(0, doc0_scores);
    
    let mut doc1_scores = std::collections::HashMap::new();
    doc1_scores.insert("test".to_string(), f32::INFINITY);
    index.add_document_with_scores(1, doc1_scores);
    
    let query = vec!["test".to_string()];
    let results = index.retrieve(&query, 10).unwrap();
    
    // Should filter out infinity
    assert!(results.iter().all(|(_, score)| score.is_finite()));
}

// ─────────────────────────────────────────────────────────────────────────────
// Conversion Correctness Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_eager_from_lazy_empty_index() {
    let lazy_index = InvertedIndex::new();
    let params = Bm25Params::default();
    
    let eager_index = EagerBm25Index::from_bm25_index(&lazy_index, params);
    
    assert_eq!(eager_index.num_docs(), 0);
    assert_eq!(eager_index.vocabulary_size(), 0);
    
    // Query should return EmptyIndex error
    let query = vec!["test".to_string()];
    let result = eager_index.retrieve(&query, 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyIndex));
}

#[cfg(feature = "bm25")]
#[test]
fn test_eager_from_lazy_single_document() {
    let mut lazy_index = InvertedIndex::new();
    lazy_index.add_document(0, &["hello".to_string(), "world".to_string()]);
    
    let params = Bm25Params::default();
    let eager_index = EagerBm25Index::from_bm25_index(&lazy_index, params);
    
    assert_eq!(eager_index.num_docs(), 1);
    assert!(eager_index.vocabulary_size() >= 2);
    
    let query = vec!["hello".to_string()];
    let results = eager_index.retrieve(&query, 10).unwrap();
    
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > 0.0);
}
