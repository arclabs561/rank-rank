//! Example demonstrating Vamana graph-based ANN with two-pass construction.

use rank_retrieve::dense::vamana::{VamanaIndex, VamanaParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dimension = 128;
    let num_vectors = 10_000;
    
    println!("=== Vamana Two-Pass Construction Example ===");
    
    // Create Vamana index with default parameters
    let params = VamanaParams {
        max_degree: 64,        // Maximum out-degree per node
        alpha: 1.3,            // Relaxation factor for RRND (typically 1.3-1.5)
        ef_construction: 200,  // Search width during construction
        ef_search: 50,         // Default search width during query
    };
    
    let mut index = VamanaIndex::new(dimension, params)?;
    
    // Add vectors
    println!("Adding {} vectors...", num_vectors);
    for i in 0..num_vectors {
        let vector: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 * 0.001).collect();
        index.add(i as u32, vector)?;
    }
    
    // Build index (two-pass construction: RRND + RND)
    println!("Building index (two-pass construction)...");
    index.build()?;
    println!("Index built successfully!");
    
    // Search
    let query: Vec<f32> = (0..dimension).map(|j| j as f32 * 0.001).collect();
    println!("Searching for top 10 nearest neighbors...");
    let results = index.search(&query, 10, 50)?;
    
    println!("Found {} results:", results.len());
    for (i, (doc_id, distance)) in results.iter().enumerate() {
        println!("  {}. doc_id={}, distance={:.4}", i + 1, doc_id, distance);
    }
    
    println!("\n=== Vamana Characteristics ===");
    println!("- Two-pass construction: RRND (first pass) + RND (second pass)");
    println!("- Competitive with HNSW on large datasets");
    println!("- Better for SSD-based serving (higher points/node ratio)");
    println!("- Higher indexing time but better graph quality");
    
    Ok(())
}
