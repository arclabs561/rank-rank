//! Error handling examples for rank-retrieve.
//!
//! Demonstrates proper error handling patterns for all retrieval methods.

use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::sparse::SparseVector;
use rank_retrieve::RetrieveError;
use rank_retrieve::{retrieve_bm25, retrieve_dense};

fn main() {
    println!("=== Error Handling Examples ===\n");

    // Example 1: Empty index error
    println!("1. Empty Index Error:");
    let empty_index = InvertedIndex::new();
    let query = vec!["test".to_string()];
    match retrieve_bm25(&empty_index, &query, 10, Bm25Params::default()) {
        Ok(results) => println!("   Retrieved {} documents", results.len()),
        Err(RetrieveError::EmptyIndex) => println!("   ✓ Correctly caught: Index is empty"),
        Err(e) => println!("   Unexpected error: {}", e),
    }

    // Example 2: Empty query error
    println!("\n2. Empty Query Error:");
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    );
    match retrieve_bm25(&index, &[], 10, Bm25Params::default()) {
        Ok(results) => println!("   Retrieved {} documents", results.len()),
        Err(RetrieveError::EmptyQuery) => println!("   ✓ Correctly caught: Query is empty"),
        Err(e) => println!("   Unexpected error: {}", e),
    }

    // Example 3: Dense dimension mismatch
    println!("\n3. Dimension Mismatch Error (Dense):");
    let mut dense = DenseRetriever::new();
    dense.add_document(0, vec![1.0, 0.0, 0.0]); // 3D
    match retrieve_dense(&dense, &[1.0, 0.0], 10) {
        // 2D query
        Ok(results) => println!("   Retrieved {} documents", results.len()),
        Err(RetrieveError::DimensionMismatch { query_dim, doc_dim }) => {
            println!(
                "   ✓ Correctly caught: Query has {} dimensions, document has {}",
                query_dim, doc_dim
            );
        }
        Err(e) => println!("   Unexpected error: {}", e),
    }

    // Example 4: Sparse vector validation
    println!("\n4. Sparse Vector Validation:");

    // Valid sparse vector
    match SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5, 0.3]) {
        Some(_) => println!("   ✓ Valid sparse vector created"),
        None => println!("   ✗ Failed to create valid vector"),
    }

    // Invalid: mismatched lengths
    match SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5]) {
        Some(_) => println!("   ✗ Should have failed (mismatched lengths)"),
        None => println!("   ✓ Correctly rejected: Mismatched indices/values lengths"),
    }

    // Invalid: unsorted indices
    match SparseVector::new(vec![2, 0, 1], vec![1.0, 0.5, 0.3]) {
        Some(_) => println!("   ✗ Should have failed (unsorted indices)"),
        None => println!("   ✓ Correctly rejected: Indices not sorted"),
    }

    // Example 5: Graceful error handling in production
    println!("\n5. Production Error Handling Pattern:");
    let mut index = InvertedIndex::new();
    index.add_document(
        0,
        &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
    );

    let query = vec!["test".to_string()];
    match retrieve_bm25(&index, &query, 10, Bm25Params::default()) {
        Ok(results) if results.is_empty() => {
            println!("   No results found (empty result set)");
        }
        Ok(results) => {
            println!("   ✓ Successfully retrieved {} documents", results.len());
            for (doc_id, score) in results.iter().take(5) {
                println!("      Doc {}: score {:.4}", doc_id, score);
            }
        }
        Err(RetrieveError::EmptyIndex) => {
            eprintln!("   Error: Index is empty. Add documents first.");
        }
        Err(RetrieveError::EmptyQuery) => {
            eprintln!("   Error: Query is empty. Provide query terms.");
        }
        Err(RetrieveError::DimensionMismatch { query_dim, doc_dim }) => {
            eprintln!(
                "   Error: Dimension mismatch (query: {}, doc: {})",
                query_dim, doc_dim
            );
        }
        Err(e) => {
            eprintln!("   Unexpected error: {}", e);
        }
    }

    println!("\n=== Error Handling Examples Complete ===");
}
