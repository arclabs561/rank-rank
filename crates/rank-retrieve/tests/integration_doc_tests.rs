//! Integration tests that validate documentation examples work end-to-end
//!
//! These tests ensure that code examples in markdown documentation files
//! are actually runnable and produce expected results.

#[cfg(test)]
mod tests {
    #[cfg(feature = "bm25")]
    use rank_retrieve::{retrieve_bm25, bm25::{Bm25Params, InvertedIndex}};
    #[cfg(feature = "dense")]
    use rank_retrieve::retrieve_dense;
    use rank_rerank::colbert;
    use rank_fusion::rrf;
    use rank_eval::binary::ndcg_at_k;
    use std::collections::HashSet;
    use crate::test_helpers::{mock_dense_embed, mock_token_embed};

    /// Validates the complete pipeline example from LATE_INTERACTION_GUIDE.md
    #[cfg(feature = "bm25")]
    #[test]
    fn test_late_interaction_guide_complete_pipeline() {
        // This validates the "Standard Approach" example from the guide
        
        // Setup: Create BM25 index
        let mut index = InvertedIndex::new();
        let documents = vec![
            (0, "machine learning algorithms neural networks"),
            (1, "deep learning neural networks artificial intelligence"),
            (2, "python programming language data science"),
        ];

        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        // Step 1: First-stage retrieval: BM25 (rank-retrieve)
        let query_terms = vec!["machine".to_string(), "learning".to_string()];
        let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default()).unwrap();
        assert!(!candidates.is_empty(), "BM25 should retrieve candidates");

        // Step 2: Rerank with MaxSim (rank-rerank)
        let query_text = "machine learning";
        let query_tokens = mock_token_embed(query_text, 128);
        
        let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = candidates.iter()
            .map(|(id, _)| {
                let doc_text = documents.iter().find(|(d_id, _)| d_id == id).unwrap().1;
                (*id, mock_token_embed(doc_text, 128))
            })
            .collect();

        let reranked = colbert::rank(&query_tokens, &doc_tokens);
        assert!(!reranked.is_empty(), "MaxSim should rerank candidates");
        assert_eq!(reranked.len(), doc_tokens.len());

        // Step 3: Optional: Apply token pooling for storage optimization
        // Pool documents at index time (50% reduction, <1% quality loss)
        let pooled_docs: Vec<_> = doc_tokens.iter()
            .map(|(id, tokens)| (*id, colbert::pool_tokens(tokens, 2).unwrap()))
            .collect();
        
