//! Trait interface for external backend integrations.
//!
//! This module provides a trait that external backends can implement to integrate
//! with rank-retrieve. It does not include full implementations of specific backends.
//!
//! # Design Philosophy
//!
//! rank-retrieve focuses on basic retrieval implementations (BM25, dense, sparse).
//! For large scale, users should integrate with specialized libraries (Tantivy, HNSW,
//! FAISS, Qdrant, Pinecone, etc.) by implementing this trait.
//!
//! This keeps rank-retrieve lightweight and avoids maintaining many backend
//! implementations, each with different APIs and dependencies.
//!
//! # Usage
//!
//! Implement the `Backend` trait for your chosen backend:
//!
//! ```rust,no_run
//! use rank_retrieve::integration::Backend;
//! use rank_retrieve::RetrieveError;
//!
//! struct MyBackend {
//!     // Your backend implementation
//! }
//!
//! impl Backend for MyBackend {
//!     fn retrieve(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
//!         // Your implementation
//!         Ok(vec![])
//!     }
//!
//!     fn add_document(&mut self, doc_id: u32, embedding: &[f32]) -> Result<(), RetrieveError> {
//!         // Your implementation
//!         Ok(())
//!     }
//!
//!     fn build(&mut self) -> Result<(), RetrieveError> {
//!         // Your implementation
//!         Ok(())
//!     }
//! }
//! ```

/// Trait for external backends that can be used as drop-in replacements
/// for basic retrievers.
///
/// Implement this trait to integrate your chosen backend (Tantivy, HNSW,
/// FAISS, Qdrant, Pinecone, etc.) with rank-retrieve.
pub trait Backend {
    /// Retrieve top-k documents for a query.
    ///
    /// Returns vector of (document_id, score) pairs sorted by score descending.
    fn retrieve(&self, query: &[f32], k: usize) -> Result<Vec<(u32, f32)>, crate::RetrieveError>;

    /// Add a document to the index.
    fn add_document(&mut self, doc_id: u32, embedding: &[f32]) -> Result<(), crate::RetrieveError>;

    /// Build/finalize the index (required for some backends before retrieval).
    fn build(&mut self) -> Result<(), crate::RetrieveError>;
}
