//! Example: BM25 variants (BM25L, BM25+).
//!
//! Demonstrates how to use BM25L and BM25+ variants for improved retrieval.
//!
//! **What are BM25 Variants?**
//! BM25L and BM25+ are improvements to standard BM25 that address specific limitations:
//! - **BM25L**: Addresses over-penalization of short documents by adding a constant delta
//! - **BM25+**: Prevents negative scores for common terms by lower-bounding the TF contribution
//!
//! **When to use each:**
//! - **Standard BM25**: Default baseline, most systems, when no clear bias is seen
//! - **BM25L**: Collections with many long documents where BM25 over-penalizes length
//! - **BM25+**: When long documents containing query terms get too low scores
//!
//! **Research evidence:**
//! - BM25L: 2-5% improvement on short documents
//! - BM25+: Prevents negative scores, more stable behavior
//! - Both variants are well-documented and tested

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, Bm25Variant, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::retrieve_bm25;

#[cfg(feature = "bm25")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== BM25 Variants Example ===\n");
    println!("Demonstrates BM25L and BM25+ variants for improved retrieval.\n");

    // ---
    // Step 1: Create index with documents of varying lengths
    // ---
    let mut index = InvertedIndex::new();

    // Short document
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    
    // Medium document (repeated terms)
    index.add_document(
        1,
        &vec![
            "machine".to_string(),
            "learning".to_string(),
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
        ],
    );

    // Long document (many terms)
    index.add_document(
        2,
        &vec![
            "machine".to_string(),
            "learning".to_string(),
            "neural".to_string(),
            "networks".to_string(),
            "deep".to_string(),
            "artificial".to_string(),
            "intelligence".to_string(),
        ],
    );

    let query = vec!["machine".to_string(), "learning".to_string()];

    // ---
    // Step 2: Compare Standard BM25 vs BM25L vs BM25+
    // ---
    println!("Query: {:?}\n", query);

    // Standard BM25
    let standard_params = Bm25Params::default();
    let standard_results = retrieve_bm25(&index, &query, 10, standard_params)?;
    println!("Standard BM25 Results:");
    for (doc_id, score) in &standard_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // BM25L (addresses over-penalization of short documents)
    let bm25l_params = Bm25Params::bm25l();
    let bm25l_results = retrieve_bm25(&index, &query, 10, bm25l_params)?;
    println!("\nBM25L Results (delta=0.5, boosts longer documents):");
    for (doc_id, score) in &bm25l_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // BM25+ (prevents negative scores)
    let bm25plus_params = Bm25Params::bm25plus();
    let bm25plus_results = retrieve_bm25(&index, &query, 10, bm25plus_params)?;
    println!("\nBM25+ Results (delta=1.0, lower-bounds scores):");
    for (doc_id, score) in &bm25plus_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 3: Custom delta values
    // ---
    println!("\n=== Custom Delta Values ===\n");

    let bm25l_custom = Bm25Params {
        k1: 1.2,
        b: 0.75,
        variant: Bm25Variant::bm25l_with_delta(1.0), // Custom delta
    };
    let bm25l_custom_results = retrieve_bm25(&index, &query, 10, bm25l_custom)?;
    println!("BM25L with custom delta=1.0:");
    for (doc_id, score) in &bm25l_custom_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 4: When to use each variant
    // ---
    println!("\n=== When to Use Each Variant ===\n");
    println!("Standard BM25:");
    println!("  - Default baseline for most systems");
    println!("  - When no clear bias toward short/long documents");
    println!("  - Most common use case\n");

    println!("BM25L:");
    println!("  - Collections with many long documents");
    println!("  - BM25 is clearly favoring short docs");
    println!("  - Long relevant docs are under-ranked\n");

    println!("BM25+:");
    println!("  - Very low or near-zero scores for long documents");
    println!("  - Want to prevent negative scores");
    println!("  - Need clearer separation between 'term present' vs 'term absent'\n");

    println!("Research evidence:");
    println!("  - BM25L: 2-5% improvement on short documents");
    println!("  - BM25+: Prevents negative scores, more stable behavior");
    println!("  - Both variants are well-documented and tested");

    Ok(())
}

#[cfg(not(feature = "bm25"))]
fn main() {
    eprintln!("This example requires the 'bm25' feature.");
    eprintln!("Run with: cargo run --example bm25_variants --features bm25");
}
