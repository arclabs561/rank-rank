//! Example: TF-IDF retrieval.
//!
//! Demonstrates how to use TF-IDF for first-stage retrieval.
//!
//! **What is TF-IDF?**
//! TF-IDF (Term Frequency-Inverse Document Frequency) is a simpler alternative to BM25,
//! calculating relevance as the product of term frequency and inverse document frequency.
//! It provides a baseline for lexical retrieval without the complexity of BM25's saturation
//! and length normalization.
//!
//! **Why use TF-IDF?**
//! - **Simplicity**: Easier to understand and implement than BM25
//! - **Fast**: Similar computational complexity to BM25
//! - **Baseline**: Useful for comparison with BM25 and other methods
//! - **Parameter-free**: No tuning required (unlike BM25's k1, b parameters)
//!
//! **When to use:**
//! - Need simpler baseline for comparison
//! - Educational/prototyping scenarios
//! - Datasets where TF-IDF outperforms BM25 (rare but documented)
//!
//! **When NOT to use:**
//! - Production systems (BM25 generally performs better)
//! - Need length normalization (TF-IDF favors longer documents)
//! - Need saturation (TF-IDF doesn't prevent term repetition abuse)

#[cfg(feature = "tfidf")]
use rank_retrieve::bm25::InvertedIndex;
#[cfg(feature = "tfidf")]
use rank_retrieve::tfidf::{retrieve_tfidf, TfIdfParams, TfVariant, IdfVariant};

#[cfg(feature = "tfidf")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== TF-IDF Retrieval Example ===\n");
    println!("TF-IDF is a simpler alternative to BM25 for lexical retrieval.\n");

    // ---
    // Step 1: Create an inverted index (reuses BM25 index structure)
    // ---
    let mut index = InvertedIndex::new();

    // ---
    // Step 2: Add documents to the index
    // ---
    index.add_document(
        0,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "algorithms".to_string(),
        ],
    );
    index.add_document(
        1,
        &["artificial".to_string(), "intelligence".to_string()],
    );
    index.add_document(
        2,
        &[
            "machine".to_string(),
            "learning".to_string(),
            "neural".to_string(),
            "networks".to_string(),
        ],
    );
    index.add_document(
        3,
        &["deep".to_string(), "learning".to_string(), "neural".to_string()],
    );

    // ---
    // Step 3: Retrieve documents using TF-IDF
    // ---
    let query = vec!["machine".to_string(), "learning".to_string()];
    
    // Default parameters (log-scaled TF, standard IDF)
    let results = retrieve_tfidf(&index, &query, 10, TfIdfParams::default())?;

    println!("Query: {:?}", query);
    println!("Results (sorted by TF-IDF score, descending):");
    for (doc_id, score) in &results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 4: Compare TF variants
    // ---
    println!("\n=== Comparing TF Variants ===\n");

    // Linear TF
    let linear_params = TfIdfParams {
        tf_variant: TfVariant::Linear,
        idf_variant: IdfVariant::Standard,
    };
    let linear_results = retrieve_tfidf(&index, &query, 10, linear_params)?;
    println!("Linear TF (tf = raw count):");
    for (doc_id, score) in &linear_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // Log-scaled TF
    let log_params = TfIdfParams {
        tf_variant: TfVariant::LogScaled,
        idf_variant: IdfVariant::Standard,
    };
    let log_results = retrieve_tfidf(&index, &query, 10, log_params)?;
    println!("\nLog-scaled TF (tf = 1 + log(count)):");
    for (doc_id, score) in &log_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // ---
    // Step 5: Compare IDF variants
    // ---
    println!("\n=== Comparing IDF Variants ===\n");

    // Standard IDF
    let standard_params = TfIdfParams {
        tf_variant: TfVariant::LogScaled,
        idf_variant: IdfVariant::Standard,
    };
    let standard_results = retrieve_tfidf(&index, &query, 10, standard_params)?;
    println!("Standard IDF (idf = log(N / df)):");
    for (doc_id, score) in &standard_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    // Smoothed IDF
    let smoothed_params = TfIdfParams {
        tf_variant: TfVariant::LogScaled,
        idf_variant: IdfVariant::Smoothed,
    };
    let smoothed_results = retrieve_tfidf(&index, &query, 10, smoothed_params)?;
    println!("\nSmoothed IDF (idf = log(1 + (N - df + 0.5) / (df + 0.5))):");
    for (doc_id, score) in &smoothed_results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }

    println!("\n=== TF-IDF vs BM25 ===\n");
    println!("TF-IDF differences from BM25:");
    println!("  - No saturation: Term frequency grows linearly");
    println!("  - No length normalization: Longer documents score higher");
    println!("  - Simpler formula: tf * idf (vs BM25's complex formula)");
    println!("  - Parameter-free: No k1, b parameters to tune");
    println!("\nUse TF-IDF when:");
    println!("  - Need simpler baseline for comparison");
    println!("  - Educational/prototyping scenarios");
    println!("  - Want parameter-free retrieval");
    println!("\nUse BM25 when:");
    println!("  - Production systems (generally better performance)");
    println!("  - Need length normalization");
    println!("  - Need saturation to prevent term repetition abuse");

    Ok(())
}

#[cfg(not(feature = "tfidf"))]
fn main() {
    eprintln!("This example requires the 'tfidf' feature.");
    eprintln!("Run with: cargo run --example tfidf_retrieval --features bm25,tfidf");
}
