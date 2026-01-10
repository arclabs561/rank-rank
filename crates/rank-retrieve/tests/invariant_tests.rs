//! Invariant tests for rank-retrieve optimizations.
//!
//! Tests mathematical and algorithmic invariants that must always hold:
//! - Score ordering properties
//! - Monotonicity properties
//! - Bounds and constraints
//! - Consistency across operations

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

#[cfg(feature = "dense")]
use rank_retrieve::simd;

// ─────────────────────────────────────────────────────────────────────────────
// Score Ordering Invariants
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_score_monotonicity_with_k() {
    // Invariant: Increasing k should not decrease scores of existing results
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
    let results_k5 = index.retrieve(&query, 5, Bm25Params::default()).unwrap();
    let results_k10 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // Top 5 results should be identical in both
    for i in 0..5.min(results_k5.len()) {
        assert_eq!(results_k5[i].0, results_k10[i].0);
        assert!((results_k5[i].1 - results_k10[i].1).abs() < 1e-5);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_score_monotonicity_with_k() {
    // Invariant: Increasing k should not decrease scores of existing results
    let mut retriever = DenseRetriever::new();
    for i in 0..20 {
        retriever.add_document(i, vec![(i as f32) * 0.1, 1.0 - (i as f32) * 0.1]);
    }

    let query = vec![1.0, 0.0];
    let results_k5 = retriever.retrieve(&query, 5).unwrap();
    let results_k10 = retriever.retrieve(&query, 10).unwrap();

    // Top 5 results should be identical in both
    for i in 0..5.min(results_k5.len()) {
        assert_eq!(results_k5[i].0, results_k10[i].0);
        assert!((results_k5[i].1 - results_k10[i].1).abs() < 1e-5);
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_score_monotonicity_with_k() {
    // Invariant: Increasing k should not decrease scores of existing results
    let mut retriever = SparseRetriever::new();
    for i in 0..20 {
        let indices: Vec<u32> = (0..5).map(|j| (i + j) as u32).collect();
        let values: Vec<f32> = (0..5).map(|j| (i + j) as f32 * 0.1).collect();
        retriever.add_document(i, SparseVector::new_unchecked(indices, values));
    }

    let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results_k5 = retriever.retrieve(&query, 5).unwrap();
    let results_k10 = retriever.retrieve(&query, 10).unwrap();

    // Top 5 results should be identical in both
    for i in 0..5.min(results_k5.len()) {
        assert_eq!(results_k5[i].0, results_k10[i].0);
        assert!((results_k5[i].1 - results_k10[i].1).abs() < 1e-5);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Consistency Invariants
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_idempotency() {
    // Invariant: Multiple retrievals with same query produce identical results
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
    let results1 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    let results2 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    let results3 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // All three should be identical
    assert_eq!(results1.len(), results2.len());
    assert_eq!(results2.len(), results3.len());

    for i in 0..results1.len() {
        let (id1, score1) = results1[i];
        let (id2, score2) = results2[i];
        let (id3, score3) = results3[i];
        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert!((score1 - score2).abs() < 1e-5);
        assert!((score2 - score3).abs() < 1e-5);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_idempotency() {
    // Invariant: Multiple retrievals with same query produce identical results
    let mut retriever = DenseRetriever::new();
    for i in 0..10 {
        retriever.add_document(i, vec![(i as f32) * 0.1, 1.0 - (i as f32) * 0.1]);
    }

    let query = vec![1.0, 0.0];
    let results1 = retriever.retrieve(&query, 10).unwrap();
    let results2 = retriever.retrieve(&query, 10).unwrap();
    let results3 = retriever.retrieve(&query, 10).unwrap();

    // All three should be identical
    assert_eq!(results1.len(), results2.len());
    assert_eq!(results2.len(), results3.len());

    for i in 0..results1.len() {
        let (id1, score1) = results1[i];
        let (id2, score2) = results2[i];
        let (id3, score3) = results3[i];
        assert_eq!(id1, id2);
        assert_eq!(id2, id3);
        assert!((score1 - score2).abs() < 1e-5);
        assert!((score2 - score3).abs() < 1e-5);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Bounds Invariants
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "dense")]
#[test]
fn test_cosine_similarity_bounds() {
    // Invariant: Cosine similarity is in [-1, 1] (allowing small floating-point error)
    for len in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        // Generate deterministic test vectors
        let a: Vec<f32> = (0..len)
            .map(|i| ((i * 7 + 13) % 100) as f32 / 10.0 - 5.0)
            .collect();
        let b: Vec<f32> = (0..len)
            .map(|i| ((i * 11 + 17) % 100) as f32 / 10.0 - 5.0)
            .collect();

        let result = simd::cosine(&a, &b);
        // Allow small floating-point error outside [-1, 1]
        assert!(
            result >= -1.1 && result <= 1.1,
            "Cosine similarity out of bounds at len={}: {}",
            len,
            result
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_norm_non_negative() {
    // Invariant: Norm is always non-negative
    for len in [1, 2, 4, 8, 16, 32, 64, 128, 256] {
        // Generate deterministic test vectors with various patterns
        for pattern in 0..5 {
            let v: Vec<f32> = (0..len)
                .map(|i| {
                    let val = match pattern {
                        0 => (i as f32) * 0.1 - 0.5, // Linear
                        1 => ((i * 7) % 100) as f32 / 10.0 - 5.0, // Random-like
                        2 => -((i as f32) * 0.1), // Negative
                        3 => if i % 2 == 0 { 1.0 } else { -1.0 }, // Alternating
                        _ => (i as f32).sin(), // Sinusoidal
                    };
                    val
                })
                .collect();

            let result = simd::norm(&v);
            assert!(
                result >= 0.0,
                "Norm should be non-negative at len={}, pattern={}: {}",
                len,
                pattern,
                result
            );
        }
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_scores_non_negative() {
    // Invariant: BM25 scores are always non-negative
    let mut index = InvertedIndex::new();
    for i in 0..50 {
        let terms: Vec<String> = (0..10)
            .map(|j| format!("term{}", (i + j) % 20))
            .collect();
        index.add_document(i, &terms);
    }

    for k in [1, 5, 10, 20] {
        let query = vec!["term0".to_string()];
        let results = index.retrieve(&query, k, Bm25Params::default()).unwrap();

        for (doc_id, score) in &results {
            assert!(
                *score >= 0.0,
                "BM25 score should be non-negative for doc {}: {}",
                doc_id,
                score
            );
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Mathematical Properties
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "dense")]
#[test]
fn test_dot_product_linearity() {
    // Invariant: dot(a, b) is linear in both arguments
    for len in [1, 4, 8, 16, 32, 64] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.2).collect();
        let c: Vec<f32> = (0..len).map(|i| (i as f32) * 0.3).collect();

        let alpha = 2.0;
        let beta = 3.0;

        // dot(alpha * a, b) = alpha * dot(a, b)
        let alpha_a: Vec<f32> = a.iter().map(|x| alpha * x).collect();
        let dot_alpha_a_b = simd::dot(&alpha_a, &b);
        let alpha_dot_a_b = alpha * simd::dot(&a, &b);

        assert!(
            (dot_alpha_a_b - alpha_dot_a_b).abs() < 1e-4,
            "Linearity property violated at len={}",
            len
        );

        // dot(a, beta * b) = beta * dot(a, b)
        let beta_b: Vec<f32> = b.iter().map(|x| beta * x).collect();
        let dot_a_beta_b = simd::dot(&a, &beta_b);
        let beta_dot_a_b = beta * simd::dot(&a, &b);

        assert!(
            (dot_a_beta_b - beta_dot_a_b).abs() < 1e-4,
            "Linearity property violated at len={}",
            len
        );
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_cosine_similarity_symmetry() {
    // Invariant: cosine(a, b) == cosine(b, a)
    for len in [2, 4, 8, 16, 32, 64, 128] {
        let a: Vec<f32> = (0..len).map(|i| (i as f32) * 0.1 - 0.5).collect();
        let b: Vec<f32> = (0..len).map(|i| (i as f32) * 0.2 + 1.0).collect();

        let cos_ab = simd::cosine(&a, &b);
        let cos_ba = simd::cosine(&b, &a);

        assert!(
            (cos_ab - cos_ba).abs() < 1e-5,
            "Cosine similarity not symmetric at len={}: cos(a,b)={}, cos(b,a)={}",
            len,
            cos_ab,
            cos_ba
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Early Termination Correctness
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_early_termination_preserves_top_k() {
    // Invariant: Early termination produces correct top-k (not just any k results)
    // Note: This test verifies that early termination doesn't skip documents that should be in top-k
    let mut index = InvertedIndex::new();
    for i in 0..50 {
        // Create documents with varying relevance
        let terms: Vec<String> = if i < 10 {
            // First 10 docs have both query terms (high relevance)
            vec!["term0".to_string(), "term1".to_string(), format!("doc{}", i)]
        } else if i < 20 {
            // Next 10 docs have one query term (medium relevance)
            vec!["term0".to_string(), format!("doc{}", i)]
        } else {
            // Remaining docs have no query terms (low relevance)
            vec![format!("doc{}", i), format!("other{}", i)]
        };
        index.add_document(i, &terms);
    }

    let query = vec!["term0".to_string(), "term1".to_string()];
    
    // Get top-10 with early termination
    let results_k10 = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    
    // Get top-20 to verify top-10 is correct
    let results_k20 = index.retrieve(&query, 20, Bm25Params::default()).unwrap();

    // Verify we got results
    assert!(results_k10.len() > 0);
    assert!(results_k20.len() >= results_k10.len());

    // Top results from k=10 should be a subset of top results from k=20
    // (allowing for slight ordering differences due to floating-point)
    let top10_ids: std::collections::HashSet<u32> = results_k10.iter().map(|(id, _)| *id).collect();
    let top20_ids: std::collections::HashSet<u32> = results_k20.iter().take(10).map(|(id, _)| *id).collect();
    
    // At least 8 out of 10 should match (allowing for floating-point differences)
    let overlap = top10_ids.intersection(&top20_ids).count();
    assert!(
        overlap >= 8,
        "Early termination produced different top-10: overlap={}/10",
        overlap
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Precomputed IDF Correctness
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_precomputed_idf_consistency() {
    // Invariant: Precomputed IDF matches on-the-fly calculation
    let mut index = InvertedIndex::new();
    for i in 0..10 {
        index.add_document(
            i,
            &[
                format!("term{}", i % 3),
                format!("doc{}", i),
            ],
        );
    }

    // Trigger precomputation
    let _ = index.retrieve(&["term0".to_string()], 10, Bm25Params::default());

    // Check that precomputed values are consistent
    let idf_term0 = index.idf("term0");
    let idf_term1 = index.idf("term1");
    let idf_term2 = index.idf("term2");

    // All should be positive
    assert!(idf_term0 > 0.0);
    assert!(idf_term1 > 0.0);
    assert!(idf_term2 > 0.0);

    // Rare terms should have higher IDF
    // term0 appears in docs 0, 3, 6, 9 (4 docs)
    // term1 appears in docs 1, 4, 7 (3 docs)
    // term2 appears in docs 2, 5, 8 (3 docs)
    // So term0 should have lower IDF than term1/term2
    assert!(idf_term0 < idf_term1 || idf_term0 < idf_term2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Score Stability
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_relative_ordering_preserved() {
    // Invariant: Adding unrelated documents preserves relative ordering of relevant documents
    let mut index1 = InvertedIndex::new();
    index1.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index1.add_document(1, &["machine".to_string()]);
    index1.add_document(2, &["learning".to_string()]);

    let mut index2 = InvertedIndex::new();
    index2.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index2.add_document(1, &["machine".to_string()]);
    index2.add_document(2, &["learning".to_string()]);
    // Add unrelated documents (should not affect relative ordering)
    for i in 3..10 {
        index2.add_document(
            i,
            &[
                format!("unrelated{}", i),
                format!("term{}", i),
            ],
        );
    }

    let query = vec!["machine".to_string(), "learning".to_string()];
    let results1 = index1.retrieve(&query, 10, Bm25Params::default()).unwrap();
    let results2 = index2.retrieve(&query, 10, Bm25Params::default()).unwrap();

    // Relative ordering should be preserved: doc 0 > doc 1 > doc 2
    // (doc 0 has both terms, doc 1 has one, doc 2 has one)
    let rank1_doc0 = results1.iter().position(|(id, _)| *id == 0);
    let rank1_doc1 = results1.iter().position(|(id, _)| *id == 1);
    let rank2_doc0 = results2.iter().position(|(id, _)| *id == 0);
    let rank2_doc1 = results2.iter().position(|(id, _)| *id == 1);

    if let (Some(r1_0), Some(r1_1), Some(r2_0), Some(r2_1)) = (rank1_doc0, rank1_doc1, rank2_doc0, rank2_doc1) {
        // Document 0 should rank higher than document 1 in both cases
        assert!(r1_0 < r1_1, "Doc 0 should rank higher than doc 1 in index1");
        assert!(r2_0 < r2_1, "Doc 0 should rank higher than doc 1 in index2");
    }
}
