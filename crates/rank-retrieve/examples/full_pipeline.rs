//! Full pipeline example: Retrieve → Fusion → Rerank → Eval
//!
//! Demonstrates the complete ranking pipeline using all rank-* crates.

use rank_retrieve::{retrieve_bm25, retrieve_dense};
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;

fn main() {
    println!("=== Full Ranking Pipeline Example ===\n");
    
    // Stage 1: Retrieval (rank-retrieve)
    println!("Stage 1: Retrieval (10M docs → 1000 candidates)\n");
    
    // BM25 retrieval
    let mut bm25_index = InvertedIndex::new();
    bm25_index.add_document(0, &["the".to_string(), "quick".to_string(), "brown".to_string(), "fox".to_string()]);
    bm25_index.add_document(1, &["the".to_string(), "lazy".to_string(), "dog".to_string()]);
    bm25_index.add_document(2, &["quick".to_string(), "brown".to_string(), "fox".to_string(), "jumps".to_string()]);
    
    let query_terms = vec!["quick".to_string(), "fox".to_string()];
    let bm25_results = retrieve_bm25(&bm25_index, &query_terms, 1000, Bm25Params::default()).unwrap();
    
    println!("BM25 retrieved {} candidates", bm25_results.len());
    println!("Top 5: {:?}\n", &bm25_results[..5.min(bm25_results.len())]);
    
    // Dense retrieval
    let mut dense_retriever = DenseRetriever::new();
    dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
    dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);
    
    let query_embedding = [1.0, 0.0, 0.0];
    let dense_results = retrieve_dense(&dense_retriever, &query_embedding, 1000).unwrap();
    
    println!("Dense retrieved {} candidates", dense_results.len());
    println!("Top 5: {:?}\n", &dense_results[..5.min(dense_results.len())]);
    
    // Stage 2: Fusion (rank-fusion) - would use rrf_multi here
    println!("Stage 2: Fusion (combine multiple retrievers)");
    println!("  → Use rank_fusion::rrf_multi() to combine BM25 and dense results");
    println!("  → Output: Top 1000 fused candidates\n");
    
    // Stage 3: Reranking (rank-rerank) - would use maxsim here
    println!("Stage 3: Reranking (1000 → 100 candidates)");
    println!("  → Use rank_rerank::simd::maxsim_batch() for late interaction");
    println!("  → Output: Top 100 reranked candidates\n");
    
    // Stage 4: Cross-encoder (rank-rerank) - would use cross-encoder here
    println!("Stage 4: Cross-encoder (100 → 10 results)");
    println!("  → Use rank_rerank::crossencoder for final scoring");
    println!("  → Output: Top 10 final results\n");
    
    // Stage 5: Evaluation (rank-eval) - would use metrics here
    println!("Stage 5: Evaluation");
    println!("  → Use rank_eval::ndcg_at_k() to measure quality");
    println!("  → Compare against ground truth relevance\n");
    
    println!("=== Pipeline Complete ===");
    println!("\nThis example shows the structure. In production:");
    println!("1. Integrate with actual search backends (Elasticsearch, etc.)");
    println!("2. Use rank-fusion for multi-retriever fusion");
    println!("3. Use rank-rerank for MaxSim and cross-encoder reranking");
    println!("4. Use rank-eval for evaluation metrics");
}

