//! Example demonstrating index factory and parameter auto-tuning.
//!
//! This example shows how to:
//! 1. Create ANN indexes using the factory pattern (inspired by Faiss)
//! 2. Use parameter auto-tuning to find optimal parameters
//! 3. Compare different index types
//!
//! Run with:
//! ```bash
//! cargo run --example factory_and_autotune --features "hnsw,ivf_pq,benchmark"
//! ```

use rank_retrieve::dense::ann::factory::index_factory;
use rank_retrieve::dense::ann::autotune::{ParameterTuner, Criterion};
use rank_retrieve::benchmark::datasets::{create_benchmark_dataset, Dataset};
use rank_retrieve::benchmark::recall_at_k;
use rank_retrieve::RetrieveError;
use std::time::Instant;

fn main() -> Result<(), RetrieveError> {
    println!("=== Index Factory and Auto-Tune Example ===\n");
    
    // Create a synthetic dataset for demonstration
    println!("Creating synthetic dataset...");
    let dataset = create_benchmark_dataset(10000, 1000, 128, 42);
    println!("Dataset: {} training vectors, {} test vectors, dimension {}\n",
             dataset.train.len(), dataset.test.len(), dataset.dimension);
    
    // Example 1: Using index factory to create different index types
    println!("=== Example 1: Index Factory ===\n");
    
    let dimension = dataset.dimension;
    let k = 10;
    
    // Create HNSW index using factory
    println!("Creating HNSW index with factory...");
    let mut hnsw_index = index_factory(dimension, "HNSW32")?;
    
    // Add vectors
    for (i, vec) in dataset.train.iter().enumerate() {
        hnsw_index.add(i as u32, vec.clone())?;
    }
    hnsw_index.build()?;
    
    // Test search
    let query = &dataset.test[0];
    let start = Instant::now();
    let results = hnsw_index.search(query, k)?;
    let search_time = start.elapsed().as_secs_f32() * 1000.0;
    
    println!("HNSW search: {} results in {:.2}ms", results.len(), search_time);
    println!("Top result: doc_id={}, score={:.4}\n", results[0].0, results[0].1);
    
    // Create IVF-PQ index using factory
    println!("Creating IVF-PQ index with factory...");
    let mut ivf_pq_index = index_factory(dimension, "IVF1024,PQ8")?;
    
    // Add vectors
    for (i, vec) in dataset.train.iter().enumerate() {
        ivf_pq_index.add(i as u32, vec.clone())?;
    }
    ivf_pq_index.build()?;
    
    // Test search
    let start = Instant::now();
    let results = ivf_pq_index.search(query, k)?;
    let search_time = start.elapsed().as_secs_f32() * 1000.0;
    
    println!("IVF-PQ search: {} results in {:.2}ms", results.len(), search_time);
    println!("Top result: doc_id={}, score={:.4}\n", results[0].0, results[0].1);
    
    // Example 2: Parameter auto-tuning for IVF-PQ nprobe
    #[cfg(feature = "ivf_pq")]
    {
        println!("=== Example 2: Auto-Tuning IVF-PQ nprobe ===\n");
        
        // Create tuner with recall target
        let tuner = ParameterTuner::new()
            .criterion(Criterion::RecallAtK { k: 10, target: 0.90 })
            .num_test_queries(50)  // Use 50 queries for faster tuning
            .time_budget(std::time::Duration::from_secs(30));
        
        println!("Tuning nprobe parameter for IVF-PQ...");
        println!("Criterion: Recall@10 >= 0.90");
        println!("Testing nprobe values: [1, 2, 4, 8, 16, 32, 64]\n");
        
        let result = tuner.tune_ivf_pq_nprobe(
            &dataset,
            dimension,
            1024,  // num_clusters
            &[1, 2, 4, 8, 16, 32, 64],  // nprobe values to try
        )?;
        
        println!("Tuning Results:");
        println!("  Best nprobe: {}", result.best_value);
        println!("  Recall@10: {:.4}", result.recall);
        println!("  Latency: {:.2}ms", result.latency_ms);
        println!("  Criterion met: {}\n", result.criterion_met);
        
        println!("All results:");
        for (nprobe, recall, latency, score) in &result.all_results {
            println!("  nprobe={:2}: recall={:.4}, latency={:.2}ms, score={:.4}",
                     nprobe, recall, latency, score);
        }
        println!();
    }
    
    // Example 3: Auto-tuning HNSW ef_search
    #[cfg(feature = "hnsw")]
    {
        println!("=== Example 3: Auto-Tuning HNSW ef_search ===\n");
        
        // Create tuner with balanced criterion (recall + latency)
        let tuner = ParameterTuner::new()
            .criterion(Criterion::Balanced {
                k: 10,
                recall_weight: 0.7,
                latency_weight: 0.3,
            })
            .num_test_queries(50)
            .time_budget(std::time::Duration::from_secs(30));
        
        println!("Tuning ef_search parameter for HNSW...");
        println!("Criterion: Balanced (70% recall, 30% latency)");
        println!("Testing ef_search values: [10, 20, 50, 100, 200]\n");
        
        let result = tuner.tune_hnsw_ef_search(
            &dataset,
            dimension,
            32,  // m parameter
            &[10, 20, 50, 100, 200],  // ef_search values to try
        )?;
        
        println!("Tuning Results:");
        println!("  Best ef_search: {}", result.best_value);
        println!("  Recall@10: {:.4}", result.recall);
        println!("  Latency: {:.2}ms", result.latency_ms);
        println!("  Score: {:.4}\n", result.best_score);
        
        println!("All results:");
        for (ef_search, recall, latency, score) in &result.all_results {
            println!("  ef_search={:3}: recall={:.4}, latency={:.2}ms, score={:.4}",
                     ef_search, recall, latency, score);
        }
        println!();
    }
    
    // Example 4: Comparing index types
    println!("=== Example 4: Comparing Index Types ===\n");
    
    let test_queries = &dataset.test[..10.min(dataset.test.len())];
    
    // HNSW
    let mut hnsw_index = index_factory(dimension, "HNSW32")?;
    for (i, vec) in dataset.train.iter().enumerate() {
        hnsw_index.add(i as u32, vec.clone())?;
    }
    let build_start = Instant::now();
    hnsw_index.build()?;
    let build_time = build_start.elapsed().as_secs_f32();
    
    let mut hnsw_recalls = Vec::new();
    let mut hnsw_times = Vec::new();
    for query in test_queries {
        let start = Instant::now();
        let results = hnsw_index.search(query, k)?;
        let time = start.elapsed().as_secs_f32() * 1000.0;
        hnsw_times.push(time);
        
        // Compute recall (simplified - would need ground truth for real comparison)
        hnsw_recalls.push(results.len() as f32 / k as f32);
    }
    
    let avg_recall = hnsw_recalls.iter().sum::<f32>() / hnsw_recalls.len() as f32;
    let avg_time = hnsw_times.iter().sum::<f32>() / hnsw_times.len() as f32;
    
    println!("HNSW32:");
    println!("  Build time: {:.2}s", build_time);
    println!("  Avg recall (approx): {:.4}", avg_recall);
    println!("  Avg query time: {:.2}ms", avg_time);
    println!("  Memory: ~{} bytes\n", hnsw_index.size_bytes());
    
    // IVF-PQ
    let mut ivf_pq_index = index_factory(dimension, "IVF1024,PQ8")?;
    for (i, vec) in dataset.train.iter().enumerate() {
        ivf_pq_index.add(i as u32, vec.clone())?;
    }
    let build_start = Instant::now();
    ivf_pq_index.build()?;
    let build_time = build_start.elapsed().as_secs_f32();
    
    let mut ivf_pq_recalls = Vec::new();
    let mut ivf_pq_times = Vec::new();
    for query in test_queries {
        let start = Instant::now();
        let results = ivf_pq_index.search(query, k)?;
        let time = start.elapsed().as_secs_f32() * 1000.0;
        ivf_pq_times.push(time);
        
        ivf_pq_recalls.push(results.len() as f32 / k as f32);
    }
    
    let avg_recall = ivf_pq_recalls.iter().sum::<f32>() / ivf_pq_recalls.len() as f32;
    let avg_time = ivf_pq_times.iter().sum::<f32>() / ivf_pq_times.len() as f32;
    
    println!("IVF1024,PQ8:");
    println!("  Build time: {:.2}s", build_time);
    println!("  Avg recall (approx): {:.4}", avg_recall);
    println!("  Avg query time: {:.2}ms", avg_time);
    println!("  Memory: ~{} bytes\n", ivf_pq_index.size_bytes());
    
    println!("=== Example Complete ===");
    
    Ok(())
}
