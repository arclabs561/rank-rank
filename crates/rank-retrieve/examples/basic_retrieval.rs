//! Basic retrieval examples.
//!
//! Demonstrates how to use rank-retrieve for first-stage retrieval.
//!
//! This example shows the three main retrieval methods:
//! 1. **BM25**: Lexical retrieval using term frequency and inverse document frequency
//! 2. **Dense**: Semantic retrieval using cosine similarity on embeddings
//! 3. **Sparse**: Lexical retrieval using sparse vector dot product
//!
//! **When to use each:**
//! - BM25: Exact keyword matching, fast, no neural model needed
//! - Dense: Semantic matching, handles synonyms/paraphrases, requires embeddings
//! - Sparse: Learned sparse representations (SPLADE), combines lexical + semantic
//!
//! **Performance:**
//! - BM25: ~1ms for 10M docs → 1000 candidates
//! - Dense: ~1-5ms for 10M docs → 1000 candidates (with ANN index)
//! - Sparse: ~1-2ms for 10M docs → 1000 candidates

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::retrieve_bm25;
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "dense")]
use rank_retrieve::retrieve_dense;
#[cfg(feature = "sparse")]
use rank_retrieve::sparse::{SparseRetriever, SparseVector};
#[cfg(feature = "sparse")]
use rank_retrieve::retrieve_sparse;

#[cfg(feature = "bm25")]
fn example_bm25() {
    println!("=== BM25 Retrieval Example ===\n");
    println!("BM25 is a lexical retrieval method that scores documents based on");
    println!("term frequency (TF) and inverse document frequency (IDF).\n");

    // ---
    // Step 1: Create an inverted index
    // ---
    // The inverted index maps terms to the documents that contain them.
    // This allows fast lookup of documents containing specific query terms.
    let mut index = InvertedIndex::new();

    // ---
    // Step 2: Add documents to the index
    // ---
    // Documents are tokenized into terms. In production, you'd use a proper
    // tokenizer (e.g., from `tantivy` or `tantivy-tokenizer`).
    index.add_document(
        0,
        &[
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
        ],
    );
    index.add_document(
        1,
        &["the".to_string(), "lazy".to_string(), "dog".to_string()],
    );
    index.add_document(
        2,
        &[
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
            "jumps".to_string(),
        ],
    );
    index.add_document(
        3,
        &[
            "over".to_string(),
            "the".to_string(),
            "lazy".to_string(),
            "dog".to_string(),
        ],
    );

    // ---
    // Step 3: Retrieve documents matching the query
    // ---
    // BM25 scores documents based on:
    // - Term frequency (TF): How often query terms appear in the document
    // - Inverse document frequency (IDF): How rare the terms are across all documents
    // - Length normalization: Prevents bias toward longer documents
    let query = vec!["quick".to_string(), "fox".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

    println!("Query: {:?}", query);
    println!("Results (sorted by BM25 score, descending):");
    for (doc_id, score) in &results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }
    println!("\nNote: Documents 0 and 2 both contain 'quick' and 'fox',");
    println!("      but document 2 has a higher score due to additional term 'jumps'.\n");
}

#[cfg(feature = "dense")]
fn example_dense() {
    println!("=== Dense Retrieval Example ===\n");
    println!("Dense retrieval uses neural embeddings to find semantically similar documents.");
    println!("Documents and queries are encoded into dense vectors (typically 128-768 dimensions),");
    println!("and similarity is computed via cosine similarity.\n");

    let mut dense_retriever = DenseRetriever::new();

    // ---
    // Step 1: Add documents with embeddings
    // ---
    // In production, embeddings would come from a neural encoder (e.g., BERT, Sentence-BERT).
    // Embeddings should be L2-normalized for cosine similarity.
    dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
    dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);

    // ---
    // Step 2: Retrieve using query embedding
    // ---
    // Cosine similarity ranges from -1 (opposite) to 1 (identical).
    // Higher scores indicate more semantic similarity.
    let query_embedding = [1.0, 0.0, 0.0];
    let results = retrieve_dense(&dense_retriever, &query_embedding, 10).unwrap();

    println!("Query embedding: {:?}", query_embedding);
    println!("Results (sorted by cosine similarity, descending):");
    for (doc_id, score) in &results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }
    println!("\nNote: Document 0 has the highest score (1.0) because its embedding");
    println!("      is identical to the query embedding.\n");
}

#[cfg(feature = "sparse")]
fn example_sparse() {
    println!("=== Sparse Retrieval Example ===\n");
    println!("Sparse retrieval uses sparse vectors where only a few dimensions are non-zero.");
    println!("This is efficient for high-dimensional spaces (e.g., 30K+ dimensions).");
    println!("Common use cases: SPLADE, learned sparse representations.\n");

    let mut sparse_retriever = SparseRetriever::new();

    // ---
    // Step 1: Add documents with sparse vectors
    // ---
    // Sparse vectors store only non-zero dimensions. This is efficient for
    // high-dimensional spaces where most dimensions are zero.
    //
    // Format: (indices, values) where indices are sorted and correspond to
    //         non-zero dimensions in the full vector space.
    let doc0 = SparseVector::new_unchecked(vec![0, 1, 2], vec![1.0, 0.5, 0.3]);
    sparse_retriever.add_document(0, doc0);

    let doc1 = SparseVector::new_unchecked(vec![1, 2, 3], vec![0.8, 0.6, 0.4]);
    sparse_retriever.add_document(1, doc1);

    // ---
    // Step 2: Retrieve using sparse query vector
    // ---
    // Sparse dot product is computed efficiently by only processing overlapping
    // dimensions (where both query and document have non-zero values).
    let query_vector = SparseVector::new_unchecked(vec![0, 1], vec![1.0, 1.0]);
    let results = retrieve_sparse(&sparse_retriever, &query_vector, 10).unwrap();

    println!(
        "Query vector: indices {:?}, values {:?}",
        query_vector.indices, query_vector.values
    );
    println!("Results (sorted by dot product, descending):");
    for (doc_id, score) in &results {
        println!("  Doc {}: {:.4}", doc_id, score);
    }
    println!("\nNote: Document 0 has a higher score because it matches both");
    println!("      query dimensions (0 and 1), while document 1 only matches dimension 1.\n");
}

// Main function that runs all examples
#[cfg(any(feature = "bm25", feature = "dense", feature = "sparse"))]
fn main() {
    #[cfg(feature = "bm25")]
    {
        example_bm25();
        println!("\n{}", "=".repeat(50));
        println!();
    }
    #[cfg(feature = "dense")]
    {
        example_dense();
        println!("\n{}", "=".repeat(50));
        println!();
    }
    #[cfg(feature = "sparse")]
    {
        example_sparse();
    }
}

#[cfg(not(any(feature = "bm25", feature = "dense", feature = "sparse")))]
fn main() {
    eprintln!("This example requires at least one feature: bm25, dense, or sparse");
    eprintln!("Run with: cargo run --example basic_retrieval --features bm25,dense,sparse");
}
