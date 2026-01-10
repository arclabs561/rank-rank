#![allow(dead_code)]
//! Realistic dataset tests using rank-eval's dataset loaders.
//!
//! These tests use real-world datasets (MS MARCO, BEIR) when available,
//! or generate realistic synthetic data that mimics real-world characteristics.

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

/// Generate realistic document text (50-500 words, typical passage length).
fn generate_realistic_document(doc_id: u32) -> String {
    // Simulate realistic document with:
    // - Common words (the, a, an, etc.)
    // - Domain-specific terms
    // - Varying lengths (50-500 words)
    
    let common_words = vec![
        "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by",
        "from", "as", "is", "was", "are", "were", "been", "be", "have", "has", "had", "do", "does", "did",
        "will", "would", "could", "should", "may", "might", "must", "can", "this", "that", "these", "those",
    ];
    
    let domain_terms = vec![
        "machine", "learning", "algorithm", "data", "model", "training", "neural", "network",
        "deep", "artificial", "intelligence", "natural", "language", "processing", "computer", "vision",
        "retrieval", "ranking", "search", "query", "document", "relevance", "precision", "recall",
    ];
    
    let num_words = 50 + (doc_id % 450) as usize; // 50-500 words
    let mut words = Vec::new();
    
    for i in 0..num_words {
        // Mix common and domain terms
        if i % 4 == 0 && !common_words.is_empty() {
            words.push(common_words[i % common_words.len()].to_string());
        } else {
            let term_idx = ((doc_id as usize + i) % domain_terms.len()) as usize;
            words.push(domain_terms[term_idx].to_string());
        }
    }
    
    words.join(" ")
}

