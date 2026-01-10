//! Example: ColPali Multimodal Retrieval Pipeline
//!
//! Demonstrates text-to-image retrieval using ColPali-style late interaction.
//! In ColPali, document images are split into patches (e.g., 32×32 grid = 1024 patches),
//! and query text tokens align with these image patch embeddings.
//!
//! This example shows:
//! 1. BM25 retrieval on document text/metadata for first-stage candidate generation
//! 2. ColPali MaxSim reranking for text-to-image token-level matching
//! 3. Visual snippet extraction using token alignments
//! 4. Integration with rank-fusion and rank-eval

use rank_eval::binary::ndcg_at_k;
use rank_fusion::rrf;
use rank_rerank::colbert;
#[cfg(feature = "bm25")]
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
#[cfg(feature = "bm25")]
use rank_retrieve::retrieve_bm25;
use std::collections::HashSet;

// Mock ColPali encoder (in practice, use a real ColPali model)
fn encode_query_text(query: &str) -> Vec<Vec<f32>> {
    // Simplified: in practice, use a ColPali model to encode query text tokens
    query
        .split_whitespace()
        .map(|word| {
            // Mock embedding: in practice, use actual ColPali encoder
            let mut emb = vec![0.0; 128];
            for (i, c) in word.chars().enumerate() {
                emb[i % 128] = (c as u32 as f32) / 1000.0;
            }
            // L2 normalize
            let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                emb.iter_mut().for_each(|x| *x /= norm);
            }
            emb
        })
        .collect()
}

fn encode_image_patches(image_id: u32, n_patches: usize) -> Vec<Vec<f32>> {
    // Mock image patch embeddings (in practice, use ColPali to encode image patches)
    // In real ColPali, images are split into a grid (e.g., 32×32 = 1024 patches)
    (0..n_patches)
        .map(|patch_idx| {
            let mut emb = vec![0.0; 128];
            // Simulate patch-specific features
            for i in 0..128 {
                emb[i] = ((image_id as f32 * 100.0 + patch_idx as f32 * 10.0 + i as f32) / 1000.0)
                    .sin()
                    .abs();
            }
            // L2 normalize
            let norm: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                emb.iter_mut().for_each(|x| *x /= norm);
            }
            emb
        })
        .collect()
}

