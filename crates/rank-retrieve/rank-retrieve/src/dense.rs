//! Dense approximate nearest neighbor search.
//!
//! Provides interface for dense vector retrieval using ANN algorithms.
//!
//! This module provides a unified API for dense retrieval, delegating to
//! existing ANN libraries (HNSW, FAISS) when available.
//!
//! # Design
//!
//! The dense retriever stores document embeddings and provides:
//! - Indexing: Add documents with their dense embeddings
//! - Retrieval: Find nearest neighbors to a query embedding
//!
//! For production use, integrate with:
//! - HNSW (Hierarchical Navigable Small World) - fast, approximate
//! - FAISS (Facebook AI Similarity Search) - highly optimized
//! - IVF (Inverted File Index) - memory efficient

use crate::RetrieveError;

/// Dense retriever using cosine similarity.
///
/// Simple implementation using brute-force cosine similarity.
/// For large-scale use, replace with HNSW or FAISS.
pub struct DenseRetriever {
    /// Document ID -> Embedding vector
    documents: Vec<(u32, Vec<f32>)>,
}

impl DenseRetriever {
    /// Create a new dense retriever.
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }
    
    /// Add a document with its dense embedding.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document identifier
    /// * `embedding` - Dense embedding vector (should be L2-normalized for cosine similarity)
    pub fn add_document(&mut self, doc_id: u32, embedding: Vec<f32>) {
        self.documents.push((doc_id, embedding));
    }
    
    /// Compute cosine similarity between two vectors.
    ///
    /// Assumes vectors are L2-normalized (unit length).
    /// For normalized vectors, cosine = dot product.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }
    
    /// Score a document against a query using cosine similarity.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document to score
    /// * `query_embedding` - Query embedding vector
    ///
    /// # Returns
    ///
    /// Cosine similarity score (higher = more relevant)
    pub fn score(&self, doc_id: u32, query_embedding: &[f32]) -> Option<f32> {
        self.documents
            .iter()
            .find(|(id, _)| *id == doc_id)
            .map(|(_, doc_embedding)| Self::cosine_similarity(doc_embedding, query_embedding))
    }
    
    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    ///
    /// * `query_embedding` - Query embedding vector
    /// * `k` - Number of documents to retrieve
    ///
    /// # Returns
    ///
    /// Vector of (document_id, score) pairs, sorted by score descending
    ///
    /// # Errors
    ///
    /// Returns `RetrieveError::EmptyQuery` if query is empty.
    /// Returns `RetrieveError::EmptyIndex` if index has no documents.
    /// Returns `RetrieveError::DimensionMismatch` if query dimension doesn't match document dimensions.
    pub fn retrieve(&self, query_embedding: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if query_embedding.is_empty() {
            return Err(RetrieveError::EmptyQuery);
        }
        
        if self.documents.is_empty() {
            return Err(RetrieveError::EmptyIndex);
        }
        
        let query_dim = query_embedding.len();
        let mut scored: Vec<(u32, f32)> = Vec::new();
        
        for (doc_id, doc_embedding) in &self.documents {
            if doc_embedding.len() != query_dim {
                return Err(RetrieveError::DimensionMismatch {
                    query_dim,
                    doc_dim: doc_embedding.len(),
                });
            }
            let score = Self::cosine_similarity(doc_embedding, query_embedding);
            scored.push((*doc_id, score));
        }
        
        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top-k
        Ok(scored.into_iter().take(k).collect())
    }
}

impl Default for DenseRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_dense_retrieval() {
        let mut retriever = DenseRetriever::new();
        
        // Document 0: [1.0, 0.0] (normalized)
        retriever.add_document(0, vec![1.0, 0.0]);
        
        // Document 1: [0.707, 0.707] (normalized)
        retriever.add_document(1, vec![0.707, 0.707]);
        
        // Query: [1.0, 0.0]
        let query = vec![1.0, 0.0];
        
        let results = retriever.retrieve(&query, 10);
        
        // Document 0 should score 1.0 (exact match)
        // Document 1 should score 0.707 (cosine similarity)
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0);
        assert!((results[0].1 - 1.0).abs() < 0.001);
        assert!((results[1].1 - 0.707).abs() < 0.01);
    }
}

