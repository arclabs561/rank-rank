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
//! It provides implementations suitable for any scale of corpora, from small prototypes
//! to large-scale production systems.
//!
//! **Key characteristics:**
//! - In-memory indexes (no persistence)
//! - Efficient implementations with SIMD acceleration
//! - Unified API for multiple retrieval methods
//! - Designed for integration with `rank-*` ecosystem
//! - Scales from small to very large corpora
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
//! - **BM25 Retrieval**: Inverted index with Okapi BM25 scoring (supports BM25L and BM25+ variants)
//! - **TF-IDF Retrieval**: Inverted index with TF-IDF scoring (linear/log TF, standard/smoothed IDF)
//! - **Query Expansion / PRF**: Pseudo-relevance feedback to improve recall (addresses vocabulary mismatch)
//! - **Query Likelihood**: Probabilistic retrieval using language models (Jelinek-Mercer, Dirichlet smoothing)
//! - **Dense ANN**: Cosine similarity-based retrieval (ready for HNSW/FAISS integration)
//! - **Sparse Retrieval**: Lexical matching using sparse vectors
//! - **Generative Retrieval (LTRGR)**: Autoregressive models generate identifiers for passages
//!
//! **Note on Late Interaction**: For ColBERT/ColPali-style late interaction retrieval (token-level
//! matching), use `rank-retrieve` for first-stage retrieval (BM25/dense) followed by
//! `rank-rerank` for MaxSim reranking. Research shows this pipeline often matches PLAID's
//! efficiency-effectiveness trade-off (MacAvaney & Tonellotto, SIGIR 2024).
//! See `rank-rerank`'s `PLAID_AND_OPTIMIZATION.md` for details and
//! [ACM DL](https://dl.acm.org/doi/10.1145/3626772.3657856) for the paper.
//! Supports both text (ColBERT) and multimodal (ColPali) late interaction.
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
///
/// # Eager Scoring
///
/// For query-heavy workloads, see `bm25::eager::EagerBm25Index` for precomputed
/// scores (500x faster retrieval, 2-3x larger memory).
pub mod bm25;

/// TF-IDF retrieval module.
///
/// Provides TF-IDF (Term Frequency-Inverse Document Frequency) scoring for first-stage retrieval.
/// Reuses the BM25 inverted index structure with simpler scoring formula.
///
/// Implementations are available when `tfidf` feature is enabled.
#[cfg(feature = "tfidf")]
pub mod tfidf;

/// Dense approximate nearest neighbor search.
///
/// Provides cosine similarity-based retrieval for dense embeddings.
/// For large scale, integrate with HNSW or FAISS.
///
/// Implementations are available when `dense` feature is enabled.
pub mod dense;

/// SIMD-accelerated vector operations.
///
/// Provides optimized dot product, cosine similarity, and sparse dot product using SIMD instructions.
/// Automatically selects the fastest available instruction set (AVX-512, AVX2, NEON).
///
/// Available when `dense` or `sparse` feature is enabled.
#[cfg(any(feature = "dense", feature = "sparse"))]
pub mod simd;

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

/// Filtering support for vector search.
///
/// Provides filter predicates, metadata storage, post-filtering, filter fusion,
/// and integrated filtering strategies.
pub mod filtering;

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

/// Lossless compression for vector IDs in ANN indexes.
///
/// Provides compression algorithms that exploit ordering invariance in vector ID
/// collections (IVF clusters, HNSW neighbor lists) to achieve significant compression
/// ratios (5-7x for large sets).
///
/// See `compression` module documentation for details.
#[cfg(feature = "id-compression")]
pub mod compression;

/// Disk persistence for retrieval indexes.
///
/// Provides crash-safe, concurrent persistence for all retrieval methods.
/// See `persistence` module documentation and `docs/PERSISTENCE_DESIGN.md` for details.
#[cfg(feature = "persistence")]
pub mod persistence;

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

/// Query expansion and pseudo-relevance feedback (PRF).
///
/// Provides query expansion techniques to improve recall by reformulating queries
/// with semantically related terms extracted from top-ranked documents.
///
/// Implementations are available when `query-expansion` feature is enabled.
#[cfg(feature = "query-expansion")]
pub mod query_expansion;

/// Query likelihood language model retrieval.
///
/// Provides probabilistic retrieval using language models, ranking documents by
/// the probability that the document's language model generated the query: P(Q|D).
///
/// Implementations are available when `query-likelihood` feature is enabled.
#[cfg(feature = "query-likelihood")]
pub mod query_likelihood;

/// Standard ANN benchmarking utilities following ann-benchmarks methodology.
///
/// Provides utilities for benchmarking ANN algorithms following the structure
/// and metrics from ann-benchmarks (erikbern/ann-benchmarks).
pub mod benchmark;

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

