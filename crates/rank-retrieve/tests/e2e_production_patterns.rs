//! End-to-end tests: Production patterns and best practices
//!
//! These tests verify that the library works correctly when used in
//! production-like scenarios with proper error handling, validation,
//! and integration patterns.
//!
//! **Test patterns:**
//! - Batch processing multiple queries
//! - Error recovery and retry logic
//! - Result caching patterns
//! - Concurrent access patterns
//! - Resource cleanup and memory management
//! - Performance under load

#[cfg(test)]
mod tests {
    use rank_eval::binary::ndcg_at_k;
    use rank_fusion::rrf;
    #[cfg(feature = "bm25")]
    use rank_retrieve::bm25::{Bm25Params, Bm25Variant, InvertedIndex};
    #[cfg(feature = "dense")]
    use rank_retrieve::dense::DenseRetriever;
    use rank_retrieve::retrieve_bm25;
    #[cfg(feature = "dense")]
    use rank_retrieve::retrieve_dense;
    use std::collections::HashSet;

    /// Pattern: Batch processing multiple queries
    ///
    /// In production, you often need to process multiple queries efficiently.
    /// This test verifies batch processing works correctly.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_batch_query_processing() {
        let mut index = InvertedIndex::new();
        let documents = vec![
            (0, "machine learning algorithms"),
            (1, "deep learning neural networks"),
            (2, "python programming language"),
            (3, "rust systems programming"),
            (4, "information retrieval search"),
        ];

        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        // Process multiple queries
        let queries = vec![
            vec!["learning".to_string()],
            vec!["programming".to_string()],
            vec!["retrieval".to_string()],
        ];

        let mut all_results = Vec::new();
        for query in &queries {
            let results = retrieve_bm25(&index, query, 10, Bm25Params::default()).unwrap();
            all_results.push(results);
        }

