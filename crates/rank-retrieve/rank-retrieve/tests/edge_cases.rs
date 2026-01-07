//! Edge case tests for rank-retrieve.

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::sparse::SparseRetriever;
use rank_sparse::SparseVector;

#[test]
fn bm25_empty_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    
    let results = index.retrieve(&[], 10, Bm25Params::default());
    assert_eq!(results.len(), 0);
}

#[test]
fn bm25_empty_index() {
    let index = InvertedIndex::new();
    let results = index.retrieve(&["test".to_string()], 10, Bm25Params::default());
    assert_eq!(results.len(), 0);
}

#[test]
fn bm25_zero_k() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    
    let results = index.retrieve(&["test".to_string()], 0, Bm25Params::default());
    assert_eq!(results.len(), 0);
}

#[test]
fn bm25_single_term_document() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    
    let results = index.retrieve(&["test".to_string()], 10, Bm25Params::default());
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > 0.0);
}

#[test]
fn bm25_duplicate_terms() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string(), "test".to_string(), "test".to_string()]);
    
    let results = index.retrieve(&["test".to_string()], 10, Bm25Params::default());
    assert_eq!(results.len(), 1);
    assert!(results[0].1 > 0.0);
}

#[test]
fn dense_empty_query() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    
    let empty: Vec<f32> = vec![];
    let results = retriever.retrieve(&empty, 10);
    assert_eq!(results.len(), 0);
}

#[test]
fn dense_empty_index() {
    let retriever = DenseRetriever::new();
    let results = retriever.retrieve(&[1.0, 0.0], 10);
    assert_eq!(results.len(), 0);
}

#[test]
fn dense_zero_k() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    
    let results = retriever.retrieve(&[1.0, 0.0], 0);
    assert_eq!(results.len(), 0);
}

#[test]
fn dense_mismatched_dimensions() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    
    let results = retriever.retrieve(&[1.0], 10);
    assert_eq!(results.len(), 0);
}

#[test]
fn dense_zero_vector() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![0.0, 0.0]);
    
    let results = retriever.retrieve(&[1.0, 0.0], 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 0.0);
}

#[test]
fn sparse_empty_query() {
    let mut retriever = SparseRetriever::new();
    let doc_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    retriever.add_document(0, doc_vector);
    
    let query_vector = SparseVector::new(vec![], vec![]).unwrap();
    let results = retriever.retrieve(&query_vector, 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 0.0);
}

#[test]
fn sparse_empty_index() {
    let retriever = SparseRetriever::new();
    let query_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    let results = retriever.retrieve(&query_vector, 10);
    assert_eq!(results.len(), 0);
}

#[test]
fn sparse_zero_k() {
    let mut retriever = SparseRetriever::new();
    let doc_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    retriever.add_document(0, doc_vector);
    
    let query_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    let results = retriever.retrieve(&query_vector, 0);
    assert_eq!(results.len(), 0);
}

#[test]
fn sparse_no_overlap() {
    let mut retriever = SparseRetriever::new();
    let doc_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    retriever.add_document(0, doc_vector);
    
    let query_vector = SparseVector::new(vec![2, 3], vec![1.0, 0.5]).unwrap();
    let results = retriever.retrieve(&query_vector, 10);
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 0.0);
}

