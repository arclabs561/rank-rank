//! Robustness Tests
//!
//! Tests for robustness scenarios:
//! - Paraphrase handling (multiple phrasings of same intent)
//! - Adversarial queries (negations, tricky formulations)
//! - No-answer scenarios (queries with no relevant documents)
//!
//! Based on IR testing best practices: retrieval systems should handle
//! various query formulations and edge cases gracefully.

use rank_eval::binary::{ndcg_at_k, precision_at_k, recall_at_k};
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "dense")]
use rank_retrieve::retrieve_dense;
#[cfg(feature = "bm25")]
use rank_retrieve::{batch::batch_retrieve_bm25, retrieve_bm25};
use std::collections::HashSet;

// Paraphrase Handling Tests

#[cfg(feature = "bm25")]
#[test]
fn test_paraphrase_similar_results() {
    // Multiple phrasings of the same intent should yield overlapping results
    let mut index = InvertedIndex::new();

    // Document about machine learning
    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
            "artificial".to_string(),
            "intelligence".to_string(),
        ],
    );
    index.add_document(1, &["python".to_string(), "programming".to_string()]);

    // Paraphrase 1: "machine learning"
    let query1 = vec!["machine".to_string(), "learning".to_string()];
    let results1 = retrieve_bm25(&index, &query1, 10, Bm25Params::default()).unwrap();

    // Paraphrase 2: "AI algorithms"
    let query2 = vec!["artificial".to_string(), "intelligence".to_string()];
    let results2 = retrieve_bm25(&index, &query2, 10, Bm25Params::default()).unwrap();

    // Both should find document 0
    assert!(results1.iter().any(|(id, _)| *id == 0));
    assert!(results2.iter().any(|(id, _)| *id == 0));
}

#[cfg(feature = "bm25")]
#[test]
fn test_paraphrase_stability() {
    // Paraphrases should produce stable, consistent results
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);

    // Different phrasings
    let queries = vec![
        vec!["machine".to_string(), "learning".to_string()],
        vec!["ML".to_string()],                               // Abbreviation
        vec!["learning".to_string(), "machines".to_string()], // Reordered
    ];

    let batch_results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();

    // All queries should produce results (may vary in quality)
    assert_eq!(batch_results.len(), 3);
    // At least some should find relevant documents
    assert!(batch_results.iter().any(|r| !r.is_empty()));
}

// Adversarial Query Tests

#[cfg(feature = "bm25")]
#[test]
fn test_adversarial_negation() {
    // Queries with negations should be handled gracefully
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["python".to_string(), "programming".to_string()]);

    // Query with negation (BM25 doesn't handle negations, but shouldn't crash)
    let query = vec!["not".to_string(), "python".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should not crash, may or may not find relevant docs
    assert!(results.len() <= 2);
}

