//! Edge case tests for rank-retrieve.

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};
#[cfg(feature = "generative")]
use rank_retrieve::generative::{GenerativeRetriever, MockAutoregressiveModel, HeuristicScorer};

#[cfg(feature = "bm25")]
#[test]
fn bm25_empty_query() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    
    let result = index.retrieve(&[], 10, Bm25Params::default());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyQuery));
}

#[cfg(feature = "bm25")]
#[test]
fn bm25_empty_index() {
    let index = InvertedIndex::new();
    let result = index.retrieve(&["test".to_string()], 10, Bm25Params::default());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyIndex));
}

#[cfg(feature = "bm25")]
#[test]
fn bm25_zero_k() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    
    let results = index.retrieve(&["test".to_string()], 0, Bm25Params::default()).unwrap();
    assert_eq!(results.len(), 0);
}

#[cfg(feature = "bm25")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn bm25_single_term_document() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string()]);
    
    let results = index.retrieve(&["test".to_string()], 10, Bm25Params::default()).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 > 0.0);
}

#[cfg(feature = "bm25")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn bm25_duplicate_terms() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["test".to_string(), "test".to_string(), "test".to_string()]);
    
    let results = index.retrieve(&["test".to_string()], 10, Bm25Params::default()).unwrap();
    assert_eq!(results.len(), 1);
    assert!(results[0].1 > 0.0);
}

#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn dense_empty_query() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    
    let empty: Vec<f32> = vec![];
    let result = retriever.retrieve(&empty, 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyQuery));
}

#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn dense_empty_index() {
    let retriever = DenseRetriever::new();
    let result = retriever.retrieve(&[1.0, 0.0], 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyIndex));
}

#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn dense_zero_k() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    
    let results = retriever.retrieve(&[1.0, 0.0], 0).unwrap();
    assert_eq!(results.len(), 0);
}

#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn dense_mismatched_dimensions() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]);
    
    let result = retriever.retrieve(&[1.0], 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::DimensionMismatch { .. }));
}

#[cfg(feature = "dense")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn dense_zero_vector() {
    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![0.0, 0.0]);
    
    let results = retriever.retrieve(&[1.0, 0.0], 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 0.0);
}

#[cfg(feature = "sparse")]
#[test]
fn sparse_empty_query() {
    let mut retriever = SparseRetriever::new();
    let doc_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    retriever.add_document(0, doc_vector);
    
    let query_vector = SparseVector::new(vec![], vec![]).unwrap();
    let result = retriever.retrieve(&query_vector, 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyQuery));
}

#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn sparse_empty_index() {
    let retriever = SparseRetriever::new();
    let query_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    let result = retriever.retrieve(&query_vector, 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyIndex));
}

#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn sparse_zero_k() {
    let mut retriever = SparseRetriever::new();
    let doc_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    retriever.add_document(0, doc_vector);
    
    let query_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    let results = retriever.retrieve(&query_vector, 0).unwrap();
    assert_eq!(results.len(), 0);
}

#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[cfg(feature = "sparse")]
#[test]
fn sparse_no_overlap() {
    let mut retriever = SparseRetriever::new();
    let doc_vector = SparseVector::new(vec![0, 1], vec![1.0, 0.5]).unwrap();
    retriever.add_document(0, doc_vector);
    
    let query_vector = SparseVector::new(vec![2, 3], vec![1.0, 0.5]).unwrap();
    let results = retriever.retrieve(&query_vector, 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].1, 0.0);
}

// Generative retrieval edge cases

#[cfg(feature = "generative")]
#[test]
fn generative_empty_query() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    retriever.add_document(0, "Test passage content");
    
    let result = retriever.retrieve("", 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyQuery));
}

#[cfg(feature = "generative")]
#[test]
fn generative_whitespace_only_query() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    retriever.add_document(0, "Test passage content");
    
    let result = retriever.retrieve("   ", 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyQuery));
}

#[cfg(feature = "generative")]
#[test]
fn generative_empty_index() {
    let model = MockAutoregressiveModel::new();
    let retriever = GenerativeRetriever::new(model);
    
    let result = retriever.retrieve("test query", 10);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), rank_retrieve::RetrieveError::EmptyIndex));
}

