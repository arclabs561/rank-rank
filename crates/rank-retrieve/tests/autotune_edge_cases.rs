//! Comprehensive edge case tests for parameter auto-tuning.
//!
//! Tests boundary conditions, error cases, and unusual inputs.

use rank_retrieve::dense::ann::autotune::{ParameterTuner, Criterion};
use rank_retrieve::benchmark::datasets::{Dataset, create_benchmark_dataset};
use rank_retrieve::RetrieveError;
use std::time::Duration;

#[test]
fn test_autotune_empty_dataset() {
    #[cfg(feature = "ivf_pq")]
    {
        let empty_dataset = Dataset {
            train: Vec::new(),
            test: Vec::new(),
            dimension: 128,
        };
        
        let tuner = ParameterTuner::new();
        let result = tuner.tune_ivf_pq_nprobe(&empty_dataset, 128, 16, &[1, 2, 4]);
        assert!(result.is_err(), "Should reject empty dataset");
    }
}

#[test]
fn test_autotune_empty_test_set() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = Dataset {
            train: vec![vec![0.1; 128]; 100],
            test: Vec::new(),
            dimension: 128,
        };
        
        let tuner = ParameterTuner::new();
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[1, 2, 4]);
        assert!(result.is_err(), "Should reject empty test set");
    }
}

#[test]
fn test_autotune_empty_parameter_values() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[]);
        assert!(result.is_err(), "Should reject empty parameter values");
    }
}

#[test]
fn test_autotune_nprobe_exceeds_clusters() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        
        // nprobe (20) > num_clusters (16)
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[1, 2, 4, 20]);
        assert!(result.is_err(), "Should reject nprobe > num_clusters");
    }
}

#[test]
fn test_autotune_zero_parameters() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        
        // Zero nprobe
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[0, 1, 2]);
        assert!(result.is_err(), "Should reject zero nprobe");
    }
    
    #[cfg(feature = "hnsw")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        
        // Zero ef_search
        let result = tuner.tune_hnsw_ef_search(&dataset, 128, 16, &[0, 10, 20]);
        assert!(result.is_err(), "Should reject zero ef_search");
    }
}

#[test]
fn test_autotune_zero_dimension() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 0, 16, &[1, 2, 4]);
        assert!(result.is_err(), "Should reject zero dimension");
    }
}

#[test]
fn test_autotune_zero_clusters() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 0, &[1, 2, 4]);
        assert!(result.is_err(), "Should reject zero clusters");
    }
}

#[test]
fn test_autotune_single_parameter_value() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        
        // Single value should work
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[8]);
        assert!(result.is_ok(), "Should handle single parameter value");
        
        let tuning_result = result.unwrap();
        assert_eq!(tuning_result.best_value, 8);
        assert_eq!(tuning_result.all_results.len(), 1);
    }
}

#[test]
fn test_autotune_time_budget_zero() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new()
            .time_budget(Duration::from_secs(0));  // Zero budget
        
        // Should still try at least one parameter
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[1, 2, 4]);
        // May succeed with partial results or fail, but shouldn't panic
        let _ = result;
    }
}

#[test]
fn test_autotune_very_large_parameter_values() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new();
        
        // Very large nprobe (but <= num_clusters)
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 1024, &[1, 512, 1024]);
        // Should work, but may be slow
        let _ = result;  // Just check it doesn't panic
    }
}

#[test]
fn test_autotune_very_small_dataset() {
    #[cfg(feature = "ivf_pq")]
    {
        // Minimal dataset
        let dataset = Dataset {
            train: vec![vec![0.1; 64]; 10],
            test: vec![vec![0.2; 64]; 2],
            dimension: 64,
        };
        
        let tuner = ParameterTuner::new()
            .num_test_queries(2);
        
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 64, 4, &[1, 2]);
        assert!(result.is_ok(), "Should handle very small dataset");
    }
}

#[test]
fn test_autotune_more_queries_than_available() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(100, 10, 128, 42);
        let tuner = ParameterTuner::new()
            .num_test_queries(1000);  // More than available (10)
        
        // Should use all available queries
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[1, 2, 4]);
        assert!(result.is_ok(), "Should handle more queries requested than available");
        
        let tuning_result = result.unwrap();
        // Should have results (used all 10 available queries)
        assert!(!tuning_result.all_results.is_empty());
    }
}

