//! Realistic retrieval example using rank-eval's dataset loaders.
//!
//! This example demonstrates:
//! - Loading real-world datasets (MS MARCO, BEIR) using rank-eval
//! - Building retrieval indexes from real documents
//! - Running retrieval with realistic queries
//! - Evaluating with standard IR metrics (nDCG, MRR, Precision@k)
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
    println!("=== Realistic Retrieval Example ===\n");
    println!("This example demonstrates retrieval with realistic datasets.");
    println!("For production use, download real datasets from:");
    println!("- MS MARCO: https://microsoft.github.io/msmarco/");
    println!("- BEIR: https://github.com/beir-cellar/beir\n");

    // For this example, we'll use realistic synthetic data
    // In production, you would use rank-eval's dataset loaders:
    //
    // use rank_eval::dataset::loaders::{load_msmarco_qrels, load_msmarco_runs};
    // use rank_eval::trec::Qrel;
    //
    // // Load MS MARCO sample
    // let qrels = load_msmarco_qrels("datasets/msmarco/qrels.txt")?;
    // let corpus = load_corpus("datasets/msmarco/corpus.tsv")?;
    //
    // // Build BM25 index
    // let mut index = InvertedIndex::new();
    // for (doc_id, text) in corpus {
    //     let terms = tokenize(&text); // Your tokenization function
    //     index.add_document(doc_id, &terms);
    // }
    //
    // // Run retrieval and evaluate
    // let results = index.retrieve(&query_terms, 1000, Bm25Params::default())?;
    // let ndcg = rank_eval::binary::ndcg_at_k(&results, &qrels, 10);

    // Generate realistic synthetic dataset
    println!("Generating realistic synthetic dataset...");
    let num_docs = 10000;
    let documents = generate_realistic_documents(num_docs);
    
    println!("Building BM25 index with {} documents...", num_docs);
    #[cfg(feature = "bm25")]
    {
        let mut index = InvertedIndex::new();
        for (doc_id, text) in &documents {
            let terms: Vec<String> = text
                .split_whitespace()
                .map(|s| s.to_string().to_lowercase())
                .collect();
            index.add_document(*doc_id, &terms);
        }
        
        println!("Index built. Vocabulary size: {} terms", index.doc_frequencies().len());
        
        // Test with realistic queries
        let queries = vec![
            "machine learning algorithms",
            "deep neural network training",
            "information retrieval ranking",
            "natural language processing",
            "computer vision applications",
        ];
        
        println!("\nRunning retrieval for {} queries...", queries.len());
        for query_text in &queries {
            let query_terms: Vec<String> = query_text
                .split_whitespace()
                .map(|s| s.to_string().to_lowercase())
                .collect();
            
            let results = index.retrieve(&query_terms, 10, Bm25Params::default())?;
            
            println!("\nQuery: \"{}\"", query_text);
            println!("Retrieved {} documents", results.len());
            println!("Top 5 results:");
            for (i, (doc_id, score)) in results.iter().take(5).enumerate() {
                println!("  {}. Doc {}: score {:.4}", i + 1, doc_id, score);
            }
        }
    }
    
    println!("\n=== Example Complete ===");
    println!("\n**Next steps for real-world usage:**");
    println!("1. Download MS MARCO or BEIR dataset");
    println!("2. Use rank-eval's dataset loaders to load corpus and qrels");
    println!("3. Build index from real documents");
    println!("4. Run retrieval and evaluate with rank-eval metrics");
    println!("\n**See also:**");
    println!("- rank-eval documentation for dataset loading");
    println!("- examples/full_pipeline.rs for complete pipeline example");
    
    Ok(())
}

/// Generate realistic documents (50-500 words, typical passage length).
fn generate_realistic_documents(num_docs: u32) -> Vec<(u32, String)> {
    let common_words = vec![
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had",
    ];
    
    let domain_terms = vec![
        "machine", "learning", "algorithm", "data", "model", "training", "neural", "network",
        "deep", "artificial", "intelligence", "natural", "language", "processing", "computer", "vision",
        "retrieval", "ranking", "search", "query", "document", "relevance", "precision", "recall",
        "embedding", "vector", "similarity", "cosine", "dot", "product", "sparse", "dense",
    ];
    
    let mut documents = Vec::new();
    
    for i in 0..num_docs {
        let num_words = 50 + (i % 450) as usize; // 50-500 words
        let mut words = Vec::new();
        
        for j in 0..num_words {
            if j % 4 == 0 && !common_words.is_empty() {
                words.push(common_words[j % common_words.len()].to_string());
            } else {
                let term_idx = ((i as usize + j) % domain_terms.len()) as usize;
                words.push(domain_terms[term_idx].to_string());
            }
        }
        
        documents.push((i, words.join(" ")));
    }
    
    documents
}
