//! Example: Hybrid retrieval combining BM25, dense, and sparse methods.
//!
//! This demonstrates how to use multiple retrieval methods and combine their results.
//!
//! **What is Hybrid Retrieval?**
//! Hybrid retrieval combines multiple retrieval methods (lexical + semantic) to
//! improve recall and precision. Research shows that combining BM25 (lexical) with
//! dense retrieval (semantic) often outperforms either method alone.
//!
//! **Why use Hybrid Retrieval?**
//! - **Better recall**: BM25 catches exact matches, dense catches semantic matches
//! - **Better precision**: Multiple signals improve ranking quality
//! - **Robustness**: Less sensitive to query formulation or embedding quality
//! - **Research-backed**: Consistently outperforms single-method retrieval
//!
//! **Pipeline:**
//! 1. **Parallel retrieval**: Run BM25, dense, and sparse retrieval simultaneously
//! 2. **Fusion**: Combine results using rank-fusion (RRF, CombSum, etc.)
//! 3. **Reranking** (optional): Use rank-rerank for final precision
//!
//! **When to use:**
//! - Need maximum retrieval quality
//! - Have both text (for BM25) and embeddings (for dense)
//! - Can afford multiple retrieval calls (latency trade-off)
//!
//! **Performance:**
//! - BM25: ~1ms for 10M docs → 1000 candidates
//! - Dense: ~1-5ms for 10M docs → 1000 candidates (with ANN index)
//! - Sparse: ~1-2ms for 10M docs → 1000 candidates
//! - Fusion: ~1ms to combine results
//! - Total: ~4-9ms per query (parallel execution)

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::retrieve_bm25;
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "dense")]
use rank_retrieve::retrieve_dense;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};
#[cfg(feature = "sparse")]
use rank_retrieve::retrieve_sparse;

/// Hybrid retriever combining multiple retrieval methods.
#[cfg(all(feature = "bm25", feature = "dense", feature = "sparse"))]
pub struct HybridRetriever {
    #[cfg(feature = "bm25")]
    bm25_index: InvertedIndex,
    #[cfg(feature = "dense")]
    dense_retriever: DenseRetriever,
    #[cfg(feature = "sparse")]
    sparse_retriever: SparseRetriever,
}

#[cfg(all(feature = "bm25", feature = "dense", feature = "sparse"))]
impl HybridRetriever {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "bm25")]
            bm25_index: InvertedIndex::new(),
            #[cfg(feature = "dense")]
            dense_retriever: DenseRetriever::new(),
            #[cfg(feature = "sparse")]
            sparse_retriever: SparseRetriever::new(),
        }
    }

    /// Add a document with all representations.
    pub fn add_document(
        &mut self,
        doc_id: u32,
        terms: &[String],
        dense_embedding: Vec<f32>,
        sparse_vector: SparseVector,
    ) {
        #[cfg(feature = "bm25")]
        self.bm25_index.add_document(doc_id, terms);
        #[cfg(feature = "dense")]
        self.dense_retriever.add_document(doc_id, dense_embedding);
        #[cfg(feature = "sparse")]
        self.sparse_retriever.add_document(doc_id, sparse_vector);
    }

    /// Retrieve from all methods.
    pub fn retrieve_all(
        &self,
        query_terms: &[String],
        query_embedding: &[f32],
        query_sparse: &SparseVector,
        k: usize,
    ) -> (Vec<(u32, f32)>, Vec<(u32, f32)>, Vec<(u32, f32)>) {
        #[cfg(feature = "bm25")]
        let bm25_results =
            retrieve_bm25(&self.bm25_index, query_terms, k, Bm25Params::default()).unwrap();
        #[cfg(not(feature = "bm25"))]
        let bm25_results = Vec::new();

        #[cfg(feature = "dense")]
        let dense_results = retrieve_dense(&self.dense_retriever, query_embedding, k).unwrap();
        #[cfg(not(feature = "dense"))]
        let dense_results = Vec::new();

        #[cfg(feature = "sparse")]
        let sparse_results = retrieve_sparse(&self.sparse_retriever, query_sparse, k).unwrap();
        #[cfg(not(feature = "sparse"))]
        let sparse_results = Vec::new();

        (bm25_results, dense_results, sparse_results)
    }
}

#[cfg(all(feature = "bm25", feature = "dense", feature = "sparse"))]
fn main() {
    let mut retriever = HybridRetriever::new();

    // Add documents with all representations
    retriever.add_document(
        0,
        &[
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
        ],
        vec![1.0, 0.0, 0.0], // Dense embedding
        SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 0.8, 0.6]), // Sparse vector
    );

    retriever.add_document(
        1,
        &["the".to_string(), "lazy".to_string(), "dog".to_string()],
        vec![0.707, 0.707, 0.0],
        SparseVector::new_unchecked(vec![1, 3], vec![0.9, 0.7]),
    );

    // Query
    let query_terms = vec!["quick".to_string(), "fox".to_string()];
    let query_embedding = vec![1.0, 0.0, 0.0];
    let query_sparse = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);

    let (bm25_results, dense_results, sparse_results) =
        retriever.retrieve_all(&query_terms, &query_embedding, &query_sparse, 10);

    println!("BM25 Results:");
    for (doc_id, score) in &bm25_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    println!("\nDense Results:");
    for (doc_id, score) in &dense_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    println!("\nSparse Results:");
    for (doc_id, score) in &sparse_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    println!("\n=== Next Step: Use rank-fusion to combine these results ===");
    println!("Example: use rank_fusion::rrf_multi() to combine all three lists");
}

#[cfg(not(all(feature = "bm25", feature = "dense", feature = "sparse")))]
fn main() {
    eprintln!("This example requires 'bm25', 'dense', and 'sparse' features.");
    eprintln!("Run with: cargo run --example hybrid_retrieval --features bm25,dense,sparse");
}