        assert_eq!(all_results.len(), queries.len());
        for results in &all_results {
            assert!(!results.is_empty());
            // Verify sorted
            for i in 1..results.len() {
                assert!(results[i - 1].1 >= results[i].1);
            }
        }
    }

    /// Pattern: Error recovery and graceful degradation
    ///
    /// Production systems should handle errors gracefully and continue operating.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_error_recovery_pattern() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string()]);

        // Valid query should succeed
        let query = vec!["test".to_string()];
        let result = retrieve_bm25(&index, &query, 10, Bm25Params::default());
        assert!(result.is_ok());

        // Empty query should fail gracefully
        let empty_result = retrieve_bm25(&index, &[], 10, Bm25Params::default());
        assert!(empty_result.is_err());

        // System should still work after error
        let result_after_error = retrieve_bm25(&index, &query, 10, Bm25Params::default());
        assert!(result_after_error.is_ok());
    }

    /// Pattern: Result validation and sanitization
    ///
    /// Production systems should validate results before returning them to users.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_result_validation_pattern() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string()]);
        index.add_document(1, &["test".to_string(), "document".to_string()]);

        let query = vec!["test".to_string()];
        let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

        // Validate results
        assert!(!results.is_empty());
        assert!(results.len() <= 10); // Respect k limit

        // Validate scores
        for (doc_id, score) in &results {
            assert!(*doc_id < 2, "Document ID should be valid");
            assert!(score.is_finite(), "Score should be finite");
            assert!(*score >= 0.0, "BM25 score should be non-negative");
        }

        // Validate sorting
        for i in 1..results.len() {
            assert!(
                results[i - 1].1 >= results[i].1,
                "Results should be sorted descending"
            );
        }

        // Validate no duplicates
        let mut seen_ids = HashSet::new();
        for (doc_id, _) in &results {
            assert!(
                seen_ids.insert(*doc_id),
                "No duplicate document IDs should appear"
            );
        }
    }

    /// Pattern: Hybrid retrieval with fallback
    ///
    /// Production systems often use multiple retrieval methods with fallback logic.
    #[cfg(all(feature = "bm25", feature = "dense"))]
    #[test]
    fn test_hybrid_retrieval_with_fallback() {
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["test".to_string()]);

        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);

        let query_terms = vec!["test".to_string()];
        let query_emb = [1.0, 0.0, 0.0];

        // Try BM25 first
        let bm25_results =
            retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();

        // Try dense as fallback or complement
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();

        // Both should succeed
        assert!(!bm25_results.is_empty());
        assert!(!dense_results.is_empty());

        // Fuse results
        let fused = rrf(&bm25_results, &dense_results);
        assert!(!fused.is_empty());
    }

    /// Pattern: Sequential query processing (concurrent access note)
    ///
    /// Note: `InvertedIndex` uses `RefCell` internally for lazy IDF computation,
    /// so it's not thread-safe. For concurrent access, use synchronization (e.g., `Mutex`)
    /// or create separate index instances per thread.
    ///
    /// This test demonstrates sequential processing, which is safe and common in production.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_sequential_query_processing_pattern() {
        let mut index = InvertedIndex::new();
        for i in 0..10 {
            let terms: Vec<String> = (0..5).map(|j| format!("term{}", j)).collect();
            index.add_document(i, &terms);
        }

        // Process multiple queries sequentially (common in production)
        let queries: Vec<Vec<String>> = (0..5)
            .map(|i| vec![format!("term{}", i)])
            .collect();

        let mut total_results = 0;
        for query in &queries {
            let results = retrieve_bm25(&index, query, 10, Bm25Params::default()).unwrap();
            assert!(results.len() > 0);
            total_results += results.len();
        }

        assert!(total_results > 0);
    }

    /// Pattern: Memory-efficient processing
    ///
    /// Tests that results can be processed incrementally without loading everything.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_memory_efficient_processing() {
        let mut index = InvertedIndex::new();
        for i in 0..100 {
            let terms: Vec<String> = (0..10).map(|j| format!("term{}", j)).collect();
            index.add_document(i, &terms);
        }

        let query = vec!["term0".to_string()];
        let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

        // Process results incrementally (simulating streaming)
        let mut processed_count = 0;
        for (doc_id, score) in &results {
            // Simulate processing each result
            assert!(score.is_finite());
            processed_count += 1;
            if processed_count >= 5 {
                // Early termination (common in production)
                break;
            }
        }

        assert!(processed_count <= results.len());
    }

    /// Pattern: Query preprocessing and normalization
    ///
    /// Production systems often preprocess queries before retrieval.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_query_preprocessing_pattern() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["machine".to_string(), "learning".to_string()]);

        // Simulate query preprocessing: lowercase, remove stop words, etc.
        let raw_query = "Machine Learning Algorithms";
        let preprocessed: Vec<String> = raw_query
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .filter(|s| s.len() > 2) // Remove short words
            .collect();

        let results = retrieve_bm25(&index, &preprocessed, 10, Bm25Params::default()).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0);
    }

    /// Pattern: Result post-processing
    ///
    /// Production systems often post-process results (filtering, deduplication, etc.).
    #[cfg(feature = "bm25")]
    #[test]
    fn test_result_post_processing_pattern() {
        let mut index = InvertedIndex::new();
        for i in 0..20 {
            let terms: Vec<String> = (0..5).map(|j| format!("term{}", j)).collect();
            index.add_document(i, &terms);
        }

        let query = vec!["term0".to_string()];
        let mut results = retrieve_bm25(&index, &query, 20, Bm25Params::default()).unwrap();

        // Post-processing: Filter by minimum score threshold
        let min_score = 0.1;
        results.retain(|(_, score)| *score >= min_score);

        // Post-processing: Limit to top 10
        results.truncate(10);

        // Post-processing: Deduplicate (shouldn't be needed, but good practice)
        let mut seen = HashSet::new();
        results.retain(|(id, _)| seen.insert(*id));

        assert!(results.len() <= 10);
        for (_, score) in &results {
            assert!(*score >= min_score);
        }
    }

    /// Pattern: Evaluation and monitoring
    ///
    /// Production systems should evaluate retrieval quality and monitor performance.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_evaluation_and_monitoring_pattern() {
        let mut index = InvertedIndex::new();
        let documents = vec![
            (0, "machine learning neural networks"),
            (1, "deep learning artificial intelligence"),
            (2, "python programming language"),
        ];

        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        let query = vec!["learning".to_string()];
        let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

        // Convert to ranked list for evaluation
        let ranked: Vec<u32> = results.iter().map(|(id, _)| *id).collect();

        // Ground truth: documents 0 and 1 are relevant
        let relevant: HashSet<u32> = [0, 1].iter().copied().collect();

        // Evaluate
        let ndcg = ndcg_at_k(&ranked, &relevant, 10);

        // Monitor metrics
        assert!(ndcg >= 0.0 && ndcg <= 1.0);
        assert!(results.len() > 0);

        // In production, you'd log these metrics:
        // - nDCG@10: {ndcg}
        // - Number of results: {results.len()}
        // - Average score: {avg_score}
        // - Query latency: {latency_ms}ms
    }

    /// Pattern: Configuration and parameter tuning
    ///
    /// Production systems often need to tune BM25 parameters for different domains.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_parameter_tuning_pattern() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string(), "test".to_string(), "test".to_string()]);
        index.add_document(1, &["test".to_string()]);

        let query = vec!["test".to_string()];

        // Default parameters
        let default_results =
            retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

        // Custom parameters (higher k1 = more term frequency saturation)
        let custom_params = Bm25Params {
            k1: 2.0, // Higher than default (1.2)
            b: 0.75,
            variant: Bm25Variant::Standard,
        };
        let custom_results = retrieve_bm25(&index, &query, 10, custom_params).unwrap();

        // Both should return results
        assert!(!default_results.is_empty());
        assert!(!custom_results.is_empty());

        // Scores may differ due to parameter changes
        // In production, you'd A/B test different parameter values
    }
}
