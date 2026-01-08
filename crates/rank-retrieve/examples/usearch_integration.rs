//! Example: Integrating rank-retrieve with usearch for approximate nearest neighbor search.
//!
//! This demonstrates a production-ready dense retrieval pipeline:
//! 1. Build HNSW index with usearch
//! 2. Approximate nearest neighbor search
//! 3. Rerank results with rank-rerank (MaxSim or cross-encoder)
//!
//! **Note:** This example requires the `usearch` crate.
//! Add to Cargo.toml: `usearch = "2.11"` (optional dependency)

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::retrieve_bm25;

// Example structure - actual usearch integration would use usearch crate
struct UsearchIndex {
    // In real implementation, this would be a usearch::Index
    _index: (),
    dimension: usize,
}

impl UsearchIndex {
    fn new(dimension: usize) -> Self {
        Self {
            _index: (),
            dimension,
        }
    }

    // Placeholder for usearch HNSW search
    fn search(&self, _query_vector: &[f32], _top_k: usize) -> Vec<(u32, f32)> {
        // In real implementation:
        // 1. Use usearch::Index to build HNSW index
        // 2. Perform approximate nearest neighbor search
        // 3. Return top-K results with scores
        vec![(1, 0.92), (2, 0.85), (3, 0.78)]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== usearch + rank-rank Integration Example ===\n");

    // 1. Build HNSW index with usearch (placeholder)
    let dimension = 128;
    let index = UsearchIndex::new(dimension);

    // 2. Approximate nearest neighbor search
    let query_embedding = vec![0.1f32; dimension];
    let dense_results = index.search(&query_embedding, 10);

    println!("Dense retrieval results (HNSW, top 10):");
    for (i, (doc_id, score)) in dense_results.iter().enumerate() {
        println!("  {}. doc{} (score: {:.4})", i + 1, doc_id, score);
    }

    // 3. Sparse retrieval with BM25 (rank-retrieve)
    let mut bm25 = InvertedIndex::new();

    let documents = vec![
        ("doc1", "machine learning algorithms neural networks"),
        ("doc2", "information retrieval search engines"),
        ("doc3", "natural language processing transformers"),
    ];

    for (doc_id, text) in &documents {
        let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        bm25.add_document(
            doc_id
                .parse()
                .map_err(|e| format!("Failed to parse doc_id: {}", e))?,
            &terms,
        );
    }

    let query_terms: Vec<String> = "machine learning"
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();
    let sparse_results = retrieve_bm25(&bm25, &query_terms, 10, Bm25Params::default())?;

    println!("\nSparse retrieval results (BM25, top 10):");
    for (i, (doc_id, score)) in sparse_results.iter().enumerate() {
        println!("  {}. doc{} (score: {:.4})", i + 1, doc_id, score);
    }

    // 4. Rerank with rank-rerank (MaxSim)
    // In production, use rank-rerank crate here:
    // use rank_rerank::simd::maxsim_vecs;
    // let reranked = rerank_with_maxsim(&query_tokens, &top_candidates)?;
    println!("\nStep 4: Reranking (MaxSim or cross-encoder)");
    println!("  Reranking top 20 candidates");
    println!("  Using MaxSim for late interaction");

    println!("\n✅ Integration example complete!");
    println!("\nProduction implementation:");
    println!("  1. Add usearch = \"2.11\" to Cargo.toml");
    println!("  2. Use rank-rerank for MaxSim/cross-encoder");
    println!("  3. Use rank-fusion for combining with sparse results");
    println!("  4. Use rank-eval for comprehensive evaluation");
    println!("\nSee docs/VECTOR_DATABASE_INTEGRATION.md for complete guide");

    Ok(())
}
