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
//! # Current Implementation
//!
//! **Status: Comprehensive ANN implementation**
//!
//! This module provides both a simple brute-force retriever and a comprehensive
//! suite of approximate nearest neighbor (ANN) algorithms, all implemented in pure Rust.
//!
//! ## Simple Dense Retriever
//!
//! The `DenseRetriever` struct provides brute-force cosine similarity (O(n*d) where
//! n is number of documents and d is embedding dimension). This is suitable for:
//! - Any scale of corpora (from small to very large)
//! - Prototyping and research
//! - Applications where simplicity is preferred over scale
//!
//! ## ANN Algorithms (Feature-Gated)
//!
//! For large-scale retrieval, use the implemented ANN algorithms:
//!
//! ### Modern Methods (2024-2026)
//! - **HNSW** (`hnsw` feature): Hierarchical Navigable Small World - fast, high recall, O(log n)
//! - **NSW** (`nsw` feature): Flat Navigable Small World - lower memory, comparable performance
//! - **Anisotropic VQ + k-means** (`scann` feature): Quantization-based, optimized for MIPS
//! - **IVF-PQ** (`ivf_pq` feature): Memory-efficient, billion-scale capable
//! - **DiskANN** (`diskann` feature): Disk-based for very large datasets
//! - **OPT-SNG** (`sng` feature): Optimized Sparse Neighborhood Graph with 5.9× construction speedup
//! - **Vamana** (`vamana` feature): Two-pass graph construction with RRND + RND (competitive with HNSW)
//! - **SAQ** (`saq` feature): Segmented Adaptive Quantization with 80% error reduction
//! - **TurboQuant** (`turboquant` feature): Online quantization with near-optimal distortion
//!
//! ### Classic Methods
//! - **LSH** (`lsh` feature): Locality Sensitive Hashing with theoretical guarantees
//! - **Random Projection Tree Forest** (`annoy` feature): Production-proven tree-based method
//! - **KD-Tree** (`kdtree` feature): Space-partitioning for low dimensions (d < 20)
//! - **Ball Tree** (`balltree` feature): Hypersphere-based for medium dimensions (20 < d < 100)
//! - **Random Projection Tree** (`rptree` feature): Baseline tree method
//! - **K-Means Tree** (`kmeans_tree` feature): Hierarchical clustering tree for fast similarity search
//!
//! ### Supporting Methods
//! - **EVōC** (`evoc` feature): Hierarchical clustering for embeddings (alternative to k-means)
//!
//! All algorithms implement the unified `ANNIndex` trait for consistent API usage.
//!
//! **When to use ANN algorithms:**
//! - Corpus size > 100K documents
//! - Need sub-10ms retrieval latency
//! - Need approximate search for very large corpora
//! - Large-scale deployment
//!
//! **See also:**
//! - `docs/IMPLEMENTATION_STATUS_2026.md` for complete implementation status
//! - `docs/ANN_METHODS_SUMMARY.md` for algorithm comparison and use cases
//! - `examples/` for usage examples (to be added)

#[cfg(feature = "dense")]
use crate::retriever::{Retriever, RetrieverBuilder};
use crate::RetrieveError;

/// HNSW approximate nearest neighbor search (feature-gated).
///
/// Pure Rust implementation with SIMD acceleration.
#[cfg(feature = "hnsw")]
pub mod hnsw;

/// Flat Navigable Small World (NSW) - single-layer variant (feature-gated).
///
/// Achieves performance parity with HNSW in high-dimensional settings with
/// lower memory overhead. See `docs/CRITICAL_PERSPECTIVES_AND_LIMITATIONS.md`
/// for research on hierarchy effectiveness.
#[cfg(feature = "nsw")]
pub mod nsw;

/// Unified ANN algorithms module (feature-gated).
///
/// Contains HNSW, NSW, SCANN, IVF-PQ, DiskANN, and all ANN implementations.
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
pub mod ann;

/// Anisotropic Vector Quantization with k-means Partitioning (vendor: SCANN) (feature-gated).
#[cfg(feature = "scann")]
pub mod scann;

