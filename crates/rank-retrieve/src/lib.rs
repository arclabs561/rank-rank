//! First-stage retrieval for information retrieval pipelines.
//!
//! This crate provides retrieval from large corpora (10M+ documents) down to
//! manageable candidate sets (1000 candidates) for reranking.
//!
//! # Pipeline Stage
//!
//! The retrieval pipeline flows as:
//! - 10M docs -> 1000 candidates (retrieve, fast)
//! - 1000 -> 100 candidates (rerank, precise)
//! - 100 -> 10 results (cross-encoder, accurate)
//!
//! # Design Philosophy
//!
//! This crate focuses on **first-stage retrieval** as a component in IR pipelines.
//! It provides basic implementations suitable for small-medium corpora and prototyping.
//! For large scale, integrate with specialized libraries (Tantivy, HNSW, FAISS).
//!
//! **Key characteristics:**
//! - In-memory indexes (no persistence)
//! - Basic implementations (not optimized for large scale)
//! - Unified API for multiple retrieval methods
//! - Designed for integration with `rank-*` ecosystem
//!
//! **Value proposition:**
//! - Multiple retrieval methods in one crate (BM25, dense, sparse, generative)
//! - Consistent output format (all return `Vec<(u32, f32)>`) for easy integration
//! - Generative retrieval (LTRGR) implementation (unique in Rust ecosystem)
//! - Seamless integration with `rank-fusion`, `rank-rerank`, `rank-eval`
//! - Simple, concrete API matching `rank-fusion` and `rank-rerank` patterns
//!
//! See [README](../README.md) for detailed boundaries and limitations.
//! See [MOTIVATION.md](../docs/MOTIVATION.md) for competitive analysis and justification.
//!
//! # Features
//!
//! - **BM25 Retrieval**: Inverted index with Okapi BM25 scoring
//! - **Dense ANN**: Cosine similarity-based retrieval (ready for HNSW/FAISS integration)
//! - **Sparse Retrieval**: Lexical matching using sparse vectors
//! - **Generative Retrieval (LTRGR)**: Autoregressive models generate identifiers for passages
//!
//! **Note on Late Interaction**: For ColBERT-style late interaction retrieval (token-level
//! matching), use `rank-retrieve` for first-stage retrieval (BM25/dense) followed by
//! `rank-rerank` for MaxSim reranking. Research shows this pipeline often matches PLAID's
//! efficiency-effectiveness trade-off (MacAvaney & Tonellotto, SIGIR 2024).
//! See `rank-rerank`'s `PLAID_AND_OPTIMIZATION.md` for details and
//! [ACM DL](https://dl.acm.org/doi/10.1145/3626772.3657856) for the paper.
//!
//! # Example: Late Interaction Pipeline
//!
//! ```rust
//! use rank_retrieve::{retrieve_bm25, bm25::{Bm25Params, InvertedIndex}};
//! use rank_rerank::colbert;
//!
//! // 1. First-stage retrieval with BM25
//! let mut index = InvertedIndex::new();
//! index.add_document(0, &["machine".to_string(), "learning".to_string()]);
//! let candidates = retrieve_bm25(&index, &["learning".to_string()], 1000, Bm25Params::default())?;
//!
//! // 2. Rerank with MaxSim (requires token embeddings - see examples/)
//! // let query_tokens = encode_query(&query_text)?;
//! // let doc_tokens = get_document_tokens(&candidates);
//! // let reranked = colbert::rank(&query_tokens, &doc_tokens);
//!
//! // 3. Optional: Apply token pooling (50% reduction, <1% quality loss)
//! // let pooled = colbert::pool_tokens(&doc_tokens, 2)?;
//! # Ok::<(), rank_retrieve::RetrieveError>(())
//! ```
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
//! - Indexing and retrieval
//! - Basic scoring (BM25, cosine similarity)
//! - Advanced reranking (delegates to rank-rerank)
//! - List fusion (delegates to rank-fusion)

/// BM25 retrieval module.
///
/// Provides inverted index and Okapi BM25 scoring.
///
/// Implementations are available when `bm25` feature is enabled.
pub mod bm25;

/// Dense approximate nearest neighbor search.
///
/// Provides cosine similarity-based retrieval for dense embeddings.
/// For large scale, integrate with HNSW or FAISS.
///
/// Implementations are available when `dense` feature is enabled.
pub mod dense;

/// Sparse retrieval module.
///
/// Uses sparse vectors for lexical matching with dot product scoring.
///
/// Implementations are available when `sparse` feature is enabled.
pub mod sparse;

/// Batch retrieval operations.
///
/// Provides efficient batch processing for multiple queries.
pub mod batch;

/// Query routing framework (LTRR-style).
///
/// Learning to Rank Retrievers - dynamically select from pool of retrievers
/// based on query characteristics.
pub mod routing;

/// Generative retrieval with learning-to-rank (LTRGR).
///
/// Autoregressive models generate identifiers (titles, substrings, pseudo-queries)
/// for relevant passages, with LTR training to optimize passage ranking.
///
/// This module is organized as a subdirectory because it contains multiple
/// distinct components (~1177 lines total) that benefit from separation.
///
/// Implementations are available when `generative` feature is enabled.
pub mod generative;

/// Error types for retrieval operations.
pub mod error;

