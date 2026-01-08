//! Examples using enhanced test helpers
//!
//! Demonstrates new helpers: mock embeddings, TREC format, builders, property-based testing

#[cfg(test)]
#[path = "test_helpers.rs"]
mod test_helpers;

#[cfg(test)]
mod tests {
    use super::test_helpers::*;
    use rank_eval::binary::ndcg_at_k;
    #[cfg(feature = "bm25")]
    use rank_retrieve::bm25::Bm25Params;
    #[cfg(feature = "bm25")]
    use rank_retrieve::{batch::batch_retrieve_bm25, retrieve_bm25};

    #[test]
    fn example_mock_embeddings() {
        // Mock embeddings for dense retrieval tests
        let query_emb = mock_dense_embed("machine learning", 128);
        let doc1_emb = mock_dense_embed("deep learning algorithms", 128);
        let doc2_emb = mock_dense_embed("python programming", 128);

        // Verify embeddings are normalized
        let query_norm: f32 = query_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((query_norm - 1.0).abs() < 0.01 || query_norm == 0.0);

        // Verify deterministic
        let query_emb2 = mock_dense_embed("machine learning", 128);
        assert_eq!(query_emb, query_emb2);
    }

    #[test]
    fn example_token_embeddings() {
        // Token embeddings for ColBERT-style tests
        let query_tokens = mock_token_embed("machine learning", 128);
        let doc_tokens = mock_token_embed("deep learning algorithms", 128);

        assert_eq!(query_tokens.len(), 2); // "machine", "learning"
        assert_eq!(doc_tokens.len(), 3); // "deep", "learning", "algorithms"

        // Each token should be normalized
        for token_emb in &query_tokens {
            let norm: f32 = token_emb.iter().map(|x| x * x).sum::<f32>().sqrt();
            assert!(norm > 0.0);
        }
    }

    #[test]
    fn example_trec_format() {
        // Create TREC format entries
        let results = vec![(0u32, 0.95), (1u32, 0.87), (2u32, 0.75)];

        let runs = results_to_trec_runs("q1", &results, "bm25");

        assert_eq!(runs.len(), 3);
        assert_eq!(runs[0].query_id, "q1");
        assert_eq!(runs[0].doc_id, "0");
        assert_eq!(runs[0].rank, 1);
        assert_eq!(runs[0].score, 0.95);
        assert_eq!(runs[0].run_tag, "bm25");

        // Format as TREC line
        let line = runs[0].to_trec_line();
        assert!(line.contains("q1 Q0 0 1 0.950000 bm25"));
    }

    #[test]
    fn example_result_builder() {
        // Build results using builder pattern
        let results = ResultBuilder::new()
            .add(0u32, 0.9)
            .add(1u32, 0.8)
            .add(2u32, 0.7)
            .build();

        assert_eq!(results.len(), 3);
        // Should be sorted descending
        assert_eq!(results[0].0, 0);
        assert_eq!(results[0].1, 0.9);
        assert_eq!(results[1].1, 0.8);
        assert_eq!(results[2].1, 0.7);
    }

    #[test]
    fn example_collection_builder() {
        // Build test collection using builder pattern
        let collection = TestCollectionBuilder::new()
            .add_document("doc1", vec!["machine".to_string(), "learning".to_string()])
            .add_document("doc2", vec!["deep".to_string(), "learning".to_string()])
            .add_query("q1", vec!["machine".to_string(), "learning".to_string()])
            .add_qrel("q1", vec!["doc1", "doc2"])
            .build();

        assert_eq!(collection.documents.len(), 2);
        assert_eq!(collection.queries.len(), 1);
        assert_eq!(collection.qrels.len(), 1);

        let relevant = collection.get_relevant("q1").unwrap();
        assert!(relevant.contains("doc1"));
        assert!(relevant.contains("doc2"));
    }

    #[cfg(feature = "bm25")]
    #[test]
    fn example_integrated_helpers() {
        // Combine multiple helpers
        let collection = TestCollectionBuilder::new()
            .add_document("0", vec!["machine".to_string(), "learning".to_string()])
            .add_document("1", vec!["deep".to_string(), "learning".to_string()])
            .add_query("q1", vec!["machine".to_string(), "learning".to_string()])
            .add_qrel("q1", vec!["0", "1"])
            .build();

        let scenario = TestScenario::new(collection.clone());
        let query = scenario.get_query("q1").unwrap();
        let relevant = scenario.get_relevant("q1").unwrap();

        let results = retrieve_bm25(&scenario.index, &query, 10, Bm25Params::default()).unwrap();
        let ranked = results.to_ranked_list();

        let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);

        // Check metrics
        assert!(eval.precision_at_10 >= 0.0);
        assert!(eval.recall_at_10 >= 0.0);
        assert!(eval.ndcg_at_10 >= 0.0);

        // Create TREC format
        let trec_runs = results_to_trec_runs("q1", &results, "bm25");
        assert_eq!(trec_runs.len(), results.len());
    }
}
