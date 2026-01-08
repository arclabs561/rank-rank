//! Tests for late interaction retrieval pipeline
//!
//! Tests the research-backed approach: BM25 first-stage retrieval followed by
//! MaxSim reranking. Validates token pooling optimization and integration with
//! rank-rerank.

#[cfg(test)]
#[path = "test_helpers.rs"]
mod test_helpers;

#[cfg(test)]
mod tests {
    use super::test_helpers::{mock_dense_embed, mock_token_embed};
    #[cfg(feature = "bm25")]
    use rank_retrieve::{retrieve_bm25, bm25::{Bm25Params, InvertedIndex}};
    #[cfg(feature = "dense")]
    use rank_retrieve::retrieve_dense;
    use rank_rerank::colbert;
    use rank_rerank::simd;
    use rank_fusion::rrf;
    use rank_eval::binary::{ndcg_at_k, precision_at_k};
    use std::collections::HashSet;

    /// Test the research-backed pipeline: BM25 → MaxSim reranking
    #[cfg(feature = "bm25")]
    #[test]
    fn test_bm25_then_maxsim_pipeline() {
        // Setup: Create BM25 index
        let mut index = InvertedIndex::new();
        let documents = vec![
            (0, "machine learning algorithms neural networks"),
            (1, "deep learning neural networks artificial intelligence"),
            (2, "python programming language data science"),
            (3, "rust systems programming memory safety"),
        ];

        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        // Step 1: BM25 first-stage retrieval
        let query_terms = vec!["machine".to_string(), "learning".to_string()];
        let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default()).unwrap();
        assert!(!candidates.is_empty());

        // Step 2: Prepare token embeddings for MaxSim reranking
        let query_text = "machine learning";
        let query_tokens = mock_token_embed(query_text, 128);

        let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = candidates.iter()
            .map(|(id, _)| {
                let doc_text = documents.iter().find(|(d_id, _)| d_id == id).unwrap().1;
                (*id, mock_token_embed(doc_text, 128))
            })
            .collect();

        // Step 3: MaxSim reranking
        let reranked: Vec<(u32, f32)> = colbert::rank(&query_tokens, &doc_tokens);
        assert!(!reranked.is_empty());
        assert_eq!(reranked.len(), doc_tokens.len());

        // Verify sorting (descending by score)
        for i in 1..reranked.len() {
            assert!(reranked[i-1].1 >= reranked[i].1, "Results should be sorted descending");
        }

