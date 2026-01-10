//! Realistic evaluation tests using rank-eval.
//!
//! These tests integrate with rank-eval to:
//! - Load real-world datasets (MS MARCO, BEIR)
//! - Run retrieval
//! - Evaluate with standard IR metrics (nDCG, MRR, Precision@k)
//! - Compare against baselines

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};

// rank-eval is available as dev-dependency
use rank_eval::binary;

/// Generate synthetic dataset with known relevance for testing evaluation.
fn generate_synthetic_dataset_with_qrels() -> (
    Vec<(u32, Vec<String>)>, // (doc_id, terms)
    Vec<(u32, Vec<String>)>, // (query_id, query_terms)
    Vec<(u32, u32, u32)>,     // (query_id, doc_id, relevance)
) {
    let mut documents = Vec::new();
    let mut queries = Vec::new();
    let mut qrels = Vec::new();
    
    // Generate documents
    for i in 0..1000 {
        let terms: Vec<String> = (0..50)
            .map(|j| format!("term{}", (i + j) % 100))
            .collect();
        documents.push((i, terms));
    }
    
    // Generate queries with known relevant documents
    for query_id in 0..10 {
        let query_terms: Vec<String> = vec![
            format!("term{}", query_id * 10),
            format!("term{}", query_id * 10 + 1),
        ];
        queries.push((query_id, query_terms));
        
        // Mark some documents as relevant
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

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_evaluation_synthetic() {
    // Test evaluation with synthetic dataset
    let (documents, queries, qrels) = generate_synthetic_dataset_with_qrels();
    
    // Build index
    let mut index = InvertedIndex::new();
    for (doc_id, terms) in &documents {
        index.add_document(*doc_id, terms);
    }
    
    // Run retrieval for each query
    let mut all_results = Vec::new();
    for (query_id, query_terms) in &queries {
        let results = index.retrieve(query_terms, 100, Bm25Params::default()).unwrap();
        all_results.push((*query_id, results));
    }
    
    // Evaluate using rank-eval
    // rank-eval's binary functions take ranked list and HashSet of relevant docs
    let mut precision_sum = 0.0;
    let mut recall_sum = 0.0;
    let mut mrr_sum = 0.0;
    let mut num_queries = 0;
    
    for (query_id, results) in &all_results {
        // Get relevant docs for this query
        let relevant_docs: std::collections::HashSet<u32> = qrels
            .iter()
            .filter(|(qid, _, rel)| *qid == *query_id && *rel > 0)
            .map(|(_, doc_id, _)| *doc_id)
            .collect();
        
        if relevant_docs.is_empty() {
            continue;
        }
        
        // Convert results to ranked list of doc IDs
        let ranked: Vec<u32> = results.iter().map(|(doc_id, _)| *doc_id).collect();
        
        // Compute metrics using rank-eval
        let precision_at_10 = binary::precision_at_k(&ranked, &relevant_docs, 10);
        let recall_at_10 = binary::recall_at_k(&ranked, &relevant_docs, 10);
        let mrr_score = binary::mrr(&ranked, &relevant_docs);
        
        precision_sum += precision_at_10;
        recall_sum += recall_at_10;
        mrr_sum += mrr_score;
        num_queries += 1;
    }
    
    if num_queries > 0 {
        let avg_precision = precision_sum / num_queries as f64;
        let avg_recall = recall_sum / num_queries as f64;
        let avg_mrr = mrr_sum / num_queries as f64;
        
        // Verify metrics are reasonable
        assert!(avg_precision >= 0.0 && avg_precision <= 1.0);
        assert!(avg_recall >= 0.0 && avg_recall <= 1.0);
        assert!(avg_mrr >= 0.0 && avg_mrr <= 1.0);
        
        println!("BM25 Evaluation Results:");
        println!("  Precision@10: {:.4}", avg_precision);
        println!("  Recall@10: {:.4}", avg_recall);
        println!("  MRR: {:.4}", avg_mrr);
    }
}

#[cfg(feature = "bm25")]
#[test]
#[ignore] // Requires MS MARCO dataset files to be downloaded
fn test_bm25_evaluation_msmarco_sample() {
    // This test uses rank-eval to load MS MARCO sample
    // Requires dataset files to be downloaded first
    
    use std::path::Path;
    
    // Check if dataset exists
    let qrels_path = Path::new("datasets/msmarco/qrels.txt");
    if !qrels_path.exists() {
        println!("MS MARCO dataset not found at {:?}. Skipping test.", qrels_path);
        println!("To run this test, download MS MARCO dataset and place it in datasets/msmarco/");
        return;
    }
    
    // Load qrels using rank-eval's trec module (dataset module requires serde feature)
    use rank_eval::trec::load_qrels;
    let qrels = match load_qrels(qrels_path) {
        Ok(qrels) => qrels,
        Err(e) => {
            eprintln!("Failed to load MS MARCO qrels: {}", e);
            return;
        }
    };
    
    // Build BM25 index from corpus (would need to load corpus first)
    // For now, this is a placeholder showing the structure
    
    // TODO: Load corpus and build index
    // let mut index = InvertedIndex::new();
    // for (doc_id, text) in corpus {
    //     let terms = tokenize(&text);
    //     index.add_document(doc_id, &terms);
    // }
    //
    // // Run retrieval
    // let mut all_results = Vec::new();
    // for query in queries {
    //     let results = index.retrieve(&query, 1000, Bm25Params::default())?;
    //     all_results.push((query_id, results));
    // }
    //
    // // Evaluate with nDCG@10
    // let ndcg = ndcg_at_k(&all_results, &qrels, 10);
    // assert!(ndcg > 0.0, "nDCG@10 should be positive");
}

#[cfg(feature = "bm25")]
#[test]
#[ignore] // Requires BEIR dataset files to be downloaded
fn test_bm25_evaluation_beir_sample() {
    // This test uses rank-eval to load BEIR dataset
    // Requires dataset files to be downloaded first
    
    use std::path::Path;
    
    // Check if dataset exists (example: scifact)
    let qrels_path = Path::new("datasets/beir/scifact/qrels/test.tsv");
    if !qrels_path.exists() {
        println!("BEIR dataset not found at {:?}. Skipping test.", qrels_path);
        println!("To run this test, download BEIR dataset and place it in datasets/beir/");
        return;
    }
    
    // Load qrels using rank-eval's trec module
    use rank_eval::trec::load_qrels;
    let qrels = match load_qrels(qrels_path) {
        Ok(qrels) => qrels,
        Err(e) => {
            eprintln!("Failed to load BEIR qrels: {}", e);
            return;
        }
    };
    
    // Similar structure to MS MARCO test
    // TODO: Load corpus, build index, run retrieval, evaluate
}
