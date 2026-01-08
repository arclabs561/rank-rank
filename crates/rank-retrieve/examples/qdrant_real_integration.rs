//! Real-world integration example: Qdrant vector database + rank-rank crates.
//!
//! This example demonstrates a production-ready RAG pipeline using actual Qdrant client.
//! 
//! **Prerequisites:**
//! - Qdrant running on localhost:6333 (or set QDRANT_URL env var)
//! - Or use mock mode (default) - runs without Qdrant
//!
//! **To use with real Qdrant:**
//! 1. Start Qdrant: `docker run -p 6333:6333 qdrant/qdrant`
//! 2. Add to Cargo.toml: `qdrant-client = { version = "1.7", optional = true }`
//! 3. Run: `cargo run --example qdrant_real_integration --features qdrant`
//!
//! **Mock mode (default):**
//! Runs without Qdrant, demonstrates the integration pattern with simulated data.

use rank_retrieve::{retrieve_bm25};
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_fusion::rrf;
use rank_rerank::simd;
use rank_eval::binary::ndcg_at_k;
use std::collections::HashSet;
use std::env;

#[cfg(feature = "qdrant")]
use qdrant_client::{
    prelude::*,
    qdrant::{SearchPoints, Vector},
};

/// Real Qdrant integration (when feature enabled)
#[cfg(feature = "qdrant")]
async fn search_qdrant_real(
    client: &QdrantClient,
    query_embedding: &[f32],
    top_k: usize,
) -> Result<Vec<(String, f32)>, Box<dyn std::error::Error>> {
    let results = client
        .search_points(&SearchPoints {
            collection_name: "documents".to_string(),
            vector: Some(Vector {
                vector: Some(qdrant_client::qdrant::vectors::Vectors::Dense(
                    query_embedding.to_vec(),
                )),
            }),
            limit: top_k as u64,
            with_payload: Some(true.into()),
            ..Default::default()
        })
        .await?;

    Ok(results
        .result
        .into_iter()
        .enumerate()
        .map(|(i, point)| {
            let id = point.id
                .and_then(|id| match id {
                    qdrant_client::qdrant::PointId { point_id_options } => {
                        match point_id_options {
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Uuid(uuid)) => {
                                Some(uuid)
                            }
                            Some(qdrant_client::qdrant::point_id::PointIdOptions::Num(num)) => {
                                Some(num.to_string())
                            }
                            None => Some(i.to_string()),
                        }
                    }
                })
                .unwrap_or_else(|| i.to_string());
            let score = point.score.unwrap_or(0.0) as f32;
            (id, score)
        })
        .collect())
}

