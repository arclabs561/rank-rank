//! Integration tests for rank-retrieve.

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::sparse::SparseRetriever;
use rank_retrieve::RetrieveError;
use rank_sparse::SparseVector;

#[test]
fn bm25_retrieval_workflow() {
    let mut index = InvertedIndex::new();
    
    index.add_document(0, &["machine".to_string(), "learning".to_string(), "algorithms".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string(), "neural".to_string(), "networks".to_string()]);
    index.add_document(2, &["information".to_string(), "retrieval".to_string(), "search".to_string()]);
    
    let query = vec!["learning".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    
    assert_eq!(results.len(), 2);
    assert!(results[0].0 == 0 || results[0].0 == 1);
    assert!(results[1].0 == 0 || results[1].0 == 1);
    assert!(results[0].1 > 0.0);
}

#[test]
fn dense_retrieval_workflow() {
    let mut retriever = DenseRetriever::new();
    
    retriever.add_document(0, vec![0.8, 0.6, 0.0]);
    retriever.add_document(1, vec![0.0, 0.6, 0.8]);
    retriever.add_document(2, vec![0.6, 0.8, 0.0]);
    
    let query = vec![0.8, 0.6, 0.0];
    let results = retriever.retrieve(&query, 10).unwrap();
    
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > results[1].1);
}

#[test]
fn sparse_retrieval_workflow() {
    let mut retriever = SparseRetriever::new();
    
    let doc0 = SparseVector::new(vec![0, 1, 2], vec![1.0, 0.8, 0.6]).unwrap();
    let doc1 = SparseVector::new(vec![2, 3, 4], vec![0.6, 0.8, 1.0]).unwrap();
    let doc2 = SparseVector::new(vec![0, 2, 4], vec![0.5, 0.7, 0.9]).unwrap();
    
    retriever.add_document(0, doc0);
    retriever.add_document(1, doc1);
    retriever.add_document(2, doc2);
    
    let query = SparseVector::new(vec![0, 1, 2], vec![1.0, 1.0, 1.0]).unwrap();
    let results = retriever.retrieve(&query, 10).unwrap();
    
    assert_eq!(results.len(), 3);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > results[1].1);
}

#[test]
fn error_handling_empty_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    
    let result = index.retrieve(&[], 10, Bm25Params::default());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyQuery));
}

#[test]
fn error_handling_empty_index() {
    let index = InvertedIndex::new();
    let result = index.retrieve(&["test".to_string()], 10, Bm25Params::default());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::EmptyIndex));
}

#[test]
fn error_handling_dimension_mismatch() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    
    let result = retriever.retrieve(&[1.0, 0.0, 0.0], 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), RetrieveError::DimensionMismatch { .. }));
}