/// Unified retriever trait interface.
///
/// Core trait that all retrieval methods implement. Always available.
/// Specific implementations are feature-gated.
///
/// **Note:** The primary API uses concrete functions (`retrieve_bm25()`, `retrieve_dense()`, etc.)
/// for simplicity and consistency with the `rank-*` ecosystem. This trait is available for
/// custom implementations or advanced use cases.
#[deprecated(
    note = "Use concrete functions instead: retrieve_bm25(), retrieve_dense(), retrieve_sparse(). The trait is kept for backward compatibility and custom implementations."
)]
pub mod retriever;

/// Trait interface for external backend integrations.
///
/// Provides a trait that external backends can implement to integrate with
/// rank-retrieve. Does not include full implementations of specific backends.
///
/// For large scale, implement this trait for your chosen backend (Tantivy,
/// HNSW, FAISS, Qdrant, Pinecone, etc.).
pub mod integration;

pub use error::RetrieveError;

/// Concrete retrieval functions (primary API).
///
/// These functions provide a simple, direct interface for retrieval operations.
/// They match the pattern used in `rank-fusion` and `rank-rerank` for consistency.
///
/// # Example
///
/// ```rust
/// use rank_retrieve::retrieve_bm25;
/// use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
///
/// let mut index = InvertedIndex::new();
/// index.add_document(0, &["the".to_string(), "quick".to_string()]);
///
/// let query = vec!["quick".to_string()];
/// let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
/// assert!(!results.is_empty());
/// ```
#[cfg(feature = "bm25")]
pub fn retrieve_bm25(
    index: &crate::bm25::InvertedIndex,
    query: &[String],
    k: usize,
    params: crate::bm25::Bm25Params,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    index.retrieve(query, k, params)
}

/// Retrieve top-k documents using dense retrieval.
///
/// # Example
///
/// ```rust
/// use rank_retrieve::retrieve_dense;
/// use rank_retrieve::dense::DenseRetriever;
///
/// let mut retriever = DenseRetriever::new();
/// let embedding = vec![1.0, 0.0, 0.0];
/// retriever.add_document(0, embedding);
///
/// let query = [1.0, 0.0, 0.0];
/// let results = retrieve_dense(&retriever, &query, 10).unwrap();
/// assert!(!results.is_empty());
/// ```
#[cfg(feature = "dense")]
pub fn retrieve_dense(
    retriever: &crate::dense::DenseRetriever,
    query: &[f32],
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    retriever.retrieve(query, k)
}

/// Retrieve top-k documents using sparse retrieval.
///
/// # Example
///
/// ```rust
/// use rank_retrieve::retrieve_sparse;
/// use rank_retrieve::sparse::{SparseRetriever, SparseVector};
///
/// let mut retriever = SparseRetriever::new();
/// let doc = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 0.5]);
/// retriever.add_document(0, doc);
///
/// let query = SparseVector::new_unchecked(vec![0], vec![1.0]);
/// let results = retrieve_sparse(&retriever, &query, 10).unwrap();
/// assert!(!results.is_empty());
/// ```
#[cfg(feature = "sparse")]
pub fn retrieve_sparse(
    retriever: &crate::sparse::SparseRetriever,
    query: &crate::sparse::SparseVector,
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    retriever.retrieve(query, k)
}

/// Re-export commonly used types.
pub mod prelude {
    // Core types (always available)
    pub use crate::RetrieveError;

    // Concrete retrieval functions (primary API)
    #[cfg(feature = "bm25")]
    pub use crate::retrieve_bm25;
    #[cfg(feature = "dense")]
    pub use crate::retrieve_dense;
    #[cfg(feature = "sparse")]
    pub use crate::retrieve_sparse;

    // Feature-gated implementations
    #[cfg(feature = "bm25")]
    pub use crate::bm25::{Bm25Params, InvertedIndex};
    #[cfg(feature = "dense")]
    pub use crate::dense::DenseRetriever;
    #[cfg(feature = "generative")]
    pub use crate::generative::{
        AutoregressiveModel, GenerativeRetriever, HeuristicScorer, IdentifierGenerator,
        IdentifierType, LTRGRConfig, LTRGRTrainer, MultiviewIdentifier, SimpleIdentifierGenerator,
    };
    #[cfg(feature = "sparse")]
    pub use crate::sparse::SparseRetriever;
    #[cfg(feature = "sparse")]
    pub use crate::sparse::{dot_product, SparseVector};

    // Always available
    pub use crate::routing::{QueryFeatures, QueryRouter, QueryType, RetrieverId};

    // Deprecated: Trait interface (kept for backward compatibility)
    #[deprecated(
        note = "Use concrete functions instead: retrieve_bm25(), retrieve_dense(), retrieve_sparse()"
    )]
    pub use crate::retriever::{Retriever, RetrieverBuilder};
}

#[cfg(test)]
mod tests {
    use crate::retriever::Retriever;

    #[cfg(feature = "bm25")]
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

    #[cfg(feature = "bm25")]
    #[test]
    fn test_bm25_trait_interface() {
        use crate::bm25::*;
        use crate::retriever::Retriever;

        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string(), "document".to_string()]);

        let query = vec!["test".to_string()];
        let results = Retriever::retrieve(&index, &query, 10).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].0, 0);
    }
}
