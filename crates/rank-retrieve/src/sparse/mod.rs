//! Sparse retrieval module.
//!
//! Sparse retrieval using sparse vector dot products for lexical matching.
//!
//! Sparse retrieval uses sparse vectors where:
//! - Indices represent term IDs (vocabulary positions)
//! - Values represent term weights (e.g., TF-IDF, BM25)
//!
//! This enables efficient dot product computation for large vocabularies.

mod vector;

use crate::RetrieveError;

// Re-export SparseVector and dot_product for convenience
pub use self::vector::{dot_product, SparseVector};

/// Sparse retriever using sparse vector dot products.
///
/// # Performance Optimizations
///
/// - **Early termination**: Uses min-heap for k << num_documents (avoids full sort)
/// - **SIMD acceleration**: Sparse dot product uses SIMD for index comparison
/// - **Cache-friendly**: Documents stored in Vec for better cache locality
///
/// # Memory Characteristics
///
/// - Stores sparse vectors (only non-zero terms)
/// - Memory usage: O(|D| × avg_sparsity × vocab_size)
/// - For learned sparse (SPLADE): ~30K dimensions per document
pub struct SparseRetriever {
    /// Document ID -> Sparse Vector
    /// Using Vec for better cache locality and iteration performance
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
    ///
    /// # Performance
    ///
    /// O(1) amortized. The vector is stored as-is; no preprocessing required.
    /// For learned sparse vectors (SPLADE), consider using `top_k()` to reduce
    /// memory usage (typically keep top 200-500 terms).
    pub fn add_document(&mut self, doc_id: u32, vector: SparseVector) {
        self.documents.push((doc_id, vector));
    }

    /// Get the number of documents in the index.
    pub fn num_docs(&self) -> usize {
        self.documents.len()
    }

    /// Get a document's sparse vector by ID.
    pub fn get_document(&self, doc_id: u32) -> Option<&SparseVector> {
        self.documents
            .iter()
            .find(|(id, _)| *id == doc_id)
            .map(|(_, vector)| vector)
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
    ///
    /// # Performance
    ///
    /// Uses early termination optimization: maintains top-k heap during scoring
    /// to avoid full sort when k << num_documents. For k >= num_documents, uses
    /// full sort (more efficient for large k).
    pub fn retrieve(
        &self,
        query_vector: &SparseVector,
        k: usize,
    ) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if query_vector.indices.is_empty() {
            return Err(RetrieveError::EmptyQuery);
        }

        if self.documents.is_empty() {
            return Err(RetrieveError::EmptyIndex);
        }

        // Handle k=0 case
        if k == 0 {
            return Ok(Vec::new());
        }

        // Early termination optimization: use heap for k << num_documents
        if k < self.documents.len() / 2 {
            // Use min-heap for top-k (more efficient for small k)
            // Note: f32 doesn't implement Ord due to NaN, so we use a wrapper
            use std::cmp::Reverse;
            use std::collections::BinaryHeap;

            #[derive(PartialEq)]
            struct FloatOrd(f32);
            impl Eq for FloatOrd {}
            impl PartialOrd for FloatOrd {
                fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                    Some(self.cmp(other))
                }
            }
            impl Ord for FloatOrd {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
                }
            }

            let mut heap: BinaryHeap<Reverse<(FloatOrd, u32)>> = BinaryHeap::with_capacity(k + 1);

            for (doc_id, doc_vector) in &self.documents {
                let score = dot_product(query_vector, doc_vector);
                
                // Filter out NaN, Infinity, and non-positive scores
                if score.is_finite() && score > 0.0 {
                    if heap.len() < k {
                        heap.push(Reverse((FloatOrd(score), *doc_id)));
                    } else if let Some(&Reverse((FloatOrd(min_score), _))) = heap.peek() {
                        if score > min_score {
                            heap.pop();
                            heap.push(Reverse((FloatOrd(score), *doc_id)));
                        }
                    }
                }
            }

            // Extract and reverse sort
            let mut results: Vec<(u32, f32)> = heap
                .into_iter()
                .map(|Reverse((FloatOrd(score), doc_id))| (doc_id, score))
                .collect();
            results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        } else {
            // Full sort for large k (more efficient)
            let mut scored: Vec<(u32, f32)> = self
                .documents
                .iter()
                .map(|(doc_id, doc_vector)| {
                    let score = dot_product(query_vector, doc_vector);
                    (*doc_id, score)
                })
                .filter(|(_, score)| score.is_finite() && *score > 0.0)
                .collect();

            // Sort by score descending
            scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Return top-k
            Ok(scored.into_iter().take(k).collect())
        }
    }
}

impl Default for SparseRetriever {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "sparse")]
impl crate::retriever::Retriever for SparseRetriever {
    type Query = SparseVector;

    fn retrieve(&self, query: &Self::Query, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        // Use fully qualified syntax to call the inherent method, not the trait method
        SparseRetriever::retrieve(self, query, k)
    }
}

#[cfg(feature = "sparse")]
impl crate::retriever::RetrieverBuilder for SparseRetriever {
    type Content = SparseVector;

    fn add_document(&mut self, doc_id: u32, content: Self::Content) -> Result<(), RetrieveError> {
        self.add_document(doc_id, content);
        Ok(())
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

        let results = retriever.retrieve(&query, 10).unwrap();

        // Document 0 should score higher (has term 0 with weight 1.0)
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0); // doc 0 should be first
        assert!(results[0].1 > results[1].1);
    }
}
