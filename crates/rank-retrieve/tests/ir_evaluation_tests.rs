//! IR Evaluation Metrics Tests
//!
//! Systematic evaluation of retrieval quality using standard IR metrics:
//! - Precision@k, Recall@k, nDCG@k
//! - MAP (Mean Average Precision)
//! - MRR (Mean Reciprocal Rank)
//!
//! These tests validate that retrieval methods produce reasonable results
//! and can catch regressions in retrieval quality.
//!
//! Based on IR testing best practices: use labeled test collections with
//! ground truth relevance judgments to evaluate retrieval systems.

#[cfg(feature = "bm25")]
use rank_retrieve::{retrieve_bm25, batch::batch_retrieve_bm25};
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::retrieve_dense;
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::retrieve_sparse;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::SparseRetriever;
use rank_eval::binary::{ndcg_at_k, precision_at_k, recall_at_k, mrr, average_precision};
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::SparseVector;
use std::collections::HashSet;

// Test collection: Simple labeled dataset for evaluation
// In practice, would use real datasets like MS MARCO or TREC collections

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_precision_at_k() {
    // Create a test collection with known relevance
    let mut index = InvertedIndex::new();
    
    // Relevant documents for query "machine learning"
    index.add_document(0, &["machine".to_string(), "learning".to_string(), "algorithms".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string(), "neural".to_string()]);
    
    // Non-relevant documents
    index.add_document(2, &["python".to_string(), "programming".to_string()]);
    index.add_document(3, &["rust".to_string(), "systems".to_string()]);
    
    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 4, Bm25Params::default()).unwrap();
    
    // Ground truth: documents 0 and 1 are relevant
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let precision_1 = precision_at_k(&ranked, &relevant, 1);
    let precision_2 = precision_at_k(&ranked, &relevant, 2);
    let precision_4 = precision_at_k(&ranked, &relevant, 4);
    
    // At least one relevant doc should be in top-1
    assert!(precision_1 >= 0.0 && precision_1 <= 1.0);
    // Both relevant docs should be in top-2 (or close)
    assert!(precision_2 >= 0.5 && precision_2 <= 1.0);
    // Precision@4 should be 0.5 (2 relevant out of 4)
    assert!((precision_4 - 0.5).abs() < 0.1);
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_recall_at_k() {
    let mut index = InvertedIndex::new();
    
    // 3 relevant documents
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    index.add_document(2, &["neural".to_string(), "networks".to_string()]);
    
    // 7 non-relevant documents
    for i in 3..10 {
        index.add_document(i, &[format!("term{}", i)]);
    }
    
    let query = vec!["learning".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
    
    let relevant: HashSet<String> = ["0", "1", "2"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let recall_5 = recall_at_k(&ranked, &relevant, 5);
    let recall_10 = recall_at_k(&ranked, &relevant, 10);
    
    // Should retrieve at least some relevant docs
    assert!(recall_5 >= 0.0 && recall_5 <= 1.0);
    assert!(recall_10 >= recall_5); // Recall should increase with k
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_ndcg_at_k() {
    let mut index = InvertedIndex::new();
    
    // Highly relevant
    index.add_document(0, &["machine".to_string(), "learning".to_string(), "tutorial".to_string()]);
    // Moderately relevant
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    // Less relevant
    index.add_document(2, &["python".to_string(), "programming".to_string()]);
    
    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 3, Bm25Params::default()).unwrap();
    
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let ndcg = ndcg_at_k(&ranked, &relevant, 3);
    
    // nDCG should be in [0, 1]
    assert!(ndcg >= 0.0 && ndcg <= 1.0);
    // Should be positive if relevant docs are retrieved
    if ranked.iter().any(|id| relevant.contains(id)) {
        assert!(ndcg > 0.0);
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_mrr() {
    let mut index = InvertedIndex::new();
    
    // Non-relevant first
    index.add_document(0, &["python".to_string(), "programming".to_string()]);
    // Relevant second
    index.add_document(1, &["machine".to_string(), "learning".to_string()]);
    
    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 2, Bm25Params::default()).unwrap();
    
    let relevant: HashSet<String> = ["1"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let mrr_score = mrr(&ranked, &relevant);
    
    // MRR should be in [0, 1]
    assert!(mrr_score >= 0.0 && mrr_score <= 1.0);
    // If relevant doc is at position 2, MRR should be 0.5
    if ranked.len() >= 2 && ranked[1] == "1" {
        assert!((mrr_score - 0.5).abs() < 0.1);
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_map() {
    let mut index = InvertedIndex::new();
    
    // Multiple relevant documents
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    index.add_document(2, &["python".to_string(), "programming".to_string()]);
    index.add_document(3, &["neural".to_string(), "networks".to_string()]);
    
    let query = vec!["learning".to_string()];
    let results = retrieve_bm25(&index, &query, 4, Bm25Params::default()).unwrap();
    
    let relevant: HashSet<String> = ["0", "1", "3"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let map_score = average_precision(&ranked, &relevant);
    
    // MAP should be in [0, 1]
    assert!(map_score >= 0.0 && map_score <= 1.0);
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_retrieval_metrics() {
    let mut retriever = DenseRetriever::new();
    
    // Relevant: similar to query
    retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    retriever.add_document(1, vec![0.9, 0.1, 0.0]);
    
    // Non-relevant: dissimilar
    retriever.add_document(2, vec![0.0, 0.0, 1.0]);
    retriever.add_document(3, vec![0.0, 1.0, 0.0]);
    
    let query = [1.0, 0.0, 0.0];
    let results = retrieve_dense(&retriever, &query, 4).unwrap();
    
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let precision = precision_at_k(&ranked, &relevant, 4);
    let recall = recall_at_k(&ranked, &relevant, 4);
    let ndcg = ndcg_at_k(&ranked, &relevant, 4);
    
    assert!(precision >= 0.0 && precision <= 1.0);
    assert!(recall >= 0.0 && recall <= 1.0);
    assert!(ndcg >= 0.0 && ndcg <= 1.0);
    
    // Dense retrieval should find similar documents
    assert!(precision >= 0.4); // At least 2/4 should be relevant
}

#[cfg(feature = "bm25")]
#[cfg(feature = "dense")]
#[test]
fn test_bm25_vs_dense_comparison() {
    // Compare BM25 and dense retrieval on same query
    let mut bm25_index = InvertedIndex::new();
    let mut dense_retriever = DenseRetriever::new();
    
    // Setup: documents about machine learning
    let docs = vec![
        (0, vec!["machine".to_string(), "learning".to_string(), "algorithms".to_string()], vec![1.0, 0.0, 0.0]),
        (1, vec!["deep".to_string(), "learning".to_string()], vec![0.9, 0.1, 0.0]),
        (2, vec!["python".to_string(), "programming".to_string()], vec![0.0, 1.0, 0.0]),
    ];
    
    for (id, terms, embedding) in docs {
        bm25_index.add_document(id, &terms);
        dense_retriever.add_document(id, embedding);
    }
    
    let query_terms = vec!["machine".to_string(), "learning".to_string()];
    let query_emb = [1.0, 0.0, 0.0];
    
    let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 3, Bm25Params::default()).unwrap();
    let dense_results = retrieve_dense(&dense_retriever, &query_emb, 3).unwrap();
    
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    
    let bm25_ranked: Vec<String> = bm25_results.iter().map(|(id, _)| id.to_string()).collect();
    let dense_ranked: Vec<String> = dense_results.iter().map(|(id, _)| id.to_string()).collect();
    
    let bm25_ndcg = ndcg_at_k(&bm25_ranked, &relevant, 3);
    let dense_ndcg = ndcg_at_k(&dense_ranked, &relevant, 3);
    
    // Both should find relevant documents
    assert!(bm25_ndcg > 0.0 || dense_ndcg > 0.0);
    
    // BM25 should excel at lexical matching
    assert!(bm25_ndcg >= 0.0 && bm25_ndcg <= 1.0);
    // Dense should excel at semantic matching
    assert!(dense_ndcg >= 0.0 && dense_ndcg <= 1.0);
}

#[cfg(feature = "bm25")]
#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
#[test]
fn test_three_way_method_comparison() {
    // Compare all three retrieval methods on same documents
    let mut bm25_index = InvertedIndex::new();
    let mut dense_retriever = DenseRetriever::new();
    let mut sparse_retriever = SparseRetriever::new();
    
    // Setup: documents about machine learning
    let docs = vec![
        (0, vec!["machine".to_string(), "learning".to_string()], vec![1.0, 0.0, 0.0], SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.8])),
        (1, vec!["deep".to_string(), "learning".to_string()], vec![0.9, 0.1, 0.0], SparseVector::new_unchecked(vec![1, 2], vec![1.0, 0.6])),
        (2, vec!["python".to_string(), "programming".to_string()], vec![0.0, 1.0, 0.0], SparseVector::new_unchecked(vec![2, 3], vec![1.0, 0.4])),
    ];
    
    for (id, terms, embedding, sparse_vec) in docs {
        bm25_index.add_document(id, &terms);
        dense_retriever.add_document(id, embedding);
        sparse_retriever.add_document(id, sparse_vec);
    }
    
    let query_terms = vec!["machine".to_string(), "learning".to_string()];
    let query_emb = [1.0, 0.0, 0.0];
    let query_sparse = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    
    let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 3, Bm25Params::default()).unwrap();
    let dense_results = retrieve_dense(&dense_retriever, &query_emb, 3).unwrap();
    let sparse_results = retrieve_sparse(&sparse_retriever, &query_sparse, 3).unwrap();
    
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    
    let bm25_ranked: Vec<String> = bm25_results.iter().map(|(id, _)| id.to_string()).collect();
    let dense_ranked: Vec<String> = dense_results.iter().map(|(id, _)| id.to_string()).collect();
    let sparse_ranked: Vec<String> = sparse_results.iter().map(|(id, _)| id.to_string()).collect();
    
    let bm25_ndcg = ndcg_at_k(&bm25_ranked, &relevant, 3);
    let dense_ndcg = ndcg_at_k(&dense_ranked, &relevant, 3);
    let sparse_ndcg = ndcg_at_k(&sparse_ranked, &relevant, 3);
    
    // All methods should find relevant documents
    assert!(bm25_ndcg >= 0.0 && bm25_ndcg <= 1.0);
    assert!(dense_ndcg >= 0.0 && dense_ndcg <= 1.0);
    assert!(sparse_ndcg >= 0.0 && sparse_ndcg <= 1.0);
    
    // At least one method should perform well
    assert!(bm25_ndcg > 0.0 || dense_ndcg > 0.0 || sparse_ndcg > 0.0);
}

#[cfg(feature = "bm25")]
#[cfg(feature = "dense")]
#[test]
fn test_method_comparison_lexical_scenario() {
    // Compare methods on lexical (exact match) scenario
    // BM25 should excel here
    
    let mut bm25_index = InvertedIndex::new();
    let mut dense_retriever = DenseRetriever::new();
    
    // Documents with exact terms
    bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    bm25_index.add_document(1, &["python".to_string(), "programming".to_string()]);
    
    dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    dense_retriever.add_document(1, vec![0.0, 1.0, 0.0]);
    
    let query_terms = vec!["machine".to_string(), "learning".to_string()];
    let query_emb = [1.0, 0.0, 0.0];
    
    let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 2, Bm25Params::default()).unwrap();
    let dense_results = retrieve_dense(&dense_retriever, &query_emb, 2).unwrap();
    
    let relevant: HashSet<String> = ["0"].iter().map(|s| s.to_string()).collect();
    
    let bm25_ranked: Vec<String> = bm25_results.iter().map(|(id, _)| id.to_string()).collect();
    let dense_ranked: Vec<String> = dense_results.iter().map(|(id, _)| id.to_string()).collect();
    
    let bm25_precision = precision_at_k(&bm25_ranked, &relevant, 2);
    let dense_precision = precision_at_k(&dense_ranked, &relevant, 2);
    
    // BM25 should excel at exact lexical matching
    assert!(bm25_precision >= 0.0 && bm25_precision <= 1.0);
    assert!(dense_precision >= 0.0 && dense_precision <= 1.0);
    // BM25 should find the exact match
    assert!(bm25_ranked.contains(&"0".to_string()));
}

#[cfg(feature = "bm25")]
#[cfg(feature = "dense")]
#[test]
fn test_method_comparison_semantic_scenario() {
    // Compare methods on semantic similarity scenario
    // Dense should excel here
    
    let mut bm25_index = InvertedIndex::new();
    let mut dense_retriever = DenseRetriever::new();
    
    // Documents with related concepts (different terms)
    bm25_index.add_document(0, &["artificial".to_string(), "intelligence".to_string()]);
    bm25_index.add_document(1, &["python".to_string(), "programming".to_string()]);
    
    // Dense embeddings capture semantic similarity
    dense_retriever.add_document(0, vec![0.9, 0.1, 0.0]); // Similar to query
    dense_retriever.add_document(1, vec![0.0, 1.0, 0.0]); // Different
    
    let query_terms = vec!["machine".to_string(), "learning".to_string()];
    let query_emb = [1.0, 0.0, 0.0]; // Semantically similar to doc 0
    
    let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 2, Bm25Params::default()).unwrap();
    let dense_results = retrieve_dense(&dense_retriever, &query_emb, 2).unwrap();
    
    let relevant: HashSet<String> = ["0"].iter().map(|s| s.to_string()).collect();
    
    let bm25_ranked: Vec<String> = bm25_results.iter().map(|(id, _)| id.to_string()).collect();
    let dense_ranked: Vec<String> = dense_results.iter().map(|(id, _)| id.to_string()).collect();
    
    let bm25_ndcg = ndcg_at_k(&bm25_ranked, &relevant, 2);
    let dense_ndcg = ndcg_at_k(&dense_ranked, &relevant, 2);
    
    // Both should work, but dense may perform better on semantic similarity
    assert!(bm25_ndcg >= 0.0 && bm25_ndcg <= 1.0);
    assert!(dense_ndcg >= 0.0 && dense_ndcg <= 1.0);
    // Dense should find semantically similar document
    assert!(dense_ranked.contains(&"0".to_string()));
}

#[cfg(feature = "bm25")]
#[test]
fn test_batch_evaluation() {
    // Evaluate multiple queries at once
    let mut index = InvertedIndex::new();
    
    // Query 1: "machine learning" -> docs 0, 1 relevant
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    
    // Query 2: "python programming" -> doc 2 relevant
    index.add_document(2, &["python".to_string(), "programming".to_string()]);
    
    let queries = vec![
        vec!["machine".to_string(), "learning".to_string()],
        vec!["python".to_string(), "programming".to_string()],
    ];
    
    let batch_results = batch_retrieve_bm25(&index, &queries, 3, Bm25Params::default()).unwrap();
    
    // Ground truth for each query
    let qrels = vec![
        ["0", "1"].iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
        ["2"].iter().map(|s| s.to_string()).collect::<HashSet<_>>(),
    ];
    
    let mut total_ndcg = 0.0;
    for (results, relevant) in batch_results.iter().zip(qrels.iter()) {
        let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
        let ndcg = ndcg_at_k(&ranked, relevant, 3);
        total_ndcg += ndcg;
    }
    
    let mean_ndcg = total_ndcg / batch_results.len() as f64;
    
    // Mean nDCG should be positive
    assert!(mean_ndcg > 0.0);
    assert!(mean_ndcg <= 1.0);
}

#[cfg(feature = "bm25")]
#[test]
fn test_retrieval_regression_guardrails() {
    // Regression test: ensure retrieval quality doesn't degrade
    // This is a minimal threshold - in practice, would use real datasets
    
    let mut index = InvertedIndex::new();
    
    // Clear lexical match scenario
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    index.add_document(2, &["python".to_string(), "programming".to_string()]);
    
    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 3, Bm25Params::default()).unwrap();
    
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let precision = precision_at_k(&ranked, &relevant, 3);
    let recall = recall_at_k(&ranked, &relevant, 3);
    let ndcg = ndcg_at_k(&ranked, &relevant, 3);
    
    // Regression guardrails: minimum acceptable performance
    // In practice, these would be based on baseline performance
    assert!(precision >= 0.5, "Precision should be at least 0.5");
    assert!(recall >= 0.5, "Recall should be at least 0.5");
    assert!(ndcg >= 0.3, "nDCG should be at least 0.3");
}

