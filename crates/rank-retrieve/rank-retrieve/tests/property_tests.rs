//! Property-based tests for rank-retrieve.

use proptest::prelude::*;
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::sparse::SparseRetriever;
use rank_sparse::SparseVector;

proptest! {
    #[test]
    fn bm25_scores_are_non_negative(
        doc_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..100),
        query_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..20),
    ) {
        let mut index = InvertedIndex::new();
        index.add_document(0, &doc_terms);
        
        let results = index.retrieve(&query_terms, 10, Bm25Params::default());
        
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
        
        let results = index.retrieve(&query_terms, k, Bm25Params::default());
        
        prop_assert!(results.len() <= k, "retrieve() must return at most k results");
    }

    #[test]
    fn bm25_results_are_sorted_descending(
        doc_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..100),
        query_terms in prop::collection::vec(prop::string::string_regex("[a-z]{1,10}").unwrap(), 1..20),
    ) {
        let mut index = InvertedIndex::new();
        index.add_document(0, &doc_terms);
        
        let results = index.retrieve(&query_terms, 10, Bm25Params::default());
        
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
        let results = retriever.retrieve(&query, k);
        
        prop_assert!(results.len() <= k, "retrieve() must return at most k results");
    }

    #[test]
    fn dense_scores_are_in_range(
        embedding_dim in 1usize..128,
    ) {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0; embedding_dim]);
        
        let query = vec![1.0; embedding_dim];
        let results = retriever.retrieve(&query, 10);
        
        for (_, score) in results {
            prop_assert!(
                score >= -1.0 && score <= 1.0,
                "Cosine similarity must be in [-1, 1]"
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
        let results = retriever.retrieve(&query, 10);
        
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
        let results = retriever.retrieve(&query_vector, k);
        
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
        let results = retriever.retrieve(&query_vector, 10);
        
        for i in 1..results.len() {
            prop_assert!(
                results[i-1].1 >= results[i].1,
                "Results must be sorted by score descending"
            );
        }
    }
}

