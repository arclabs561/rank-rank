//! First-stage retrieval for information retrieval pipelines.
//!
//! This crate provides retrieval from large corpora (10M+ documents) down to
//! manageable candidate sets (1000 candidates) for reranking.
//!
//! # Pipeline Stage
//!
//! ```
//! 10M docs → 1000 candidates → 100 candidates → 10 results
//!     │            │                 │              │
//!     ▼            ▼                 ▼              ▼
//! [retrieve]   [rerank]         [cross-encoder]   [User]
//!   (fast)      (precise)        (accurate)
//! ```
//!
//! # Features
//!
//! - **BM25 Retrieval**: Inverted index with Okapi BM25 scoring
//! - **Dense ANN**: Cosine similarity-based retrieval (ready for HNSW/FAISS integration)
//! - **Sparse Retrieval**: Lexical matching using sparse vectors (uses rank-sparse)
//!
//! # Quick Start
//!
//! ```rust
//! use rank_retrieve::prelude::*;
//!
//! let mut index = InvertedIndex::new();
//! index.add_document(0, &["the".to_string(), "quick".to_string()]);
//! 
//! let query = vec!["quick".to_string()];
//! let results = index.retrieve(&query, 10, Bm25Params::default());
//! ```
//!
//! # Design
//!
//! This crate focuses on **retrieval** (finding candidates), not scoring.
//! Advanced scoring (MaxSim, cross-encoder) is handled by `rank-rerank`.
//!
//! **Boundaries:**
//! - ✅ Indexing and retrieval
//! - ✅ Basic scoring (BM25, cosine similarity)
//! - ❌ Advanced reranking (delegates to rank-rerank)
//! - ❌ List fusion (delegates to rank-fusion)

/// BM25 retrieval module.
///
/// Provides inverted index and Okapi BM25 scoring.
pub mod bm25;

/// Dense approximate nearest neighbor search.
///
/// Provides cosine similarity-based retrieval for dense embeddings.
/// For production use, integrate with HNSW or FAISS.
pub mod dense;

/// Sparse retrieval module.
///
/// Uses rank-sparse for sparse vector operations.
pub mod sparse;

/// Error types for retrieval operations.
pub mod error;

pub use error::RetrieveError;

/// Re-export commonly used types.
pub mod prelude {
    pub use crate::bm25::{Bm25Params, InvertedIndex};
    pub use crate::dense::DenseRetriever;
    pub use crate::sparse::SparseRetriever;
    pub use crate::RetrieveError;
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bm25_retrieval() {
        use crate::bm25::*;
        
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string(), "document".to_string()]);
        
        let query = vec!["test".to_string()];
        let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
        
        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0);
    }
}