#[test]
fn test_autotune_criterion_edge_cases() {
    // Test criterion evaluation with edge values
    let criterion = Criterion::RecallAtK { k: 10, target: 0.95 };
    
    // Perfect recall
    let (score, met) = criterion.evaluate(1.0, 10.0);
    assert_eq!(score, 1.0);
    assert!(met);
    
    // Zero recall
    let (score, met) = criterion.evaluate(0.0, 10.0);
    assert_eq!(score, 0.0);
    assert!(!met);
    
    // Exactly at threshold
    let (score, met) = criterion.evaluate(0.95, 10.0);
    assert_eq!(score, 0.95);
    assert!(met);
    
    // Just below threshold
    let (score, met) = criterion.evaluate(0.949, 10.0);
    assert_eq!(score, 0.949);
    assert!(!met);
}

#[test]
fn test_autotune_balanced_criterion_weights() {
    // Test balanced criterion with various weights
    let criterion1 = Criterion::Balanced {
        k: 10,
        recall_weight: 1.0,
        latency_weight: 0.0,
    };
    
    let (score1, _) = criterion1.evaluate(0.9, 10.0);
    assert!((score1 - 0.9).abs() < 0.01, "Should weight recall only");
    
    let criterion2 = Criterion::Balanced {
        k: 10,
        recall_weight: 0.0,
        latency_weight: 1.0,
    };
    
    let (score2, _) = criterion2.evaluate(0.9, 10.0);
    // Should weight latency only (lower latency = higher score)
    assert!(score2 < 1.0);
}

#[test]
fn test_autotune_latency_with_recall_edge_cases() {
    let criterion = Criterion::LatencyWithRecall {
        k: 10,
        min_recall: 0.90,
        max_latency_ms: 10.0,
    };
    
    // Both met
    let (score, met) = criterion.evaluate(0.95, 8.0);
    assert!(met);
    assert!(score < 0.0);  // Negative latency (lower is better)
    
    // Recall not met
    let (score, met) = criterion.evaluate(0.85, 5.0);
    assert!(!met);
    assert!(score < 0.0);  // Penalized
    
    // Latency not met
    let (score, met) = criterion.evaluate(0.95, 15.0);
    assert!(!met);
    
    // Neither met
    let (score, met) = criterion.evaluate(0.85, 15.0);
    assert!(!met);
}

#[test]
fn test_autotune_result_consistency() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(200, 20, 128, 42);
        let tuner = ParameterTuner::new()
            .num_test_queries(10);
        
        // Run multiple times
        let results: Vec<_> = (0..3)
            .map(|_| tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[1, 2, 4, 8]))
            .collect();
        
        // All should succeed
        for result in &results {
            assert!(result.is_ok());
        }
        
        // All should have same best value (deterministic)
        if let (Ok(r1), Ok(r2), Ok(r3)) = (&results[0], &results[1], &results[2]) {
            assert_eq!(r1.best_value, r2.best_value);
            assert_eq!(r2.best_value, r3.best_value);
        }
    }
}

#[test]
fn test_autotune_all_results_ordering() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(200, 20, 128, 42);
        let tuner = ParameterTuner::new()
            .num_test_queries(10);
        
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[1, 2, 4, 8]).unwrap();
        
        // All results should be in order tried
        let values: Vec<usize> = result.all_results.iter().map(|(v, _, _, _)| *v).collect();
        assert_eq!(values, vec![1, 2, 4, 8], "Results should be in order tried");
        
        // Best value should be one of the tried values
        assert!(result.all_results.iter().any(|(v, _, _, _)| *v == result.best_value),
                "Best value should be from tried values");
    }
}

#[test]
fn test_autotune_score_ordering() {
    #[cfg(feature = "ivf_pq")]
    {
        let dataset = create_benchmark_dataset(200, 20, 128, 42);
        let tuner = ParameterTuner::new()
            .criterion(Criterion::RecallAtK { k: 10, target: 0.80 })
            .num_test_queries(10);
        
        let result = tuner.tune_ivf_pq_nprobe(&dataset, 128, 16, &[1, 2, 4, 8]).unwrap();
        
        // Best score should be highest
        let best_score = result.best_score;
        for (_, _, _, score) in &result.all_results {
            assert!(*score <= best_score,
                    "Best score should be highest: {} <= {}", score, best_score);
        }
    }
}