/// IVF-PQ (Inverted File Index with Product Quantization) implementation (feature-gated).
#[cfg(feature = "ivf_pq")]
pub mod ivf_pq;

/// DiskANN (disk-based ANN) implementation (feature-gated).
#[cfg(feature = "diskann")]
pub mod diskann;

/// OPT-SNG (Optimized Sparse Neighborhood Graph) implementation (feature-gated).
#[cfg(feature = "sng")]
pub mod sng;

/// Vamana graph-based ANN (two-pass construction with RRND + RND) (feature-gated).
#[cfg(feature = "vamana")]
pub mod vamana;

/// Enhanced quantization methods (SAQ, TurboQuant) (feature-gated).
#[cfg(any(feature = "saq", feature = "turboquant"))]
pub mod quantization;

/// Classic ANN methods (LSH, Random Projection Tree Forest, trees) (feature-gated).
#[cfg(any(feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
pub mod classic;

/// EVōC (Embedding Vector Oriented Clustering) - hierarchical clustering for embeddings (feature-gated).
#[cfg(feature = "evoc")]
pub mod evoc;

/// Partitioning/clustering interface (supports k-means and EVōC).
#[cfg(any(feature = "scann", feature = "evoc"))]
pub mod partitioning;

/// Dense retriever using cosine similarity.
///
/// Simple implementation using brute-force cosine similarity.
/// For large-scale use, replace with HNSW or FAISS.
pub struct DenseRetriever {
    /// Document ID -> Embedding vector
    documents: Vec<(u32, Vec<f32>)>,
    /// Optional metadata store for filtering
    metadata: Option<crate::filtering::MetadataStore>,
}

