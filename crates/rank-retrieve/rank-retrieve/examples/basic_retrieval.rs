//! Basic retrieval examples.
//!
//! Demonstrates how to use rank-retrieve for first-stage retrieval.

use rank_retrieve::{bm25::*, dense::*, sparse::*};
use rank_sparse::SparseVector;

fn main() {
    println!("=== BM25 Retrieval Example ===\n");
    
    // Create an inverted index
    let mut index = InvertedIndex::new();
    
    // Add documents
    index.add_document(0, &["the".to_string(), "quick".to_string(), "brown".to_string(), "fox".to_string()]);
    index.add_document(1, &["the".to_string(), "lazy".to_string(), "dog".to_string()]);
    index.add_document(2, &["quick".to_string(), "brown".to_string(), "fox".to_string(), "jumps".to_string()]);
    index.add_document(3, &["over".to_string(), "the".to_string(), "lazy".to_string(), "dog".to_string()]);
    
    // Query
    let query = vec!["quick".to_string(), "fox".to_string()];
    let results = index.retrieve(&query, 10, Bm25Params::default().unwrap().unwrap()).unwrap();
    
    println!("Query: {:?}", query);
    println!("Results:");
    for (doc_id, score) in results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }
    
    println!("\n=== Dense Retrieval Example ===\n");
    
    let mut dense_retriever = DenseRetriever::new();
    
    // Add documents with embeddings (normalized)
    dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
    dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);
    
    // Query embedding
    let query_embedding = vec![1.0, 0.0, 0.0];
    let results = dense_retriever.retrieve(&query_embedding, 10).unwrap();
    
    println!("Query embedding: {:?}", query_embedding);
    println!("Results:");
    for (doc_id, score) in results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }
    
    println!("\n=== Sparse Retrieval Example ===\n");
    
    let mut sparse_retriever = SparseRetriever::new();
    
    // Document 0: terms 0, 1, 2
    let doc0 = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
    sparse_retriever.add_document(0, doc0);
    
    // Document 1: terms 1, 2, 3
    let doc1 = SparseVector::new_unchecked(vec![1, 2, 3], vec![0.8, 0.6, 0.4]);
    sparse_retriever.add_document(1, doc1);
    
    // Query: terms 0, 1
    let query_vector = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = sparse_retriever.retrieve(&query_vector, 10).unwrap();
    
    println!("Query vector: indices {:?}, values {:?}", query_vector.indices, query_vector.values);
    println!("Results:");
    for (doc_id, score) in results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }
}

