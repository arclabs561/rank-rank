//! End-to-end tests: Realistic retrieval scenarios
//!
//! These tests simulate real-world use cases to ensure the library works
//! correctly in production-like scenarios.
//!
//! **Test scenarios:**
//! - E-commerce product search
//! - Document search (technical documentation)
//! - Question answering (RAG pipeline)
//! - Multilingual retrieval
//! - Long-tail query handling

#[cfg(test)]
mod tests {
    use rank_eval::binary::ndcg_at_k;
    use rank_fusion::rrf;
    use rank_rerank::colbert;
    #[cfg(feature = "bm25")]
    use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
    #[cfg(feature = "dense")]
    use rank_retrieve::dense::DenseRetriever;
    use rank_retrieve::retrieve_bm25;
    #[cfg(feature = "dense")]
    use rank_retrieve::retrieve_dense;
    use std::collections::HashSet;

    /// Scenario: E-commerce product search
    ///
    /// User searches for products with a query like "wireless headphones noise cancelling".
    /// The system should:
    /// 1. Retrieve products matching keywords (BM25)
    /// 2. Optionally fuse with semantic search (dense)
    /// 3. Rerank to prioritize most relevant products
    #[cfg(feature = "bm25")]
    #[test]
    fn test_ecommerce_product_search() {
        // Product catalog
        let products = vec![
            (0, "Sony WH-1000XM4 Wireless Noise Cancelling Headphones"),
            (1, "Bose QuietComfort 35 II Wireless Bluetooth Headphones"),
            (2, "Apple AirPods Pro Wireless Earbuds with Active Noise Cancellation"),
            (3, "Sennheiser HD 450BT Wireless Headphones"),
            (4, "JBL Tune 750BTNC Wireless On-Ear Headphones with Noise Cancelling"),
        ];

        let mut index = InvertedIndex::new();
        for (id, title) in &products {
            let terms: Vec<String> = title
                .split_whitespace()
                .map(|s| s.to_string().to_lowercase())
                .collect();
            index.add_document(*id, &terms);
        }

        // User query
        let query = "wireless headphones noise cancelling";
        let query_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_string().to_lowercase())
            .collect();

        // Retrieve top products
        let results = retrieve_bm25(&index, &query_terms, 10, Bm25Params::default()).unwrap();

        // Verify results
        assert!(!results.is_empty());
        assert!(results.len() <= 5); // Should not exceed number of products

        // Top result should be highly relevant
        let top_id = results[0].0;
        let top_title = products.iter().find(|(id, _)| *id == top_id).unwrap().1;
        assert!(
            top_title.to_lowercase().contains("wireless")
                && top_title.to_lowercase().contains("noise")
        );

