//! End-to-end tests: Full pipeline with all rank-* crates
//!
//! Tests complete pipeline: retrieve → fuse → rerank → eval
//! using actual rank-fusion, rank-rerank, and rank-eval crates.
//!
//! **Test Organization:**
//! - This file: General pipeline tests with dense embeddings
//! - `late_interaction_tests.rs`: Specialized tests for ColBERT/MaxSim token-level matching
//!   (tests the research-backed BM25 → MaxSim pipeline)

#[cfg(test)]
mod tests {
    use rank_retrieve::{retrieve_bm25, retrieve_dense, retrieve_sparse};
    use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
    use rank_retrieve::dense::DenseRetriever;
    use rank_retrieve::sparse::{SparseRetriever, SparseVector};
    use rank_retrieve::batch;
    use rank_fusion::{rrf, rrf_multi, combsum, RrfConfig};
    use rank_rerank::explain::{RerankerInput, Candidate, RerankMethod, rerank_batch};
    use rank_eval::binary::{ndcg_at_k, precision_at_k, recall_at_k, mrr};
    use std::collections::HashSet;

    #[test]
    fn test_concrete_functions_with_fusion() {
        // Test that concrete functions work directly with rank-fusion
        // No conversion needed - u32 works directly
        
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);
        bm25_index.add_document(2, &["python".to_string(), "programming".to_string()]);
        
