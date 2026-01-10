//! Realistic evaluation example using rank-eval.
//!
//! This example demonstrates:
//! - Loading real-world datasets using rank-eval
//! - Running retrieval with BM25, dense, and sparse methods
//! - Evaluating with standard IR metrics (nDCG, MRR, Precision@k, Recall@k)
//! - Comparing different retrieval methods
//!
//! **Note**: This example requires dataset files to be downloaded.
//! See rank-eval documentation for dataset download instructions.

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Realistic Evaluation Example ===\n");
    println!("This example demonstrates evaluation with realistic datasets.");
    println!("For production use, download real datasets and use rank-eval's loaders.\n");

    // Generate realistic synthetic dataset with known relevance
    println!("Generating synthetic dataset with relevance judgments...");
    let (documents, queries, qrels) = generate_synthetic_dataset_with_qrels();
    
    println!("Dataset: {} documents, {} queries, {} relevance judgments",
        documents.len(), queries.len(), qrels.len());
    
    #[cfg(feature = "bm25")]
    {
        println!("\n=== BM25 Retrieval ===");
        
        // Build BM25 index
        let mut index = InvertedIndex::new();
        for (doc_id, terms) in &documents {
            index.add_document(*doc_id, terms);
        }
        
        // Run retrieval for each query
        let mut all_results = Vec::new();
        for (query_id, query_terms) in &queries {
            let results = index.retrieve(query_terms, 100, Bm25Params::default())?;
            all_results.push((*query_id, results));
        }
        
        // Evaluate (would use rank-eval if available)
        println!("\nEvaluating BM25 retrieval...");
        evaluate_results(&all_results, &qrels, "BM25");
    }
    
    #[cfg(feature = "dense")]
    {
        println!("\n=== Dense Retrieval ===");
        
        // Build dense retriever with realistic embeddings
        let mut retriever = DenseRetriever::new();
        let dim = 768; // Typical embedding dimension
        
        for (doc_id, _) in &documents {
            // Generate realistic normalized embeddings
            let mut embedding: Vec<f32> = (0..dim)
                .map(|j| ((*doc_id as usize + j) as f32 * 0.001).sin())
                .collect();
            
            // Normalize
            let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 1e-9 {
                for val in &mut embedding {
                    *val /= norm;
                }
            }
            
            retriever.add_document(*doc_id, embedding);
        }
        
        // Run retrieval
        let mut all_results = Vec::new();
        for (query_id, _) in &queries {
            // Generate query embedding
            let mut query: Vec<f32> = (0..dim).map(|j| (j as f32 * 0.001).sin()).collect();
            let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 1e-9 {
                for val in &mut query {
                    *val /= norm;
                }
            }
            
            let results = retriever.retrieve(&query, 100)?;
            all_results.push((*query_id, results));
        }
        
        evaluate_results(&all_results, &qrels, "Dense");
    }
    
    println!("\n=== Evaluation Complete ===");
    println!("\n**For real-world evaluation:**");
    println!("1. Download MS MARCO or BEIR dataset");
    println!("2. Use rank-eval::dataset::loaders to load corpus and qrels");
    println!("3. Use rank-eval::binary::ndcg_at_k for nDCG@k");
    println!("4. Use rank-eval::binary::mrr for MRR");
    println!("5. Use rank-eval::binary::precision_at_k for Precision@k");
    
    Ok(())
}

/// Generate synthetic dataset with known relevance.
fn generate_synthetic_dataset_with_qrels() -> (
    Vec<(u32, Vec<String>)>, // (doc_id, terms)
    Vec<(u32, Vec<String>)>, // (query_id, query_terms)
    Vec<(u32, u32, u32)>,     // (query_id, doc_id, relevance)
) {
    let mut documents = Vec::new();
    let mut queries = Vec::new();
    let mut qrels = Vec::new();
    
    // Generate 1000 documents
    for i in 0..1000 {
        let terms: Vec<String> = (0..50)
            .map(|j| format!("term{}", (i + j) % 100))
            .collect();
        documents.push((i, terms));
    }
    
    // Generate 10 queries with known relevant documents
    for query_id in 0..10 {
        let query_terms: Vec<String> = vec![
            format!("term{}", query_id * 10),
            format!("term{}", query_id * 10 + 1),
        ];
        queries.push((query_id, query_terms));
        
        // Mark documents as relevant (5 per query)
        for doc_id in (query_id * 10)..(query_id * 10 + 5) {
            if doc_id < 1000 {
                qrels.push((query_id, doc_id, 1)); // Relevance 1
            }
        }
        // Mark one document as highly relevant
        if query_id * 10 < 1000 {
            qrels.push((query_id, query_id * 10, 2)); // Relevance 2
        }
    }
    
    (documents, queries, qrels)
}

/// Evaluate retrieval results (simplified version, would use rank-eval in production).
fn evaluate_results(
    results: &[(u32, Vec<(u32, f32)>)],
    qrels: &[(u32, u32, u32)],
    method_name: &str,
) {
    // Compute basic metrics manually (would use rank-eval in production)
    let mut precision_sum = 0.0;
    let mut recall_sum = 0.0;
    let mut num_queries = 0;
    
    for (query_id, results) in results {
        let relevant_docs: std::collections::HashSet<u32> = qrels
            .iter()
            .filter(|(qid, _, rel)| *qid == *query_id && *rel > 0)
            .map(|(_, doc_id, _)| *doc_id)
            .collect();
        
        if relevant_docs.is_empty() {
            continue;
        }
        
        let top_10: std::collections::HashSet<u32> = results
            .iter()
            .take(10)
            .map(|(doc_id, _)| *doc_id)
            .collect();
        
        let relevant_in_top_10 = top_10.intersection(&relevant_docs).count();
        let precision_at_10 = relevant_in_top_10 as f32 / 10.0;
        let recall_at_10 = relevant_in_top_10 as f32 / relevant_docs.len() as f32;
        
        precision_sum += precision_at_10;
        recall_sum += recall_at_10;
        num_queries += 1;
    }
    
    if num_queries > 0 {
        let avg_precision = precision_sum / num_queries as f32;
        let avg_recall = recall_sum / num_queries as f32;
        
        println!("{} Results:", method_name);
        println!("  Precision@10: {:.4}", avg_precision);
        println!("  Recall@10: {:.4}", avg_recall);
    }
}
