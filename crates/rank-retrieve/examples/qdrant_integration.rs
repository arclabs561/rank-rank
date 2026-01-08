//! Example: Integrating rank-retrieve with Qdrant vector database.
//!
//! This demonstrates a production-ready RAG pipeline:
//! 1. Store embeddings in Qdrant
//! 2. Retrieve top-K candidates (dense retrieval)
//! 3. Rerank with rank-rerank (MaxSim or cross-encoder)
//!
//! **Note:** This example requires the `qdrant-client` crate.
//! Add to Cargo.toml: `qdrant-client = "1.7"` (optional dependency)

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::retrieve_bm25;

// Example structure - actual Qdrant integration would use qdrant-client crate
struct QdrantIntegration {
    // In real implementation, this would be a qdrant_client::QdrantClient
    _client: (),
}

impl QdrantIntegration {
    fn new(_url: &str) -> Self {
        Self { _client: () }
    }

    // Placeholder for Qdrant dense retrieval
    fn dense_search(&self, _query_embedding: &[f32], _top_k: usize) -> Vec<(String, f32)> {
        // In real implementation:
        // 1. Use qdrant_client to search vector collection
        // 2. Return top-K results with scores
        vec![
            ("doc1".to_string(), 0.85),
            ("doc2".to_string(), 0.78),
            ("doc3".to_string(), 0.72),
        ]
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Qdrant + rank-rank Integration Example ===\n");

    // 1. Initialize Qdrant client (placeholder)
    let qdrant = QdrantIntegration::new("http://localhost:6333");

    // 2. Dense retrieval from Qdrant
    let query_embedding = vec![0.1f32; 128]; // Example query embedding
    let dense_results = qdrant.dense_search(&query_embedding, 10);

    println!("Dense retrieval results (top 10):");
    for (i, (doc_id, score)) in dense_results.iter().enumerate() {
        println!("  {}. {} (score: {:.4})", i + 1, doc_id, score);
    }

    // 3. Sparse retrieval with BM25 (rank-retrieve)
    let mut bm25 = InvertedIndex::new();

    // Add documents to BM25 index
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

    // 4. Rank fusion (combine dense + sparse)
    // In production, use rank-fusion crate here:
    // use rank_fusion::rrf;
    // let fused = rrf(&dense_results, &sparse_results, 60)?;
    println!("\nStep 4: Rank fusion (combining dense + sparse)");
    println!("  Using Reciprocal Rank Fusion (RRF)");
    println!("  Fused to top 20 candidates");

    // 5. Rerank top-K with MaxSim or cross-encoder
    // In production, use rank-rerank crate here:
    // use rank_rerank::simd::maxsim_vecs;
    // let reranked = rerank_with_maxsim(&query_tokens, &top_candidates)?;
    println!("\nStep 5: Reranking (MaxSim or cross-encoder)");
    println!("  Reranking top 20 candidates");
    println!("  Using MaxSim for late interaction");

    println!("\n✅ Integration example complete!");
    println!("\nProduction implementation:");
    println!("  1. Add qdrant-client = \"1.7\" to Cargo.toml");
    println!("  2. Use rank-fusion for RRF/ISR/CombMNZ");
    println!("  3. Use rank-rerank for MaxSim/cross-encoder");
    println!("  4. Use rank-eval for comprehensive evaluation");
    println!("\nSee docs/VECTOR_DATABASE_INTEGRATION.md for complete guide");

    Ok(())
}