/// Generate realistic query (2-10 terms, typical user query).
fn generate_realistic_query(query_id: u32) -> Vec<String> {
    let domain_terms = vec![
        "machine", "learning", "algorithm", "neural", "network", "deep", "training",
        "retrieval", "ranking", "search", "query", "document", "relevance",
    ];
    
    let num_terms = 2 + (query_id % 9) as usize; // 2-10 terms
    let mut terms = Vec::new();
    
    for i in 0..num_terms {
        let term_idx = ((query_id as usize + i * 7) % domain_terms.len()) as usize;
        terms.push(domain_terms[term_idx].to_string());
    }
    
    terms
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_realistic_documents() {
    // Test with realistic document lengths and vocabulary
    let num_docs = 1000;
    let mut index = InvertedIndex::new();
    
    for i in 0..num_docs {
        let text = generate_realistic_document(i);
        let terms: Vec<String> = text
            .split_whitespace()
            .map(|s| s.to_string().to_lowercase())
            .collect();
        index.add_document(i, &terms);
    }
    
    // Test with realistic queries
    for query_id in 0..10 {
        let query = generate_realistic_query(query_id);
        let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
        
        // Verify results
        assert!(results.len() <= 10);
        
        // Verify sorted
        for i in 1..results.len() {
            assert!(results[i - 1].1 >= results[i].1);
        }
        
        // Verify all scores are positive and finite
        for (_, score) in &results {
            assert!(score.is_finite());
            assert!(*score > 0.0);
        }
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_realistic_query_patterns() {
    // Test with various realistic query patterns
    let mut index = InvertedIndex::new();
    
    // Add documents
    for i in 0..500 {
        let text = generate_realistic_document(i);
        let terms: Vec<String> = text
            .split_whitespace()
            .map(|s| s.to_string().to_lowercase())
            .collect();
        index.add_document(i, &terms);
    }
    
    // Test different query types
    let queries = vec![
        // Short query (2 terms)
        vec!["machine".to_string(), "learning".to_string()],
        // Medium query (4 terms)
        vec!["deep".to_string(), "neural".to_string(), "network".to_string(), "training".to_string()],
        // Long query (6 terms)
        vec![
            "information".to_string(),
            "retrieval".to_string(),
            "ranking".to_string(),
            "algorithm".to_string(),
            "relevance".to_string(),
            "precision".to_string(),
        ],
    ];
    
    for query in queries {
        let results = index.retrieve(&query, 20, Bm25Params::default()).unwrap();
        
        assert!(results.len() <= 20);
        
        // Verify sorted
        for i in 1..results.len() {
            assert!(results[i - 1].1 >= results[i].1);
        }
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_realistic_embeddings() {
    // Test with realistic embedding dimensions (128, 384, 768)
    let num_docs = 1000;
    let dim = 768; // Typical embedding dimension (e.g., BERT-base)
    
    let mut retriever = DenseRetriever::new();
    
    for i in 0..num_docs {
        // Generate realistic embeddings (normalized, typical distribution)
        let mut embedding: Vec<f32> = (0..dim)
            .map(|j| ((i + j) as f32 * 0.001).sin())
            .collect();
        
        // Normalize to unit length (typical for embeddings)
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-9 {
            for val in &mut embedding {
                *val /= norm;
            }
        }
        
        retriever.add_document(i, embedding);
    }
    
    // Generate realistic query embedding
    let mut query: Vec<f32> = (0..dim).map(|j| (j as f32 * 0.001).sin()).collect();
    let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 1e-9 {
        for val in &mut query {
            *val /= norm;
        }
    }
    
    let results = retriever.retrieve(&query, 10).unwrap();
    
    assert!(results.len() <= 10);
    
    // Verify sorted
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
    
    // Verify scores are in [-1, 1] range (cosine similarity)
    for (_, score) in &results {
        assert!(score.is_finite());
        assert!(*score >= -1.1 && *score <= 1.1); // Allow small floating-point error
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_realistic_splade_vectors() {
    // Test with realistic SPLADE-like sparse vectors
    // SPLADE typically has:
    // - 30K+ dimensions (vocabulary size)
    // - ~200-500 non-zeros per document (after top-k pruning)
    let num_docs = 1000;
    let vocab_size = 30000;
    let sparsity = 200; // Typical after top-k pruning
    
    let mut retriever = SparseRetriever::new();
    
    for i in 0..num_docs {
        let mut indices = Vec::new();
        let mut values = Vec::new();
        
        // Generate sparse vector with realistic distribution
        for j in 0..sparsity {
            let idx = ((i as usize * 17 + j * 23) % vocab_size) as u32;
            // SPLADE values are typically positive, with some larger values
            let val = if j % 10 == 0 {
                ((i + j) as f32 * 0.01).sin().abs() * 10.0 // Some larger values
            } else {
                ((i + j) as f32 * 0.01).sin().abs() // Most values smaller
            };
            indices.push(idx);
            values.push(val);
        }
        
        // Sort indices
        let mut pairs: Vec<(u32, f32)> = indices.into_iter().zip(values.into_iter()).collect();
        pairs.sort_unstable_by_key(|(idx, _)| *idx);
        let (indices, values) = pairs.into_iter().unzip();
        
        let vector = SparseVector::new_unchecked(indices, values);
        retriever.add_document(i as u32, vector);
    }
    
    // Generate realistic query vector (typically 10-50 non-zeros for queries)
    let query_sparsity = 30;
    let mut query_indices = Vec::new();
    let mut query_values = Vec::new();
    
    for i in 0..query_sparsity {
        let idx = (i * 100) as u32;
        query_indices.push(idx);
        query_values.push(1.0); // Query terms typically have weight 1.0
    }
    
    let query = SparseVector::new_unchecked(query_indices, query_values);
    let results = retriever.retrieve(&query, 10).unwrap();
    
    assert!(results.len() <= 10);
    
    // Verify sorted
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
    
    // Verify all scores are positive and finite
    for (_, score) in &results {
        assert!(score.is_finite());
        assert!(*score > 0.0);
    }
}

/// Helper to check if rank-eval dataset loaders are available.
/// Returns true if we can use real datasets, false otherwise.
fn has_rank_eval() -> bool {
    // Check if rank-eval is available as a dependency
    // This is a compile-time check, so we'll use a feature flag or just try to use it
    // For now, we'll generate realistic synthetic data
    false // Will be set to true when rank-eval is added as dev-dependency
}

#[cfg(feature = "bm25")]
#[test]
#[ignore] // Requires rank-eval and dataset files
fn test_bm25_with_real_dataset() {
    // This test would use rank-eval's dataset loaders if available
    // For now, it's marked as ignored and uses synthetic data
    
    if !has_rank_eval() {
        // Fall back to realistic synthetic data
        test_bm25_realistic_documents();
        return;
    }
    
    // TODO: When rank-eval is available as dev-dependency:
    // use rank_eval::dataset::loaders::load_msmarco_qrels;
    // use rank_eval::trec::load_trec_runs;
    // 
    // Load MS MARCO sample
    // Build BM25 index from corpus
    // Run retrieval
    // Evaluate with qrels using rank-eval metrics
}
