//! Tests for tree-based ANN methods: KD-Tree, Ball Tree, Random Projection Tree, K-Means Tree.

#[cfg(any(feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
use rank_retrieve::dense::ann::ANNIndex;

#[cfg(any(feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
fn generate_test_vectors(num_vectors: usize, dimension: usize, seed: u64) -> Vec<Vec<f32>> {
    use rand::Rng;
    use rand::SeedableRng;
    use rand::rngs::StdRng;
    
    let mut rng = StdRng::seed_from_u64(seed);
    let mut vectors = Vec::new();
    
    for _ in 0..num_vectors {
        let mut vec = Vec::with_capacity(dimension);
        let mut norm = 0.0;
        
        for _ in 0..dimension {
            let val = rng.gen::<f32>() * 2.0 - 1.0;
            vec.push(val);
            norm += val * val;
        }
        
        // Normalize
        let norm = norm.sqrt();
        if norm > 0.0 {
            for val in vec.iter_mut() {
                *val /= norm;
            }
        }
        
        vectors.push(vec);
    }
    
    vectors
}

#[cfg(feature = "kdtree")]
#[test]
fn test_kdtree_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::kdtree::{KDTreeIndex, KDTreeParams};
    
    let dimension = 16;  // Low dimension for KD-Tree
    let num_vectors = 100;
    let vectors = generate_test_vectors(num_vectors, dimension, 42);
    
    let mut index = KDTreeIndex::new(dimension, KDTreeParams::default())?;
    
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    
    index.build()?;
    
    let query = &vectors[0];
    let results = index.search(query, 10)?;
    
    assert!(!results.is_empty(), "Should return results");
    assert!(results.len() <= 10, "Should return at most k results");
    
    Ok(())
}

#[cfg(feature = "balltree")]
#[test]
fn test_balltree_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::balltree::{BallTreeIndex, BallTreeParams};
    
    let dimension = 64;  // Medium dimension for Ball Tree
    let num_vectors = 100;
    let vectors = generate_test_vectors(num_vectors, dimension, 42);
    
    let mut index = BallTreeIndex::new(dimension, BallTreeParams::default())?;
    
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    
    index.build()?;
    
    let query = &vectors[0];
    let results = index.search(query, 10)?;
    
    assert!(!results.is_empty(), "Should return results");
    assert!(results.len() <= 10, "Should return at most k results");
    
    Ok(())
}

#[cfg(feature = "rptree")]
#[test]
fn test_rptree_basic() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::random_projection::{RPTreeIndex, RPTreeParams};
    
    let dimension = 128;
    let num_vectors = 100;
    let vectors = generate_test_vectors(num_vectors, dimension, 42);
    
    let mut index = RPTreeIndex::new(dimension, RPTreeParams::default())?;
    
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    
    index.build()?;
    
    let query = &vectors[0];
    let results = index.search(query, 10)?;
    
    assert!(!results.is_empty(), "Should return results");
    assert!(results.len() <= 10, "Should return at most k results");
    
    Ok(())
}

#[cfg(all(feature = "kdtree", feature = "dense"))]
#[test]
fn test_kdtree_ann_index_trait() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::kdtree::{KDTreeIndex, KDTreeParams};
    
    let dimension = 16;
    let num_vectors = 50;
    let vectors = generate_test_vectors(num_vectors, dimension, 42);
    
    let mut index = KDTreeIndex::new(dimension, KDTreeParams::default())?;
    
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    
    index.build()?;
    
    // Test search works
    let query = &vectors[0];
    let results = index.search(query, 10)?;
    assert!(!results.is_empty());
    
    Ok(())
}

#[cfg(all(feature = "balltree", feature = "dense"))]
#[test]
fn test_balltree_ann_index_trait() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::balltree::{BallTreeIndex, BallTreeParams};
    
    let dimension = 64;
    let num_vectors = 50;
    let vectors = generate_test_vectors(num_vectors, dimension, 42);
    
    let mut index = BallTreeIndex::new(dimension, BallTreeParams::default())?;
    
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    
    index.build()?;
    
    // Test search works
    let query = &vectors[0];
    let results = index.search(query, 10)?;
    assert!(!results.is_empty());
    
    Ok(())
}

#[cfg(all(feature = "rptree", feature = "dense"))]
#[test]
fn test_rptree_ann_index_trait() -> Result<(), RetrieveError> {
    use rank_retrieve::dense::classic::trees::random_projection::{RPTreeIndex, RPTreeParams};
    
    let dimension = 128;
    let num_vectors = 50;
    let vectors = generate_test_vectors(num_vectors, dimension, 42);
    
    let mut index = RPTreeIndex::new(dimension, RPTreeParams::default())?;
    
    for (i, vec) in vectors.iter().enumerate() {
        index.add(i as u32, vec.clone())?;
    }
    
    index.build()?;
    
    // Test search works
    let query = &vectors[0];
    let results = index.search(query, 10)?;
    assert!(!results.is_empty());
    
    Ok(())
}