impl DenseRetriever {
    /// Create a new dense retriever.
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
            metadata: None,
        }
    }

    /// Create a new dense retriever with metadata support for filtering.
    pub fn with_metadata() -> Self {
        Self {
            documents: Vec::new(),
            metadata: Some(crate::filtering::MetadataStore::new()),
        }
    }

    /// Add metadata for a document (enables filtering).
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document identifier
    /// * `metadata` - Document metadata (field -> category ID mapping)
    pub fn add_metadata(
        &mut self,
        doc_id: u32,
        metadata: crate::filtering::DocumentMetadata,
    ) -> Result<(), RetrieveError> {
        if let Some(ref mut store) = self.metadata {
            store.add(doc_id, metadata);
            Ok(())
        } else {
            Err(RetrieveError::Other(
                "Metadata store not initialized. Use DenseRetriever::with_metadata()".to_string(),
            ))
        }
    }

    /// Get reference to metadata store (for faceting).
    ///
    /// Returns `None` if metadata store is not initialized.
    pub fn metadata(&self) -> Option<&crate::filtering::MetadataStore> {
        self.metadata.as_ref()
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
    ///
    /// Uses SIMD-accelerated dot product when available for better performance.
    fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        // Use SIMD-accelerated dot product for normalized vectors
        #[cfg(any(feature = "dense", feature = "sparse"))]
        {
            crate::simd::dot(a, b)
        }
        #[cfg(not(any(feature = "dense", feature = "sparse")))]
        {
            // Fallback to portable dot product when SIMD is not available
            a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
        }
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
    ///
    /// # Performance
    ///
    /// This uses brute-force cosine similarity (O(n*d) where n is documents, d is dimension).
    /// For large corpora (>100K documents), consider using optimized backends:
    /// - `rank-retrieve::integration::hnsw::HNSWBackend` (feature: `hnsw`)
    /// - `rank-retrieve::integration::usearch::UsearchBackend` (feature: `usearch`)
    /// - `rank-retrieve::integration::faiss::FaissBackend` (feature: `faiss`)
    pub fn retrieve(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if query_embedding.is_empty() {
            return Err(RetrieveError::EmptyQuery);
        }

        if self.documents.is_empty() {
            return Err(RetrieveError::EmptyIndex);
        }

        let query_dim = query_embedding.len();
        
        // Handle k=0 case
        if k == 0 {
            return Ok(Vec::new());
        }

        // Early termination optimization: use heap for k << num_documents
        if k < self.documents.len() / 2 {
            // Use min-heap for top-k (more efficient for small k)
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

            for (doc_id, doc_embedding) in &self.documents {
                if doc_embedding.len() != query_dim {
                    return Err(RetrieveError::DimensionMismatch {
                        query_dim,
                        doc_dim: doc_embedding.len(),
                    });
                }
                let score = Self::cosine_similarity(doc_embedding, query_embedding);
                
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

            // Extract and sort by score descending
            let mut results: Vec<(u32, f32)> = heap
                .into_iter()
                .map(|Reverse((FloatOrd(score), doc_id))| (doc_id, score))
                .collect();
            results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            Ok(results)
        } else {
            // Full sort for large k (more efficient)
            let mut scored: Vec<(u32, f32)> = Vec::with_capacity(self.documents.len());

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
            scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Return top-k
            Ok(scored.into_iter().take(k).collect())
        }
    }

    /// Retrieve top-k documents with post-filtering.
    ///
    /// Performs ANN search, then filters results by metadata predicate.
    /// Uses oversampling to ensure k results when filters are strict.
    ///
    /// # Arguments
    ///
    /// * `query_embedding` - Query embedding vector
    /// * `k` - Number of documents to retrieve (after filtering)
    /// * `filter` - Filter predicate
    ///
    /// # Returns
    ///
    /// Vector of (document_id, score) pairs matching the filter, sorted by score descending
    ///
    /// # Errors
    ///
    /// Returns `RetrieveError::Other` if metadata store is not initialized or filter is too strict.
    pub fn retrieve_with_filter(
        &self,
        query_embedding: &[f32],
        k: usize,
        filter: &crate::filtering::FilterPredicate,
    ) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if self.metadata.is_none() {
            return Err(RetrieveError::Other(
                "Metadata store not initialized. Use DenseRetriever::with_metadata()".to_string(),
            ));
        }

        let metadata_store = self.metadata.as_ref().unwrap();

        // Estimate filter selectivity
        let selectivity = metadata_store
            .estimate_selectivity(filter)
            .unwrap_or(0.5); // Default to 50% if can't estimate

        // Oversample: search more candidates if filter is strict
        // Formula: search k * (1 / selectivity) candidates, with minimum k
        let oversample_factor = (1.0 / selectivity.max(0.01)).ceil() as usize;
        let search_k = (k * oversample_factor).max(k);

        // Perform standard retrieval with oversampling
        let candidates = self.retrieve(query_embedding, search_k)?;

        // Post-filter: keep only matching documents
        let filtered: Vec<(u32, f32)> = candidates
            .into_iter()
            .filter(|(doc_id, _)| metadata_store.matches(*doc_id, filter))
            .take(k)
            .collect();

        if filtered.len() < k {
            return Err(RetrieveError::Other(format!(
                "Filter too strict: only {} documents match (requested {})",
                filtered.len(),
                k
            )));
        }

        Ok(filtered)
    }
}

#[cfg(feature = "dense")]
impl Retriever for DenseRetriever {
    type Query = [f32];

    fn retrieve(&self, query: &Self::Query, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        self.retrieve(query, k)
    }
}

#[cfg(feature = "dense")]
impl RetrieverBuilder for DenseRetriever {
    type Content = Vec<f32>;

    fn add_document(&mut self, doc_id: u32, content: Self::Content) -> Result<(), RetrieveError> {
        self.add_document(doc_id, content);
        Ok(())
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

        let results = retriever.retrieve(&query, 10).unwrap();

        // Document 0 should score 1.0 (exact match)
        // Document 1 should score 0.707 (cosine similarity)
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, 0);
        assert!((results[0].1 - 1.0).abs() < 0.001);
        assert!((results[1].1 - 0.707).abs() < 0.01);
    }
}