/// Mock Qdrant integration (when feature disabled)
#[cfg(not(feature = "qdrant"))]
fn search_qdrant_mock(
    _url: &str,
    _query_embedding: &[f32],
    top_k: usize,
) -> Vec<(String, f32)> {
    println!("[Mock Mode] Simulating Qdrant search (add --features qdrant for real integration)");
    
    // Simulate dense retrieval results
    (0..top_k.min(5))
        .map(|i| {
            let score = 0.95 - (i as f32 * 0.1);
            (format!("qdrant_doc_{}", i + 1), score)
        })
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Real-World Qdrant + rank-rank Integration ===\n");
    
    let qdrant_url = env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
    let query = "machine learning algorithms";
    let query_embedding = vec![0.1f32; 128]; // Example 128-dim embedding
    
    // Step 1: Dense retrieval from Qdrant
    println!("📚 Step 1: Dense Retrieval (Qdrant)");
    
    let dense_results = {
        #[cfg(feature = "qdrant")]
        {
            let client = QdrantClient::from_url(&qdrant_url).build()?;
            let results = futures::executor::block_on(search_qdrant_real(
                &client,
                &query_embedding,
                100,
            ))?;
            println!("✅ Retrieved {} documents from Qdrant", results.len());
            results
        }
        
        #[cfg(not(feature = "qdrant"))]
        {
            let results = search_qdrant_mock(&qdrant_url, &query_embedding, 100);
            println!("✅ Retrieved {} documents (mock mode)", results.len());
            results
        }
    };
    
    // Step 2: Sparse retrieval with BM25 (rank-retrieve)
    println!("\n🔍 Step 2: Sparse Retrieval (BM25)");
    
    let mut bm25_index = InvertedIndex::new();
    let documents = vec![
        (0, "machine learning algorithms neural networks deep learning"),
        (1, "information retrieval search engines ranking"),
        (2, "natural language processing transformers BERT GPT"),
        (3, "computer vision image recognition convolutional networks"),
        (4, "reinforcement learning Q-learning policy gradients"),
    ];
    
    for (doc_id, text) in &documents {
        let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        bm25_index.add_document(*doc_id, &terms);
    }
    
    let query_terms: Vec<String> = query.split_whitespace().map(|s| s.to_string()).collect();
    let sparse_results = retrieve_bm25(&bm25_index, &query_terms, 100, Bm25Params::default())?;
    
    // Convert to fusion format (String IDs for compatibility)
    let dense_fusion: Vec<(String, f32)> = dense_results;
    let sparse_fusion: Vec<(String, f32)> = sparse_results
        .iter()
        .map(|(id, score)| (id.to_string(), *score))
        .collect();
    
    println!("✅ Retrieved {} documents with BM25", sparse_fusion.len());
    
    // Step 3: Rank fusion (rank-fusion)
    println!("\n🔀 Step 3: Rank Fusion (RRF)");
    
    let fused = rrf(&dense_fusion, &sparse_fusion);
    println!("✅ Fused to {} results", fused.len());
    for (i, (id, score)) in fused.iter().take(5).enumerate() {
        println!("   {}. {} (score: {:.4})", i + 1, id, score);
    }
    
    // Step 4: Rerank with MaxSim (rank-rerank)
    println!("\n🎯 Step 4: Reranking (MaxSim)");
    
    // Simulate query and document token embeddings for MaxSim
    let query_tokens = vec![
        vec![1.0, 0.0, 0.0],  // "machine"
        vec![0.0, 1.0, 0.0],  // "learning"
    ];
    
    let top_candidates: Vec<(String, Vec<Vec<f32>>)> = fused
        .iter()
        .take(10)
        .map(|(id, _)| {
            // Simulate document token embeddings
            let doc_tokens = vec![
                vec![0.9, 0.1, 0.0],
                vec![0.1, 0.9, 0.0],
            ];
            (id.clone(), doc_tokens)
        })
        .collect();
    
    let mut reranked: Vec<(String, f32)> = top_candidates
        .iter()
        .map(|(id, doc_tokens)| {
            let score = simd::maxsim_vecs(&query_tokens, doc_tokens);
            (id.clone(), score)
        })
        .collect();
    
    reranked.sort_by(|a, b| b.1.total_cmp(&a.1));
    
    println!("✅ Reranked {} candidates", reranked.len());
    for (i, (id, score)) in reranked.iter().take(5).enumerate() {
        println!("   {}. {} (score: {:.4})", i + 1, id, score);
    }
    
    // Step 5: Evaluate with rank-eval
    println!("\n📊 Step 5: Evaluation (nDCG)");
    
    let ranked: Vec<String> = reranked.iter().map(|(id, _)| id.clone()).collect();
    let relevant: HashSet<String> = ["0", "qdrant_doc_1"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    
    let ndcg = ndcg_at_k(&ranked, &relevant, 10);
    println!("✅ nDCG@10: {:.4}", ndcg);
    
    println!("\n✅ Complete pipeline executed successfully!");
    println!("\nProduction setup:");
    println!("  1. Start Qdrant: docker run -p 6333:6333 qdrant/qdrant");
    println!("  2. Add qdrant-client to Cargo.toml");
    println!("  3. Run with --features qdrant");
    println!("  4. See docs/VECTOR_DATABASE_INTEGRATION.md for details");
    
    Ok(())
}

