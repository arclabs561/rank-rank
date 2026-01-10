//! Tests for HNSW seed selection strategies.

#[cfg(feature = "hnsw")]
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams, SeedSelectionStrategy, NeighborhoodDiversification};

#[cfg(feature = "hnsw")]
#[test]
fn test_stacked_nsw_seed_selection() {
    let params = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::StackedNSW,
        neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
        ..Default::default()
    };
    
    let mut index = HNSWIndex::with_params(128, params).unwrap();
    
    // Add vectors
    for i in 0..100 {
        let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
        index.add(i as u32, vector).unwrap();
    }
    
    index.build().unwrap();
    
    // Search
    let query: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    let results = index.search(&query, 10, 50).unwrap();
    
    assert_eq!(results.len(), 10);
    assert!(results[0].1.is_finite());
}

#[cfg(feature = "hnsw")]
#[test]
fn test_k_sampled_random_seed_selection() {
    let params = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::KSampledRandom { k: 10 },
        neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
        ..Default::default()
    };
    
    let mut index = HNSWIndex::with_params(128, params).unwrap();
    
    // Add vectors
    for i in 0..100 {
        let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
        index.add(i as u32, vector).unwrap();
    }
    
    index.build().unwrap();
    
    // Search
    let query: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    let results = index.search(&query, 10, 50).unwrap();
    
    assert_eq!(results.len(), 10);
    assert!(results[0].1.is_finite());
}

#[cfg(feature = "hnsw")]
#[test]
fn test_rnd_neighborhood_diversification() {
    let params = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::StackedNSW,
        neighborhood_diversification: NeighborhoodDiversification::RelativeNeighborhood,
        ..Default::default()
    };
    
    let mut index = HNSWIndex::with_params(128, params).unwrap();
    
    // Add vectors
    for i in 0..100 {
        let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
        index.add(i as u32, vector).unwrap();
    }
    
    index.build().unwrap();
    
    // Search
    let query: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    let results = index.search(&query, 10, 50).unwrap();
    
    assert_eq!(results.len(), 10);
    assert!(results[0].1.is_finite());
}

#[cfg(feature = "hnsw")]
#[test]
fn test_mond_neighborhood_diversification() {
    let params = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::StackedNSW,
        neighborhood_diversification: NeighborhoodDiversification::MaximumOriented {
            min_angle_degrees: 60.0,
        },
        ..Default::default()
    };
    
    let mut index = HNSWIndex::with_params(128, params).unwrap();
    
    // Add vectors
    for i in 0..100 {
        let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
        index.add(i as u32, vector).unwrap();
    }
    
    index.build().unwrap();
    
    // Search
    let query: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    let results = index.search(&query, 10, 50).unwrap();
    
    assert_eq!(results.len(), 10);
    assert!(results[0].1.is_finite());
}

#[cfg(feature = "hnsw")]
#[test]
fn test_rrnd_neighborhood_diversification() {
    let params = HNSWParams {
        m: 16,
        m_max: 16,
        ef_construction: 200,
        ef_search: 50,
        seed_selection: SeedSelectionStrategy::StackedNSW,
        neighborhood_diversification: NeighborhoodDiversification::RelaxedRelative {
            alpha: 1.3,
        },
        ..Default::default()
    };
    
    let mut index = HNSWIndex::with_params(128, params).unwrap();
    
    // Add vectors
    for i in 0..100 {
        let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
        index.add(i as u32, vector).unwrap();
    }
    
    index.build().unwrap();
    
    // Search
    let query: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    let results = index.search(&query, 10, 50).unwrap();
    
    assert_eq!(results.len(), 10);
    assert!(results[0].1.is_finite());
}
