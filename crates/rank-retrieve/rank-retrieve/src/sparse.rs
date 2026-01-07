//! Sparse retrieval module.
//!
//! Uses rank-sparse for sparse vector operations in lexical matching.
//!
//! Sparse retrieval uses sparse vectors where:
//! - Indices represent term IDs (vocabulary positions)
//! - Values represent term weights (e.g., TF-IDF, BM25)
//!
//! This enables efficient dot product computation for large vocabularies.

use rank_sparse::{SparseVector, dot_product};
use crate::RetrieveError;

/// Sparse retriever using sparse vector dot products.
pub struct SparseRetriever {
    /// Document ID -> Sparse Vector
    documents: Vec<(u32, SparseVector)>,
}

impl SparseRetriever {
    /// Create a new sparse retriever.
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }
    
    /// Add a document with its sparse vector representation.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document identifier
    /// * `vector` - Sparse vector representation (indices = term IDs, values = term weights)
    pub fn add_document(&mut self, doc_id: u32, vector: SparseVector) {
        self.documents.push((doc_id, vector));
    }
    
    /// Score a document against a query using dot product.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document to score
    /// * `query_vector` - Sparse vector representation of query
    ///
    /// # Returns
    ///
    /// Dot product score (higher = more relevant)
    pub fn score(&self, doc_id: u32, query_vector: &SparseVector) -> Option<f32> {
        self.documents
            .iter()
            .find(|(id, _)| *id == doc_id)
            .map(|(_, doc_vector)| dot_product(query_vector, doc_vector))
    }
    
    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    ///
    /// * `query_vector` - Sparse vector representation of query
    /// * `k` - Number of documents to retrieve
    ///
    /// # Returns
    ///
    /// Vector of (document_id, score) pairs, sorted by score descending
    ///
    /// # Errors
    ///
    /// Returns `RetrieveError::EmptyQuery` if query vector is empty.
    /// Returns `RetrieveError::EmptyIndex` if index has no documents.
    pub fn retrieve(&self, query_vector: &SparseVector, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if query_vector.indices.is_empty() {
            return Err(RetrieveError::EmptyQuery);
        }
        
        if self.documents.is_empty() {
            return Err(RetrieveError::EmptyIndex);
        }
        
        let mut scored: Vec<(u32, f32)> = self.documents
            .iter()
            .map(|(doc_id, doc_vector)| {
                let score = dot_product(query_vector, doc_vector);
                (*doc_id, score)
            })
            .collect();
        
        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top-k
        Ok(scored.into_iter().take(k).collect())
    }
}

impl Default for SparseRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sparse_retrieval() {
        let mut retriever = SparseRetriever::new();
        
        // Document 0: terms 0, 1, 2 with weights 1.0, 0.5, 0.3
        let doc0 = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
        retriever.add_document(0, doc0);
        
        // Document 1: terms 1, 2, 3 with weights 0.8, 0.6, 0.4
        let doc1 = SparseVector::new_unchecked(vec![1, 2, 3], vec![0.8, 0.6, 0.4]);
        retriever.add_document(1, doc1);
        
        // Query: terms 0, 1 with weights 1.0, 1.0
        let query = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
        
        let results = retriever.retrieve(&query, 10);
        
        // Document 0 should score higher (has term 0 with weight 1.0)
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // doc 0 should be first
        assert!(results[0].1 > results[1].1);
    }
}

