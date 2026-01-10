//! Example demonstrating HNSW seed selection strategies.
//!
//! Based on 2025 research findings:
//! - StackedNSW (SN): Best for billion-scale datasets
//! - KSampledRandom (KS): Best for medium-scale (1M-25GB) with lower indexing overhead

use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, SeedSelectionStrategy, NeighborhoodDiversification};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dimension = 128;
    let num_vectors = 10_000;
    
    // Example 1: StackedNSW (default, best for large-scale)
    println!("=== Example 1: StackedNSW (default) ===");
    let params_sn = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::StackedNSW,  // Default
        neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,  // Default
        ..Default::default()
    };
    
    let mut index_sn = HNSWIndex::with_params(dimension, params_sn)?;
    
    // Add vectors
    for i in 0..num_vectors {
        let vector: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 * 0.001).collect();
        index_sn.add(i as u32, vector)?;
    }
    
    index_sn.build()?;
    
    // Search
    let query: Vec<f32> = (0..dimension).map(|j| j as f32 * 0.001).collect();
    let results = index_sn.search(&query, 10, 50)?;
    println!("Found {} results", results.len());
    println!("Top result: doc_id={}, distance={:.4}", results[0].0, results[0].1);
    
    // Example 2: KSampledRandom (best for medium-scale)
    println!("\n=== Example 2: KSampledRandom (medium-scale) ===");
    let params_ks = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::KSampledRandom { k: 50 },  // Sample 50 random seeds
        neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
        ..Default::default()
    };
    
    let mut index_ks = HNSWIndex::with_params(dimension, params_ks)?;
    
    // Add vectors
    for i in 0..num_vectors {
        let vector: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 * 0.001).collect();
        index_ks.add(i as u32, vector)?;
    }
    
    index_ks.build()?;
    
    // Search
    let results = index_ks.search(&query, 10, 50)?;
    println!("Found {} results", results.len());
    println!("Top result: doc_id={}, distance={:.4}", results[0].0, results[0].1);
    
    // Example 3: MOND (Maximum-Oriented Neighborhood Diversification)
    println!("\n=== Example 3: MOND Neighborhood Diversification ===");
    let params_mond = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::StackedNSW,
        neighborhood_diversification: NeighborhoodDiversification::MaximumOriented {
            min_angle_degrees: 60.0,  // 60° minimum angle between neighbors
        },
        ..Default::default()
    };
    
    let mut index_mond = HNSWIndex::with_params(dimension, params_mond)?;
    
    // Add vectors
    for i in 0..num_vectors {
        let vector: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 * 0.001).collect();
        index_mond.add(i as u32, vector)?;
    }
    
    index_mond.build()?;
    
    // Search
    let results = index_mond.search(&query, 10, 50)?;
    println!("Found {} results", results.len());
    println!("Top result: doc_id={}, distance={:.4}", results[0].0, results[0].1);
    
    // Example 4: RRND (Relaxed Relative Neighborhood Diversification)
    println!("\n=== Example 4: RRND Neighborhood Diversification ===");
    let params_rrnd = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::StackedNSW,
        neighborhood_diversification: NeighborhoodDiversification::RelaxedRelative {
            alpha: 1.3,  // Relaxation factor (typically 1.3-1.5)
        },
        ..Default::default()
    };
    
    let mut index_rrnd = HNSWIndex::with_params(dimension, params_rrnd)?;
    
    // Add vectors
    for i in 0..num_vectors {
        let vector: Vec<f32> = (0..dimension).map(|j| (i + j) as f32 * 0.001).collect();
        index_rrnd.add(i as u32, vector)?;
    }
    
    index_rrnd.build()?;
    
    // Search
    let results = index_rrnd.search(&query, 10, 50)?;
    println!("Found {} results", results.len());
    println!("Top result: doc_id={}, distance={:.4}", results[0].0, results[0].1);
    
    println!("\n=== Recommendations ===");
    println!("- StackedNSW: Use for billion-scale datasets (best scalability)");
    println!("- KSampledRandom: Use for medium-scale (1M-25GB) with lower indexing overhead");
    println!("- RND: Best overall ND strategy (highest pruning, smallest graphs)");
    println!("- MOND: Second-best ND strategy (angle-based diversification)");
    println!("- RRND: Less effective, creates larger graphs");
    
    Ok(())
}