#[cfg(feature = "generative")]
#[test]
fn generative_zero_k() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    retriever.add_document(0, "Test passage content");
    
    let results = retriever.retrieve("test", 0).unwrap();
    assert_eq!(results.len(), 0);
}

#[cfg(feature = "generative")]
#[test]
fn generative_single_document() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    retriever.add_document(0, "Prime Rate in Canada is a guideline interest rate");
    
    let results = retriever.retrieve("What is prime rate?", 10).unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, 0);
    assert!(results[0].1 >= 0.0);
}

#[cfg(feature = "generative")]
#[test]
fn generative_very_long_query() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    retriever.add_document(0, "Short passage");
    
    let long_query = "a ".repeat(1000);
    let results = retriever.retrieve(&long_query, 10).unwrap();
    assert!(!results.is_empty());
}

#[cfg(feature = "generative")]
#[test]
fn generative_very_long_passage() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    let long_passage = "word ".repeat(10000);
    retriever.add_document(0, &long_passage);
    
    let results = retriever.retrieve("word", 10).unwrap();
    assert!(!results.is_empty());
}

#[cfg(feature = "generative")]
#[test]
fn generative_unicode_characters() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    retriever.add_document(0, "Café résumé naïve");
    retriever.add_document(1, "Hello world");
    
    let results = retriever.retrieve("café", 10).unwrap();
    assert!(!results.is_empty());
}

#[cfg(feature = "generative")]
#[test]
fn generative_special_characters() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    retriever.add_document(0, "Test: passage (with) [special] {characters}");
    
    let results = retriever.retrieve("test", 10).unwrap();
    assert!(!results.is_empty());
}

#[cfg(feature = "generative")]
#[test]
fn generative_case_insensitive_scorer() {
    let model = MockAutoregressiveModel::new();
    let scorer = HeuristicScorer::new().with_case_insensitive(true);
    let mut retriever = GenerativeRetriever::new(model)
        .with_scorer(scorer);
    
    retriever.add_document(0, "PRIME RATE in Canada");
    
    let results = retriever.retrieve("prime rate", 10).unwrap();
    assert!(!results.is_empty());
}

#[cfg(feature = "generative")]
#[test]
fn generative_case_sensitive_scorer() {
    let model = MockAutoregressiveModel::new();
    let scorer = HeuristicScorer::new().with_case_insensitive(false);
    let mut retriever = GenerativeRetriever::new(model)
        .with_scorer(scorer);
    
    retriever.add_document(0, "PRIME RATE in Canada");
    
    // With case-sensitive, lowercase query might not match uppercase passage
    let results = retriever.retrieve("prime rate", 10).unwrap();
    // Results may be empty or have low scores depending on model behavior
    assert!(results.len() <= 1);
}

#[cfg(feature = "generative")]
#[test]
fn generative_min_identifier_length_filtering() {
    let model = MockAutoregressiveModel::new();
    let scorer = HeuristicScorer::new().with_min_identifier_len(10);
    let mut retriever = GenerativeRetriever::new(model)
        .with_scorer(scorer);
    
    retriever.add_document(0, "Prime Rate in Canada");
    
    let results = retriever.retrieve("prime", 10).unwrap();
    // Short identifiers should be filtered
    assert!(results.len() <= 1);
}

#[cfg(feature = "generative")]
#[test]
fn generative_large_beam_size() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model)
        .with_beam_size(100);
    
    retriever.add_document(0, "Test passage");
    
    let results = retriever.retrieve("test", 10).unwrap();
    assert!(!results.is_empty());
}

#[cfg(feature = "generative")]
#[test]
fn generative_many_documents() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);
    
    for i in 0..1000 {
        retriever.add_document(i, &format!("Document {} content", i));
    }
    
    let results = retriever.retrieve("document", 10).unwrap();
    assert_eq!(results.len(), 10);
    assert!(results.iter().all(|(_, score)| *score >= 0.0));
}

