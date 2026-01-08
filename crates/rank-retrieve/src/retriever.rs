//! Unified retriever trait interface.
//!
//! This module defines the core trait that all retrieval methods implement.
//! The trait is always available, while specific implementations are
//! feature-gated to keep the core lightweight.
//!
//! # Design Philosophy
//!
//! The `Retriever` trait provides a unified interface for all retrieval methods:
//! - BM25 (lexical matching)
//! - Dense (semantic similarity)
//! - Sparse (sparse vector matching)
//! - Generative (identifier-based)
//!
//! This enables:
//! - Easy switching between methods
//! - Hybrid search (combining multiple methods)
//! - Polymorphic code that works with any retriever
//! - Integration with `rank-fusion` and `rank-rerank`
//!
//! # Usage
//!
//! The trait is deprecated in favor of concrete functions. See `rank_retrieve::retrieve_bm25`,
//! `rank_retrieve::retrieve_dense`, etc. for the primary API.
//!
//! # Feature-Gated Implementations
//!
//! Specific implementations are available via feature flags:
//! - `bm25`: BM25 retrieval (`InvertedIndex`)
//! - `dense`: Dense retrieval (`DenseRetriever`)
//! - `sparse`: Sparse retrieval (`SparseRetriever`)
//! - `generative`: Generative retrieval (`GenerativeRetriever`)
//!
//! The trait itself is always available, allowing users to implement their own
//! retrievers or integrate with external backends.

use crate::RetrieveError;

/// Unified trait for all retrieval methods.
///
/// This trait provides a common interface for BM25, dense, sparse, and generative
/// retrieval. All retrieval methods return `Vec<(u32, f32)>` where:
/// - `u32` is the document ID
/// - `f32` is the relevance score (higher is better)
///
/// # Type Parameters
///
/// The trait is generic over query type `Q` to support different query formats:
/// - `&str` or `String` for text queries (BM25, generative)
/// - `&[f32]` for dense embeddings
/// - `SparseVector` for sparse queries
///
/// # Implementations
///
/// Implementations are feature-gated:
/// - `bm25`: `InvertedIndex` implements `Retriever<Query = Vec<String>>`
/// - `dense`: `DenseRetriever` implements `Retriever<Query = &[f32]>`
/// - `sparse`: `SparseRetriever` implements `Retriever<Query = SparseVector>`
/// - `generative`: `GenerativeRetriever` implements `Retriever<Query = &str>`
pub trait Retriever {
    /// Query type for this retriever.
    type Query: ?Sized;

    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query (type depends on retriever: text, embedding, sparse vector)
    /// * `k` - Number of top documents to retrieve
    ///
    /// # Returns
    ///
    /// Vector of `(document_id, score)` pairs sorted by score descending.
    /// Returns error if retrieval fails (empty index, invalid query, etc.).
    fn retrieve(&self, query: &Self::Query, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError>;
}

/// Trait for retrievers that can add documents.
///
/// Not all retrievers support adding documents after construction
/// (e.g., some backends require building the index first).
pub trait RetrieverBuilder {
    /// Add a document to the index.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document identifier
    /// * `content` - Document content (type depends on retriever)
    ///
    /// # Returns
    ///
    /// Error if document addition fails (duplicate ID, invalid content, etc.).
    fn add_document(&mut self, doc_id: u32, content: Self::Content) -> Result<(), RetrieveError>;

    /// Content type for this retriever builder.
    type Content;

    /// Build/finalize the index (optional, some retrievers don't need this).
    ///
    /// Some retrievers require a build step before retrieval (e.g., HNSW index construction).
    /// Others are ready immediately after adding documents.
    fn build(&mut self) -> Result<(), RetrieveError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock retriever for testing the trait interface.
    struct MockRetriever {
        results: Vec<(u32, f32)>,
    }

    impl Retriever for MockRetriever {
        type Query = &'static str;

        fn retrieve(&self, _query: &Self::Query, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
            Ok(self.results.iter().take(k).copied().collect())
        }
    }

    #[test]
    fn test_retriever_trait() {
        let retriever = MockRetriever {
            results: vec![(1, 0.9), (2, 0.8), (3, 0.7)],
        };

        let results = retriever.retrieve(&"test", 2).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (1, 0.9));
        assert_eq!(results[1], (2, 0.8));
    }

    #[test]
    fn test_polymorphic_retrieval() {
        fn search<R: Retriever<Query = &'static str>>(
            retriever: &R,
            query: &'static str,
            k: usize,
        ) -> Result<Vec<(u32, f32)>, RetrieveError> {
            retriever.retrieve(&query, k)
        }

        let retriever = MockRetriever {
            results: vec![(1, 0.9), (2, 0.8)],
        };

        let results = search(&retriever, "test", 2).unwrap();
        assert_eq!(results.len(), 2);
    }
}

