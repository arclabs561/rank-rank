//! End-to-end tests: rank-retrieve → rank-fusion → rank-eval
//!
//! Tests integration with rank-fusion and rank-eval using actual crates.
//! For full pipeline tests with all crates, see `e2e_full_pipeline.rs`.

#[cfg(test)]
mod tests {
    use rank_eval::binary::ndcg_at_k;
    use rank_fusion::rrf;
    use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
    use rank_retrieve::dense::DenseRetriever;
    use rank_retrieve::{retrieve_bm25, retrieve_dense};
    use std::collections::HashSet;

    #[test]
    fn test_complete_pipeline_real() {
        // Real integration test using actual crates:
        // 1. Retrieve (rank-retrieve)
        // 2. Fusion (rank-fusion) - REAL
        // 3. Eval (rank-eval) - REAL

        // Step 1: Retrieve candidates using multiple methods
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(
            0,
            &["machine", "learning", "tutorial"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
        bm25_index.add_document(
            1,
            &["deep", "learning", "neural"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );
        bm25_index.add_document(
            2,
            &["python", "programming"]
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>(),
        );

        let query_terms = vec!["machine".to_string(), "learning".to_string()];
        let bm25_results =
            retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();

        assert!(!bm25_results.is_empty());

        // Dense retrieval
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);

        let query_emb = [1.0, 0.0, 0.0];
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();

        assert!(!dense_results.is_empty());

        // Step 2: Fusion using real rank-fusion crate
        // u32 works directly with rank-fusion (no conversion needed)
        let fused = rrf(&bm25_results, &dense_results);
        assert!(!fused.is_empty());

        // Step 3: Evaluate using real rank-eval crate
        let ranked: Vec<String> = fused.iter().map(|(id, _)| id.to_string()).collect();
        let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();

        let ndcg = ndcg_at_k(&ranked, &relevant, 10);
        assert!(ndcg >= 0.0 && ndcg <= 1.0, "nDCG should be in [0, 1]");

        // Verify pipeline produces valid outputs
        assert!(fused.iter().all(|(_, score)| score.is_finite()));
        assert!(!ranked.is_empty());
    }

    #[test]
    fn test_pipeline_error_propagation() {
        // Test that errors propagate correctly through the pipeline

        // Empty index should fail at retrieval stage
        let empty_index = InvertedIndex::new();
        let query = vec!["test".to_string()];
        let result = retrieve_bm25(&empty_index, &query, 10, Bm25Params::default());
        assert!(result.is_err());

        // Empty query should fail at retrieval stage
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string()]);
        let result = retrieve_bm25(&index, &[], 10, Bm25Params::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_pipeline_data_flow() {
        // Test that data flows correctly through pipeline stages

        // Stage 1: Retrieve
        let mut index = InvertedIndex::new();
        for i in 0..10 {
            let terms: Vec<String> = (0..5).map(|j| format!("term{}", j)).collect();
            index.add_document(i, &terms);
        }

        let query = vec!["term0".to_string(), "term1".to_string()];
        let retrieved = retrieve_bm25(&index, &query, 5, Bm25Params::default()).unwrap();

        // Verify retrieval output format
        assert!(!retrieved.is_empty());
        assert!(retrieved.len() <= 5);
        assert!(retrieved
            .iter()
            .all(|(doc_id, score)| { *doc_id < 10 && score.is_finite() && *score >= 0.0 }));

        // Verify sorted descending
        for i in 1..retrieved.len() {
            assert!(retrieved[i - 1].1 >= retrieved[i].1);
        }
    }
}
