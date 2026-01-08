//! Batch retrieval operations.
//!
//! Provides efficient batch processing for multiple queries, enabling better
//! performance through vectorization and parallelization opportunities.
//!
//! # Example
//!
//! ```rust
//! use rank_retrieve::batch::batch_retrieve_bm25;
//! use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
//!
//! let mut index = InvertedIndex::new();
//! index.add_document(0, &["machine".to_string(), "learning".to_string()]);
//!
//! let queries = vec![
//!     vec!["machine".to_string()],
//!     vec!["learning".to_string()],
//! ];
//!
//! let results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();
//! assert_eq!(results.len(), 2);
//! ```

#[cfg(feature = "bm25")]
use crate::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use crate::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use crate::sparse::{SparseRetriever, SparseVector};
use crate::RetrieveError;

/// Retrieve top-k documents for multiple queries using BM25.
///
/// Processes queries in batch, which can be more efficient than individual
/// retrievals for large numbers of queries.
///
/// # Arguments
///
/// * `index` - BM25 inverted index
/// * `queries` - Vector of query term vectors
/// * `k` - Number of documents to retrieve per query
/// * `params` - BM25 parameters
///
/// # Returns
///
/// Vector of results, one per query. Each result is a vector of (document_id, score) pairs.
///
/// Available when `bm25` feature is enabled.
#[cfg(feature = "bm25")]
pub fn batch_retrieve_bm25(
    index: &InvertedIndex,
    queries: &[Vec<String>],
    k: usize,
    params: Bm25Params,
) -> Result<Vec<Vec<(u32, f32)>>, RetrieveError> {
    let mut results = Vec::with_capacity(queries.len());
    
    for query in queries {
        let result = index.retrieve(query, k, params)?;
        results.push(result);
    }
    
    Ok(results)
}

/// Retrieve top-k documents for multiple queries using dense retrieval.
///
/// # Arguments
///
/// * `retriever` - Dense retriever
/// * `queries` - Vector of query embedding vectors
/// * `k` - Number of documents to retrieve per query
///
/// # Returns
///
/// Vector of results, one per query. Each result is a vector of (document_id, score) pairs.
///
/// Available when `dense` feature is enabled.
#[cfg(feature = "dense")]
pub fn batch_retrieve_dense(
    retriever: &DenseRetriever,
    queries: &[Vec<f32>],
    k: usize,
) -> Result<Vec<Vec<(u32, f32)>>, RetrieveError> {
    let mut results = Vec::with_capacity(queries.len());
    
    for query in queries {
        let result = retriever.retrieve(query, k)?;
        results.push(result);
    }
    
    Ok(results)
}

/// Retrieve top-k documents for multiple queries using sparse retrieval.
///
/// # Arguments
///
/// * `retriever` - Sparse retriever
/// * `queries` - Vector of query sparse vectors
/// * `k` - Number of documents to retrieve per query
///
/// # Returns
///
/// Vector of results, one per query. Each result is a vector of (document_id, score) pairs.
///
/// Available when `sparse` feature is enabled.
#[cfg(feature = "sparse")]
pub fn batch_retrieve_sparse(
    retriever: &SparseRetriever,
    queries: &[SparseVector],
    k: usize,
) -> Result<Vec<Vec<(u32, f32)>>, RetrieveError> {
    let mut results = Vec::with_capacity(queries.len());
    
    for query in queries {
        let result = retriever.retrieve(query, k)?;
        results.push(result);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "bm25")]
    #[test]
    fn test_batch_retrieve_bm25() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);

        let queries = vec![
            vec!["machine".to_string()],
            vec!["artificial".to_string()],
        ];

        let results = batch_retrieve_bm25(&index, &queries, 10, Bm25Params::default()).unwrap();
        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
    }

    #[cfg(feature = "dense")]
    #[test]
    fn test_batch_retrieve_dense() {
        let mut retriever = DenseRetriever::new();
        retriever.add_document(0, vec![1.0, 0.0]);
        retriever.add_document(1, vec![0.0, 1.0]);

        let queries = vec![
            vec![1.0, 0.0],
            vec![0.0, 1.0],
        ];

        let results = batch_retrieve_dense(&retriever, &queries, 10).unwrap();
        assert_eq!(results.len(), 2);
        assert!(!results[0].is_empty());
        assert!(!results[1].is_empty());
    }

    #[cfg(feature = "sparse")]
    #[test]
    fn test_batch_retrieve_sparse() {
        let mut retriever = SparseRetriever::new();
        let doc0 = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]);
        retriever.add_document(0, doc0);

        let queries = vec![
            SparseVector::new_unchecked(vec![0], vec![1.0]),
            SparseVector::new_unchecked(vec![1], vec![1.0]),
        ];

        let results = batch_retrieve_sparse(&retriever, &queries, 10).unwrap();
        assert_eq!(results.len(), 2);
    }
}

