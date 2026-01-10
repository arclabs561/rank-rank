//! Comprehensive tests for all ANN algorithms.

use rank_retrieve::RetrieveError;
use rand::Rng;

/// Generate random normalized vectors for testing.
fn generate_test_vectors(num: usize, dimension: usize) -> Vec<Vec<f32>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::new();
    
    for _ in 0..num {
        let mut vec = Vec::with_capacity(dimension);
        let mut norm = 0.0;
        
        // Generate random vector
        for _ in 0..dimension {
            let val = rng.gen::<f32>() * 2.0 - 1.0;
            norm += val * val;
            vec.push(val);
        }
        
        // Normalize
        let norm = norm.sqrt();
        if norm > 0.0 {
            for val in &mut vec {
                *val /= norm;
            }
        }
        
        vectors.push(vec);
    }
    
    vectors
}

/// Test basic functionality of an ANN index.
fn test_ann_basic<F>(create_index: F) -> Result<(), RetrieveError>
where
    F: Fn(usize) -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError>,
{
    let dimension = 128;
    let num_vectors = 100;
    let k = 10;
    
    // Generate test vectors
    let vectors = generate_test_vectors(num_vectors, dimension);
    
    // Create and build index
    let mut index = create_index(dimension)?;
    
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    
    index.build()?;
    
    // Test search
    let query = &vectors[0];
    let results = index.search(query, k)?;
    
    // Verify results
    assert!(!results.is_empty(), "Search should return results");
    assert!(results.len() <= k, "Should return at most k results");
    
    // Verify distances are non-decreasing
    for i in 1..results.len() {
        assert!(
            results[i].1 >= results[i - 1].1,
            "Distances should be non-decreasing"
        );
    }
    
    // First result should be the query vector itself (distance = 0)
    if !results.is_empty() {
        assert_eq!(results[0].0, 0, "First result should be query vector");
        assert!(results[0].1 < 0.01, "Distance to self should be ~0");
    }
    
    Ok(())
}

#[cfg(feature = "hnsw")]
#[test]
fn test_hnsw_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};
    
    test_ann_basic(|dim| {
        let params = HNSWParams::default();
        let index = HNSWIndex::new(dim, params.m, params.m_max)?;
        Ok(Box::new(index))
    })
}

#[cfg(feature = "nsw")]
#[test]
fn test_nsw_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::nsw::{NSWIndex, NSWParams};
    
    test_ann_basic(|dim| {
        let params = NSWParams::default();
        let index = NSWIndex::with_params(dim, params)?;
        Ok(Box::new(index))
    })
}

#[cfg(feature = "sng")]
#[test]
fn test_sng_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::sng::{SNGIndex, SNGParams};
    
    test_ann_basic(|dim| {
        let params = SNGParams::default();
        let index = SNGIndex::new(dim, params)?;
        Ok(Box::new(index))
    })
}

#[cfg(feature = "scann")]
#[test]
fn test_scann_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::scann::{SCANNIndex, SCANNParams};
    
    test_ann_basic(|dim| {
        let params = SCANNParams::default();
        let index = SCANNIndex::new(dim, params)?;
        Ok(Box::new(index))
    })
}

#[cfg(feature = "ivf_pq")]
#[test]
fn test_ivf_pq_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
    
    test_ann_basic(|dim| {
        let params = IVFPQParams::default();
        let index = IVFPQIndex::new(dim, params)?;
        Ok(Box::new(index))
    })
}

#[cfg(feature = "kmeans_tree")]
#[test]
fn test_kmeans_tree_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};
    
    test_ann_basic(|dim| {
        let params = KMeansTreeParams::default();
        let index = KMeansTreeIndex::new(dim, params)?;
        Ok(Box::new(index))
    })
}

#[cfg(feature = "kmeans_tree")]
#[test]
fn test_kmeans_tree_recall() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};
    
    test_recall(|dim| {
        let params = KMeansTreeParams::default();
        let index = KMeansTreeIndex::new(dim, params)?;
        Ok(Box::new(index))
    }, 0.6)  // Expect at least 60% recall (tree methods typically lower than graph methods)
}

/// Test recall against brute-force.
fn test_recall<F>(create_index: F, expected_recall: f32) -> Result<(), RetrieveError>
where
    F: Fn(usize) -> Result<Box<dyn rank_retrieve::dense::ann::ANNIndex>, RetrieveError>,
{
    let dimension = 64;
    let num_vectors = 1000;
    let k = 10;
    
    // Generate test vectors
    let vectors = generate_test_vectors(num_vectors, dimension);
    
    // Create ANN index
    let mut ann_index = create_index(dimension)?;
    for (i, vec) in vectors.iter().enumerate() {
        ann_index.add(i as u32, vec.clone())?;
    }
    ann_index.build()?;
    
    // Brute-force search for ground truth
    let query = &vectors[0];
    let mut brute_force_results: Vec<(u32, f32)> = vectors
        .iter()
        .enumerate()
        .map(|(i, vec)| {
            let dist = 1.0 - rank_retrieve::simd::dot(query, vec);
            (i as u32, dist)
        })
        .collect();
    brute_force_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let ground_truth: Vec<u32> = brute_force_results.iter().take(k).map(|(id, _)| *id).collect();
    
    // ANN search
    let ann_results = ann_index.search(query, k)?;
    let ann_ids: Vec<u32> = ann_results.iter().map(|(id, _)| *id).collect();
    
    // Calculate recall
    let intersection: Vec<u32> = ground_truth
        .iter()
        .filter(|id| ann_ids.contains(id))
        .copied()
        .collect();
    let recall = intersection.len() as f32 / k as f32;
    
    assert!(
        recall >= expected_recall,
        "Recall {} should be >= {}",
        recall,
        expected_recall
    );
    
    Ok(())
}

#[cfg(feature = "hnsw")]
#[test]
fn test_hnsw_recall() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};
    
    test_recall(|dim| {
        let params = HNSWParams::default();
        let index = HNSWIndex::new(dim, params.m, params.m_max)?;
        Ok(Box::new(index))
    }, 0.8)  // Expect at least 80% recall
}

#[cfg(feature = "sng")]
#[test]
fn test_sng_recall() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::sng::{SNGIndex, SNGParams};
    
    test_recall(|dim| {
        let params = SNGParams::default();
        let index = SNGIndex::new(dim, params)?;
        Ok(Box::new(index))
    }, 0.7)  // Expect at least 70% recall
}
