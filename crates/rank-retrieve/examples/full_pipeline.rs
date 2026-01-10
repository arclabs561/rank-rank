//! Full pipeline example: Retrieve → Fusion → Rerank → Eval
//!
//! Demonstrates the complete ranking pipeline using all rank-* crates.
//!
//! This example shows the typical production pipeline:
//! 1. **Retrieval** (rank-retrieve): Fast first-stage retrieval from large corpus
//! 2. **Fusion** (rank-fusion): Combine results from multiple retrievers
//! 3. **Reranking** (rank-rerank): Precise token-level or cross-encoder reranking
//! 4. **Evaluation** (rank-eval): Measure quality with metrics like nDCG
//!
//! **Performance characteristics:**
//! - Retrieval: ~1ms for 10M docs → 1000 candidates (BM25 or dense ANN)
//! - Fusion: <1ms for combining multiple result lists
//! - Reranking: ~10-50ms for 1000 → 100 candidates (MaxSim)
//! - Cross-encoder: ~100-500ms for 100 → 10 candidates
//!
//! **In production:**
//! - Use specialized backends (Tantivy, Qdrant, Elasticsearch) for retrieval
//! - Pre-compute embeddings for reranking
//! - Cache frequently accessed results
//! - Use batch processing for multiple queries

#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::retrieve_bm25;
#[cfg(feature = "dense")]
use rank_retrieve::dense::DenseRetriever;
#[cfg(feature = "dense")]
use rank_retrieve::retrieve_dense;

