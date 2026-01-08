//! Comprehensive tests for the unified Retriever trait interface.
//!
//! These tests verify that:
//! 1. All implementations correctly implement the Retriever trait
//! 2. Polymorphic code works with any retriever
//! 3. Feature flags work correctly (trait available even without implementations)

use rank_retrieve::retriever::Retriever;
use rank_retrieve::RetrieveError;

/// Generic function that works with any retriever.
fn search<R: Retriever>(
    retriever: &R,
    query: &R::Query,
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    retriever.retrieve(query, k)
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_trait_interface() {
    use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
    use rank_retrieve::retriever::RetrieverBuilder;

    let mut index = InvertedIndex::new();
    index
        .add_document(0, vec!["test".to_string(), "document".to_string()])
        .unwrap();
    index
        .add_document(1, vec!["another".to_string(), "test".to_string()])
        .unwrap();

    let query = vec!["test".to_string()];
    let results = search(&index, &query, 10).unwrap();

    assert!(!results.is_empty());
    assert!(results.iter().any(|(id, _)| *id == 0 || *id == 1));
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_trait_interface() {
    use rank_retrieve::dense::DenseRetriever;
    use rank_retrieve::retriever::RetrieverBuilder;

    let mut retriever = DenseRetriever::new();
    retriever.add_document(0, vec![1.0, 0.0]).unwrap();
    retriever.add_document(1, vec![0.0, 1.0]).unwrap();

    let query = [1.0, 0.0];
    let results = search(&retriever, &query, 10).unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].0, 0);
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_trait_interface() {
    use rank_retrieve::retriever::RetrieverBuilder;
    use rank_retrieve::sparse::{SparseRetriever, SparseVector};

    let mut retriever = SparseRetriever::new();
    let doc0 = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]);
    retriever.add_document(0, doc0).unwrap();

    let query = SparseVector::new_unchecked(vec![0], vec![1.0]);
    let results = search(&retriever, &query, 10).unwrap();

    assert!(!results.is_empty());
    assert_eq!(results[0].0, 0);
}

#[cfg(all(feature = "bm25", feature = "dense"))]
#[test]
fn test_polymorphic_hybrid_search() {
    use rank_retrieve::bm25::InvertedIndex;
    use rank_retrieve::dense::DenseRetriever;
    use rank_retrieve::retriever::RetrieverBuilder;

    // Setup BM25 retriever
    let mut bm25 = InvertedIndex::new();
    bm25.add_document(0, vec!["machine".to_string(), "learning".to_string()])
        .unwrap();

    // Setup dense retriever
    let mut dense = DenseRetriever::new();
    dense.add_document(0, vec![1.0, 0.0]).unwrap();

    // Both can be used polymorphically
    let bm25_query = vec!["machine".to_string()];
    let bm25_results = search(&bm25, &bm25_query, 10).unwrap();

    let dense_query = [1.0, 0.0];
    let dense_results = search(&dense, &dense_query, 10).unwrap();

    assert!(!bm25_results.is_empty());
    assert!(!dense_results.is_empty());
}

#[test]
fn test_trait_available_without_implementations() {
    // This test verifies that the trait is available even when no implementation
    // features are enabled. This is important for users who want to implement
    // their own retrievers or integrate with external backends.

    // The trait should be importable
    use rank_retrieve::retriever::Retriever;

    // We can define functions that use the trait
    fn _generic_search<R: Retriever>(
        _retriever: &R,
        _query: &R::Query,
        _k: usize,
    ) -> Result<Vec<(u32, f32)>, RetrieveError> {
        // This would work with any retriever implementation
        Ok(vec![])
    }

    // Test passes if trait is available
    assert!(true);
}
