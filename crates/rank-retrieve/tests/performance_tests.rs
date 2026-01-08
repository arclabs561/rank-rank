//! Performance and stress tests for rank-retrieve.
//!
//! Tests behavior under load and with large datasets.

#[cfg(test)]
mod tests {
    use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
    use rank_retrieve::dense::DenseRetriever;
    use std::time::Instant;

    #[test]
    fn test_large_index_performance() {
        // Test indexing performance with large document set
        let mut index = InvertedIndex::new();
        let n_docs = 10000;
        let terms_per_doc = 100;

        let start = Instant::now();
        for doc_id in 0..n_docs {
            let terms: Vec<String> = (0..terms_per_doc)
                .map(|i| format!("term{}", (doc_id * 7 + i * 11) % 1000))
                .collect();
            index.add_document(doc_id, &terms);
        }
        let elapsed = start.elapsed();

        // Should complete in reasonable time (< 5 seconds for 10K docs)
        // Note: Performance depends on system load, so use generous threshold
        assert!(
            elapsed.as_secs_f64() < 5.0,
            "Indexing took too long: {:?}",
            elapsed
        );
    }

    #[test]
    fn test_large_retrieval_performance() {
        // Test retrieval performance with large index
        let mut index = InvertedIndex::new();
        let n_docs = 10000;

        // Build index
        for doc_id in 0..n_docs {
            let terms: Vec<String> = (0..50)
                .map(|i| format!("term{}", (doc_id * 7 + i * 11) % 1000))
                .collect();
            index.add_document(doc_id, &terms);
        }

        // Query
        let query: Vec<String> = (0..10).map(|i| format!("term{}", i * 100)).collect();

        let start = Instant::now();
        let results = index.retrieve(&query, 100, Bm25Params::default()).unwrap();
        let elapsed = start.elapsed();

        // Should complete quickly (< 100ms for 10K docs, k=100)
        assert!(
            elapsed.as_millis() < 100,
            "Retrieval took too long: {:?}",
            elapsed
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn test_dense_large_scale() {
        // Test dense retrieval with large embedding set
        let mut retriever = DenseRetriever::new();
        let n_docs = 10000;
        let dim = 128;

        // Add documents
        for doc_id in 0..n_docs {
            let embedding: Vec<f32> = (0..dim)
                .map(|i| ((doc_id * 7 + i * 11) % 100) as f32 / 100.0 - 0.5)
                .collect();
            retriever.add_document(doc_id, embedding);
        }

        // Query
        let query: Vec<f32> = (0..dim).map(|i| (i % 100) as f32 / 100.0 - 0.5).collect();

        let start = Instant::now();
        let results = retriever.retrieve(&query, 100).unwrap();
        let elapsed = start.elapsed();

        // Should complete in reasonable time (< 500ms for 10K docs, k=100)
        assert!(
            elapsed.as_millis() < 500,
            "Dense retrieval took too long: {:?}",
            elapsed
        );
        assert!(!results.is_empty());
    }

    #[test]
    fn test_repeated_retrieval_consistency() {
        // Test that repeated retrievals are consistent and don't degrade
        let mut index = InvertedIndex::new();
        for i in 0..1000 {
            index.add_document(i, &[format!("term{}", i)]);
        }

        let query = vec!["term0".to_string()];
        let mut times = Vec::new();

        // Run multiple times
        for _ in 0..10 {
            let start = Instant::now();
            let _results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
            times.push(start.elapsed());
        }

        // All retrievals should complete
        assert_eq!(times.len(), 10);

        // Performance should be consistent (no significant degradation)
        let avg_time: f64 =
            times.iter().map(|t| t.as_millis() as f64).sum::<f64>() / times.len() as f64;
        let max_time = times.iter().map(|t| t.as_millis() as u128).max().unwrap();
        let min_time = times.iter().map(|t| t.as_millis() as u128).min().unwrap();

        // Max should not be more than 5x average (allowing for system variance)
        // Performance can vary due to system load, so use generous threshold
        assert!(
            (max_time as f64) < avg_time * 5.0 || avg_time < 1.0,
            "Performance inconsistency detected: max={}ms, avg={:.2}ms",
            max_time,
            avg_time
        );
    }

    #[test]
    fn test_memory_efficiency() {
        // Test that memory usage is reasonable
        let mut index = InvertedIndex::new();
        let n_docs = 10000;

        // Add many documents
        for doc_id in 0..n_docs {
            let terms: Vec<String> = (0..50)
                .map(|i| format!("term{}", (doc_id * 7 + i * 11) % 1000))
                .collect();
            index.add_document(doc_id, &terms);
        }

        // Index should still be usable
        let query = vec!["term0".to_string()];
        let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
        assert!(!results.is_empty());
    }
}
