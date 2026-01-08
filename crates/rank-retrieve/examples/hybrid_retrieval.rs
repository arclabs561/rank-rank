//! Hybrid retrieval example.
//!
//! Demonstrates combining BM25, dense, and sparse retrieval,
//! then using rank-fusion to combine results.

use rank_retrieve::{retrieve_bm25, retrieve_dense, retrieve_sparse};
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

/// Hybrid retriever combining multiple retrieval methods.
pub struct HybridRetriever {
    bm25_index: InvertedIndex,
    dense_retriever: DenseRetriever,
    sparse_retriever: SparseRetriever,
}

impl HybridRetriever {
    pub fn new() -> Self {
        Self {
            bm25_index: InvertedIndex::new(),
            dense_retriever: DenseRetriever::new(),
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
        self.bm25_index.add_document(doc_id, terms);
        self.dense_retriever.add_document(doc_id, dense_embedding);
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
        let bm25_results = retrieve_bm25(&self.bm25_index, query_terms, k, Bm25Params::default()).unwrap();
        let dense_results = retrieve_dense(&self.dense_retriever, query_embedding, k).unwrap();
        let sparse_results = retrieve_sparse(&self.sparse_retriever, query_sparse, k).unwrap();
        
        (bm25_results, dense_results, sparse_results)
    }
}

fn main() {
    let mut retriever = HybridRetriever::new();
    
    // Add documents with all representations
    retriever.add_document(
        0,
        &["the".to_string(), "quick".to_string(), "brown".to_string(), "fox".to_string()],
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
    
    let (bm25_results, dense_results, sparse_results) = retriever.retrieve_all(
        &query_terms,
        &query_embedding,
        &query_sparse,
        10,
    );
    
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

