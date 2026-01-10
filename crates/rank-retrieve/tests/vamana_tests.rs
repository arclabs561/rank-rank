//! Tests for Vamana graph-based ANN.

#[cfg(feature = "vamana")]
use rank_retrieve::dense::vamana::{VamanaIndex, VamanaParams};

#[cfg(feature = "vamana")]
#[test]
fn test_vamana_create() {
    let params = VamanaParams::default();
    let index = VamanaIndex::new(128, params);
    assert!(index.is_ok());
}

#[cfg(feature = "vamana")]
#[test]
fn test_vamana_add_and_build() {
    let params = VamanaParams {
        max_degree: 32,
        alpha: 1.3,
        ef_construction: 100,
        ef_search: 50,
    };
    
    let mut index = VamanaIndex::new(128, params).unwrap();
    
    // Add vectors
    for i in 0..100 {
        let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 * 0.001).collect();
        index.add(i as u32, vector).unwrap();
    }
    
    // Build index (two-pass construction)
    index.build().unwrap();
    
    // Search
    let query: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    let results = index.search(&query, 10, 50).unwrap();
    
    assert_eq!(results.len(), 10);
    assert!(results[0].1.is_finite());
}

#[cfg(feature = "vamana")]
#[test]
fn test_vamana_empty_index() {
    let params = VamanaParams::default();
    let mut index = VamanaIndex::new(128, params).unwrap();
    
    // Building empty index should fail
    assert!(index.build().is_err());
}

#[cfg(feature = "vamana")]
#[test]
fn test_vamana_search_before_build() {
    let params = VamanaParams::default();
    let mut index = VamanaIndex::new(128, params).unwrap();
    
    let vector: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    index.add(0, vector).unwrap();
    
    // Searching before build should fail
    let query: Vec<f32> = (0..128).map(|j| j as f32 * 0.001).collect();
    assert!(index.search(&query, 10, 50).is_err());
}
