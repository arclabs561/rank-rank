//! Comprehensive error handling tests for rank-retrieve.
//!
//! Tests all error conditions and edge cases.

#[cfg(test)]
mod tests {
    use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
    use rank_retrieve::dense::DenseRetriever;
    use rank_retrieve::sparse::SparseRetriever;
    use rank_retrieve::sparse::SparseVector;
    use rank_retrieve::RetrieveError;

    #[test]
    fn test_bm25_empty_index_error() {
        let index = InvertedIndex::new();
        let result = index.retrieve(
            &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            10,
            Bm25Params::default(),
        );
        assert!(result.is_err());
        match result {
            Err(RetrieveError::EmptyIndex) => {}
            _ => panic!("Expected EmptyIndex error"),
        }
    }

    #[test]
    fn test_bm25_empty_query_error() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        let result = index.retrieve(&[], 10, Bm25Params::default());
        assert!(result.is_err());
        match result {
            Err(RetrieveError::EmptyQuery) => {}
            _ => panic!("Expected EmptyQuery error"),
        }
    }

    #[test]
    fn test_dense_empty_index_error() {
        let retriever = DenseRetriever::new();
        let result = retriever.retrieve(&[1.0, 0.0, 0.0], 10);
        assert!(result.is_err());
        match result {
            Err(RetrieveError::EmptyIndex) => {}
            _ => panic!("Expected EmptyIndex error"),
        }
    }

    #[test]
    fn test_dense_empty_query_error() {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        let result = retriever.retrieve(&[], 10);
        assert!(result.is_err());
        match result {
            Err(RetrieveError::EmptyQuery) => {}
            _ => panic!("Expected EmptyQuery error"),
        }
    }

    #[test]
    fn test_dense_dimension_mismatch_error() {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0, 0.0, 0.0]); // 3D
        
        // Query with wrong dimension
        let result = retriever.retrieve(&[1.0, 0.0], 10); // 2D
        assert!(result.is_err());
        match result {
            Err(RetrieveError::DimensionMismatch { query_dim, doc_dim }) => {
                assert_eq!(doc_dim, 3);
                assert_eq!(query_dim, 2);
            }
            _ => panic!("Expected DimensionMismatch error"),
        }
    }

    #[test]
    fn test_dense_dimension_mismatch_multiple_docs() {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0, 0.0, 0.0]); // 3D
        retriever.add_document(1, vec![0.0, 1.0, 0.0]); // 3D
        
        // Query with wrong dimension
        let result = retriever.retrieve(&[1.0, 0.0, 0.0, 0.0], 10); // 4D
        assert!(result.is_err());
        match result {
            Err(RetrieveError::DimensionMismatch { query_dim, doc_dim }) => {
                assert_eq!(doc_dim, 3);
                assert_eq!(query_dim, 4);
            }
            _ => panic!("Expected DimensionMismatch error"),
        }
    }

    #[test]
    fn test_sparse_empty_index_error() {
        let retriever = SparseRetriever::new();
        let query = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
        let result = retriever.retrieve(&query, 10);
        assert!(result.is_err());
        match result {
            Err(RetrieveError::EmptyIndex) => {}
            _ => panic!("Expected EmptyIndex error"),
        }
    }

    #[test]
    fn test_sparse_empty_query_error() {
        let mut retriever = SparseRetriever::new();
        let doc = SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5, 0.3]).unwrap();
        retriever.add_document(0, doc);
        
        // Empty query vector
        let empty_query = SparseVector::new(vec![], vec![]).unwrap();
        let result = retriever.retrieve(&empty_query, 10);
        assert!(result.is_err());
        match result {
            Err(RetrieveError::EmptyQuery) => {}
            _ => panic!("Expected EmptyQuery error"),
        }
    }

    #[test]
    fn test_sparse_vector_validation() {
        // Mismatched indices and values lengths
        let result = SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5]);
        assert!(result.is_none());
        
        // Empty vectors - valid (empty sparse vector)
        let result = SparseVector::new(vec![], vec![]);
        assert!(result.is_some()); // Empty is valid
        
        // Unsorted indices
        let result = SparseVector::new(vec![2, 0, 1], vec![1.0, 0.5, 0.3]);
        assert!(result.is_none());
        
        // Duplicate indices
        let result = SparseVector::new(vec![0, 1, 1], vec![1.0, 0.5, 0.3]);
        assert!(result.is_none());
        
        // Valid sparse vector
        let result = SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
        assert!(result.is_some());
    }

    #[test]
    fn test_bm25_idf_edge_cases() {
        let mut index = InvertedIndex::new();
        
        // Single document
        index.add_document(0, &["term"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        let idf = index.idf("term");
        assert!(idf >= 0.0);
        
        // Term not in index
        let idf_missing = index.idf("missing");
        assert_eq!(idf_missing, 0.0);
        
        // All documents have the term
        index.add_document(1, &["term"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        index.add_document(2, &["term"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        let idf_common = index.idf("term");
        assert!(idf_common < idf); // Common term should have lower IDF
    }

    #[test]
    fn test_dense_score_nonexistent_doc() {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0, 0.0, 0.0]);
        
        // Score non-existent document
        let score = retriever.score(999, &[1.0, 0.0, 0.0]);
        assert!(score.is_none());
    }

    #[test]
    fn test_sparse_score_nonexistent_doc() {
        let mut retriever = SparseRetriever::new();
        let doc = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
        retriever.add_document(0, doc);
        
        let query = SparseVector::new(vec![0, 1], vec![1.0, 1.0]).unwrap();
        
        // Score non-existent document
        let score = retriever.score(999, &query);
        assert!(score.is_none());
    }

    #[test]
    fn test_retrieval_k_zero() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        
        let result = index.retrieve(
            &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            0,
            Bm25Params::default(),
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_retrieval_k_larger_than_docs() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        index.add_document(1, &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        
        // Request more results than available
        let result = index.retrieve(
            &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            100,
            Bm25Params::default(),
        );
        assert!(result.is_ok());
        let results = result.unwrap();
        assert!(results.len() <= 2); // Should return at most 2 documents
    }

    #[test]
    fn test_bm25_params_edge_cases() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>());
        
        // Extreme k1 parameter
        let params_high_k1 = Bm25Params { k1: 100.0, b: 0.75 };
        let result = index.retrieve(
            &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            10,
            params_high_k1,
        );
        assert!(result.is_ok());
        
        // Extreme b parameter
        let params_high_b = Bm25Params { k1: 1.2, b: 1.0 };
        let result = index.retrieve(
            &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
            10,
            params_high_b,
        );
        assert!(result.is_ok());
    }
}

