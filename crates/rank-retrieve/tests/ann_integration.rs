//! Integration tests for ANN algorithms.

use rank_retrieve::RetrieveError;
use rand::Rng;

/// Test that all ANN methods implement the unified API.
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
#[test]
fn test_unified_api() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::ann::ANNIndex;
    
    let dimension = 128;
    let num_vectors = 100;
    let k = 10;
    
    // Generate test vectors
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();
    for _ in 0..num_vectors {
        let mut vec = vec![0.0f32; dimension];
        let mut norm = 0.0f32;
        for val in &mut vec {
            *val = rng.gen::<f32>() * 2.0 - 1.0;
            norm += *val * *val;
        }
        let norm = f32::sqrt(norm);
        if norm > 0.0 {
            for val in &mut vec {
                *val /= norm;
            }
        }
        vectors.push(vec);
    }
    
    // Test each implemented method
    #[cfg(feature = "hnsw")]
    {
        use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(dimension, params.m, params.m_max)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(&vectors[0], k, params.ef_search)?;
        assert!(!results.is_empty());
    }
    
    #[cfg(feature = "sng")]
    {
        use rank_retrieve::dense::sng::{SNGIndex, SNGParams};
        let params = SNGParams::default();
        let mut index = SNGIndex::new(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(&vectors[0], k)?;
        assert!(!results.is_empty());
    }
    
    #[cfg(feature = "lsh")]
    {
        use rank_retrieve::dense::classic::lsh::{LSHIndex, LSHParams};
        let params = LSHParams::default();
        let mut index = LSHIndex::new(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(&vectors[0], k)?;
        assert!(!results.is_empty());
    }
    
    #[cfg(feature = "annoy")]
    {
        use rank_retrieve::dense::classic::trees::annoy::{AnnoyIndex, AnnoyParams};
        let params = AnnoyParams::default();
        let mut index = AnnoyIndex::new(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(&vectors[0], k)?;
        assert!(!results.is_empty());
    }
    
    #[cfg(feature = "nsw")]
    {
        use rank_retrieve::dense::nsw::{NSWIndex, NSWParams};
        let params = NSWParams::default();
        let ef_search = params.ef_search;
        let mut index = NSWIndex::with_params(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(&vectors[0], k, ef_search)?;
        assert!(!results.is_empty());
    }
    
    #[cfg(feature = "kmeans_tree")]
    {
        use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};
        let params = KMeansTreeParams::default();
        let mut index = KMeansTreeIndex::new(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(&vectors[0], k)?;
        assert!(!results.is_empty());
    }
    
    Ok(())
}

/// Test cross-method consistency.
#[cfg(any(feature = "hnsw", feature = "nsw", feature = "scann", feature = "ivf_pq", feature = "diskann", feature = "sng", feature = "lsh", feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
#[test]
fn test_cross_method_consistency() -> Result<(), RetrieveError> {
    let dimension = 64;
    let num_vectors = 500;
    let k = 5;
    
    // Generate test vectors
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();
    for _ in 0..num_vectors {
        let mut vec = vec![0.0f32; dimension];
        let mut norm = 0.0f32;
        for val in &mut vec {
            *val = rng.gen::<f32>() * 2.0 - 1.0;
            norm += *val * *val;
        }
        let norm = f32::sqrt(norm);
        if norm > 0.0 {
            for val in &mut vec {
                *val /= norm;
            }
        }
        vectors.push(vec);
    }
    
    let query = &vectors[0];
    
    // Get results from different methods
    let mut results_map = std::collections::HashMap::new();
    
    #[cfg(feature = "hnsw")]
    {
        use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};
        let params = HNSWParams::default();
        let mut index = HNSWIndex::new(dimension, params.m, params.m_max)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(query, k, params.ef_search)?;
        results_map.insert("hnsw", results.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    }
    
    #[cfg(feature = "sng")]
    {
        use rank_retrieve::dense::sng::{SNGIndex, SNGParams};
        let params = SNGParams::default();
        let mut index = SNGIndex::new(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(query, k)?;
        results_map.insert("sng", results.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    }
    
    #[cfg(feature = "lsh")]
    {
        use rank_retrieve::dense::classic::lsh::{LSHIndex, LSHParams};
        let params = LSHParams::default();
        let mut index = LSHIndex::new(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(query, k)?;
        results_map.insert("lsh", results.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    }
    
    #[cfg(feature = "nsw")]
    {
        use rank_retrieve::dense::nsw::{NSWIndex, NSWParams};
        let params = NSWParams::default();
        let ef_search = params.ef_search;
        let mut index = NSWIndex::with_params(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(query, k, ef_search)?;
        results_map.insert("nsw", results.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    }
    
    #[cfg(feature = "kmeans_tree")]
    {
        use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};
        let params = KMeansTreeParams::default();
        let mut index = KMeansTreeIndex::new(dimension, params)?;
        for (i, vec) in vectors.iter().enumerate() {
            index.add(i as u32, vec.clone())?;
        }
        index.build()?;
        let results = index.search(query, k)?;
        results_map.insert("kmeans_tree", results.iter().map(|(id, _)| *id).collect::<Vec<_>>());
    }
    
    // Check that methods return reasonable results (at least some overlap)
    let methods: Vec<&str> = results_map.keys().copied().collect();
    if methods.len() >= 2 {
        for i in 0..methods.len() {
            for j in (i + 1)..methods.len() {
                let results_i = &results_map[methods[i]];
                let results_j = &results_map[methods[j]];
                
                let intersection: Vec<u32> = results_i
                    .iter()
                    .filter(|id| results_j.contains(id))
                    .copied()
                    .collect();
                
                // Methods should have at least some overlap (not necessarily all)
                // This is a sanity check, not a strict requirement
                assert!(
                    intersection.len() > 0 || results_i.len() == 0 || results_j.len() == 0,
                    "Methods {} and {} should have some overlap or both return empty",
                    methods[i],
                    methods[j]
                );
            }
        }
    }
    
    Ok(())
}