#[cfg(all(feature = "bm25", feature = "dense"))]
fn main() {
    println!("=== Full Ranking Pipeline Example ===\n");
    println!("This example demonstrates a complete RAG/search pipeline.");
    println!("In production, each stage would use optimized backends.\n");

    // ---
    // Stage 1: Retrieval (rank-retrieve)
    // ---
    // Retrieval narrows down from a large corpus (10M+ documents) to a
    // manageable candidate set (typically 100-1000 candidates).
    //
    // This stage prioritizes speed and recall over precision.
    // Multiple retrieval methods can be used in parallel:
    // - BM25: Lexical matching (exact keyword matches)
    // - Dense: Semantic similarity (handles synonyms, paraphrases)
    // - Sparse: Learned sparse representations (SPLADE, etc.)
    //
    // Performance: ~1ms for 10M docs → 1000 candidates
    println!("Stage 1: Retrieval (10M docs → 1000 candidates)\n");

    // ### BM25 Retrieval
    //
    // BM25 is a lexical retrieval method that scores documents based on
    // term frequency (TF) and inverse document frequency (IDF).
    //
    // **When to use:**
    // - Exact keyword matching is important
    // - Query terms are specific (e.g., "Python async programming")
    // - Fast retrieval is required (no neural model needed)
    //
    // **Limitations:**
    // - Doesn't handle synonyms (e.g., "car" vs "automobile")
    // - Requires exact term matches
    let mut bm25_index = InvertedIndex::new();
    bm25_index.add_document(
        0,
        &[
            "the".to_string(),
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
        ],
    );
    bm25_index.add_document(
        1,
        &["the".to_string(), "lazy".to_string(), "dog".to_string()],
    );
    bm25_index.add_document(
        2,
        &[
            "quick".to_string(),
            "brown".to_string(),
            "fox".to_string(),
            "jumps".to_string(),
        ],
    );

    let query_terms = vec!["quick".to_string(), "fox".to_string()];
    let bm25_results =
        retrieve_bm25(&bm25_index, &query_terms, 1000, Bm25Params::default()).unwrap();

    println!("BM25 retrieved {} candidates", bm25_results.len());
    println!("Top 5: {:?}\n", &bm25_results[..5.min(bm25_results.len())]);

    // ### Dense Retrieval
    //
    // Dense retrieval uses neural embeddings to find semantically similar documents.
    // Documents and queries are encoded into dense vectors (typically 128-768 dimensions),
    // and similarity is computed via cosine similarity or dot product.
    //
    // **When to use:**
    // - Semantic matching is important (synonyms, paraphrases)
    // - Query intent is complex (e.g., "how to optimize database queries")
    // - Multilingual retrieval (cross-lingual embeddings)
    //
    // **In production:**
    // - Use approximate nearest neighbor (ANN) indexes: HNSW, FAISS, Qdrant
    // - Pre-compute document embeddings offline
    // - Use batch encoding for queries
    let mut dense_retriever = DenseRetriever::new();
    dense_retriever.add_document(0, vec![1.0, 0.0, 0.0]);
    dense_retriever.add_document(1, vec![0.707, 0.707, 0.0]);
    dense_retriever.add_document(2, vec![0.0, 1.0, 0.0]);

    let query_embedding = [1.0, 0.0, 0.0];
    let dense_results = retrieve_dense(&dense_retriever, &query_embedding, 1000).unwrap();

    println!("Dense retrieved {} candidates", dense_results.len());
    println!(
        "Top 5: {:?}\n",
        &dense_results[..5.min(dense_results.len())]
    );

    // ---
    // Stage 2: Fusion (rank-fusion)
    // ---
    // Fusion combines results from multiple retrievers to improve recall and diversity.
    //
    // **Common fusion methods:**
    // - RRF (Reciprocal Rank Fusion): Rank-based, robust to score differences
    // - CombSUM: Sum of scores (requires score normalization)
    // - Weighted: Learned weights for each retriever
    //
    // **When to use:**
    // - Multiple retrieval methods available (BM25 + dense + sparse)
    // - Different retrievers excel at different query types
    // - Need to balance precision and recall
    //
    // Performance: <1ms for combining multiple result lists
    println!("Stage 2: Fusion (combine multiple retrievers)");
    println!("  → Use rank_fusion::rrf_multi() to combine BM25 and dense results");
    println!("  → Output: Top 1000 fused candidates\n");

    // ---
    // Stage 3: Reranking (rank-rerank)
    // ---
    // Reranking refines the candidate set using more expensive but accurate methods.
    //
    // **MaxSim (ColBERT-style):**
    // - Token-level matching between query and document
    // - Handles partial matches and fine-grained relevance
    // - Performance: ~10-50ms for 1000 → 100 candidates
    //
    // **When to use:**
    // - Need precise ranking of top candidates
    // - Query-document interaction is important
    // - Can afford moderate latency (~50ms)
    println!("Stage 3: Reranking (1000 → 100 candidates)");
    println!("  → Use rank_rerank::simd::maxsim_batch() for late interaction");
    println!("  → Output: Top 100 reranked candidates\n");

    // ---
    // Stage 4: Cross-encoder (rank-rerank)
    // ---
    // Cross-encoders provide the highest quality but are the slowest.
    // They encode query and document together, allowing full interaction.
    //
    // **When to use:**
    // - Final ranking of top 10-100 candidates
    // - Quality is more important than latency
    // - Can afford ~100-500ms per query
    //
    // Performance: ~100-500ms for 100 → 10 candidates
    println!("Stage 4: Cross-encoder (100 → 10 results)");
    println!("  → Use rank_rerank::crossencoder for final scoring");
    println!("  → Output: Top 10 final results\n");

    // ---
    // Stage 5: Evaluation (rank-eval)
    // ---
    // Evaluation measures the quality of the ranking pipeline.
    //
    // **Common metrics:**
    // - nDCG@k: Normalized Discounted Cumulative Gain (handles graded relevance)
    // - Precision@k: Fraction of top-k results that are relevant
    // - Recall@k: Fraction of relevant documents in top-k
    // - MRR: Mean Reciprocal Rank (for single relevant document)
    //
    // **In production:**
    // - Use A/B testing to compare pipeline variants
    // - Track metrics over time to detect regressions
    // - Use user feedback (clicks, dwell time) as relevance signals
    println!("Stage 5: Evaluation");
    println!("  → Use rank_eval::ndcg_at_k() to measure quality");
    println!("  → Compare against ground truth relevance\n");

    println!("=== Pipeline Complete ===");
    println!("\n**Next steps for production:**");
    println!("1. Integrate with search backends:");
    println!("   - BM25: Tantivy, Elasticsearch, Meilisearch");
    println!("   - Dense: Qdrant, Pinecone, Weaviate, FAISS");
    println!("2. Optimize each stage:");
    println!("   - Retrieval: Use ANN indexes (HNSW) for dense retrieval");
    println!("   - Fusion: Cache fused results for common queries");
    println!("   - Reranking: Pre-compute document embeddings, use batch processing");
    println!("3. Monitor and evaluate:");
    println!("   - Track latency and throughput for each stage");
    println!("   - Measure quality metrics (nDCG, Precision, Recall)");
    println!("   - Use A/B testing to compare pipeline variants");
    println!("\n**See other examples:**");
    println!("- examples/late_interaction_pipeline.rs: BM25 → MaxSim pipeline");
    println!("- examples/colpali_multimodal_pipeline.rs: Text-to-image retrieval");
    println!("- examples/qdrant_real_integration.rs: Production Qdrant integration");
}

#[cfg(not(all(feature = "bm25", feature = "dense")))]
fn main() {
    eprintln!("This example requires both 'bm25' and 'dense' features.");
    eprintln!("Run with: cargo run --example full_pipeline --features bm25,dense");
}
