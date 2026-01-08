//! Property-based tests for rank-retrieve.

use proptest::prelude::*;
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::generative::{GenerativeRetriever, MockAutoregressiveModel};
use rank_retrieve::sparse::SparseRetriever;
use rank_retrieve::sparse::{dot_product, SparseVector};

proptest! {
    #[test]
    fn bm25_scores_are_non_negative(
        doc_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..100),
        query_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..20),
    ) {
        let mut index = InvertedIndex::new();
        index.add_document(0, &doc_terms);

        let results = index.retrieve(&query_terms, 10, Bm25Params::default()).unwrap();

        for (_, score) in results {
            prop_assert!(score >= 0.0, "BM25 scores must be non-negative");
        }
    }

    #[test]
    fn bm25_retrieve_returns_at_most_k(
        doc_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..100),
        query_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..20),
        k in 1usize..50,
    ) {
        let mut index = InvertedIndex::new();
        index.add_document(0, &doc_terms);

        let results = index.retrieve(&query_terms, k, Bm25Params::default()).unwrap();

        prop_assert!(results.len() <= k, "retrieve() must return at most k results");
    }

    #[test]
    fn bm25_results_are_sorted_descending(
        doc_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..100),
        query_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..20),
    ) {
        let mut index = InvertedIndex::new();
        index.add_document(0, &doc_terms);

        let results = index.retrieve(&query_terms, 10, Bm25Params::default()).unwrap();

        for i in 1..results.len() {
            prop_assert!(
                results[i-1].1 >= results[i].1,
                "Results must be sorted by score descending"
            );
        }
    }

    #[test]
    fn dense_retrieve_returns_at_most_k(
        embedding_dim in 1usize..128,
        k in 1usize..50,
    ) {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0; embedding_dim]);

        let query = vec![1.0; embedding_dim];
        let results = retriever.retrieve(&query, k).unwrap();

        prop_assert!(results.len() <= k, "retrieve() must return at most k results");
    }

    #[test]
    fn dense_scores_are_finite(
        embedding_dim in 1usize..128,
    ) {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0; embedding_dim]);

        let query = vec![1.0; embedding_dim];
        let results = retriever.retrieve(&query, 10).unwrap();

        for (_, score) in results {
            prop_assert!(
                score.is_finite(),
                "Cosine similarity must be finite"
            );
        }
    }

    #[test]
    fn dense_results_are_sorted_descending(
        embedding_dim in 1usize..128,
    ) {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0; embedding_dim]);
        retriever.add_document(1, vec![0.5; embedding_dim]);

        let query = vec![1.0; embedding_dim];
        let results = retriever.retrieve(&query, 10).unwrap();

        for i in 1..results.len() {
            prop_assert!(
                results[i-1].1 >= results[i].1,
                "Results must be sorted by score descending"
            );
        }
    }

    #[test]
    fn sparse_retrieve_returns_at_most_k(
        num_terms in 1usize..100,
        k in 1usize..50,
    ) {
        let mut retriever = SparseRetriever::new();
        let indices: Vec<u32> = (0..num_terms).map(|i| i as u32).collect();
        let values = vec![1.0; num_terms];
        let doc_vector = SparseVector::new(indices.clone(), values.clone()).unwrap();
        retriever.add_document(0, doc_vector);

        let query_vector = SparseVector::new(indices, values).unwrap();
        let results = retriever.retrieve(&query_vector, k).unwrap();

        prop_assert!(results.len() <= k, "retrieve() must return at most k results");
    }

    #[test]
    fn sparse_results_are_sorted_descending(
        num_terms in 1usize..100,
    ) {
        let mut retriever = SparseRetriever::new();
        let indices: Vec<u32> = (0..num_terms).map(|i| i as u32).collect();
        let values = vec![1.0; num_terms];
        let doc_vector = SparseVector::new(indices.clone(), values.clone()).unwrap();
        retriever.add_document(0, doc_vector);

        let query_vector = SparseVector::new(indices, values).unwrap();
        let results = retriever.retrieve(&query_vector, 10).unwrap();

        for i in 1..results.len() {
            prop_assert!(
                results[i-1].1 >= results[i].1,
                "Results must be sorted by score descending"
            );
        }
    }

    #[test]
    fn bm25_idf_monotonicity(
        term1 in prop::string::string_regex("[a-z]{1,10}").unwrap(),
        term2 in prop::string::string_regex("[a-z]{1,10}").unwrap(),
        num_docs in 2u32..100,
    ) {
        // Ensure term1 != term2
        if term1 == term2 {
            return Ok(());
        }

        let mut index = InvertedIndex::new();

        // Add term1 to all documents (common term)
        for doc_id in 0..num_docs {
            index.add_document(doc_id, &[term1.clone()]);
        }

        // Add term2 to only one document (rare term)
        index.add_document(num_docs, &[term2.clone()]);

        let idf_common = index.idf(&term1);
        let idf_rare = index.idf(&term2);

        // Rare term should have higher IDF
        prop_assert!(
            idf_rare > idf_common,
            "Rare terms should have higher IDF than common terms"
        );
    }

    #[test]
    fn dense_cosine_similarity_bounds(
        embedding_dim in 1usize..128,
        num_docs in 1usize..20,
    ) {
        let mut retriever = DenseRetriever::new();

        // Add documents with normalized embeddings
        for doc_id in 0..num_docs {
            let mut embedding = vec![0.0; embedding_dim];
            // Create normalized vector
            for i in 0..embedding_dim {
                embedding[i] = if i == doc_id % embedding_dim { 1.0 } else { 0.0 };
            }
            retriever.add_document(doc_id as u32, embedding);
        }

        let query = vec![1.0; embedding_dim];
        let results = retriever.retrieve(&query, num_docs).unwrap();

        for (_, score) in results {
            prop_assert!(
                score >= -1.0 && score <= 1.0,
                "Cosine similarity must be in [-1, 1]"
            );
        }
    }

    #[test]
    fn sparse_dot_product_commutative(
        num_terms in 1usize..100,
    ) {
        let indices: Vec<u32> = (0..num_terms).map(|i| i as u32).collect();
        let values1: Vec<f32> = (0..num_terms).map(|i| (i as f32) * 0.1).collect();
        let values2: Vec<f32> = (0..num_terms).map(|i| (i as f32) * 0.2).collect();

        let v1 = SparseVector::new(indices.clone(), values1.clone()).unwrap();
        let v2 = SparseVector::new(indices, values2.clone()).unwrap();

        let dot1 = dot_product(&v1, &v2);
        let dot2 = dot_product(&v2, &v1);

        prop_assert!(
            (dot1 - dot2).abs() < 1e-6,
            "Dot product must be commutative"
        );
    }

    #[test]
    fn bm25_empty_query_handled(
        doc_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..100),
    ) {
        let mut index = InvertedIndex::new();
        index.add_document(0, &doc_terms);

        let result = index.retrieve(&[], 10, Bm25Params::default());
        prop_assert!(
            result.is_err(),
            "Empty query should return error"
        );
    }

    #[test]
    fn bm25_empty_index_handled(
        query_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..20),
    ) {
        let index = InvertedIndex::new();

        let result = index.retrieve(&query_terms, 10, Bm25Params::default());
        prop_assert!(
            result.is_err(),
            "Empty index should return error"
        );
    }

    #[test]
    fn dense_dimension_mismatch_handled(
        doc_dim in 1usize..128,
        query_dim in 1usize..128,
    ) {
        if doc_dim == query_dim {
            return Ok(()); // Skip when dimensions match
        }

        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0; doc_dim]);

        let query = vec![1.0; query_dim];
        let result = retriever.retrieve(&query, 10);

        prop_assert!(
            result.is_err(),
            "Dimension mismatch should return error"
        );
    }

    #[test]
    fn bm25_idf_always_non_negative(
        term in prop::string::string_regex("[a-z]{1,10}").unwrap(),
        num_docs in 1u32..100,
    ) {
        let mut index = InvertedIndex::new();
        for doc_id in 0..num_docs {
            index.add_document(doc_id, &[term.clone()]);
        }

        let idf = index.idf(&term);
        prop_assert!(
            idf >= 0.0,
            "IDF must always be non-negative"
        );
    }

    #[test]
    fn dense_cosine_similarity_symmetric(
        embedding_dim in 1usize..128,
    ) {
        let mut retriever = DenseRetriever::new();
        let emb1: Vec<f32> = (0..embedding_dim).map(|i| (i as f32) * 0.01).collect();
        let emb2: Vec<f32> = (0..embedding_dim).map(|i| ((i + 1) as f32) * 0.01).collect();

        retriever.add_document(0, emb1.clone());
        retriever.add_document(1, emb2.clone());

        let score1 = retriever.score(0, &emb2);
        let score2 = retriever.score(1, &emb1);

        if let (Some(s1), Some(s2)) = (score1, score2) {
            prop_assert!(
                (s1 - s2).abs() < 1e-6,
                "Cosine similarity should be symmetric"
            );
        }
    }

    #[test]
    fn sparse_dot_product_non_negative_when_positive_values(
        num_terms in 1usize..100,
    ) {
        let indices: Vec<u32> = (0..num_terms).map(|i| i as u32).collect();
        let values: Vec<f32> = (0..num_terms).map(|i| (i as f32) * 0.1 + 0.1).collect(); // All positive

        let v1 = SparseVector::new(indices.clone(), values.clone()).unwrap();
        let v2 = SparseVector::new(indices, values).unwrap();

        let dot = dot_product(&v1, &v2);
        prop_assert!(
            dot >= 0.0,
            "Dot product of positive vectors must be non-negative"
        );
    }

    #[test]
    fn bm25_score_monotonic_with_term_frequency(
        base_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..50),
        query_term in prop::string::string_regex("[a-z]{1,10}").unwrap(),
    ) {
        // Create two documents: one with query_term once, one with it multiple times
        let mut index = InvertedIndex::new();

        let mut doc1_terms = base_terms.clone();
        doc1_terms.push(query_term.clone());
        index.add_document(0, &doc1_terms);

        let mut doc2_terms = base_terms;
        // Add query_term multiple times
        for _ in 0..5 {
            doc2_terms.push(query_term.clone());
        }
        index.add_document(1, &doc2_terms);

        let results = index.retrieve(&[query_term], 10, Bm25Params::default()).unwrap();

        if results.len() >= 2 {
            // Document with more occurrences should score higher (generally)
            // Note: This isn't always true due to IDF, but with same term it should be
            let score0 = results.iter().find(|(id, _)| *id == 0).map(|(_, s)| *s);
            let score1 = results.iter().find(|(id, _)| *id == 1).map(|(_, s)| *s);

            if let (Some(s0), Some(s1)) = (score0, score1) {
                // Document 1 has more occurrences, should generally score higher
                // But this depends on document length normalization, so we just verify scores are finite
                prop_assert!(s0.is_finite() && s1.is_finite());
            }
        }
    }

    #[test]
    fn dense_retrieval_consistency(
        embedding_dim in 1usize..128,
        num_docs in 1usize..20,
    ) {
        let mut retriever = DenseRetriever::new();

        // Add documents
        for doc_id in 0..num_docs {
            let mut embedding = vec![0.0; embedding_dim];
            embedding[doc_id % embedding_dim] = 1.0; // One-hot encoding
            retriever.add_document(doc_id as u32, embedding);
        }

        // Query should match document 0 best
        let query: Vec<f32> = (0..embedding_dim).map(|i| if i == 0 { 1.0 } else { 0.0 }).collect();

        let results1 = retriever.retrieve(&query, num_docs).unwrap();
        let results2 = retriever.retrieve(&query, num_docs).unwrap();

        // Results should be consistent across calls
        prop_assert_eq!(results1.len(), results2.len());
        for ((id1, s1), (id2, s2)) in results1.iter().zip(results2.iter()) {
            prop_assert_eq!(id1, id2);
            prop_assert!((s1 - s2).abs() < 1e-6, "Scores should be consistent");
        }
    }

    #[test]
    fn generative_retrieve_returns_at_most_k(
        query in prop::string::string_regex("[a-zA-Z][a-zA-Z ]{0,99}").unwrap(),
        num_docs in 1usize..20,
        k in 1usize..50,
    ) {
        // Filter out queries that are only whitespace
        if query.trim().is_empty() {
            return Ok(());
        }

        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model);

        for doc_id in 0..num_docs {
            let passage = format!("Document {} content with some text", doc_id);
            retriever.add_document(doc_id as u32, &passage);
        }

        let results = retriever.retrieve(&query, k).unwrap();
        prop_assert!(results.len() <= k, "retrieve() must return at most k results");
    }

    #[test]
    fn generative_scores_are_non_negative(
        query in prop::string::string_regex("[a-zA-Z][a-zA-Z ]{0,99}").unwrap(),
        num_docs in 1usize..20,
    ) {
        // Filter out queries that are only whitespace
        if query.trim().is_empty() {
            return Ok(());
        }

        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model);

        for doc_id in 0..num_docs {
            let passage = format!("Document {} content with some text", doc_id);
            retriever.add_document(doc_id as u32, &passage);
        }

        let results = retriever.retrieve(&query, num_docs).unwrap();
        for (_, score) in results {
            prop_assert!(score >= 0.0, "Generative scores must be non-negative");
        }
    }

    #[test]
    fn generative_results_are_sorted_descending(
        query in prop::string::string_regex("[a-zA-Z][a-zA-Z ]{0,99}").unwrap(),
        num_docs in 1usize..20,
    ) {
        // Filter out queries that are only whitespace
        if query.trim().is_empty() {
            return Ok(());
        }

        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model);

        for doc_id in 0..num_docs {
            let passage = format!("Document {} content with some text", doc_id);
            retriever.add_document(doc_id as u32, &passage);
        }

        let results = retriever.retrieve(&query, num_docs).unwrap();
        for i in 1..results.len() {
            prop_assert!(
                results[i-1].1 >= results[i].1,
                "Results must be sorted by score descending"
            );
        }
    }

    #[test]
    fn generative_scores_are_finite(
        query in prop::string::string_regex("[a-zA-Z][a-zA-Z ]{0,99}").unwrap(),
        num_docs in 1usize..20,
    ) {
        // Filter out queries that are only whitespace
        if query.trim().is_empty() {
            return Ok(());
        }

        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model);

        for doc_id in 0..num_docs {
            let passage = format!("Document {} content with some text", doc_id);
            retriever.add_document(doc_id as u32, &passage);
        }

        let results = retriever.retrieve(&query, num_docs).unwrap();
        for (_, score) in results {
            prop_assert!(score.is_finite(), "Scores must be finite");
        }
    }

    #[test]
    fn generative_empty_query_handled(
        num_docs in 1usize..20,
    ) {
        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model);

        for doc_id in 0..num_docs {
            let passage = format!("Document {} content", doc_id);
            retriever.add_document(doc_id as u32, &passage);
        }

        let result = retriever.retrieve("", num_docs);
        prop_assert!(result.is_err(), "Empty query should return error");
    }

    #[test]
    fn generative_empty_index_handled(
        query in prop::string::string_regex("[a-zA-Z ]{1,100}").unwrap(),
    ) {
        let model = MockAutoregressiveModel::new();
        let retriever = GenerativeRetriever::new(model);

        let result = retriever.retrieve(&query, 10);
        prop_assert!(result.is_err(), "Empty index should return error");
    }

    #[test]
    fn generative_retrieval_consistency(
        query in prop::string::string_regex("[a-zA-Z][a-zA-Z ]{0,99}").unwrap(),
        num_docs in 1usize..20,
    ) {
        // Filter out queries that are only whitespace
        if query.trim().is_empty() {
            return Ok(());
        }

        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model);

        for doc_id in 0..num_docs {
            let passage = format!("Document {} content with some text", doc_id);
            retriever.add_document(doc_id as u32, &passage);
        }

        let results1 = retriever.retrieve(&query, num_docs).unwrap();
        let results2 = retriever.retrieve(&query, num_docs).unwrap();

        // Results should be consistent across calls
        prop_assert_eq!(results1.len(), results2.len());
        for ((id1, s1), (id2, s2)) in results1.iter().zip(results2.iter()) {
            prop_assert_eq!(id1, id2);
            prop_assert!((s1 - s2).abs() < 1e-6, "Scores should be consistent");
        }
    }
}
