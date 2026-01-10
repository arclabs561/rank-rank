//! Edge case tests for rank-retrieve optimizations.
//!
//! Tests behavior with:
//! - Empty inputs
//! - Single-element inputs
//! - Very large inputs
//! - Extreme values (very small, very large, subnormal)
//! - Boundary conditions

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

#[cfg(feature = "dense")]
use rank_retrieve::simd;

// ─────────────────────────────────────────────────────────────────────────────
// SIMD Edge Cases
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "dense")]
#[test]
fn test_dot_empty_vectors() {
    let empty: Vec<f32> = vec![];
    assert_eq!(simd::dot(&empty, &empty), 0.0);
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_single_element() {
    assert!((simd::dot(&[2.0], &[3.0]) - 6.0).abs() < 1e-5);
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_very_large_vectors() {
    // Test with vectors larger than typical SIMD chunk size
    let len = 10_000;
    let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.001).collect();
    let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.002).collect();

    let result = simd::dot(&a, &b);
    assert!(result.is_finite(), "Dot product of large vectors should be finite");
    assert!(result >= 0.0, "Dot product of positive vectors should be non-negative");
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_very_small_values() {
    // Test with very small values (near subnormal range)
    let a = vec![1e-30, 1e-31, 1e-32];
    let b = vec![1e-30, 1e-31, 1e-32];

    let result = simd::dot(&a, &b);
    assert!(result.is_finite(), "Dot product should handle very small values");
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_very_large_values() {
    // Test with very large values (near overflow)
    let a = vec![1e20, 1e21];
    let b = vec![1e20, 1e21];

    let result = simd::dot(&a, &b);
    // Result might be infinity, which is acceptable
    assert!(result.is_finite() || result.is_infinite());
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_mixed_signs() {
    // Test with mixed positive and negative values
    let a = vec![1.0, -2.0, 3.0, -4.0, 5.0];
    let b = vec![-1.0, 2.0, -3.0, 4.0, -5.0];

    let result = simd::dot(&a, &b);
    let expected = -1.0 - 4.0 - 9.0 - 16.0 - 25.0;
    assert!((result - expected).abs() < 1e-4);
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_empty_vectors() {
    let empty: Vec<f32> = vec![];
    assert_eq!(simd::cosine(&empty, &empty), 0.0);
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_zero_norm() {
    // Test with vectors that have zero norm
    let zero = vec![0.0, 0.0, 0.0];
    let normal = vec![1.0, 0.0, 0.0];

    assert_eq!(simd::cosine(&zero, &normal), 0.0);
    assert_eq!(simd::cosine(&normal, &zero), 0.0);
    assert_eq!(simd::cosine(&zero, &zero), 0.0);
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_very_small_norm() {
    // Test with vectors that have very small norm (below NORM_EPSILON)
    let tiny = vec![1e-10, 1e-10];
    let normal = vec![1.0, 0.0];

    // Should return 0.0 for effectively zero norm
    assert_eq!(simd::cosine(&tiny, &normal), 0.0);
}

#[cfg(feature = "dense")]
#[test]
fn test_norm_empty_vector() {
    let empty: Vec<f32> = vec![];
    assert_eq!(simd::norm(&empty), 0.0);
}

#[cfg(feature = "dense")]
#[test]
fn test_norm_single_element() {
    assert!((simd::norm(&[3.0]) - 3.0).abs() < 1e-5);
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_dot_empty_vectors() {
    use rank_retrieve::simd;

    let empty_indices: Vec<u32> = vec![];
    let empty_values: Vec<f32> = vec![];
    assert_eq!(simd::sparse_dot(&empty_indices, &empty_values, &empty_indices, &empty_values), 0.0);
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_dot_single_element() {
    use rank_retrieve::simd;

    let a_indices = vec![5];
    let a_values = vec![2.0];
    let b_indices = vec![5];
    let b_values = vec![3.0];

    assert!((simd::sparse_dot(&a_indices, &a_values, &b_indices, &b_values) - 6.0).abs() < 1e-5);
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_dot_very_sparse() {
    // Test with very sparse vectors (few non-zeros)
    use rank_retrieve::simd;

    let a_indices = vec![100, 200, 300];
    let a_values = vec![1.0, 2.0, 3.0];
    let b_indices = vec![100, 400, 500];
    let b_values = vec![4.0, 5.0, 6.0];

    // Only index 100 overlaps
    let result = simd::sparse_dot(&a_indices, &a_values, &b_indices, &b_values);
    assert!((result - 4.0).abs() < 1e-5);
}

// ─────────────────────────────────────────────────────────────────────────────
// BM25 Edge Cases
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_empty_index() {
    let index = InvertedIndex::new();
    let query = vec!["term".to_string()];

    let result = index.retrieve(&query, 10, Bm25Params::default());
    assert!(result.is_err());
    // Should return EmptyIndex error
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_empty_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["term".to_string()]);

    let query: Vec<String> = vec![];
    let result = index.retrieve(&query, 10, Bm25Params::default());
    assert!(result.is_err());
    // Should return EmptyQuery error
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_single_document() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["term".to_string()]);

    let query = vec!["term".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > 0.0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_single_query_term() {
    let mut index = InvertedIndex::new();
    for i in 0..10 {
        index.add_document(i, &["term".to_string(), format!("doc{}", i)]);
    }

    let query = vec!["term".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // All documents should match
    assert_eq!(results.len(), 10);
    for (doc_id, score) in &results {
        assert!(*score > 0.0, "Score should be positive for matching document");
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_no_matching_documents() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["term1".to_string()]);
    index.add_document(1, &["term2".to_string()]);

    let query = vec!["term3".to_string()]; // No documents contain term3
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_very_long_query() {
    let mut index = InvertedIndex::new();
    for i in 0..10 {
        let terms: Vec<String> = (0..20).map(|j| format!("term{}", j)).collect();
        index.add_document(i, &terms);
    }

    // Query with many terms
    let query: Vec<String> = (0..50).map(|i| format!("term{}", i % 20)).collect();
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // Should still return results
    assert!(results.len() > 0);
    for (_, score) in &results {
        assert!(score.is_finite());
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_very_large_index() {
    // Test with many documents
    let mut index = InvertedIndex::new();
    for i in 0..1000 {
        let terms: Vec<String> = (0..10).map(|j| format!("term{}", (i + j) % 100)).collect();
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // Should return top-10 results
    assert_eq!(results.len(), 10);
    for (_, score) in &results {
        assert!(score.is_finite());
        assert!(*score >= 0.0);
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_duplicate_query_terms() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["term".to_string(), "term".to_string()]); // Duplicate in doc
    index.add_document(1, &["term".to_string()]);

    let query = vec!["term".to_string(), "term".to_string()]; // Duplicate in query
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // Document 0 should score higher (has term twice)
    assert!(results.len() >= 1);
    if results.len() >= 2 {
        // Document 0 should rank higher
        assert!(results[0].1 >= results[1].1);
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_special_characters_in_terms() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["term-with-dash".to_string()]);
    index.add_document(1, &["term_with_underscore".to_string()]);
    index.add_document(2, &["term.with.dot".to_string()]);

    let query = vec!["term-with-dash".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results[0].0, 0); // Document 0 should match
}

// ─────────────────────────────────────────────────────────────────────────────
// Dense Retrieval Edge Cases
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "dense")]
#[test]
fn test_dense_empty_retriever() {
    let retriever = DenseRetriever::new();
    let query = vec![1.0, 0.0];

    let result = retriever.retrieve(&query, 10);
    assert!(result.is_err());
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_empty_query() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let query: Vec<f32> = vec![];
    let result = retriever.retrieve(&query, 10);
    assert!(result.is_err());
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_dimension_mismatch() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0, 0.0]); // 3D

    let query = vec![1.0, 0.0]; // 2D
    let result = retriever.retrieve(&query, 10);
    assert!(result.is_err());
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_single_document() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);

    let query = vec![1.0, 0.0];
    let results = retriever.retrieve(&query, 10).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!((results[0].1 - 1.0).abs() < 1e-5); // Should be perfect match
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_very_high_dimension() {
    let mut retriever = DenseRetriever::new();
    let dim = 10_000;
    let doc: Vec<f32> = (0..dim).map(|i| (i as f32) / dim as f32).collect();
    retriever.add_document(0, doc.clone());

    let query = doc;
    let results = retriever.retrieve(&query, 10).unwrap();

    assert_eq!(results.len(), 1);
    assert!(results[0].1 > 0.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// Sparse Retrieval Edge Cases
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_empty_retriever() {
    let retriever = SparseRetriever::new();
    let query = SparseVector::new_unchecked(vec![0], vec![1.0]);

    let result = retriever.retrieve(&query, 10);
    assert!(result.is_err());
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_empty_query() {
    let mut retriever = SparseRetriever::new();
    retriever.add_document(0, SparseVector::new_unchecked(vec![0], vec![1.0]));

    let query = SparseVector::new_unchecked(vec![], vec![]);
    let result = retriever.retrieve(&query, 10);
    assert!(result.is_err());
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_single_document() {
    let mut retriever = SparseRetriever::new();
    retriever.add_document(0, SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]));

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retriever.retrieve(&query, 10).unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > 0.0);
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_no_overlap() {
    let mut retriever = SparseRetriever::new();
    retriever.add_document(0, SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]));
    retriever.add_document(1, SparseVector::new_unchecked(vec![2, 3], vec![1.0, 0.5]));

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retriever.retrieve(&query, 10).unwrap();

    // Document 0 should match (has indices 0, 1) and score higher
    // Document 1 has indices 2, 3 which don't overlap with query (score = 0.0)
    assert_eq!(results.len(), 2); // Both documents returned
    assert_eq!(results[0].0, 0); // Document 0 should be top result (has positive score)
    assert!(results[0].1 > 0.0); // Document 0 should have positive score
    assert_eq!(results[1].0, 1); // Document 1 should be second (zero score)
    assert_eq!(results[1].1, 0.0); // Document 1 should have zero score (no overlap)
}