        let query = vec!["learning".to_string()];
        let bm25_results = retrieve_bm25(&bm25_index, &query, 10, Bm25Params::default()).unwrap();
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);
        
        let query_emb = [1.0, 0.0, 0.0];
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        
        // Direct fusion - u32 works with rank-fusion
        let fused = rrf(&bm25_results, &dense_results);
        
        assert!(!fused.is_empty());
        assert!(fused.iter().all(|(id, score)| {
            *id < 3 && score.is_finite() && *score >= 0.0
        }));
        
        // Verify sorted descending
        for i in 1..fused.len() {
            assert!(fused[i-1].1 >= fused[i].1);
        }
    }

    #[test]
    fn test_three_way_fusion() {
        // Test fusion of BM25, dense, and sparse retrieval
        
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);
        bm25_index.add_document(2, &["python".to_string(), "programming".to_string()]);
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);
        
        let mut sparse_retriever = SparseRetriever::new();
        let doc0_sparse = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]);
        let doc1_sparse = SparseVector::new_unchecked(vec![1, 2], vec![1.0, 0.5]);
        let doc2_sparse = SparseVector::new_unchecked(vec![0, 2], vec![1.0, 0.5]);
        sparse_retriever.add_document(0, doc0_sparse);
        sparse_retriever.add_document(1, doc1_sparse);
        sparse_retriever.add_document(2, doc2_sparse);
        
        let query_terms = vec!["learning".to_string()];
        let query_emb = [1.0, 0.0, 0.0];
        let query_sparse = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        let sparse_results = retrieve_sparse(&sparse_retriever, &query_sparse, 10).unwrap();
        
        // Three-way fusion
        let fused = rrf_multi(&[&bm25_results, &dense_results, &sparse_results], RrfConfig::default());
        
        assert!(!fused.is_empty());
        // Fusion combines results but doesn't create new documents - should have at most
        // the union of all input documents (3 in this test)
        assert!(fused.len() <= 3);
        
        // Verify all results are valid
        assert!(fused.iter().all(|(id, score)| {
            *id < 3 && score.is_finite() && *score >= 0.0
        }));
    }

    #[test]
    fn test_batch_retrieval_e2e() {
        // Test batch retrieval with fusion
        
        let mut bm25_index = InvertedIndex::new();
        for i in 0..5 {
            let terms: Vec<String> = (0..3).map(|j| format!("term{}", j)).collect();
            bm25_index.add_document(i, &terms);
        }
        
        let queries = vec![
            vec!["term0".to_string()],
            vec!["term1".to_string()],
            vec!["term2".to_string()],
        ];
        
        let batch_results = batch::batch_retrieve_bm25(&bm25_index, &queries, 5, Bm25Params::default()).unwrap();
        
        assert_eq!(batch_results.len(), 3);
        
        // Fuse results from multiple queries
        let fused = rrf_multi(&[&batch_results[0], &batch_results[1], &batch_results[2]], RrfConfig::default());
        
        assert!(!fused.is_empty());
    }

    #[test]
    fn test_retrieve_fuse_rerank_pipeline() {
        // Complete pipeline: retrieve → fuse → rerank
        
        // Step 1: Retrieve
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);
        bm25_index.add_document(2, &["python".to_string(), "programming".to_string()]);
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);
        
        let query_terms = vec!["learning".to_string()];
        let query_emb = [1.0, 0.0, 0.0];
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        
        // Step 2: Fuse
        let fused = rrf(&bm25_results, &dense_results);
        assert!(!fused.is_empty());
        
        // Step 3: Rerank
        // Store embeddings in a vector that outlives the candidates
        let doc_embeddings: Vec<Vec<f32>> = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.707, 0.707, 0.0],
            vec![0.0, 1.0, 0.0],
        ];
        
        let candidates: Vec<Candidate<u32>> = fused.iter()
            .map(|(id, score)| {
                let embedding = doc_embeddings.get(*id as usize)
                    .map(|e| e.as_slice())
                    .unwrap_or(&[0.0, 0.0, 1.0]);
                Candidate {
                    id: *id,
                    original_score: *score,
                    dense_embedding: Some(embedding),
                    token_embeddings: None,
                    text: None,
                }
            })
            .collect();
        
        let input = RerankerInput {
            query_dense: Some(&query_emb),
            query_tokens: None,
            candidates,
        };
        
        let reranked = rerank_batch(input, RerankMethod::DenseCosine, 10);
        
        assert!(!reranked.is_empty());
        assert!(reranked.iter().all(|r| r.score.is_finite()));
        
        // Verify sorted descending
        for i in 1..reranked.len() {
            assert!(reranked[i-1].score >= reranked[i].score);
        }
    }

    #[test]
    fn test_retrieve_fuse_eval_pipeline() {
        // Complete pipeline: retrieve → fuse → eval
        
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);
        bm25_index.add_document(2, &["python".to_string(), "programming".to_string()]);
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);
        
        let query_terms = vec!["learning".to_string()];
        let query_emb = [1.0, 0.0, 0.0];
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        
        // Fuse
        let fused = rrf(&bm25_results, &dense_results);
        
        // Evaluate
        let ranked: Vec<String> = fused.iter().map(|(id, _)| id.to_string()).collect();
        let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
        
        let ndcg = ndcg_at_k(&ranked, &relevant, 10);
        let precision = precision_at_k(&ranked, &relevant, 10);
        let recall = recall_at_k(&ranked, &relevant, 10);
        let mrr_score = mrr(&ranked, &relevant);
        
        assert!(ndcg >= 0.0 && ndcg <= 1.0);
        assert!(precision >= 0.0 && precision <= 1.0);
        assert!(recall >= 0.0 && recall <= 1.0);
        assert!(mrr_score >= 0.0 && mrr_score <= 1.0);
    }

    #[test]
    fn test_combsum_fusion_e2e() {
        // Test CombSUM fusion (score-based) with retrieval results
        
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        
        let query_terms = vec!["learning".to_string()];
        let query_emb = [1.0, 0.0, 0.0];
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        
        // CombSUM uses scores (unlike RRF which uses ranks)
        let fused = combsum(&bm25_results, &dense_results);
        
        assert!(!fused.is_empty());
        assert!(fused.iter().all(|(_, score)| score.is_finite()));
        
        // Verify sorted descending
        for i in 1..fused.len() {
            assert!(fused[i-1].1 >= fused[i].1);
        }
    }

    #[test]
    fn test_multi_fusion_methods() {
        // Test multiple fusion methods with same retrieval results
        
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        
        let query_terms = vec!["learning".to_string()];
        let query_emb = [1.0, 0.0, 0.0];
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        
        // Test different fusion methods
        let rrf_fused = rrf(&bm25_results, &dense_results);
        let combsum_fused = combsum(&bm25_results, &dense_results);
        
        assert!(!rrf_fused.is_empty());
        assert!(!combsum_fused.is_empty());
        
        // Both should produce valid results
        assert!(rrf_fused.iter().all(|(id, score)| {
            *id < 2 && score.is_finite() && *score >= 0.0
        }));
        assert!(combsum_fused.iter().all(|(id, score)| {
            *id < 2 && score.is_finite() && *score >= 0.0
        }));
    }

    #[test]
    fn test_error_propagation_through_pipeline() {
        // Test that errors from retrieval propagate correctly
        
        // Empty index
        let empty_index = InvertedIndex::new();
        let result = retrieve_bm25(&empty_index, &["test".to_string()], 10, Bm25Params::default());
        assert!(result.is_err());
        
        // Empty query
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string()]);
        let result = retrieve_bm25(&index, &[], 10, Bm25Params::default());
        assert!(result.is_err());
        
        // Dimension mismatch for dense
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0]);
        let result = retrieve_dense(&dense_retriever, &[1.0, 0.0, 0.0], 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_large_scale_retrieval_fusion() {
        // Test with larger document sets
        
        let mut bm25_index = InvertedIndex::new();
        let mut dense_retriever = DenseRetriever::new();
        
        // Add 100 documents
        for i in 0..100 {
            let terms: Vec<String> = (0..10).map(|j| format!("term{}", j)).collect();
            bm25_index.add_document(i, &terms);
            
            let embedding: Vec<f32> = (0..128).map(|j| (i as f32 + j as f32) / 200.0).collect();
            dense_retriever.add_document(i, embedding);
        }
        
        let query_terms = vec!["term0".to_string(), "term1".to_string()];
        let query_emb: Vec<f32> = (0..128).map(|j| (j as f32) / 200.0).collect();
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 50, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 50).unwrap();
        
        assert_eq!(bm25_results.len(), 50);
        assert_eq!(dense_results.len(), 50);
        
        // Fuse
        let fused = rrf_multi(&[&bm25_results, &dense_results], RrfConfig::default());
        
        assert!(!fused.is_empty());
        // Fusion combines results but doesn't create new documents - should have at most
        // the union of all unique documents from both input lists (100 documents in corpus)
        assert!(fused.len() <= 100);
    }

    #[test]
    fn test_output_format_consistency() {
        // Verify all retrieval methods return consistent format
        
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["test".to_string()]);
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0]);
        
        let mut sparse_retriever = SparseRetriever::new();
        let doc_sparse = SparseVector::new_unchecked(vec![0], vec![1.0]);
        sparse_retriever.add_document(0, doc_sparse);
        
        let query_terms = vec!["test".to_string()];
        let query_emb = [1.0, 0.0];
        let query_sparse = SparseVector::new_unchecked(vec![0], vec![1.0]);
        
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        let sparse_results = retrieve_sparse(&sparse_retriever, &query_sparse, 10).unwrap();
        
        // All should return Vec<(u32, f32)>
        assert!(bm25_results.iter().all(|(id, score)| {
            *id < 10 && score.is_finite()
        }));
        assert!(dense_results.iter().all(|(id, score)| {
            *id < 10 && score.is_finite()
        }));
        assert!(sparse_results.iter().all(|(id, score)| {
            *id < 10 && score.is_finite()
        }));
        
        // All should be sorted descending
        for results in [&bm25_results, &dense_results, &sparse_results] {
            for i in 1..results.len() {
                assert!(results[i-1].1 >= results[i].1);
            }
        }
    }

    #[test]
    fn test_complete_pipeline_all_stages() {
        // Full pipeline: retrieve → fuse → rerank → eval
        
        // Setup
        let mut bm25_index = InvertedIndex::new();
        bm25_index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        bm25_index.add_document(1, &["deep".to_string(), "learning".to_string()]);
        bm25_index.add_document(2, &["python".to_string(), "programming".to_string()]);
        
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);
        
        let query_terms = vec!["learning".to_string()];
        let query_emb = [1.0, 0.0, 0.0];
        
        // Stage 1: Retrieve
        let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();
        
        // Stage 2: Fuse
        let fused = rrf(&bm25_results, &dense_results);
        assert!(!fused.is_empty());
        
        // Stage 3: Rerank
        // Store embeddings in a vector that outlives the candidates
        let doc_embeddings: Vec<Vec<f32>> = vec![
            vec![1.0, 0.0, 0.0],
            vec![0.707, 0.707, 0.0],
            vec![0.0, 1.0, 0.0],
        ];
        
        let candidates: Vec<Candidate<u32>> = fused.iter()
            .map(|(id, score)| {
                let embedding = doc_embeddings.get(*id as usize)
                    .map(|e| e.as_slice())
                    .unwrap_or(&[0.0, 0.0, 1.0]);
                Candidate {
                    id: *id,
                    original_score: *score,
                    dense_embedding: Some(embedding),
                    token_embeddings: None,
                    text: None,
                }
            })
            .collect();
        
        let input = RerankerInput {
            query_dense: Some(&query_emb),
            query_tokens: None,
            candidates,
        };
        
        let reranked = rerank_batch(input, RerankMethod::DenseCosine, 10);
        assert!(!reranked.is_empty());
        
        // Stage 4: Evaluate
        let ranked: Vec<String> = reranked.iter().map(|r| r.id.to_string()).collect();
        let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
        
        let ndcg = ndcg_at_k(&ranked, &relevant, 10);
        assert!(ndcg >= 0.0 && ndcg <= 1.0);
    }
}