/// Three-way retrieval: Combine BM25, dense, and sparse retrieval.
///
/// Research shows that combining full-text (BM25), dense, and sparse retrieval
/// provides optimal results for RAG applications. This function retrieves from
/// all three methods and returns separate result lists for fusion.
///
/// # Arguments
///
/// * `bm25_index` - BM25 inverted index (full-text search)
/// * `dense_retriever` - Dense retriever (semantic search)
/// * `sparse_retriever` - Sparse retriever (learned sparse, e.g., SPLADE)
/// * `query_terms` - Query terms for BM25
/// * `query_embedding` - Query embedding for dense retrieval
/// * `query_sparse` - Query sparse vector for sparse retrieval
/// * `k` - Number of candidates to retrieve from each method
/// * `bm25_params` - BM25 parameters
///
/// # Returns
///
/// Tuple of (bm25_results, dense_results, sparse_results) for fusion
///
/// # Example
///
/// ```rust,no_run
/// use rank_retrieve::retrieve_three_way;
/// use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
/// use rank_retrieve::dense::DenseRetriever;
/// use rank_retrieve::sparse::{SparseRetriever, SparseVector};
///
/// let bm25_index = InvertedIndex::new();
/// let dense_retriever = DenseRetriever::new();
/// let sparse_retriever = SparseRetriever::new();
///
/// let (bm25_results, dense_results, sparse_results) = retrieve_three_way(
///     &bm25_index,
///     &dense_retriever,
///     &sparse_retriever,
///     &["query".to_string()],
///     &[0.1; 128],
///     &SparseVector::new_unchecked(vec![0], vec![1.0]),
///     1000,
///     Bm25Params::default(),
/// ).unwrap();
///
/// // Fuse results using rank-fusion
/// ```
#[cfg(all(feature = "bm25", feature = "dense", feature = "sparse"))]
pub fn retrieve_three_way(
    bm25_index: &crate::bm25::InvertedIndex,
    dense_retriever: &crate::dense::DenseRetriever,
    sparse_retriever: &crate::sparse::SparseRetriever,
    query_terms: &[String],
    query_embedding: &[f32],
    query_sparse: &crate::sparse::SparseVector,
    k: usize,
    bm25_params: crate::bm25::Bm25Params,
) -> Result<(Vec<(u32, f32)>, Vec<(u32, f32)>, Vec<(u32, f32)>), RetrieveError> {
    let bm25_results = retrieve_bm25(bm25_index, query_terms, k, bm25_params)?;
    let dense_results = retrieve_dense(dense_retriever, query_embedding, k)?;
    let sparse_results = retrieve_sparse(sparse_retriever, query_sparse, k)?;
    Ok((bm25_results, dense_results, sparse_results))
}

/// Retrieve top-k documents using TF-IDF scoring.
///
/// TF-IDF is a simpler alternative to BM25, providing baseline lexical retrieval.
/// Reuses the BM25 inverted index structure.
///
/// # Example
///
/// ```rust
/// use rank_retrieve::retrieve_tfidf;
/// use rank_retrieve::bm25::InvertedIndex;
/// use rank_retrieve::tfidf::TfIdfParams;
///
/// let mut index = InvertedIndex::new();
/// index.add_document(0, &["machine".to_string(), "learning".to_string()]);
///
/// let query = vec!["machine".to_string()];
/// let results = retrieve_tfidf(&index, &query, 10, TfIdfParams::default()).unwrap();
/// assert!(!results.is_empty());
/// ```
#[cfg(feature = "tfidf")]
pub fn retrieve_tfidf(
    index: &crate::bm25::InvertedIndex,
    query: &[String],
    k: usize,
    params: crate::tfidf::TfIdfParams,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    crate::tfidf::retrieve_tfidf(index, query, k, params)
}

/// Re-export commonly used types.
pub mod prelude {
    // Core types (always available)
    pub use crate::RetrieveError;

    // Concrete retrieval functions (primary API)
    #[cfg(feature = "bm25")]
    pub use crate::retrieve_bm25;
    #[cfg(feature = "tfidf")]
    pub use crate::retrieve_tfidf;
    #[cfg(feature = "query-expansion")]
    pub use crate::query_expansion::{expand_query_with_prf_bm25, ExpansionMethod, QueryExpander};
    #[cfg(feature = "query-likelihood")]
    pub use crate::query_likelihood::{retrieve_query_likelihood, QueryLikelihoodParams, SmoothingMethod};
    #[cfg(feature = "dense")]
    pub use crate::retrieve_dense;
    #[cfg(feature = "sparse")]
    pub use crate::retrieve_sparse;

    // Feature-gated implementations
    #[cfg(feature = "bm25")]
    pub use crate::bm25::{Bm25Params, Bm25Variant, InvertedIndex};
    #[cfg(feature = "tfidf")]
    pub use crate::tfidf::{TfIdfParams, TfVariant, IdfVariant};
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