#[cfg(feature = "bm25")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("ColPali Multimodal Retrieval Pipeline Example\n");
    println!("This demonstrates text-to-image retrieval using ColPali-style late interaction.");
    println!("In ColPali, document images are split into patches (e.g., 32×32 grid = 1024 patches),");
    println!("and query text tokens align with these image patch embeddings.\n");

    // Setup: Create BM25 index on document text/metadata
    // In practice, you might index OCR text, captions, or metadata associated with images
    let mut index = InvertedIndex::new();
    let documents = vec![
        (0, "revenue chart Q3 financial report"),
        (1, "machine learning neural network diagram"),
        (2, "python programming code example"),
        (3, "rust systems programming memory safety"),
        (4, "information retrieval search ranking"),
    ];

    for (id, text) in &documents {
        let terms: Vec<String> = text.split_whitespace().map(|s| s.to_string()).collect();
        index.add_document(*id, &terms);
    }

    // Query: Text query for image retrieval
    let query_text = "revenue Q3 chart";
    let query_terms: Vec<String> = query_text
        .split_whitespace()
        .map(|s| s.to_string())
        .collect();

    // Step 1: First-stage retrieval with BM25 on text/metadata (rank-retrieve)
    // This narrows down from all documents to candidates
    let candidates = retrieve_bm25(&index, &query_terms, 1000, Bm25Params::default())?;
    println!("BM25 retrieved {} candidates from text/metadata", candidates.len());

    // Step 2: Prepare token embeddings for ColPali reranking
    let query_tokens = encode_query_text(query_text);
    println!("Query: '{}' ({} text tokens)", query_text, query_tokens.len());

    // Get image patch embeddings for candidates
    // In practice, these would be pre-computed and stored
    let doc_image_patches: Vec<(u32, Vec<Vec<f32>>)> = candidates
        .iter()
        .map(|(id, _)| {
            // In real ColPali, this would be 1024 patches from a 32×32 grid
            // For demo, use fewer patches
            let n_patches = 16; // Simplified: real ColPali uses 1024 patches
            (*id, encode_image_patches(*id, n_patches))
        })
        .collect();

    println!(
        "Document images: {} patches per image (in practice, ColPali uses 1024 patches)",
        doc_image_patches[0].1.len()
    );

    // Step 3: Rerank with ColPali MaxSim (rank-rerank)
    // Text query tokens align with image patch embeddings
    let reranked = colbert::rank(&query_tokens, &doc_image_patches);
    println!("\nReranked results (text-to-image):");
    for (i, (id, score)) in reranked.iter().take(5).enumerate() {
        let doc_text = documents.iter().find(|(d_id, _)| d_id == id).unwrap().1;
        println!("  {}. Doc {} ({}): {:.4}", i + 1, id, doc_text, score);
    }

    // Step 4: Visual snippet extraction using token alignments
    println!("\nVisual snippet extraction example:");
    if let Some((top_id, top_patches)) = doc_image_patches.iter().find(|(id, _)| {
        reranked.first().map(|(r_id, _)| r_id == id).unwrap_or(false)
    }) {
        let alignments = colbert::alignments(&query_tokens, top_patches);
        println!("  Top document {} alignments:", top_id);
        for (q_idx, patch_idx, score) in &alignments {
            let token = query_terms.get(*q_idx).map(|s| s.as_str()).unwrap_or("unknown");
            println!(
                "    Query token '{}' (idx {}) → Image patch {} (similarity: {:.3})",
                token, q_idx, patch_idx, score
            );
        }

        // Extract highlighted patches for visual snippet
        let highlighted_patches = colbert::highlight(&query_tokens, top_patches, 0.7);
        println!(
            "  Highlighted image patch indices: {:?}",
            highlighted_patches
        );
        println!(
            "  These patches can be extracted as visual snippets to show users which"
        );
        println!("  regions of the document image are relevant to their query.");
    }

    // Step 5: Optional - Hybrid retrieval with fusion
    // Combine text-based BM25 with image-based ColPali
    println!("\nHybrid retrieval (BM25 text + ColPali image, fused with RRF):");
    // Convert to String IDs for fusion
    let bm25_string: Vec<(String, f32)> = candidates
        .iter()
        .map(|(id, score)| (id.to_string(), *score))
        .collect();
    let colpali_string: Vec<(String, f32)> = reranked
        .iter()
        .map(|(id, score)| (id.to_string(), *score))
        .collect();

    let fused = rrf(&bm25_string, &colpali_string);
    for (i, (id, score)) in fused.iter().take(5).enumerate() {
        let doc_text = documents
            .iter()
            .find(|(d_id, _)| d_id.to_string() == *id)
            .map(|(_, text)| *text)
            .unwrap_or("unknown");
        println!("  {}. Doc {} ({}): {:.4}", i + 1, id, doc_text, score);
    }

    // Step 6: Evaluation
    let ranked_ids: Vec<String> = reranked.iter().map(|(id, _)| id.to_string()).collect();
    let relevant: HashSet<String> = ["0"].iter().map(|s| s.to_string()).collect(); // Doc 0 is relevant
    let ndcg = ndcg_at_k(&ranked_ids, &relevant, 10);
    println!("\nEvaluation: nDCG@10 = {:.4}", ndcg);

    println!("\n✅ ColPali multimodal pipeline complete!");
    println!("   - BM25 first-stage retrieval on text/metadata (rank-retrieve)");
    println!("   - ColPali MaxSim reranking for text-to-image matching (rank-rerank)");
    println!("   - Visual snippet extraction using token alignments");
    println!("   - Same MaxSim mechanism works for both text-text (ColBERT) and text-image (ColPali)");

    Ok(())
}

#[cfg(not(feature = "bm25"))]
fn main() {
    eprintln!("This example requires the 'bm25' feature. Run with: cargo run --example colpali_multimodal_pipeline --features bm25");
}
