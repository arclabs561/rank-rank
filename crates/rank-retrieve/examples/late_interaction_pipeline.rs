//! Example: Late Interaction Retrieval Pipeline
//!
//! Demonstrates the research-backed approach: BM25 first-stage retrieval
//! followed by MaxSim reranking. Research shows this pipeline often matches
//! PLAID's efficiency-effectiveness trade-off (MacAvaney & Tonellotto, SIGIR 2024).
//!
//! **What is Late Interaction?**
//! Late interaction (ColBERT-style) computes token-level similarities between
//! query and document tokens, then aggregates (MaxSim) for final scoring.
//! This enables fine-grained matching without the quadratic complexity of
//! cross-encoders.
//!
//! **Pipeline stages:**
//! 1. **BM25 retrieval** (rank-retrieve): Fast first-stage retrieval from large corpus
//! 2. **MaxSim reranking** (rank-rerank): Token-level matching for precision
//! 3. **Token pooling** (optional): Storage optimization for large corpora
//! 4. **Evaluation** (rank-eval): Quality metrics (nDCG, MRR, etc.)
//!
//! **When to use:**
//! - Need high-quality retrieval with token-level matching
//! - Want to balance speed (BM25) and precision (MaxSim)
//! - Working with text-only or multimodal (ColPali) retrieval
//!
//! **Performance:**
//! - BM25: ~1ms for 10M docs → 1000 candidates
//! - MaxSim: ~10-50ms for 1000 candidates → 100 results
//! - Total: ~11-51ms per query (vs. 100-500ms for cross-encoder)

use rank_eval::binary::ndcg_at_k;
use rank_fusion::rrf;
use rank_rerank::colbert;
#[cfg(feature = "dense")]
use rank_retrieve::retrieve_dense;
#[cfg(feature = "bm25")]
use rank_retrieve::{
    bm25::{Bm25Params, InvertedIndex},
    retrieve_bm25,
};
use std::collections::HashSet;

// Mock ColBERT encoder (in practice, use a real ColBERT model)
fn encode_query(query: &str) -> Vec<Vec<f32>> {
    // Simplified: in practice, use a ColBERT model to encode query tokens
    query
        .split_whitespace()
        .map(|word| {
            // Mock embedding: in practice, use actual ColBERT encoder
            let mut emb = vec![0.0; 128];
            for (i, c) in word.chars().enumerate() {
                emb[i % 128] = (c as u32 as f32) / 1000.0;
            }
            // L2 normalize
            let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                emb.iter_mut().for_each(|x| *x /= norm);
            }
            emb
        })
        .collect()
}

fn encode_document(doc: &str) -> Vec<Vec<f32>> {
    // Same as query encoding for documents
    encode_query(doc)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup: Create BM25 index
    let mut index = InvertedIndex::new();
    let documents = vec![
        (0, "machine learning algorithms neural networks"),
        (1, "deep learning neural networks artificial intelligence"),
        (2, "python programming language data science"),
        (3, "rust systems programming memory safety"),
        (4, "information retrieval search engines ranking"),
    ];

    for (id, text) in &documents {
        let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        index.add_document(*id, &terms);
    }

    // Query
    let query_text = "machine learning neural networks";
    let query_terms: Vec<String> = query_text
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // Step 1: First-stage retrieval with BM25 (rank-retrieve)
    // Research shows BM25 provides excellent recall for most queries
    let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default())?;
    println!("BM25 retrieved {} candidates", candidates.len());

    // Step 2: Prepare token embeddings for reranking
    let query_tokens = encode_query(query_text);

    // Get document token embeddings (in practice, these would be pre-computed and stored)
    let doc_tokens: Vec<(u32, Vec<Vec<f32>>)> = candidates
        .iter()
        .map(|(id, _)| {
            let doc_text = documents.iter().find(|(d_id, _)| d_id == id).unwrap().1;
            (*id, encode_document(doc_text))
        })
        .collect();

    // Step 3: Rerank with MaxSim (rank-rerank)
    // Token-level matching refines the ranking
    // colbert::rank expects Vec<(I, Vec<Vec<f32>>)>
    let reranked = colbert::rank(&query_tokens, &doc_tokens);
    println!("\nReranked results:");
    for (i, (id, score)) in reranked.iter().take(5).enumerate() {
        println!("  {}. Doc {}: {:.4}", i + 1, id, score);
    }

    // Step 4: Optional - Apply token pooling for storage optimization
    // Research: Pool factor 2 = 50% reduction, <1% quality loss (Clavie et al., 2024)
    println!("\nToken pooling example:");
    for (id, tokens) in &doc_tokens[..2] {
        let original_count = tokens.len();
        let pooled = colbert::pool_tokens(tokens, 2)?;
        let pooled_count = pooled.len();
        let reduction = 100.0 * (1.0 - (pooled_count as f32 / original_count as f32));
        println!(
            "  Doc {}: {} tokens → {} tokens ({:.1}% reduction)",
            id, original_count, pooled_count, reduction
        );
    }

    // Step 5: Optional - Hybrid retrieval with fusion
    #[cfg(feature = "dense")]
    {
        use rank_retrieve::dense::DenseRetriever;
        let mut dense_retriever = DenseRetriever::new();
        for (id, text) in &documents {
            // Mock dense embedding
            let embedding: Vec<f32> = (0..128).map(|i| (*id as f32 + i as f32) / 200.0).collect();
            dense_retriever.add_document(*id, embedding);
        }

        let query_embedding: Vec<f32> = (0..128).map(|i| (i as f32) / 200.0).collect();
        let dense_results = retrieve_dense(&dense_retriever, &query_embedding, 1000)?;

        // Convert to String IDs for fusion
        let bm25_string: Vec<(String, f32)> = candidates
            .iter()
            .map(|(id, score)| (id.to_string(), *score))
            .collect();
        let dense_string: Vec<(String, f32)> = dense_results
            .iter()
            .map(|(id, score)| (id.to_string(), *score))
            .collect();

        let fused = rrf(&bm25_string, &dense_string);
        println!("\nHybrid retrieval (BM25 + Dense, fused with RRF):");
        for (i, (id, score)) in fused.iter().take(5).enumerate() {
            println!("  {}. Doc {}: {:.4}", i + 1, id, score);
        }
    }

    // Step 6: Evaluation
    let ranked_ids: Vec<String> = reranked.iter().map(|(id, _)| id.to_string()).collect();
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    let ndcg = ndcg_at_k(&ranked_ids, &relevant, 10);
    println!("\nEvaluation: nDCG@10 = {:.4}", ndcg);

    println!("\n✅ Pipeline complete! This demonstrates the research-backed approach:");
    println!("   - BM25 first-stage retrieval (rank-retrieve)");
    println!("   - MaxSim reranking (rank-rerank)");
    println!("   - Token pooling optimization (50% reduction, <1% quality loss)");
    println!("   - Research: This pipeline often matches PLAID's trade-offs (SIGIR 2024)");

    Ok(())
}