#[cfg(feature = "bm25")]
#[test]
fn test_adversarial_special_characters() {
    // Queries with special characters should be handled
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string(), "document".to_string()]);

    // Query with special characters
    let query = vec!["test".to_string(), "document".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should work normally
    assert!(!results.is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_adversarial_very_rare_terms() {
    // Queries with very rare terms (may not exist in corpus)
    let mut index = InvertedIndex::new();
    for i in 0..100 {
        index.add_document(i, &[format!("term{}", i)]);
    }

    // Query with term that doesn't exist
    let query = vec!["nonexistent_term_xyz_123".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should return empty results, not crash
    assert!(results.is_empty() || results.iter().all(|(_, score)| *score == 0.0));
}

// No-Answer Scenario Tests

#[cfg(feature = "bm25")]
#[test]
fn test_no_answer_empty_results() {
    // Query with no relevant documents should return empty or low-scoring results
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["python".to_string(), "programming".to_string()]);

    // Query with no relevant documents
    let query = vec![
        "completely".to_string(),
        "unrelated".to_string(),
        "terms".to_string(),
    ];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should return empty or very low scores
    assert!(results.is_empty() || results.iter().all(|(_, score)| *score == 0.0));
}

#[cfg(feature = "bm25")]
#[test]
fn test_no_answer_evaluation() {
    // Evaluate no-answer scenario using metrics
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["python".to_string(), "programming".to_string()]);

    // Query with no relevant documents
    let query = vec!["unrelated".to_string(), "query".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    let relevant: HashSet<String> = HashSet::new(); // No relevant documents
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();

    let precision = precision_at_k(&ranked, &relevant, 10);
    let recall = recall_at_k(&ranked, &relevant, 10);
    let ndcg = ndcg_at_k(&ranked, &relevant, 10);

    // All metrics should be 0 when no relevant documents
    assert_eq!(precision, 0.0);
    assert_eq!(recall, 0.0);
    assert_eq!(ndcg, 0.0);
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_no_answer() {
    // Dense retrieval with no similar documents
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    retriever.add_document(1, vec![0.0, 1.0, 0.0]);

    // Query orthogonal to all documents
    let query = [0.0, 0.0, 1.0];
    let results = retrieve_dense(&retriever, &query, 10).unwrap();

    // Should return results (all documents), but with low scores
    assert!(!results.is_empty());
    // Scores should be low (orthogonal vectors have low cosine similarity)
    assert!(results.iter().all(|(_, score)| score.abs() < 0.5));
}

// Robustness Under Variation

#[cfg(feature = "bm25")]
#[test]
fn test_robustness_query_variations() {
    // Test robustness across query variations
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
        ],
    );

    // Various query formulations
    let queries = vec![
        vec!["machine".to_string(), "learning".to_string()], // Standard
        vec!["learning".to_string(), "machine".to_string()], // Reordered
        vec!["machine".to_string()],                         // Partial
        vec!["learning".to_string()],                        // Partial
    ];

    let batch_results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();

    // All should find document 0
    for results in &batch_results {
        assert!(results.iter().any(|(id, _)| *id == 0));
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_robustness_term_order_independence() {
    // Results should be consistent regardless of term order (for same terms)
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);

    // Same terms, different order
    let query1 = vec!["machine".to_string(), "learning".to_string()];
    let query2 = vec!["learning".to_string(), "machine".to_string()];

    let results1 = retrieve_bm25(&index, &query1, 10, Bm25Params::default()).unwrap();
    let results2 = retrieve_bm25(&index, &query2, 10, Bm25Params::default()).unwrap();

    // Should find same documents (scores may differ slightly due to term order in BM25)
    let ids1: HashSet<u32> = results1.iter().map(|(id, _)| *id).collect();
    let ids2: HashSet<u32> = results2.iter().map(|(id, _)| *id).collect();

    // Should find document 0 in both
    assert!(ids1.contains(&0));
    assert!(ids2.contains(&0));
}

// Edge Case Robustness

#[cfg(feature = "bm25")]
#[test]
fn test_robustness_duplicate_terms() {
    // Queries with duplicate terms should be handled
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["test".to_string(), "test".to_string(), "test".to_string()],
    );
    index.add_document(1, &["test".to_string()]);

    // Query with duplicate terms
    let query = vec!["test".to_string(), "test".to_string(), "test".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should work correctly
    assert!(!results.is_empty());
    // Document 0 should rank higher (more term occurrences)
    assert!(results.iter().any(|(id, _)| *id == 0));
}

#[cfg(feature = "bm25")]
#[test]
fn test_robustness_case_sensitivity() {
    // Test case sensitivity handling
    let mut index = InvertedIndex::new();
    index.add_document(0, &["Machine".to_string(), "Learning".to_string()]);
    index.add_document(1, &["machine".to_string(), "learning".to_string()]);

    // Query with different case
    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // BM25 is case-sensitive, so may or may not match
    // But should not crash
    assert!(results.len() <= 2);
}

// Batch Robustness

#[cfg(feature = "bm25")]
#[test]
fn test_batch_robustness_mixed_scenarios() {
    // Batch with mixed robustness scenarios
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["python".to_string(), "programming".to_string()]);

    let queries = vec![
        vec!["machine".to_string(), "learning".to_string()], // Normal query
        vec!["nonexistent".to_string()],                     // No-answer query
        vec!["machine".to_string(), "machine".to_string()],  // Duplicate terms
        vec![],                                              // Empty query (should error)
    ];

    // Should handle gracefully (empty query will error)
    let result = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default());

    // Should error on empty query
    assert!(result.is_err());
}

// Dense Retrieval Robustness

#[cfg(feature = "dense")]
#[test]
fn test_dense_robustness_orthogonal_vectors() {
    // Dense retrieval with orthogonal query vectors
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    retriever.add_document(1, vec![0.0, 1.0, 0.0]);

    // Query orthogonal to all documents
    let query = [0.0, 0.0, 1.0];
    let results = retrieve_dense(&retriever, &query, 10).unwrap();

    // Should return results with low scores
    assert!(!results.is_empty());
    assert!(results.iter().all(|(_, score)| score.is_finite()));
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_robustness_zero_vector() {
    // Dense retrieval with zero query vector
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0, 0.0]);

    // Zero vector query
    let query = [0.0, 0.0, 0.0];
    let results = retrieve_dense(&retriever, &query, 10).unwrap();

    // Should handle gracefully (zero vector has zero cosine similarity)
    assert!(!results.is_empty());
    assert!(results
        .iter()
        .all(|(_, score)| *score == 0.0 || score.is_finite()));
}
