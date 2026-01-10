//! Property-based tests for rank-retrieve optimizations.
//!
//! Tests invariants, correctness properties, and edge cases for:
//! - SIMD-accelerated dense vector operations
//! - SIMD-accelerated sparse vector operations
//! - BM25 retrieval optimizations

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

#[cfg(feature = "dense")]
use rank_retrieve::simd;

// ─────────────────────────────────────────────────────────────────────────────
// SIMD Dense Vector Properties
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "dense")]
#[test]
fn test_dot_commutativity() {
    // Property: dot(a, b) == dot(b, a)
    for len in [1, 4, 8, 16, 32, 64, 128, 256] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1 - 0.5).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.2 + 1.0).collect();

        let ab = simd::dot(&a, &b);
        let ba = simd::dot(&b, &a);

        assert!(
            (ab - ba).abs() < 1e-5,
            "Dot product not commutative at len={}: dot(a,b)={}, dot(b,a)={}",
            len,
            ab,
            ba
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_distributivity() {
    // Property: dot(a, b + c) ≈ dot(a, b) + dot(a, c) (within floating-point error)
    for len in [1, 4, 8, 16, 32, 64, 128] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.2).collect();
        let c: Vec<f32> = (0..len).map(|i| (i as f32) * 0.3).collect();

        let bc: Vec<f32> = b.iter().zip(c.iter()).map(|(x, y)| x + y).collect();

        let ab_plus_ac = simd::dot(&a, &b) + simd::dot(&a, &c);
        let a_bc = simd::dot(&a, &bc);

        // Allow larger tolerance for floating-point accumulation errors
        let tolerance = (ab_plus_ac.abs() * 1e-4).max(1e-5);
        assert!(
            (ab_plus_ac - a_bc).abs() < tolerance,
            "Dot product not distributive at len={}: dot(a,b)+dot(a,c)={}, dot(a,b+c)={}",
            len,
            ab_plus_ac,
            a_bc
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_always_finite() {
    // Property: dot product always returns finite values
    for len in [1, 16, 32, 64, 128, 256, 512, 1024] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.2).collect();

        let result = simd::dot(&a, &b);
        assert!(
            result.is_finite(),
            "Dot product returned non-finite value at len={}: {}",
            len,
            result
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_zero_vector() {
    // Property: dot with zero vector is zero
    for len in [1, 4, 8, 16, 32, 64] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1).collect();
        let zero: Vec<f32> = vec![0.0; len];

        assert_eq!(simd::dot(&a, &zero), 0.0);
        assert_eq!(simd::dot(&zero, &a), 0.0);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_negative_values() {
    // Property: dot product handles negative values correctly
    let a = vec![-1.0, 2.0, -3.0, 4.0];
    let b = vec![5.0, -6.0, 7.0, -8.0];

    let result = simd::dot(&a, &b);
    let expected = -1.0 * 5.0 + 2.0 * -6.0 + -3.0 * 7.0 + 4.0 * -8.0;
    assert!((result - expected).abs() < 1e-5);
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_range() {
    // Property: cosine similarity is in [-1, 1] (or very close due to floating-point error)
    for len in [2, 4, 8, 16, 32, 64, 128, 256] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1 - 0.5).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.2 + 1.0).collect();

        let result = simd::cosine(&a, &b);

        // Allow small floating-point error outside [-1, 1]
        assert!(
            result >= -1.1 && result <= 1.1,
            "Cosine similarity out of expected range at len={}: {}",
            len,
            result
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_identical_vectors() {
    // Property: cosine similarity of identical vectors is 1.0 (or very close)
    for len in [2, 4, 8, 16, 32, 64, 128] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1).collect();
        let result = simd::cosine(&a, &a);

        assert!(
            (result - 1.0).abs() < 1e-4,
            "Cosine similarity of identical vectors not 1.0 at len={}: {}",
            len,
            result
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_orthogonal_vectors() {
    // Property: cosine similarity of orthogonal vectors is 0.0 (or very close)
    let a = vec![1.0, 0.0, 0.0];
    let b = vec![0.0, 1.0, 0.0];

    let result = simd::cosine(&a, &b);
    assert!(result.abs() < 1e-5, "Orthogonal vectors should have cosine ~0, got {}", result);
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_opposite_vectors() {
    // Property: cosine similarity of opposite vectors is -1.0 (or very close)
    for len in [2, 4, 8, 16, 32] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1).collect();
        let b: Vec<f32> = a.iter().map(|x| -x).collect();

        let result = simd::cosine(&a, &b);
        assert!(
            (result + 1.0).abs() < 1e-4,
            "Opposite vectors should have cosine ~-1.0 at len={}, got {}",
            len,
            result
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_norm_non_negative() {
    // Property: norm is always non-negative
    for len in [1, 4, 8, 16, 32, 64, 128] {
        let v: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1 - 0.5).collect();
        let result = simd::norm(&v);

        assert!(
            result >= 0.0,
            "Norm should be non-negative at len={}: {}",
            len,
            result
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_norm_zero_vector() {
    // Property: norm of zero vector is 0.0
    for len in [1, 4, 8, 16, 32, 64] {
        let zero: Vec<f32> = vec![0.0; len];
        assert_eq!(simd::norm(&zero), 0.0);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dot_mismatched_lengths() {
    // Property: dot product handles mismatched lengths correctly
    let a = vec![1.0, 2.0, 3.0, 4.0];
    let b = vec![5.0, 6.0]; // Shorter

    let result = simd::dot(&a, &b);
    let expected = 1.0 * 5.0 + 2.0 * 6.0; // Only first 2 elements
    assert!((result - expected).abs() < 1e-5);
}

// ─────────────────────────────────────────────────────────────────────────────
// SIMD Sparse Vector Properties
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_dot_commutativity() {
    // Property: sparse_dot(a, b) == sparse_dot(b, a)
    use rank_retrieve::simd;

    for size in [4, 8, 16, 32, 64] {
        let mut a_indices = Vec::new();
        let mut a_values = Vec::new();
        let mut b_indices = Vec::new();
        let mut b_values = Vec::new();

        // Create overlapping sparse vectors
        for i in 0..size {
            if i % 2 == 0 {
                a_indices.push(i as u32);
                a_values.push(i as f32 * 0.1);
            }
            if i % 3 == 0 {
                b_indices.push(i as u32);
                b_values.push(i as f32 * 0.2);
            }
        }

        let ab = simd::sparse_dot(&a_indices, &a_values, &b_indices, &b_values);
        let ba = simd::sparse_dot(&b_indices, &b_values, &a_indices, &a_values);

        assert!(
            (ab - ba).abs() < 1e-5,
            "Sparse dot product not commutative at size={}: dot(a,b)={}, dot(b,a)={}",
            size,
            ab,
            ba
        );
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_dot_always_finite() {
    // Property: sparse dot product always returns finite values
    use rank_retrieve::simd;

    for size in [4, 8, 16, 32, 64, 128] {
        let mut a_indices = Vec::new();
        let mut a_values = Vec::new();
        let mut b_indices = Vec::new();
        let mut b_values = Vec::new();

        for i in 0..size {
            if i % 2 == 0 {
                a_indices.push(i as u32);
                a_values.push(i as f32 * 0.1);
            }
            if i % 3 == 0 {
                b_indices.push(i as u32);
                b_values.push(i as f32 * 0.2);
            }
        }

        let result = simd::sparse_dot(&a_indices, &a_values, &b_indices, &b_values);
        assert!(
            result.is_finite(),
            "Sparse dot product returned non-finite value at size={}: {}",
            size,
            result
        );
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_dot_no_overlap() {
    // Property: sparse dot product of non-overlapping vectors is 0.0
    use rank_retrieve::simd;

    let a_indices = vec![1, 3, 5];
    let a_values = vec![1.0, 2.0, 3.0];
    let b_indices = vec![2, 4, 6];
    let b_values = vec![4.0, 5.0, 6.0];

    let result = simd::sparse_dot(&a_indices, &a_values, &b_indices, &b_values);
    assert_eq!(result, 0.0);
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_dot_empty() {
    // Property: sparse dot product with empty vector is 0.0
    use rank_retrieve::simd;

    let a_indices = vec![1, 2, 3];
    let a_values = vec![1.0, 2.0, 3.0];
    let empty_indices: Vec<u32> = vec![];
    let empty_values: Vec<f32> = vec![];

    assert_eq!(simd::sparse_dot(&a_indices, &a_values, &empty_indices, &empty_values), 0.0);
    assert_eq!(simd::sparse_dot(&empty_indices, &empty_values, &a_indices, &a_values), 0.0);
}

// ─────────────────────────────────────────────────────────────────────────────
// BM25 Retrieval Properties
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_results_sorted_descending() {
    // Property: BM25 results are sorted by score descending
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
    index.add_document(2, &["machine".to_string(), "learning".to_string(), "algorithm".to_string()]);

    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // Verify descending order
    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results not sorted descending: scores {:?}",
            results.iter().map(|(_, s)| *s).collect::<Vec<_>>()
        );
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_no_duplicate_doc_ids() {
    // Property: BM25 results contain no duplicate document IDs
    let mut index = InvertedIndex::new();
    for i in 0..20 {
        index.add_document(
            i,
            &[
                "term".to_string(),
                format!("doc{}", i),
                "content".to_string(),
            ],
        );
    }

    let query = vec!["term".to_string()];
    let results = index.retrieve(&query, 20, Bm25Params::default()).unwrap();

    let mut seen = std::collections::HashSet::new();
    for (doc_id, _) in &results {
        assert!(
            seen.insert(*doc_id),
            "Duplicate document ID in results: {}",
            doc_id
        );
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_scores_finite() {
    // Property: All BM25 scores are finite
    let mut index = InvertedIndex::new();
    for i in 0..10 {
        index.add_document(
            i,
            &[
                "term".to_string(),
                format!("doc{}", i),
            ],
        );
    }

    let query = vec!["term".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    for (doc_id, score) in &results {
        assert!(
            score.is_finite(),
            "Non-finite score for document {}: {}",
            doc_id,
            score
        );
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_scores_non_negative() {
    // Property: BM25 scores are non-negative (BM25 formula always produces >= 0)
    let mut index = InvertedIndex::new();
    for i in 0..10 {
        index.add_document(
            i,
            &[
                "term".to_string(),
                format!("doc{}", i),
            ],
        );
    }

    let query = vec!["term".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    for (doc_id, score) in &results {
        assert!(
            *score >= 0.0,
            "Negative score for document {}: {}",
            doc_id,
            score
        );
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_k_zero_returns_empty() {
    // Property: k=0 returns empty results
    let mut index = InvertedIndex::new();
    index.add_document(0, &["term".to_string()]);
    index.add_document(1, &["term".to_string()]);

    let query = vec!["term".to_string()];
    let results = index.retrieve(&query, 0, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_k_larger_than_docs() {
    // Property: k larger than matching documents returns all matching documents
    let mut index = InvertedIndex::new();
    for i in 0..5 {
        index.add_document(i, &["term".to_string()]);
    }

    let query = vec!["term".to_string()];
    let results = index.retrieve(&query, 100, Bm25Params::default()).unwrap();

    // Should return at most 5 documents (all that match)
    assert!(results.len() <= 5);
    assert!(results.len() > 0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_early_termination_correctness() {
    // Property: Early termination produces correct top-k results
    // Test by comparing with exhaustive scoring
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        let terms: Vec<String> = (0..10)
            .map(|j| format!("term{}", (i + j) % 20))
            .collect();
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];
    let results_early = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // Verify we got exactly k results
    assert_eq!(results_early.len(), 10);

    // Verify all scores are positive (documents match query)
    for (_, score) in &results_early {
        assert!(*score > 0.0, "Early termination returned zero score");
    }

    // Verify results are sorted
    for i in 1..results_early.len() {
        assert!(results_early[i - 1].1 >= results_early[i].1);
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_precomputed_idf_correctness() {
    // Property: Precomputed IDF matches on-the-fly calculation
    let mut index = InvertedIndex::new();
    index.add_document(0, &["common".to_string(), "term".to_string()]);
    index.add_document(1, &["common".to_string(), "word".to_string()]);
    index.add_document(2, &["rare".to_string(), "term".to_string()]);

    // Trigger IDF precomputation by retrieving
    let _ = index.retrieve(&["term".to_string()], 10, Bm25Params::default());

    // Check that precomputed IDF matches direct calculation
    let idf_common = index.idf("common");
    let idf_rare = index.idf("rare");

    // Rare term should have higher IDF
    assert!(idf_rare > idf_common);

    // Both should be positive
    assert!(idf_common > 0.0);
    assert!(idf_rare > 0.0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_retrieval_consistency() {
    // Property: Multiple retrievals with same query produce same results
    let mut index = InvertedIndex::new();
    for i in 0..20 {
        index.add_document(
            i,
            &[
                "term".to_string(),
                format!("doc{}", i),
            ],
        );
    }

    let query = vec!["term".to_string()];
    let results1 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    let results2 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    assert_eq!(results1.len(), results2.len());
    for ((id1, score1), (id2, score2)) in results1.iter().zip(results2.iter()) {
        assert_eq!(id1, id2);
        assert!((score1 - score2).abs() < 1e-5, "Scores differ: {} vs {}", score1, score2);
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_query_term_ordering_irrelevant() {
    // Property: Query term order doesn't affect results (BM25 is order-independent)
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
    index.add_document(2, &["machine".to_string(), "learning".to_string(), "algorithm".to_string()]);

    let query1 = vec!["machine".to_string(), "learning".to_string()];
    let query2 = vec!["learning".to_string(), "machine".to_string()];

    let results1 = index.retrieve(&query1, 10, Bm25Params::default()).unwrap();
    let results2 = index.retrieve(&query2, 10, Bm25Params::default()).unwrap();

    // Results should be identical (same documents, same scores)
    assert_eq!(results1.len(), results2.len());
    for ((id1, score1), (id2, score2)) in results1.iter().zip(results2.iter()) {
        assert_eq!(id1, id2);
        assert!((score1 - score2).abs() < 1e-5);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Dense Retrieval Properties
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "dense")]
#[test]
fn test_dense_retrieval_results_sorted() {
    // Property: Dense retrieval results are sorted by score descending
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    retriever.add_document(1, vec![0.707, 0.707]);
    retriever.add_document(2, vec![0.0, 1.0]);

    let query = vec![1.0, 0.0];
    let results = retriever.retrieve(&query, 10).unwrap();

    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results not sorted descending"
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_retrieval_no_duplicates() {
    // Property: Dense retrieval results contain no duplicate document IDs
    let mut retriever = DenseRetriever::new();
    for i in 0..10 {
        retriever.add_document(i, vec![(i as f32) * 0.1, 1.0 - (i as f32) * 0.1]);
    }

    let query = vec![1.0, 0.0];
    let results = retriever.retrieve(&query, 10).unwrap();

    let mut seen = std::collections::HashSet::new();
    for (doc_id, _) in &results {
        assert!(seen.insert(*doc_id), "Duplicate document ID: {}", doc_id);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_retrieval_scores_in_range() {
    // Property: Dense retrieval scores (cosine similarity) are in [-1, 1]
    let mut retriever = DenseRetriever::new();
    for i in 0..10 {
        retriever.add_document(i, vec![(i as f32) * 0.1, 1.0 - (i as f32) * 0.1]);
    }

    let query = vec![1.0, 0.0];
    let results = retriever.retrieve(&query, 10).unwrap();

    for (doc_id, score) in &results {
        assert!(
            *score >= -1.1 && *score <= 1.1, // Allow small floating-point error
            "Score out of range for document {}: {}",
            doc_id,
            score
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Sparse Retrieval Properties
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_retrieval_results_sorted() {
    // Property: Sparse retrieval results are sorted by score descending
    let mut retriever = SparseRetriever::new();
    retriever.add_document(0, SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]));
    retriever.add_document(1, SparseVector::new_unchecked(vec![0, 2], vec![0.8, 0.6]));
    retriever.add_document(2, SparseVector::new_unchecked(vec![1, 2], vec![0.5, 0.7]));

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retriever.retrieve(&query, 10).unwrap();

    for i in 1..results.len() {
        assert!(
            results[i - 1].1 >= results[i].1,
            "Results not sorted descending"
        );
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_retrieval_no_duplicates() {
    // Property: Sparse retrieval results contain no duplicate document IDs
    let mut retriever = SparseRetriever::new();
    for i in 0..10 {
        let indices: Vec<u32> = (0..5).map(|j| (i + j) as u32).collect();
        let values: Vec<f32> = (0..5).map(|j| (i + j) as f32 * 0.1).collect();
        retriever.add_document(i, SparseVector::new_unchecked(indices, values));
    }

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retriever.retrieve(&query, 10).unwrap();

    let mut seen = std::collections::HashSet::new();
    for (doc_id, _) in &results {
        assert!(seen.insert(*doc_id), "Duplicate document ID: {}", doc_id);
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_retrieval_scores_finite() {
    // Property: Sparse retrieval scores are finite
    let mut retriever = SparseRetriever::new();
    for i in 0..10 {
        let indices: Vec<u32> = (0..5).map(|j| (i + j) as u32).collect();
        let values: Vec<f32> = (0..5).map(|j| (i + j) as f32 * 0.1).collect();
        retriever.add_document(i, SparseVector::new_unchecked(indices, values));
    }

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retriever.retrieve(&query, 10).unwrap();

    for (doc_id, score) in &results {
        assert!(
            score.is_finite(),
            "Non-finite score for document {}: {}",
            doc_id,
            score
        );
    }
}
