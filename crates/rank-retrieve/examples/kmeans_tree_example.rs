//! Example: K-Means Tree for Hierarchical Clustering Search
//!
//! Demonstrates how to use the K-Means Tree algorithm for approximate nearest neighbor search.
//! K-Means Tree is a hierarchical clustering structure that recursively partitions the data
//! space using k-means clustering at each node.
//!
//! **When to use K-Means Tree:**
//! - Medium to large datasets (1000+ vectors)
//! - When you need a tree-based method with good accuracy
//! - Hierarchical clustering structure is beneficial
//! - Complement to other tree methods (KD-Tree, Ball Tree)
//!
//! **Performance:**
//! - Build time: O(n log n) with k-means clustering
//! - Search time: O(log n) tree traversal
//! - Memory: Moderate (tree structure + vectors)
//!
//! **Parameters:**
//! - `num_clusters`: Number of clusters per node (default: 16)
//! - `max_depth`: Maximum tree depth (default: 10)
//! - `max_leaf_size`: Maximum vectors in a leaf node (default: 100)
//! - `max_iterations`: Max iterations for k-means (default: 50)

#[cfg(feature = "kmeans_tree")]
use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};
#[cfg(feature = "kmeans_tree")]
use rank_retrieve::RetrieveError;

#[cfg(feature = "kmeans_tree")]
fn main() -> Result<(), RetrieveError> {
    println!("=== K-Means Tree Example ===\n");

    let dimension = 128;
    let num_vectors = 2000;
    let k = 10;

    // Generate sample vectors with some structure
    println!("Generating {} vectors of dimension {}...", num_vectors, dimension);
    let mut vectors = Vec::new();
    for i in 0..num_vectors {
        let mut vec = vec![0.0f32; dimension];
        // Create vectors with some structure (clusters)
        let cluster_id = i / 400; // 5 clusters
        for j in 0..dimension {
            vec[j] = ((cluster_id * dimension + j) as f32) * 0.01 + 
                     (i as f32) * 0.0001; // Add some variation
        }
        // L2 normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        for x in &mut vec {
            *x /= norm;
        }
        vectors.push((i as u32, vec));
    }
    println!("Generated {} vectors\n", num_vectors);

    // Create K-Means Tree with default parameters
    println!("Creating K-Means Tree index...");
    let params = KMeansTreeParams::default();
    println!("Parameters: num_clusters={}, max_depth={}, max_leaf_size={}, max_iterations={}", 
             params.num_clusters, params.max_depth, params.max_leaf_size, params.max_iterations);
    
    let mut index = KMeansTreeIndex::new(dimension, params)?;

    // Add vectors
    println!("Adding vectors to index...");
    for (id, vec) in &vectors {
        index.add(*id, vec.clone())?;
    }

    // Build the tree
    println!("Building K-Means Tree (this may take a moment)...");
    index.build()?;
    println!("Tree built successfully!\n");

    // Test queries
    println!("=== Testing Queries ===\n");

    // Query 1: Search for vectors similar to first vector
    let query1 = &vectors[0].1;
    println!("Query 1: Searching for vectors similar to vector 0");
    let results1 = index.search(query1, k)?;
    println!("   Retrieved {} results", results1.len());
    println!("   Top 5: {:?}\n", &results1[..5.min(results1.len())]);

    // Query 2: Search for vectors from a different cluster
    let query2 = &vectors[800].1; // From a different cluster
    println!("Query 2: Searching for vectors similar to vector 800");
    let results2 = index.search(query2, k)?;
    println!("   Retrieved {} results", results2.len());
    println!("   Top 5: {:?}\n", &results2[..5.min(results2.len())]);

    // Query 3: Custom parameters
    println!("Query 3: Using custom parameters (num_clusters=4, max_depth=8)");
    let custom_params = KMeansTreeParams {
        num_clusters: 4,
        max_depth: 8,
        max_leaf_size: 50,
        max_iterations: 30,
    };
    let mut custom_index = KMeansTreeIndex::new(dimension, custom_params)?;
    for (id, vec) in &vectors[..1000] { // Use subset for faster build
        custom_index.add(*id, vec.clone())?;
    }
    custom_index.build()?;
    let results3 = custom_index.search(query1, k)?;
    println!("   Retrieved {} results", results3.len());
    println!("   Top 5: {:?}\n", &results3[..5.min(results3.len())]);

    // Performance comparison
    println!("=== Performance Notes ===");
    println!("- K-Means Tree builds a hierarchical clustering structure");
    println!("- Each node uses k-means to partition data into clusters");
    println!("- Search traverses the tree by finding closest clusters");
    println!("- Good balance between accuracy and speed");
    println!("\n**Comparison with other methods:**");
    println!("- vs. KD-Tree: Better for high dimensions, handles clusters well");
    println!("- vs. Ball Tree: Similar performance, different partitioning");
    println!("- vs. HNSW: Lower memory, but may have lower recall");

    Ok(())
}

#[cfg(not(feature = "kmeans_tree"))]
fn main() {
    println!("This example requires the 'kmeans_tree' feature.");
    println!("Run with: cargo run --example kmeans_tree_example --features dense,kmeans_tree");
}