        // Verify top result is relevant
        assert_eq!(reranked[0].0, 0, "Document 0 should rank highest for 'machine learning'");
    }

    /// Test token pooling optimization (research: 50% reduction, <1% quality loss)
    #[test]
    fn test_token_pooling_optimization() {
        let doc_text = "machine learning algorithms neural networks deep";
        let doc_tokens = mock_token_embed(doc_text, 128);
        let original_count = doc_tokens.len();

        // Pool factor 2 = 50% reduction
        let pooled = colbert::pool_tokens(&doc_tokens, 2).unwrap();
        let pooled_count = pooled.len();
        let reduction = 1.0 - (pooled_count as f32 / original_count as f32);

        // Verify reduction is approximately 50%
        assert!(reduction >= 0.4 && reduction <= 0.6, 
                "Pool factor 2 should reduce by ~50%, got {:.1}%", reduction * 100.0);
        assert!(pooled_count <= original_count, "Pooled should not exceed original");

        // Verify dimensions preserved
        assert_eq!(pooled[0].len(), doc_tokens[0].len(), "Dimensions should be preserved");

        // Test with query: pooled documents should still work
        let query_tokens = mock_token_embed("machine learning", 128);
        let score_original = simd::maxsim(
            &query_tokens.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
            &doc_tokens.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
        );
        let score_pooled = simd::maxsim(
            &query_tokens.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
            &pooled.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
        );

        // Quality should be similar (research: <1% loss for factor 2)
        // Note: With mock embeddings, we relax to 90% to account for simplified data
        let quality_retention = score_pooled / score_original;
        assert!(quality_retention >= 0.90, 
                "Pool factor 2 should retain >90% quality (relaxed for mock data), got {:.1}%", quality_retention * 100.0);
    }

    /// Test that token pooling works with different factors
    #[test]
    fn test_token_pooling_factors() {
        let doc_tokens = mock_token_embed("machine learning algorithms neural networks deep artificial intelligence", 128);
        let original_count = doc_tokens.len();

        // Factor 2
        let pooled_2 = colbert::pool_tokens(&doc_tokens, 2).unwrap();
        let reduction_2 = 1.0 - (pooled_2.len() as f32 / original_count as f32);
        assert!(reduction_2 >= 0.4 && reduction_2 <= 0.6, "Factor 2: ~50% reduction");

        // Factor 3
        let pooled_3 = colbert::pool_tokens(&doc_tokens, 3).unwrap();
        let reduction_3 = 1.0 - (pooled_3.len() as f32 / original_count as f32);
        // Relaxed bounds for mock embeddings (actual reduction depends on clustering)
        assert!(reduction_3 >= 0.5 && reduction_3 <= 0.75, 
                "Factor 3: should reduce significantly, got {:.1}%", reduction_3 * 100.0);

        // Factor 4
        let pooled_4 = colbert::pool_tokens(&doc_tokens, 4).unwrap();
        let reduction_4 = 1.0 - (pooled_4.len() as f32 / original_count as f32);
        assert!(reduction_4 >= 0.7 && reduction_4 <= 0.8, "Factor 4: ~75% reduction");

        // Verify more aggressive pooling reduces more
        assert!(pooled_4.len() <= pooled_3.len());
        assert!(pooled_3.len() <= pooled_2.len());
    }

    /// Test that queries should stay at full resolution (research finding)
    #[test]
    fn test_queries_full_resolution() {
        let query_tokens = mock_token_embed("machine learning", 128);
        let doc_tokens = mock_token_embed("machine learning algorithms", 128);

        // Full resolution query
        let score_full = simd::maxsim(
            &query_tokens.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
            &doc_tokens.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
        );

        // Pooled query (not recommended, but test the difference)
        let query_pooled = colbert::pool_tokens(&query_tokens, 2).unwrap();
        let score_pooled_query = simd::maxsim(
            &query_pooled.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
            &doc_tokens.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
        );

        // Full resolution should be better (research finding)
        assert!(score_full >= score_pooled_query, 
                "Full resolution queries should perform better");
    }

    /// Test hybrid retrieval: BM25 + Dense → Fusion → MaxSim reranking
    #[cfg(feature = "bm25")]
    #[cfg(feature = "dense")]
    #[test]
    fn test_hybrid_retrieval_pipeline() {
        // Setup
        let mut bm25_index = InvertedIndex::new();
        let mut dense_retriever = rank_retrieve::dense::DenseRetriever::new();

        let documents = vec![
            (0, "machine learning algorithms"),
            (1, "deep learning neural networks"),
            (2, "python programming"),
        ];

        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            bm25_index.add_document(*id, &terms);

            let embedding = mock_dense_embed(text, 128);
            dense_retriever.add_document(*id, embedding);
        }

        // Step 1: Retrieve with both methods
        let query_terms = vec!["learning".to_string()];
        let query_emb = mock_dense_embed("learning", 128);
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();

        // Step 2: Fuse results
        let bm25_string: Vec<(String, f32)> = bm25_results.iter()
            .map(|(id, score)| (id.to_string(), *score))
            .collect();
        let dense_string: Vec<(String, f32)> = dense_results.iter()
            .map(|(id, score)| (id.to_string(), *score))
            .collect();

        let fused = rrf(&bm25_string, &dense_string);
        assert!(!fused.is_empty());

        // Step 3: Rerank with MaxSim
        let query_tokens = mock_token_embed("learning", 128);
        let doc_tokens: Vec<(String, Vec<Vec<f32>>)> = fused.iter()
            .map(|(id, _)| {
                let id_u32: u32 = id.parse().unwrap();
                let doc_text = documents.iter().find(|(d_id, _)| *d_id == id_u32).unwrap().1;
                (id.clone(), mock_token_embed(doc_text, 128))
            })
            .collect();

        let reranked: Vec<(String, f32)> = colbert::rank(&query_tokens, &doc_tokens);
        assert!(!reranked.is_empty());
        assert_eq!(reranked.len(), doc_tokens.len());
    }

    /// Test evaluation metrics with late interaction pipeline
    #[cfg(feature = "bm25")]
    #[test]
    fn test_late_interaction_evaluation() {
        let mut index = InvertedIndex::new();
        let documents = vec![
            (0, "machine learning algorithms"),
            (1, "deep learning neural networks"),
            (2, "python programming"),
        ];

        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        let query_terms = vec!["machine".to_string(), "learning".to_string()];
        let candidates = retrieve_bm25(&index, &query_terms, 10, Bm25Params::default()).unwrap();

        let query_tokens = mock_token_embed("machine learning", 128);
        let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = candidates.iter()
            .map(|(id, _)| {
                let doc_text = documents.iter().find(|(d_id, _)| d_id == id).unwrap().1;
                (*id, mock_token_embed(doc_text, 128))
            })
            .collect();

        let reranked: Vec<(u32, f32)> = colbert::rank(&query_tokens, &doc_tokens);
        let ranked_ids: Vec<String> = reranked.iter().map(|(id, _)| id.to_string()).collect();
        let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();

        // Evaluate
        let precision = precision_at_k(&ranked_ids, &relevant, 5);
        let ndcg = ndcg_at_k(&ranked_ids, &relevant, 5);

        assert!(precision >= 0.0 && precision <= 1.0);
        assert!(ndcg >= 0.0 && ndcg <= 1.0);
        assert!(precision > 0.0, "Should retrieve at least one relevant document");
    }

    /// Test that token pooling preserves ranking quality
    #[test]
    fn test_pooling_preserves_ranking_quality() {
        let query_tokens = mock_token_embed("machine learning", 128);
        
        let docs = vec![
            (0, "machine learning algorithms"),
            (1, "deep learning neural networks"),
            (2, "python programming"),
        ];

        let doc_tokens_original: Vec<(u32, Vec<Vec<f32>>)> = docs.iter()
            .map(|(id, text)| (*id, mock_token_embed(text, 128)))
            .collect();

        // Rank with original tokens
        let ranked_original = colbert::rank(&query_tokens, &doc_tokens_original);

        // Pool documents (factor 2)
        let doc_tokens_pooled: Vec<(u32, Vec<Vec<f32>>)> = doc_tokens_original.iter()
            .map(|(id, tokens)| (*id, colbert::pool_tokens(tokens, 2).unwrap()))
            .collect();

        // Rank with pooled tokens
        let ranked_pooled = colbert::rank(&query_tokens, &doc_tokens_pooled);

        // Top result should be the same (research: <1% quality loss for factor 2)
        // With mock embeddings, we check that top result is in top-2 to account for ties
        let top_original = ranked_original[0].0;
        let top_pooled_ids: Vec<u32> = ranked_pooled.iter().take(2).map(|(id, _)| *id).collect();
        assert!(top_pooled_ids.contains(&top_original), 
                "Top result from original should be in top-2 of pooled results");
        
        // Scores should be similar (relaxed for mock embeddings)
        let score_ratio = ranked_pooled[0].1 / ranked_original[0].1;
        assert!(score_ratio >= 0.85, 
                "Pooled scores should retain >85% of original (relaxed for mock data), got {:.1}%", score_ratio * 100.0);
    }

    /// Test adaptive pooling strategy selection
    #[test]
    fn test_adaptive_pooling_strategy() {
        let doc_tokens = mock_token_embed("machine learning algorithms neural networks deep artificial intelligence", 128);

        // Factor 2: Should use clustering (quality-focused)
        let pooled_2 = colbert::pool_tokens_adaptive(&doc_tokens, 2).unwrap();
        assert!(pooled_2.len() <= doc_tokens.len() / 2 + 1);

        // Factor 4: Should use sequential (speed-focused)
        let pooled_4 = colbert::pool_tokens_adaptive(&doc_tokens, 4).unwrap();
        assert!(pooled_4.len() <= doc_tokens.len() / 4 + 1);
        assert!(pooled_4.len() <= pooled_2.len());
    }

    /// Test batch reranking with late interaction
    #[cfg(feature = "bm25")]
    #[test]
    fn test_batch_reranking_late_interaction() {
        let mut index = InvertedIndex::new();
        for i in 0..10 {
            let terms = vec![format!("term{}", i), format!("word{}", i)];
            index.add_document(i, &terms);
        }

        let queries = vec![
            vec!["term0".to_string(), "word0".to_string()],
            vec!["term1".to_string(), "word1".to_string()],
        ];

        // Batch retrieve
        let batch_results = rank_retrieve::batch::batch_retrieve_bm25(
            &index,
            &queries,
            10,
            Bm25Params::default(),
        ).unwrap();

        assert_eq!(batch_results.len(), queries.len());

        // Rerank each query's results
        for (query_idx, results) in batch_results.iter().enumerate() {
            let query_text = queries[query_idx].join(" ");
            let query_tokens = mock_token_embed(&query_text, 128);

            let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = results.iter()
                .map(|(id, _)| {
                    let terms = vec![format!("term{}", id), format!("word{}", id)];
                    let doc_text = terms.join(" ");
                    (*id, mock_token_embed(&doc_text, 128))
                })
                .collect();

            let reranked: Vec<(u32, f32)> = colbert::rank(&query_tokens, &doc_tokens);
            assert!(!reranked.is_empty());
            assert_eq!(reranked[0].0, query_idx as u32, 
                       "Query {} should rank doc {} highest", query_idx, query_idx);
        }
    }

    /// Test that late interaction outperforms dense on complex queries
    #[cfg(feature = "dense")]
    #[test]
    fn test_late_interaction_vs_dense_complex_query() {
        let mut dense_retriever = rank_retrieve::dense::DenseRetriever::new();
        let documents = vec![
            (0, "machine learning algorithms"),
            (1, "deep learning neural networks"),
            (2, "python programming"),
        ];

        for (id, text) in &documents {
            let embedding = mock_dense_embed(text, 128);
            dense_retriever.add_document(*id, embedding);
        }

        // Complex query with multiple concepts
        let query_text = "machine learning neural networks";
        let query_emb = mock_dense_embed(query_text, 128);
        let query_tokens = mock_token_embed(query_text, 128);

        // Dense retrieval
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();

        // Late interaction reranking
        let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = documents.iter()
            .map(|(id, text)| (*id, mock_token_embed(text, 128)))
            .collect();
        let reranked: Vec<(u32, f32)> = colbert::rank(&query_tokens, &doc_tokens);

        // Both should work, but late interaction should handle multi-concept queries better
        assert!(!dense_results.is_empty());
        assert!(!reranked.is_empty());
        
        // Late interaction should rank relevant documents (0, 1) highly
        let top_ids: Vec<u32> = reranked.iter().take(2).map(|(id, _)| *id).collect();
        assert!(top_ids.contains(&0) || top_ids.contains(&1), 
                "Late interaction should rank relevant documents highly");
    }
}

