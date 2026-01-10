//! Example: Using ANN (Approximate Nearest Neighbor) algorithms.
//!
//! Demonstrates how to use the various ANN algorithms implemented in rank-retrieve.
//! All algorithms implement the unified `ANNIndex` trait for consistent API usage.
//!
//! **Available Algorithms:**
//! - **HNSW**: Hierarchical Navigable Small World (graph-based, high recall)
//! - **NSW**: Flat Navigable Small World (lower memory, comparable performance)
//! - **Anisotropic VQ + k-means (SCANN)**: Quantization-based, optimized for MIPS
//! - **IVF-PQ**: Memory-efficient, billion-scale capable
//! - **LSH**: Locality Sensitive Hashing (theoretical guarantees)
//! - **Random Projection Tree Forest (Annoy)**: Production-proven tree-based
//! - **KD-Tree, Ball Tree, Random Projection Tree, K-Means Tree**: Classic tree methods
//!
//! **When to use each:**
//! - **HNSW/NSW**: General purpose, high-dimensional data, high recall needed
//! - **Anisotropic VQ + k-means (SCANN)**: MIPS queries, very large datasets
//! - **IVF-PQ**: Memory-constrained, billion-scale datasets
//! - **LSH**: Hash-based systems, theoretical guarantees needed
//! - **Tree methods**: Low-medium dimensions, simple baseline needed
//!
//! **Performance:**
//! - Construction: O(n log n) for most methods
//! - Search: O(log n) for graph/tree methods, O(n/k) for quantization methods
//! - Memory: ~1.5-2x for graph methods, ~0.1x for quantization methods

#[cfg(feature = "hnsw")]
use rank_retrieve::dense::ann::ANNIndex;
#[cfg(feature = "hnsw")]
use rank_retrieve::dense::hnsw::{HNSWIndex, HNSWParams};

#[cfg(feature = "nsw")]
use rank_retrieve::dense::nsw::{NSWIndex, NSWParams};

#[cfg(feature = "scann")]
use rank_retrieve::dense::scann::{SCANNIndex, SCANNParams};

#[cfg(feature = "lsh")]
use rank_retrieve::dense::classic::lsh::{LSHIndex, LSHParams};

#[cfg(feature = "annoy")]
use rank_retrieve::dense::classic::trees::annoy::{AnnoyIndex, AnnoyParams};

#[cfg(feature = "kdtree")]
use rank_retrieve::dense::classic::trees::kdtree::{KDTreeIndex, KDTreeParams};

#[cfg(feature = "balltree")]
use rank_retrieve::dense::classic::trees::balltree::{BallTreeIndex, BallTreeParams};

#[cfg(feature = "rptree")]
use rank_retrieve::dense::classic::trees::random_projection::{RPTreeIndex, RPTreeParams};

#[cfg(feature = "kmeans_tree")]
use rank_retrieve::dense::classic::trees::kmeans_tree::{KMeansTreeIndex, KMeansTreeParams};

#[cfg(feature = "ivf_pq")]
use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};

#[cfg(feature = "diskann")]
use rank_retrieve::dense::diskann::{DiskANNIndex, DiskANNParams};

