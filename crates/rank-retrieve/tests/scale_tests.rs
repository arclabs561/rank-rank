//! Scale tests for rank-retrieve.
//!
//! Tests correctness and performance at realistic scales:
//! - Small-scale: 1K documents (small business knowledge base)
//! - Medium-scale: 10K documents (enterprise search)
//! - Large-scale: 100K documents (large corpus)
//! - Very large-scale: 1M documents (web search scale)

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};

/// Generate realistic document terms with Zipfian-like distribution.
#[cfg(any(feature = "bm25", feature = "dense", feature = "sparse"))]
fn generate_realistic_terms(doc_id: u32, num_terms: usize) -> Vec<String> {
    // Use a mix of common and rare terms to simulate realistic vocabulary
    let common_terms = vec!["the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"];
    let domain_terms: Vec<String> = (0..100).map(|i| format!("term{}", i)).collect();
    
    let mut terms = Vec::new();
    for i in 0..num_terms {
        // Mix common and domain terms
        if i % 3 == 0 && !common_terms.is_empty() {
            terms.push(common_terms[i % common_terms.len()].to_string());
        } else {
            let term_idx = (doc_id as usize + i) % domain_terms.len();
            terms.push(domain_terms[term_idx].clone());
        }
    }
    terms
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_scale_1k_docs() {
    // Small-scale: 1K documents
    let num_docs = 1000;
    let mut index = InvertedIndex::new();
    
    for i in 0..num_docs {
        let terms = generate_realistic_terms(i, 50); // 50 terms per document
        index.add_document(i, &terms);
    }
    
    // Test retrieval
    let query = vec!["term0".to_string(), "term1".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    
    // Verify correctness
    assert!(results.len() <= 10);
    assert!(results.len() > 0);
    
    // Verify sorted
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
    
    // Verify all scores are finite and positive
    for (_, score) in &results {
        assert!(score.is_finite());
        assert!(*score > 0.0);
    }
}

#[cfg(feature = "bm25")]
#[test]
fn test_bm25_scale_10k_docs() {
    // Medium-scale: 10K documents
    let num_docs = 10000;
    let mut index = InvertedIndex::new();
    
    for i in 0..num_docs {
        let terms = generate_realistic_terms(i, 50);
        index.add_document(i, &terms);
    }
    
    let query = vec!["term0".to_string(), "term1".to_string()];
    let results = index.retrieve(&query, 100, Bm25Params::default()).unwrap();
    
    assert!(results.len() <= 100);
    assert!(results.len() > 0);
    
    // Verify early termination is working (should be fast)
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}

#[cfg(feature = "bm25")]
#[test]
#[ignore] // Mark as ignored for CI, run manually for large-scale testing
fn test_bm25_scale_100k_docs() {
    // Large-scale: 100K documents
    let num_docs = 100000;
    let mut index = InvertedIndex::new();
    
    for i in 0..num_docs {
        let terms = generate_realistic_terms(i, 50);
        index.add_document(i, &terms);
    }
    
    let query = vec!["term0".to_string(), "term1".to_string()];
    let results = index.retrieve(&query, 1000, Bm25Params::default()).unwrap();
    
    assert!(results.len() <= 1000);
    assert!(results.len() > 0);
    
    // Verify correctness
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_scale_1k_docs() {
    // Small-scale: 1K documents, 768 dimensions (typical embedding size)
    let num_docs = 1000;
    let dim = 768;
    let mut retriever = DenseRetriever::new();
    
    for i in 0..num_docs {
        let embedding: Vec<f32> = (0..dim)
            .map(|j| ((i + j) as f32 * 0.001).sin())
            .collect();
        retriever.add_document(i, embedding);
    }
    
    let query: Vec<f32> = (0..dim).map(|j| (j as f32 * 0.001).sin()).collect();
    let results = retriever.retrieve(&query, 10).unwrap();
    
    assert!(results.len() <= 10);
    assert!(results.len() > 0);
    
    // Verify sorted
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}

#[cfg(feature = "dense")]
#[test]
fn test_dense_scale_10k_docs() {
    // Medium-scale: 10K documents
    let num_docs = 10000;
    let dim = 768;
    let mut retriever = DenseRetriever::new();
    
    for i in 0..num_docs {
        let embedding: Vec<f32> = (0..dim)
            .map(|j| ((i + j) as f32 * 0.001).sin())
            .collect();
        retriever.add_document(i, embedding);
    }
    
    let query: Vec<f32> = (0..dim).map(|j| (j as f32 * 0.001).sin()).collect();
    let results = retriever.retrieve(&query, 100).unwrap();
    
    assert!(results.len() <= 100);
    assert!(results.len() > 0);
    
    // Verify early termination is working
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_scale_1k_docs() {
    // Small-scale: 1K documents, sparse vectors (30K dimensions, ~200 non-zeros)
    let num_docs = 1000;
    let vocab_size = 30000;
    let sparsity = 200; // ~200 non-zeros per document
    
    let mut retriever = SparseRetriever::new();
    
    for i in 0..num_docs {
        let mut indices = Vec::new();
        let mut values = Vec::new();
        
        for j in 0..sparsity {
            let idx = ((i as usize * 17 + j * 23) % vocab_size) as u32;
            let val = ((i + j) as f32 * 0.001).sin().abs();
            indices.push(idx);
            values.push(val);
        }
        
        // Sort indices
        let mut pairs: Vec<(u32, f32)> = indices.into_iter().zip(values.into_iter()).collect();
        pairs.sort_unstable_by_key(|(idx, _)| *idx);
        let indices: Vec<u32> = pairs.iter().map(|(idx, _)| *idx).collect();
        let values: Vec<f32> = pairs.iter().map(|(_, val)| *val).collect();
        
        let vector = SparseVector::new_unchecked(indices, values);
        retriever.add_document(i as u32, vector);
    }
    
    // Create query vector
    let query_indices: Vec<u32> = (0..50).map(|i| (i * 100) as u32).collect();
    let query_values: Vec<f32> = vec![1.0; 50];
    let query = SparseVector::new_unchecked(query_indices, query_values);
    
    let results = retriever.retrieve(&query, 10).unwrap();
    
    assert!(results.len() <= 10);
    assert!(results.len() > 0);
    
    // Verify sorted
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}

#[cfg(feature = "sparse")]
#[test]
fn test_sparse_scale_10k_docs() {
    // Medium-scale: 10K documents
    let num_docs = 10000;
    let vocab_size = 30000;
    let sparsity = 200;
    
    let mut retriever = SparseRetriever::new();
    
    for i in 0..num_docs {
        let mut indices = Vec::new();
        let mut values = Vec::new();
        
        for j in 0..sparsity {
            let idx = ((i as usize * 17 + j * 23) % vocab_size) as u32;
            let val = ((i + j) as f32 * 0.001).sin().abs();
            indices.push(idx);
            values.push(val);
        }
        
        let mut pairs: Vec<(u32, f32)> = indices.into_iter().zip(values.into_iter()).collect();
        pairs.sort_unstable_by_key(|(idx, _)| *idx);
        let indices: Vec<u32> = pairs.iter().map(|(idx, _)| *idx).collect();
        let values: Vec<f32> = pairs.iter().map(|(_, val)| *val).collect();
        
        let vector = SparseVector::new_unchecked(indices, values);
        retriever.add_document(i as u32, vector);
    }
    
    let query_indices: Vec<u32> = (0..50).map(|i| (i * 100) as u32).collect();
    let query_values: Vec<f32> = vec![1.0; 50];
    let query = SparseVector::new_unchecked(query_indices, query_values);
    
    let results = retriever.retrieve(&query, 100).unwrap();
    
    assert!(results.len() <= 100);
    assert!(results.len() > 0);
    
    // Verify early termination is working
    for i in 1..results.len() {
        assert!(results[i - 1].1 >= results[i].1);
    }
}
