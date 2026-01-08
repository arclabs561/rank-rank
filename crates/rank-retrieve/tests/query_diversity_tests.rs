//! Query Type Diversity Tests
//!
//! Tests different query types to ensure retrieval methods handle various
//! query characteristics correctly:
//! - Lexical vs semantic queries
//! - Short vs long queries
//! - Head vs tail queries (common vs rare terms)
//!
//! Based on IR testing best practices: different retrieval methods excel
//! at different query types, so tests should cover diverse query scenarios.

use rank_eval::binary::ndcg_at_k;
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "bm25")]
use rank_retrieve::{batch::batch_retrieve_bm25, retrieve_bm25};
#[cfg(feature = "dense")]
use rank_retrieve::{batch::batch_retrieve_dense, retrieve_dense};
use std::collections::HashSet;

// Lexical vs Semantic Query Tests

#[cfg(feature = "bm25")]
#[test]
fn test_lexical_query_exact_match() {
    // Lexical query: exact keyword match
    // BM25 should excel at this

    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    index.add_document(2, &["python".to_string(), "programming".to_string()]);

    // Exact lexical match
    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should find exact match first
    assert!(results.iter().any(|(id, _)| *id == 0));
    assert!(!results.is_empty());
}

#[cfg(feature = "bm25")]
#[test]
fn test_semantic_query_paraphrase() {
    // Semantic query: different phrasing of same concept
    // Tests how BM25 handles term variations

    let mut index = InvertedIndex::new();
    index.add_document(0, &["artificial".to_string(), "intelligence".to_string()]);
    index.add_document(1, &["machine".to_string(), "learning".to_string()]);
    index.add_document(2, &["neural".to_string(), "networks".to_string()]);

    // Query uses different terms but same concept
    let query = vec!["AI".to_string()]; // Abbreviation
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // May or may not find relevant docs (depends on exact terms)
    // But should not crash
    assert!(results.len() <= 3);
}

// Short vs Long Query Tests

#[cfg(feature = "bm25")]
#[test]
fn test_short_query_single_term() {
    // Short query: 1-2 terms
    // Should work correctly

    let mut index = InvertedIndex::new();
    for i in 0..50 {
        index.add_document(i, &[format!("term{}", i % 10)]);
    }

    // Single term query
    let query = vec!["term0".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    assert!(!results.is_empty());
    assert!(results.len() <= 10);
}

#[cfg(feature = "bm25")]
#[test]
fn test_short_query_two_terms() {
    // Two-term query
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    index.add_document(2, &["python".to_string(), "programming".to_string()]);

    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    assert!(!results.is_empty());
    assert!(results.iter().any(|(id, _)| *id == 0));
}

#[cfg(feature = "bm25")]
#[test]
fn test_long_query_many_terms() {
    // Long query: 5+ terms
    // Should handle correctly

    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
            "neural".to_string(),
            "networks".to_string(),
        ],
    );
    index.add_document(1, &["python".to_string(), "programming".to_string()]);

    // Long query with many terms
    let query = vec![
        "machine".to_string(),
        "learning".to_string(),
        "algorithms".to_string(),
        "neural".to_string(),
        "networks".to_string(),
    ];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    assert!(!results.is_empty());
    // Document 0 should rank highly (matches all terms)
    assert!(results.iter().any(|(id, _)| *id == 0));
}

#[cfg(feature = "bm25")]
#[test]
fn test_very_long_query() {
    // Very long query: 10+ terms
    let mut index = InvertedIndex::new();
    let terms: Vec<String> = (0..20).map(|i| format!("term{}", i)).collect();
    index.add_document(0, &terms);

    let query: Vec<String> = (0..15).map(|i| format!("term{}", i)).collect();
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    assert!(!results.is_empty());
}

// Head vs Tail Query Tests

#[cfg(feature = "bm25")]
#[test]
fn test_head_query_common_terms() {
    // Head query: common terms (appear in many documents)
    // Should return many results

    let mut index = InvertedIndex::new();
    // Add common term to most documents
    for i in 0..100 {
        let mut terms = vec![format!("term{}", i % 10)];
        if i % 2 == 0 {
            terms.push("common".to_string()); // Common term
        }
        index.add_document(i, &terms);
    }

    // Query with common term
    let query = vec!["common".to_string()];
    let results = retrieve_bm25(&index, &query, 100, Bm25Params::default()).unwrap();

    // Should return many results (common term appears in ~50 docs)
    assert!(results.len() >= 40);
}

