//! Integration tests for index factory and auto-tuning.
//!
//! These tests verify that the factory and autotune work together correctly
//! and handle edge cases properly.

use rank_retrieve::dense::ann::factory::index_factory;
use rank_retrieve::dense::ann::autotune::{ParameterTuner, Criterion};
use rank_retrieve::benchmark::datasets::create_benchmark_dataset;
use rank_retrieve::RetrieveError;

#[cfg(feature = "hnsw")]
#[test]
fn test_factory_hnsw_integration() -> Result<(), RetrieveError> {
    let mut index = index_factory(128, "HNSW32")?;
    
    // Add vectors
    for i in 0..100 {
        let vec = vec![0.1; 128];
        index.add(i, vec)?;
    }
    
    // Build
    index.build()?;
    
    // Search
    let query = vec![0.15; 128];
    let results = index.search(&query, 10)?;
    
    assert!(!results.is_empty());
    assert!(results.len() <= 10);
    
    Ok(())
}

#[cfg(feature = "ivf_pq")]
#[test]
fn test_factory_ivf_pq_integration() -> Result<(), RetrieveError> {
    let mut index = index_factory(128, "IVF16,PQ8")?;
    
    // Add vectors
    for i in 0..100 {
        let vec = vec![0.1; 128];
        index.add(i, vec)?;
    }
    
    // Build
    index.build()?;
    
    // Search
    let query = vec![0.15; 128];
    let results = index.search(&query, 10)?;
    
    assert!(!results.is_empty());
    
    Ok(())
}

#[cfg(all(feature = "ivf_pq", feature = "benchmark"))]
#[test]
fn test_autotune_integration() -> Result<(), RetrieveError> {
    let dataset = create_benchmark_dataset(500, 50, 128, 42);
    
    let tuner = ParameterTuner::new()
        .criterion(Criterion::RecallAtK { k: 10, target: 0.80 })
        .num_test_queries(20);  // Small for fast tests
    
    let result = tuner.tune_ivf_pq_nprobe(
        &dataset,
        128,
        16,
        &[1, 2, 4, 8],  // Small set for fast tests
    )?;
    
    // Verify result structure
    assert!(!result.all_results.is_empty());
    assert!(result.recall >= 0.0 && result.recall <= 1.0);
    assert!(result.latency_ms >= 0.0);
    assert!(result.best_value > 0);
    
    // Verify all_results contains the best value
    let best_found = result.all_results.iter()
        .any(|(val, _, _, _)| *val == result.best_value);
    assert!(best_found);
    
    Ok(())
}

#[cfg(all(feature = "hnsw", feature = "benchmark"))]
#[test]
fn test_autotune_hnsw_integration() -> Result<(), RetrieveError> {
    let dataset = create_benchmark_dataset(500, 50, 128, 42);
    
    let tuner = ParameterTuner::new()
        .criterion(Criterion::Balanced {
            k: 10,
            recall_weight: 0.7,
            latency_weight: 0.3,
        })
        .num_test_queries(20);
    
    let result = tuner.tune_hnsw_ef_search(
        &dataset,
        128,
        16,
        &[10, 20, 50],  // Small set for fast tests
    )?;
    
    assert!(!result.all_results.is_empty());
    assert!(result.recall >= 0.0 && result.recall <= 1.0);
    assert!(result.latency_ms >= 0.0);
    
    Ok(())
}

#[cfg(all(feature = "hnsw", feature = "ivf_pq"))]
#[test]
fn test_factory_multiple_types() -> Result<(), RetrieveError> {
    let dimension = 128;
    let test_vector = vec![0.1; dimension];
    
    // Test HNSW
    let mut hnsw = index_factory(dimension, "HNSW32")?;
    hnsw.add(0, test_vector.clone())?;
    hnsw.build()?;
    let _results = hnsw.search(&test_vector, 1)?;
    
    // Test IVF-PQ
    let mut ivf_pq = index_factory(dimension, "IVF16,PQ8")?;
    ivf_pq.add(0, test_vector.clone())?;
    ivf_pq.build()?;
    let _results = ivf_pq.search(&test_vector, 1)?;
    
    Ok(())
}

#[cfg(feature = "hnsw")]
#[test]
fn test_factory_error_messages() {
    // Test that error messages are helpful
    let result = index_factory(128, "INVALID");
    assert!(result.is_err());
    let err = result.unwrap_err();
    let err_msg = format!("{}", err);
    assert!(err_msg.contains("Unsupported") || err_msg.contains("INVALID"));
    
    // Test dimension validation
    let result = index_factory(0, "HNSW32");
    assert!(result.is_err());
    
    // Test parameter validation
    let result = index_factory(128, "HNSW0");
    assert!(result.is_err());
}

#[cfg(all(feature = "ivf_pq", feature = "benchmark"))]
#[test]
fn test_autotune_criteria() -> Result<(), RetrieveError> {
    let dataset = create_benchmark_dataset(200, 20, 64, 42);
    
    // Test RecallAtK criterion
    let tuner1 = ParameterTuner::new()
        .criterion(Criterion::RecallAtK { k: 10, target: 0.90 })
        .num_test_queries(10);
    
    let result1 = tuner1.tune_ivf_pq_nprobe(&dataset, 64, 8, &[1, 2, 4])?;
    assert!(!result1.all_results.is_empty());
    
    // Test LatencyWithRecall criterion
    let tuner2 = ParameterTuner::new()
        .criterion(Criterion::LatencyWithRecall {
            k: 10,
            min_recall: 0.80,
            max_latency_ms: 100.0,
        })
        .num_test_queries(10);
    
    let result2 = tuner2.tune_ivf_pq_nprobe(&dataset, 64, 8, &[1, 2, 4])?;
    assert!(!result2.all_results.is_empty());
    
    Ok(())
}