        // All results should contain at least one query term
        for (id, _) in &results {
            let title = products.iter().find(|(p_id, _)| p_id == id).unwrap().1;
            let title_lower = title.to_lowercase();
            assert!(
                query_terms.iter().any(|term| title_lower.contains(term)),
                "Result should contain at least one query term"
            );
        }
    }

    /// Scenario: Technical documentation search
    ///
    /// Developer searches documentation for "async await error handling".
    /// The system should retrieve relevant documentation pages.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_technical_documentation_search() {
        let docs = vec![
            (0, "Rust async programming guide async await futures"),
            (1, "Error handling in Rust Result Option unwrap"),
            (2, "Async error handling with tokio spawn await"),
            (3, "Rust ownership borrowing references"),
            (4, "Async await patterns error propagation"),
        ];

        let mut index = InvertedIndex::new();
        for (id, text) in &docs {
            let terms: Vec<String> = text
                .split_whitespace()
                .map(|s| s.to_string().to_lowercase())
                .collect();
            index.add_document(*id, &terms);
        }

        let query = "async await error handling";
        let query_terms: Vec<String> = query
            .split_whitespace()
            .map(|s| s.to_string().to_lowercase())
            .collect();

        let results = retrieve_bm25(&index, &query_terms, 10, Bm25Params::default()).unwrap();

        assert!(!results.is_empty());

        // Most relevant docs should be ranked highly
        let top_ids: Vec<u32> = results.iter().take(3).map(|(id, _)| *id).collect();
        assert!(
            top_ids.contains(&2) || top_ids.contains(&4),
            "Docs about async error handling should be ranked highly"
        );
    }

    /// Scenario: Question answering (RAG pipeline)
    ///
    /// User asks a question, system retrieves relevant passages, then generates answer.
    /// This tests the retrieval stage of RAG.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_rag_question_answering() {
        // Knowledge base passages
        let passages = vec![
            (0, "Python is a high-level programming language known for its simplicity."),
            (1, "Rust is a systems programming language focused on memory safety."),
            (2, "Machine learning uses algorithms to learn patterns from data."),
            (3, "Neural networks are inspired by biological neurons in the brain."),
            (4, "Information retrieval finds relevant documents from large collections."),
        ];

        let mut index = InvertedIndex::new();
        for (id, text) in &passages {
            let terms: Vec<String> = text
                .split_whitespace()
                .map(|s| s.to_string().to_lowercase())
                .collect();
            index.add_document(*id, &terms);
        }

        // User question
        let question = "what is python programming language";
        let query_terms: Vec<String> = question
            .split_whitespace()
            .map(|s| s.to_string().to_lowercase())
            .collect();

        let results = retrieve_bm25(&index, &query_terms, 5, Bm25Params::default()).unwrap();

        assert!(!results.is_empty());

        // Most relevant passage should be about Python
        let top_id = results[0].0;
        assert_eq!(top_id, 0, "Python passage should be ranked highest");
    }

    /// Scenario: Hybrid retrieval (BM25 + Dense)
    ///
    /// Combines lexical (BM25) and semantic (dense) retrieval for better recall.
    #[cfg(all(feature = "bm25", feature = "dense"))]
    #[test]
    fn test_hybrid_retrieval_scenario() {
        // Documents
        let documents = vec![
            (0, "machine learning algorithms neural networks"),
            (1, "deep learning artificial intelligence"),
            (2, "python programming data science"),
        ];

        // BM25 index
        let mut bm25_index = InvertedIndex::new();
        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            bm25_index.add_document(*id, &terms);
        }

        // Dense retriever
        let mut dense_retriever = DenseRetriever::new();
        dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
        dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);

        // Query
        let query_terms = vec!["learning".to_string()];
        let query_emb = [1.0, 0.0, 0.0];

        // Retrieve from both methods
        let bm25_results =
            retrieve_bm25(&bm25_index, &query_terms, 10, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&dense_retriever, &query_emb, 10).unwrap();

        // Fuse results
        let fused = rrf(&bm25_results, &dense_results);

        assert!(!fused.is_empty());
        assert!(fused.len() <= 3); // Should not exceed number of documents

        // Verify sorting
        for i in 1..fused.len() {
            assert!(fused[i - 1].1 >= fused[i].1);
        }
    }

    /// Scenario: Late interaction pipeline (BM25 → MaxSim)
    ///
    /// Tests the research-backed pipeline: BM25 first-stage retrieval
    /// followed by MaxSim reranking for token-level matching.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_late_interaction_pipeline_scenario() {
        use super::test_helpers::{mock_dense_embed, mock_token_embed};

        // Documents
        let documents = vec![
            (0, "machine learning algorithms neural networks"),
            (1, "deep learning artificial intelligence neural networks"),
            (2, "python programming language data science"),
        ];

        let mut index = InvertedIndex::new();
        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        // Step 1: BM25 first-stage retrieval
        let query_terms = vec!["neural".to_string(), "networks".to_string()];
        let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default()).unwrap();
        assert!(!candidates.is_empty());

        // Step 2: Prepare token embeddings for MaxSim reranking
        let query_tokens = mock_token_embed("neural networks", 128);
        let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = candidates
            .iter()
            .map(|(id, _)| {
                let doc_text = documents.iter().find(|(d_id, _)| d_id == id).unwrap().1;
                let tokens: Vec<Vec<f32>> = doc_text
                    .split_whitespace()
                    .map(|word| mock_dense_embed(word, 128))
                    .collect();
                (*id, tokens)
            })
            .collect();

        // Step 3: Rerank with MaxSim
        let reranked = colbert::rank(&query_tokens, &doc_tokens);

        assert!(!reranked.is_empty());
        assert_eq!(reranked.len(), doc_tokens.len());

        // Verify sorting (descending by score)
        for i in 1..reranked.len() {
            assert!(
                reranked[i - 1].1 >= reranked[i].1,
                "Results should be sorted descending"
            );
        }

        // Top results should be about neural networks
        let top_ids: Vec<u32> = reranked.iter().take(2).map(|(id, _)| *id).collect();
        assert!(
            top_ids.contains(&0) || top_ids.contains(&1),
            "Documents about neural networks should be ranked highly"
        );
    }

    /// Scenario: Long-tail query handling
    ///
    /// Tests retrieval for rare, specific queries that may have few matching documents.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_long_tail_query_handling() {
        let documents = vec![
            (0, "rust programming language memory safety"),
            (1, "python async await coroutines"),
            (2, "javascript promises async programming"),
            (3, "go goroutines concurrency"),
            (4, "erlang actor model distributed systems"),
        ];

        let mut index = InvertedIndex::new();
        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        // Long-tail query: very specific, few matches
        let query = "erlang actor model distributed";
        let query_terms: Vec<String> = query.split_whitespace().map(|s| s.to_string()).collect();

        let results = retrieve_bm25(&index, &query_terms, 10, Bm25Params::default()).unwrap();

        // Should still return results even for rare queries
        assert!(!results.is_empty());

        // Most relevant document should be about Erlang
        let top_id = results[0].0;
        assert_eq!(top_id, 4, "Erlang document should be ranked highest");
    }

    /// Scenario: Evaluation of retrieval quality
    ///
    /// Tests that retrieval results can be evaluated using standard metrics.
    #[cfg(feature = "bm25")]
    #[test]
    fn test_retrieval_evaluation() {
        let documents = vec![
            (0, "machine learning neural networks"),
            (1, "deep learning artificial intelligence"),
            (2, "python programming language"),
            (3, "rust systems programming"),
            (4, "information retrieval search"),
        ];

        let mut index = InvertedIndex::new();
        for (id, text) in &documents {
            let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
            index.add_document(*id, &terms);
        }

        let query = "machine learning";
        let query_terms: Vec<String> = query.split_whitespace().map(|s| s.to_string()).collect();

        let results = retrieve_bm25(&index, &query_terms, 10, Bm25Params::default()).unwrap();

        // Convert to ranked list for evaluation
        let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();

        // Ground truth: documents 0 and 1 are relevant
        let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();

        // Evaluate
        let ndcg = ndcg_at_k(&ranked, &relevant, 10);

        // nDCG should be > 0 if relevant documents are retrieved
        assert!(ndcg >= 0.0 && ndcg <= 1.0);

        // If top results are relevant, nDCG should be reasonably high
        if ranked[0] == "0" || ranked[0] == "1" {
            assert!(ndcg > 0.3, "nDCG should be > 0.3 if top result is relevant");
        }
    }
}

// Import test helpers
#[cfg(test)]
mod test_helpers {
    use super::*;

    pub fn mock_token_embed(text: &str, dim: usize) -> Vec<Vec<f32>> {
        text.split_whitespace()
            .map(|word| mock_dense_embed(word, dim))
            .collect()
    }

    pub fn mock_dense_embed(text: &str, dim: usize) -> Vec<f32> {
        let mut emb = vec![0.0; dim];
        for (i, c) in text.chars().enumerate() {
            emb[i % dim] = (c as u32 as f32) / 1000.0;
        }
        // L2 normalize
        let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            emb.iter_mut().for_each(|x| *x /= norm);
        }
        emb
    }
}