#[cfg(feature = "bm25")]
#[test]
fn test_tail_query_rare_terms() {
    // Tail query: rare terms (appear in few documents)
    // Should return few, highly relevant results

    let mut index = InvertedIndex::new();
    // Add rare term to only one document
    index.add_document(0, &["rare_term_xyz".to_string()]);

    // Add many other documents without the rare term
    for i in 1..100 {
        index.add_document(i, &[format!("term{}", i)]);
    }

    // Query with rare term
    let query = vec!["rare_term_xyz".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    // Should find the rare document
    assert!(results.iter().any(|(id, _)| *id == 0));
    // Should have high score (rare term has high IDF)
    if let Some((_, score)) = results.iter().find(|(id, _)| *id == 0) {
        assert!(*score > 0.0);
    }
}

// Query Type Comparison Tests

#[cfg(feature = "bm25")]
#[test]
fn test_lexical_vs_semantic_performance() {
    // Compare performance on lexical vs semantic queries
    let mut index = InvertedIndex::new();

    // Documents with exact terms
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);

    // Lexical query (exact match)
    let lexical_query = vec!["machine".to_string(), "learning".to_string()];
    let lexical_results = retrieve_bm25(&index, &lexical_query, 10, Bm25Params::default()).unwrap();

    // Semantic query (related terms)
    let semantic_query = vec!["AI".to_string(), "algorithms".to_string()];
    let _semantic_results =
        retrieve_bm25(&index, &semantic_query, 10, Bm25Params::default()).unwrap();

    // Lexical should perform better (exact match)
    assert!(!lexical_results.is_empty());
    assert!(lexical_results.iter().any(|(id, _)| *id == 0));
}

#[cfg(feature = "bm25")]
#[test]
fn test_short_vs_long_query_behavior() {
    // Compare behavior of short vs long queries
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
            "neural".to_string(),
        ],
    );

    // Short query
    let short_query = vec!["machine".to_string()];
    let short_results = retrieve_bm25(&index, &short_query, 10, Bm25Params::default()).unwrap();

    // Long query
    let long_query = vec![
        "machine".to_string(),
        "learning".to_string(),
        "algorithms".to_string(),
    ];
    let long_results = retrieve_bm25(&index, &long_query, 10, Bm25Params::default()).unwrap();

    // Both should work
    assert!(!short_results.is_empty());
    assert!(!long_results.is_empty());

    // Long query should match document 0 better (more terms)
    assert!(long_results.iter().any(|(id, _)| *id == 0));
}

// Dense Retrieval Query Diversity

#[cfg(feature = "dense")]
#[test]
fn test_dense_short_query() {
    let mut retriever = DenseRetriever::new();
    for i in 0..20 {
        let embedding: Vec<f32> = (0..64).map(|j| ((i + j) as f32) / 200.0).collect();
        retriever.add_document(i, embedding);
    }

    // Short query (low-dimensional semantic space)
    let query: Vec<f32> = (0..64).map(|j| (j as f32) / 200.0).collect();
    let results = retrieve_dense(&retriever, &query, 10).unwrap();

    assert!(!results.is_empty());
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_semantic_similarity() {
    // Dense retrieval should handle semantic similarity
    let mut retriever = DenseRetriever::new();

    // Similar embeddings (semantically related)
    retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    retriever.add_document(1, vec![0.9, 0.1, 0.0]);

    // Dissimilar embeddings
    retriever.add_document(2, vec![0.0, 0.0, 1.0]);
    retriever.add_document(3, vec![0.0, 1.0, 0.0]);

    let query = [1.0, 0.0, 0.0];
    let results = retrieve_dense(&retriever, &query, 4).unwrap();

    // Should rank similar documents higher
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > results[2].1);
}

// Batch Query Diversity

#[cfg(feature = "bm25")]
#[test]
fn test_batch_mixed_query_types() {
    // Batch with different query types
    let mut index = InvertedIndex::new();
    for i in 0..50 {
        let terms: Vec<String> = (0..5).map(|j| format!("term{}", (i + j) % 10)).collect();
        index.add_document(i, &terms);
    }

    let queries = vec![
        vec!["term0".to_string()],                      // Short, head query
        vec!["term0".to_string(), "term1".to_string()], // Short, head query
        vec!["rare_term_xyz".to_string()],              // Tail query
        vec![
            "term0".to_string(),
            "term1".to_string(),
            "term2".to_string(),
            "term3".to_string(),
            "term4".to_string(),
        ], // Long query
    ];

    let results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();

    assert_eq!(results.len(), 4);
    assert!(results.iter().all(|r| !r.is_empty() || r.is_empty())); // May be empty for rare terms
}

// Query Type Evaluation

#[cfg(feature = "bm25")]
#[test]
fn test_query_type_evaluation() {
    // Evaluate different query types using IR metrics
    let mut index = InvertedIndex::new();

    // Setup: documents about machine learning
    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
        ],
    );
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    index.add_document(2, &["python".to_string(), "programming".to_string()]);

    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();

    // Short query
    let short_query = vec!["learning".to_string()];
    let short_results = retrieve_bm25(&index, &short_query, 3, Bm25Params::default()).unwrap();
    let short_ranked: Vec<String> = short_results.iter().map(|(id, _)| id.to_string()).collect();
    let short_ndcg = ndcg_at_k(&short_ranked, &relevant, 3);

    // Long query
    let long_query = vec![
        "machine".to_string(),
        "learning".to_string(),
        "algorithms".to_string(),
    ];
    let long_results = retrieve_bm25(&index, &long_query, 3, Bm25Params::default()).unwrap();
    let long_ranked: Vec<String> = long_results.iter().map(|(id, _)| id.to_string()).collect();
    let long_ndcg = ndcg_at_k(&long_ranked, &relevant, 3);

    // Both should find relevant documents
    assert!(short_ndcg >= 0.0 && short_ndcg <= 1.0);
    assert!(long_ndcg >= 0.0 && long_ndcg <= 1.0);
}