        // Verify pooling reduced storage
        assert!(pooled_docs.iter().all(|(_, tokens)| tokens.len() <= doc_tokens[0].1.len()));
    }

    /// Validates the hybrid retrieval example from LATE_INTERACTION_GUIDE.md
    #[cfg(feature = "bm25")]
    #[cfg(feature = "dense")]
    #[test]
    fn test_late_interaction_guide_hybrid_retrieval() {
        // Validates the "Complete Pipeline" example
        
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

        // Step 1: Retrieve (rank-retrieve)
        let query_terms = vec!["learning".to_string()];
        let query_emb = mock_dense_embed("learning", 128);
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 1000, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 1000).unwrap();

        // Step 2: Rerank (rank-rerank)
        let query_tokens = mock_token_embed("learning", 128);
        let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = bm25_results.iter()
            .map(|(id, _)| {
                let doc_text = documents.iter().find(|(d_id, _)| d_id == id).unwrap().1;
                (*id, mock_token_embed(doc_text, 128))
            })
            .collect();

        let reranked = colbert::rank(&query_tokens, &doc_tokens);
        assert!(!reranked.is_empty());

        // Step 3: Optional: Fuse multiple retrievers (rank-fusion)
        let bm25_string: Vec<(String, f32)> = bm25_results.iter()
            .map(|(id, score)| (id.to_string(), *score))
            .collect();
        let dense_string: Vec<(String, f32)> = dense_results.iter()
            .map(|(id, score)| (id.to_string(), *score))
            .collect();

        let fused = rrf(&bm25_string, &dense_string);
        assert!(!fused.is_empty());

        // Step 4: Evaluate (rank-eval)
        let ranked_ids: Vec<String> = reranked.iter().map(|(id, _)| id.to_string()).collect();
        let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
        let ndcg = ndcg_at_k(&ranked_ids, &relevant, 10);
        
        assert!(ndcg >= 0.0 && ndcg <= 1.0);
    }

    /// Validates token pooling example from colbert.rs documentation
    #[test]
    fn test_colbert_docs_pooling_example() {
        // Validates the example in colbert::pool_tokens documentation
        
        let doc_tokens = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.9, 0.1, 0.0],  // similar to first
            vec![0.0, 1.0, 0.0],
            vec![0.0, 0.9, 0.1],  // similar to third
        ];

        // Pool factor 2 = 50% reduction, ~0% quality loss (research-backed)
        let pooled = colbert::pool_tokens(&doc_tokens, 2).unwrap();
        assert_eq!(pooled.len(), 2);  // 4 tokens → 2 pooled tokens
        assert_eq!(pooled[0].len(), doc_tokens[0].len());  // Dimensions preserved
    }

    /// Validates the decision tree from PLAID_AND_OPTIMIZATION.md
    #[cfg(feature = "bm25")]
    #[test]
    fn test_plaid_optimization_decision_tree() {
        // Validates the decision framework documented in PLAID_AND_OPTIMIZATION.md
        
        // "Do you need high recall beyond BM25? No → Use rank-retrieve (BM25) → rank-rerank (MaxSim)"
        let mut index = InvertedIndex::new();
        for i in 0..10 {
            let terms = vec![format!("term{}", i), format!("word{}", i)];
            index.add_document(i, &terms);
        }
        
        let query_terms = vec!["term0".to_string()];
        let candidates = retrieve_bm25(&index, &query_terms, 100, Bm25Params::default()).unwrap();
        
        // BM25 provides good recall for standard queries
        assert!(!candidates.is_empty());
        assert!(candidates.iter().any(|(id, _)| *id == 0));
        
        // This validates the "standard approach" path in the decision tree
    }

    /// Validates research claim: "Pool documents at index time, keep queries at full resolution"
    #[test]
    fn test_pooling_best_practice() {
        // Validates the research-backed practice from documentation
        
        let doc_tokens = mock_token_embed("machine learning algorithms neural networks", 128);
        let query_tokens = mock_token_embed("machine learning", 128);
        
        // Pool documents (index time) - recommended
        let pooled_doc = colbert::pool_tokens(&doc_tokens, 2).unwrap();
        assert!(pooled_doc.len() < doc_tokens.len());
        
        // Keep queries at full resolution (query time) - recommended
        let score_with_full_query = rank_rerank::simd::maxsim(
            &query_tokens.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
            &pooled_doc.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
        );
        
        // Pool query (not recommended, but test the difference)
        let pooled_query = colbert::pool_tokens(&query_tokens, 2).unwrap();
        let score_with_pooled_query = rank_rerank::simd::maxsim(
            &pooled_query.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
            &pooled_doc.iter().map(|v| v.as_slice()).collect::<Vec<_>>(),
        );
        
        // Full resolution query should perform better (research finding)
        assert!(score_with_full_query >= score_with_pooled_query,
                "Full resolution queries should perform better");
    }

    /// Validates the research-backed pool factor guide from documentation
    #[test]
    fn test_pool_factor_guide() {
        // Validates the research-backed recommendations from colbert.rs docs
        
        let tokens = mock_token_embed("machine learning algorithms neural networks deep artificial intelligence", 128);
        let original_count = tokens.len();
        
        // Factor 2: Default choice (50% reduction, ~0% loss)
        let pooled_2 = colbert::pool_tokens(&tokens, 2).unwrap();
        let reduction_2 = 1.0 - (pooled_2.len() as f32 / original_count as f32);
        assert!(reduction_2 >= 0.4 && reduction_2 <= 0.6, "Factor 2: ~50% reduction");
        
        // Factor 3: Good tradeoff (66% reduction, ~1% loss)
        let pooled_3 = colbert::pool_tokens(&tokens, 3).unwrap();
        let reduction_3 = 1.0 - (pooled_3.len() as f32 / original_count as f32);
        assert!(reduction_3 >= 0.5 && reduction_3 <= 0.75, "Factor 3: ~66% reduction");
        
        // Factor 4+: More aggressive (75%+ reduction, 3-5% loss)
        let pooled_4 = colbert::pool_tokens(&tokens, 4).unwrap();
        let reduction_4 = 1.0 - (pooled_4.len() as f32 / original_count as f32);
        assert!(reduction_4 >= 0.7 && reduction_4 <= 0.85, "Factor 4: ~75%+ reduction");
    }
}

#[cfg(test)]
#[path = "test_helpers.rs"]
mod test_helpers;

