//! Error handling examples for rank-retrieve.
//!
//! Demonstrates proper error handling patterns for all retrieval methods.
//!
//! This example shows how to handle common error cases:
//! 1. **EmptyIndex**: Index has no documents
//! 2. **EmptyQuery**: Query has no terms
//! 3. **DimensionMismatch**: Query and document embeddings have different dimensions
//! 4. **SparseVector validation**: Invalid sparse vector construction
//!
//! **Production patterns:**
//! - Use `match` for exhaustive error handling
//! - Provide user-friendly error messages
//! - Log errors for debugging
//! - Gracefully degrade when possible

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::retrieve_bm25;
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "dense")]
use rank_retrieve::retrieve_dense;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::SparseVector;
use rank_retrieve::RetrieveError;

fn main() {
    println!("=== Error Handling Examples ===\n");
    println!("This example demonstrates how to handle errors gracefully in production.\n");

    // ---
    // Example 1: Empty Index Error
    // ---
    // Occurs when trying to retrieve from an index with no documents.
    // Common in production when index hasn't been populated yet.
    #[cfg(feature = "bm25")]
    {
        println!("1. Empty Index Error:");
        let empty_index = InvertedIndex::new();
        let query = vec!["test".to_string()];
        match retrieve_bm25(&empty_index, &query, 10, Bm25Params::default()) {
            Ok(results) => println!("   Retrieved {} documents", results.len()),
            Err(RetrieveError::EmptyIndex) => {
                println!("   ✓ Correctly caught: Index is empty");
                println!("   → Production: Log error, return empty results or error to user");
            }
            Err(e) => println!("   Unexpected error: {}", e),
        }

        // ---
        // Example 2: Empty Query Error
        // ---
        // Occurs when query has no terms (empty vector).
        // Common when user submits empty search or tokenization fails.
        println!("\n2. Empty Query Error:");
        let mut index = InvertedIndex::new();
        index.add_document(
            0,
            &["test"].iter().map(|s| s.to_string()).collect::<Vec<_>>(),
        );
        match retrieve_bm25(&index, &[], 10, Bm25Params::default()) {
            Ok(results) => println!("   Retrieved {} documents", results.len()),
            Err(RetrieveError::EmptyQuery) => {
                println!("   ✓ Correctly caught: Query is empty");
                println!("   → Production: Validate query before retrieval, return helpful error");
            }
            Err(e) => println!("   Unexpected error: {}", e),
        }
    }

    // ---
    // Example 3: Dimension Mismatch Error (Dense)
    // ---
    // Occurs when query embedding and document embeddings have different dimensions.
    // Common when using different embedding models or incorrect preprocessing.
    #[cfg(feature = "dense")]
    {
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
                println!("   → Production: Validate dimensions at encoding time, use consistent models");
            }
            Err(e) => println!("   Unexpected error: {}", e),
        }
    }

    // ---
    // Example 4: Sparse Vector Validation
    // ---
    // Sparse vectors must have sorted indices and matching lengths.
    // Validation prevents runtime errors from invalid data.
    #[cfg(feature = "sparse")]
    {
        println!("\n4. Sparse Vector Validation:");

        // Valid sparse vector
        match SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5, 0.3]) {
            Some(_) => println!("   ✓ Valid sparse vector created"),
            None => println!("   ✗ Failed to create valid vector"),
        }

        // Invalid: mismatched lengths
        match SparseVector::new(vec![0, 1, 2], vec![1.0, 0.5]) {
            Some(_) => println!("   ✗ Should have failed (mismatched lengths)"),
            None => {
                println!("   ✓ Correctly rejected: Mismatched indices/values lengths");
                println!("   → Production: Validate sparse vectors at construction time");
            }
        }

        // Invalid: unsorted indices
        match SparseVector::new(vec![2, 0, 1], vec![1.0, 0.5, 0.3]) {
            Some(_) => println!("   ✗ Should have failed (unsorted indices)"),
            None => {
                println!("   ✓ Correctly rejected: Indices not sorted");
                println!("   → Production: Sort indices before creating sparse vectors");
            }
        }
    }

    // ---
    // Example 5: Production Error Handling Pattern
    // ---
    // Shows a complete error handling pattern suitable for production use.
    // Includes logging, user-friendly messages, and graceful degradation.
    #[cfg(feature = "bm25")]
    {
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
                println!("   → Production: Return empty results, suggest query refinement");
            }
            Ok(results) => {
                println!("   ✓ Successfully retrieved {} documents", results.len());
                for (doc_id, score) in results.iter().take(5) {
                    println!("      Doc {}: score {:.4}", doc_id, score);
                }
            }
            Err(RetrieveError::EmptyIndex) => {
                eprintln!("   Error: Index is empty. Add documents first.");
                println!("   → Production: Log error, return 503 Service Unavailable");
            }
            Err(RetrieveError::EmptyQuery) => {
                eprintln!("   Error: Query is empty. Provide query terms.");
                println!("   → Production: Validate query before retrieval, return 400 Bad Request");
            }
            Err(RetrieveError::DimensionMismatch { query_dim, doc_dim }) => {
                eprintln!(
                    "   Error: Dimension mismatch (query: {}, doc: {})",
                    query_dim, doc_dim
                );
                println!("   → Production: Validate dimensions at encoding time, log mismatch");
            }
            Err(e) => {
                eprintln!("   Unexpected error: {}", e);
                println!("   → Production: Log full error for debugging, return 500 Internal Server Error");
            }
        }
    }

    println!("\n=== Error Handling Examples Complete ===");
    println!("\n**Key takeaways for production:**");
    println!("1. Always handle all error variants explicitly");
    println!("2. Provide user-friendly error messages");
    println!("3. Log errors with context for debugging");
    println!("4. Return appropriate HTTP status codes (400, 500, 503)");
    println!("5. Validate inputs before calling retrieval functions");
}
