//! Example: Query likelihood language model retrieval.
//!
//! Demonstrates probabilistic retrieval using language models, ranking documents by
//! the probability that the document's language model generated the query: P(Q|D).
//!
//! **What is Query Likelihood?**
//! Query likelihood models rank documents by P(Q|D) - the probability that query Q
//! was generated from document D's language model. This inverts the traditional
//! relevance question: instead of "how relevant is D to Q?", we ask "how likely
//! is Q to be generated from D?"
//!
//! **Smoothing Techniques:**
//! - **Jelinek-Mercer**: Interpolates document and corpus language models
//! - **Dirichlet**: Bayesian approach with automatic length adaptation
//!
//! **When to use:**
//! - Research/prototyping scenarios
//! - Queries where probabilistic approach helps
//! - When theoretical grounding is important
//! - As a baseline for comparison with BM25/TF-IDF

#[cfg(feature = "query-likelihood")]
use rank_retrieve::bm25::InvertedIndex;
#[cfg(feature = "query-likelihood")]
use rank_retrieve::query_likelihood::{retrieve_query_likelihood, QueryLikelihoodParams, SmoothingMethod};

#[cfg(feature = "query-likelihood")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Query Likelihood Language Model Retrieval Example ===\n");
    println!("Demonstrates probabilistic retrieval using language models.\n");

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

    let query = vec!["machine".to_string(), "learning".to_string()];
    println!("Query: {:?}", query);

    // ---
    // Step 2: Query likelihood with Dirichlet smoothing (default)
    // ---
    println!("\n=== Dirichlet Smoothing (Default) ===\n");
    let dirichlet_params = QueryLikelihoodParams {
        smoothing: SmoothingMethod::dirichlet(),
    };
    let dirichlet_results = retrieve_query_likelihood(&index, &query, 10, dirichlet_params)?;

    println!("Results (Dirichlet smoothing, μ=1000):");
    for (doc_id, score) in &dirichlet_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 3: Query likelihood with Jelinek-Mercer smoothing
    // ---
    println!("\n=== Jelinek-Mercer Smoothing ===\n");
    let jm_params = QueryLikelihoodParams {
        smoothing: SmoothingMethod::jelinek_mercer(),
    };
    let jm_results = retrieve_query_likelihood(&index, &query, 10, jm_params)?;

    println!("Results (Jelinek-Mercer smoothing, λ=0.5):");
    for (doc_id, score) in &jm_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 4: Compare smoothing parameters
    // ---
    println!("\n=== Comparing Smoothing Parameters ===\n");

    // Jelinek-Mercer with different lambda values
    println!("Jelinek-Mercer with different lambda values:");
    for lambda in [0.1, 0.3, 0.5, 0.7, 0.9] {
        let params = QueryLikelihoodParams {
            smoothing: SmoothingMethod::jelinek_mercer_with_lambda(lambda),
        };
        let results = retrieve_query_likelihood(&index, &query, 10, params)?;
        println!("  λ={}: Doc {}: {:.4}", lambda, results[0].0, results[0].1);
    }

    // Dirichlet with different mu values
    println!("\nDirichlet with different mu values:");
    for mu in [100.0, 500.0, 1000.0, 2000.0] {
        let params = QueryLikelihoodParams {
            smoothing: SmoothingMethod::dirichlet_with_mu(mu),
        };
        let results = retrieve_query_likelihood(&index, &query, 10, params)?;
        println!("  μ={}: Doc {}: {:.4}", mu, results[0].0, results[0].1);
    }

    // ---
    // Step 5: Handle unseen terms (smoothing advantage)
    // ---
    println!("\n=== Handling Unseen Terms (Smoothing Advantage) ===\n");
    let unseen_query = vec!["quantum".to_string(), "computing".to_string()];
    println!("Query with unseen terms: {:?}", unseen_query);

    let results = retrieve_query_likelihood(&index, &unseen_query, 10, QueryLikelihoodParams::default())?;
    println!("Results (smoothing allows non-zero scores even for unseen terms):");
    for (doc_id, score) in &results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 6: Comparison with BM25 (for context)
    // ---
    println!("\n=== Comparison with BM25 (for context) ===\n");
    #[cfg(feature = "bm25")]
    {
        use rank_retrieve::bm25::Bm25Params;
        use rank_retrieve::retrieve_bm25;
        let bm25_results = retrieve_bm25(&index, &query, 10, Bm25Params::default())?;
        println!("BM25 results:");
        for (doc_id, score) in &bm25_results {
            println!("  Doc {}: {:.4}", doc_id, score);
        }
    }
    #[cfg(not(feature = "bm25"))]
    {
        println!("(Enable 'bm25' feature to see BM25 comparison)");
    }

    // ---
    // Step 7: Best practices summary
    // ---
    println!("\n=== Best Practices ===\n");
    println!("Smoothing parameter recommendations:");
    println!("  - Jelinek-Mercer λ: 0.1-0.7 (default: 0.5)");
    println!("    * Lower λ (0.1-0.3): More weight to corpus, better for short documents");
    println!("    * Higher λ (0.5-0.7): More weight to document, better for long documents");
    println!("  - Dirichlet μ: 50-2000 (default: 1000)");
    println!("    * Higher μ: More smoothing, better for short documents");
    println!("    * Lower μ: Less smoothing, better for long documents");
    println!("\nWhen to use:");
    println!("  - Research/prototyping scenarios");
    println!("  - Queries where probabilistic approach helps");
    println!("  - When theoretical grounding is important");
    println!("  - As a baseline for comparison with BM25/TF-IDF");
    println!("\nResearch evidence:");
    println!("  - Query likelihood with Dirichlet smoothing is competitive with BM25");
    println!("  - Often performs better on short queries");
    println!("  - Better theoretical foundation than BM25 (probabilistic vs. heuristic)");

    Ok(())
}

#[cfg(not(feature = "query-likelihood"))]
fn main() {
    eprintln!("This example requires the 'query-likelihood' feature.");
    eprintln!("Run with: cargo run --example query_likelihood --features bm25,query-likelihood");
}