#[cfg(feature = "sng")]
use rank_retrieve::dense::sng::{SNGIndex, SNGParams};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ANN Algorithms Example ===\n");

    let dimension = 128;
    let num_vectors = 1000;
    let k = 10;

    // Generate some sample vectors
    let mut vectors = Vec::new();
    for i in 0..num_vectors {
        let mut vec = vec![0.0f32; dimension];
        // Create vectors with some structure
        for j in 0..dimension {
            vec[j] = ((i * dimension + j) as f32) * 0.01;
        }
        // L2 normalize
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        for x in &mut vec {
            *x /= norm;
        }
        vectors.push((i as u32, vec));
    }

    // Query vector
    let mut query = vec![0.0f32; dimension];
    for j in 0..dimension {
        query[j] = (j as f32) * 0.01;
    }
    let norm: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    for x in &mut query {
        *x /= norm;
    }

    // Example 1: HNSW
    #[cfg(feature = "hnsw")]
    {
        println!("1. HNSW (Hierarchical Navigable Small World)");
        let params = HNSWParams::default();
        let mut index = HNSWIndex::with_params(dimension, params.clone())?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k, params.ef_search)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 2: NSW (Flat Navigable Small World)
    #[cfg(feature = "nsw")]
    {
        println!("2. NSW (Flat Navigable Small World)");
        let params = NSWParams::default();
        let mut index = NSWIndex::with_params(dimension, params.clone())?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k, params.ef_search)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 3: Anisotropic VQ + k-means (SCANN)
    #[cfg(feature = "scann")]
    {
        println!("3. Anisotropic VQ + k-means (vendor: SCANN)");
        let params = SCANNParams {
            num_partitions: 256,
            num_reorder: 100,
            quantization_bits: 8,
        };
        let mut index = SCANNIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 4: LSH
    #[cfg(feature = "lsh")]
    {
        println!("4. LSH (Locality Sensitive Hashing)");
        let params = LSHParams::default();
        let mut index = LSHIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 5: Random Projection Tree Forest (Annoy)
    #[cfg(feature = "annoy")]
    {
        println!("5. Random Projection Tree Forest (vendor: Annoy)");
        let params = AnnoyParams::default();
        let mut index = AnnoyIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 6: KD-Tree (for low dimensions)
    #[cfg(feature = "kdtree")]
    {
        if dimension < 20 {
            println!("6. KD-Tree (for low dimensions)");
            let params = KDTreeParams::default();
            let mut index = KDTreeIndex::new(dimension, params)?;

            for (id, vec) in &vectors {
                index.add(*id, vec.clone())?;
            }
            index.build()?;

            let results = index.search(&query, k)?;
            println!("   Retrieved {} results", results.len());
            println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
        }
    }

    // Example 7: Ball Tree (for medium dimensions)
    #[cfg(feature = "balltree")]
    {
        if dimension >= 20 && dimension < 100 {
            println!("7. Ball Tree (for medium dimensions)");
            let params = BallTreeParams::default();
            let mut index = BallTreeIndex::new(dimension, params)?;

            for (id, vec) in &vectors {
                index.add(*id, vec.clone())?;
            }
            index.build()?;

            let results = index.search(&query, k)?;
            println!("   Retrieved {} results", results.len());
            println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
        }
    }

    // Example 8: Random Projection Tree
    #[cfg(feature = "rptree")]
    {
        println!("8. Random Projection Tree");
        let params = RPTreeParams::default();
        let mut index = RPTreeIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 8b: K-Means Tree
    #[cfg(feature = "kmeans_tree")]
    {
        println!("8b. K-Means Tree (Hierarchical Clustering Tree)");
        let params = KMeansTreeParams::default();
        let mut index = KMeansTreeIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 9: IVF-PQ
    #[cfg(feature = "ivf_pq")]
    {
        println!("9. IVF-PQ (Inverted File Index with Product Quantization)");
        let params = IVFPQParams::default();
        let mut index = IVFPQIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 10: DiskANN
    #[cfg(feature = "diskann")]
    {
        println!("10. DiskANN (Disk-based ANN)");
        let params = DiskANNParams::default();
        let mut index = DiskANNIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    // Example 11: OPT-SNG
    #[cfg(feature = "sng")]
    {
        println!("11. OPT-SNG (Optimized Sparse Neighborhood Graph)");
        let params = SNGParams::default();
        let mut index = SNGIndex::new(dimension, params)?;

        for (id, vec) in &vectors {
            index.add(*id, vec.clone())?;
        }
        index.build()?;

        let results = index.search(&query, k)?;
        println!("   Retrieved {} results", results.len());
        println!("   Top 3: {:?}\n", &results[..3.min(results.len())]);
    }

    println!("=== Example Complete ===");
    println!("\n**Next steps:**");
    println!("- See docs/ANN_METHODS_SUMMARY.md for algorithm comparison");
    println!("- See docs/IMPLEMENTATION_STATUS_2026.md for implementation status");
    println!("- Use unified ANNIndex trait for algorithm-agnostic code");

    Ok(())
}
