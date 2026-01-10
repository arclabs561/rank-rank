//! Example: Query expansion with pseudo-relevance feedback (PRF).
//!
//! Demonstrates how to use query expansion to improve recall by reformulating
//! queries with semantically related terms extracted from top-ranked documents.
//!
//! **What is Query Expansion / PRF?**
//! Query expansion reformulates queries to include semantically related terms,
//! addressing vocabulary mismatch - a key problem in first-stage retrieval.
//!
//! **Pseudo-Relevance Feedback (PRF):**
//! 1. Initial retrieval with original query
//! 2. Extract terms from top-k feedback documents
//! 3. Select best expansion terms
//! 4. Re-retrieve with expanded query
//!
//! **Research-backed best practices (2024):**
//! - Small PRF depth: Top-3 to top-10 feedback docs (default: 5)
//! - Limited expansion: 3-10 terms typically optimal (default: 5)
//! - Original query dominance: Expansion weight 0.3-0.7 (default: 0.5)
//! - Structured features: Prioritize rare, discriminative terms
//!
//! **When to use:**
//! - Queries with vocabulary mismatch
//! - Low recall scenarios
//! - Domain-specific terminology
//! - Research/prototyping

#[cfg(feature = "query-expansion")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "query-expansion")]
use rank_retrieve::query_expansion::{expand_query_with_prf_bm25, ExpansionMethod, QueryExpander};
#[cfg(feature = "query-expansion")]
use rank_retrieve::retrieve_bm25;

#[cfg(feature = "query-expansion")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Query Expansion / PRF Example ===\n");
    println!("Demonstrates query expansion to improve recall.\n");

    // ---
    // Step 1: Create index with documents
    // ---
    let mut index = InvertedIndex::new();

    // Document about machine learning
    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
            "neural".to_string(),
            "networks".to_string(),
        ],
    );

    // Document about artificial intelligence
    index.add_document(
        1,
        &[
            "artificial".to_string(),
            "intelligence".to_string(),
            "machine".to_string(),
            "learning".to_string(),
        ],
    );

    // Document about deep learning
    index.add_document(
        2,
        &[
            "deep".to_string(),
            "learning".to_string(),
            "neural".to_string(),
            "networks".to_string(),
            "algorithms".to_string(),
        ],
    );

    // ---
    // Step 2: Original query (abbreviated - vocabulary mismatch)
    // ---
    let query = vec!["ml".to_string()]; // Abbreviated query

    println!("Original query: {:?}", query);
    println!("\nInitial retrieval (without expansion):");
    let original_results = retrieve_bm25(&index, &query, 10, Bm25Params::default())?;
    if original_results.is_empty() {
        println!("  No results found (vocabulary mismatch: 'ml' not in documents)");
    } else {
        for (doc_id, score) in &original_results {
            println!("  Doc {}: {:.4}", doc_id, score);
        }
    }

    // ---
    // Step 3: Query expansion with PRF
    // ---
    println!("\n=== Query Expansion with PRF ===\n");

    // Default expander (IDF-weighted, PRF depth=5, max terms=5)
    let expander = QueryExpander::default();
    let expanded_results = expand_query_with_prf_bm25(
        &index,
        &query,
        5,  // initial_k: Get top 5 for feedback
        10, // final_k: Return top 10 after expansion
        &expander,
        retrieve_bm25,
    )?;

    println!("Expanded query results (PRF with IDF-weighted selection):");
    for (doc_id, score) in &expanded_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 4: Compare expansion methods
    // ---
    println!("\n=== Comparing Expansion Methods ===\n");

    // Term frequency method
    let tf_expander = QueryExpander::new()
        .with_method(ExpansionMethod::TermFrequency)
        .with_prf_depth(5)
        .with_max_expansion_terms(5);
    let tf_results = expand_query_with_prf_bm25(
        &index,
        &query,
        5,
        10,
        &tf_expander,
        retrieve_bm25,
    )?;
    println!("Term Frequency method:");
    for (doc_id, score) in &tf_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // Robertson Selection Value method
    let rsv_expander = QueryExpander::new()
        .with_method(ExpansionMethod::RobertsonSelection)
        .with_prf_depth(5)
        .with_max_expansion_terms(5);
    let rsv_results = expand_query_with_prf_bm25(
        &index,
        &query,
        5,
        10,
        &rsv_expander,
        retrieve_bm25,
    )?;
    println!("\nRobertson Selection Value method:");
    for (doc_id, score) in &rsv_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // IDF-weighted method (default)
    println!("\nIDF-weighted method (default):");
    for (doc_id, score) in &expanded_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 5: Tuning PRF parameters
    // ---
    println!("\n=== Tuning PRF Parameters ===\n");

    // Shallow PRF (fewer feedback docs)
    let shallow_expander = QueryExpander::new()
        .with_prf_depth(3)
        .with_max_expansion_terms(3);
    let shallow_results = expand_query_with_prf_bm25(
        &index,
        &query,
        5,
        10,
        &shallow_expander,
        retrieve_bm25,
    )?;
    println!("Shallow PRF (depth=3, max_terms=3):");
    for (doc_id, score) in &shallow_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // Deeper PRF (more feedback docs)
    let deep_expander = QueryExpander::new()
        .with_prf_depth(10)
        .with_max_expansion_terms(10);
    let deep_results = expand_query_with_prf_bm25(
        &index,
        &query,
        10,
        10,
        &deep_expander,
        retrieve_bm25,
    )?;
    println!("\nDeeper PRF (depth=10, max_terms=10):");
    for (doc_id, score) in &deep_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 6: Best practices summary
    // ---
    println!("\n=== Best Practices ===\n");
    println!("Research-backed recommendations (2024):");
    println!("  - PRF depth: 3-10 feedback docs (default: 5)");
    println!("  - Max expansion terms: 3-10 (default: 5)");
    println!("  - Expansion weight: 0.3-0.7 (default: 0.5)");
    println!("  - Method: IDF-weighted for vocabulary mismatch");
    println!("  - Term selection: Prioritize rare, discriminative terms");
    println!("\nWhen to use:");
    println!("  - Queries with vocabulary mismatch");
    println!("  - Low recall scenarios");
    println!("  - Domain-specific terminology");
    println!("\nWhen NOT to use:");
    println!("  - Queries already have high recall");
    println!("  - Latency is critical (PRF adds 2x retrieval cost)");
    println!("  - Query drift is a concern (use shallow PRF)");

    Ok(())
}

#[cfg(not(feature = "query-expansion"))]
fn main() {
    eprintln!("This example requires the 'query-expansion' feature.");
    eprintln!("Run with: cargo run --example query_expansion --features bm25,query-expansion");
}
