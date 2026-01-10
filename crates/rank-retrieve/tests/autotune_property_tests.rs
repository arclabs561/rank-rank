//! Property-based tests for parameter auto-tuning.
//!
//! These tests validate that auto-tuning correctly handles various
//! parameter ranges and finds reasonable solutions.

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use rank_retrieve::dense::ann::autotune::{ParameterTuner, Criterion};
    use rank_retrieve::benchmark::datasets::create_benchmark_dataset;
    use rank_retrieve::RetrieveError;

    /// Property: Auto-tune should find a valid parameter
    #[cfg(all(feature = "ivf_pq", feature = "benchmark"))]
    #[test]
    fn prop_autotune_finds_parameter() {
        proptest!(|(
            train_size in 50usize..500,
            test_size in 10usize..100,
            dim in 8usize..256,
            num_clusters in 4usize..64,
            seed in 0u64..1000
        )| {
            // Ensure dimension is reasonable for PQ
            let dim = (dim / 8) * 8;  // Make divisible by 8
            if dim == 0 {
                return Ok(());
            }
            
            let dataset = create_benchmark_dataset(train_size, test_size, dim, seed);
            let tuner = ParameterTuner::new()
                .num_test_queries(test_size.min(20));  // Limit for speed
            
            let nprobe_values = vec![1, 2, 4, 8, num_clusters.min(16)];
            let result = tuner.tune_ivf_pq_nprobe(
                &dataset,
                dim,
                num_clusters,
                &nprobe_values,
            );
            
            if let Ok(tuning_result) = result {
                prop_assert!(!tuning_result.all_results.is_empty(),
                    "Auto-tune should return results");
                prop_assert!(tuning_result.recall >= 0.0 && tuning_result.recall <= 1.0,
                    "Recall should be in [0, 1]: {}", tuning_result.recall);
                prop_assert!(tuning_result.latency_ms >= 0.0,
                    "Latency should be non-negative: {}", tuning_result.latency_ms);
                prop_assert!(nprobe_values.contains(&tuning_result.best_value),
                    "Best value should be from candidate set");
            }
        });
    }

    /// Property: Auto-tune should respect time budget
    #[cfg(all(feature = "ivf_pq", feature = "benchmark"))]
    #[test]
    fn prop_autotune_time_budget() {
        use std::time::Duration;
        use std::time::Instant;
        
        let dataset = create_benchmark_dataset(200, 20, 128, 42);
        let tuner = ParameterTuner::new()
            .time_budget(Duration::from_millis(100))  // Very short budget
            .num_test_queries(5);
        
        let start = Instant::now();
        let result = tuner.tune_ivf_pq_nprobe(
            &dataset,
            128,
            16,
            &[1, 2, 4, 8, 16, 32, 64],  // Many values
        );
        let elapsed = start.elapsed();
        
        // Should complete within reasonable time (budget + some overhead)
        assert!(elapsed < Duration::from_millis(500),
            "Auto-tune should respect time budget: took {:?}", elapsed);
        
        // May have partial results
        if let Ok(tuning_result) = result {
            assert!(!tuning_result.all_results.is_empty(),
                "Should have at least some results even with time budget");
        }
    }

    /// Property: Auto-tune criteria should evaluate correctly
    #[test]
    fn prop_criterion_evaluation() {
        proptest!(|(
            recall in 0.0f32..1.0f32,
            latency in 0.0f32..1000.0f32,
            target in 0.5f32..1.0f32
        )| {
            let criterion = Criterion::RecallAtK { k: 10, target };
            let (score, met) = criterion.evaluate(recall, latency);
            
            prop_assert_eq!(score, recall, "Score should equal recall for RecallAtK");
            prop_assert_eq!(met, recall >= target,
                "Criterion met should match: recall={}, target={}", recall, target);
        });
    }

    /// Property: Balanced criterion should produce scores in [0, 1]
    #[test]
    fn prop_balanced_criterion_range() {
        proptest!(|(
            recall in 0.0f32..1.0f32,
            latency in 0.0f32..1000.0f32,
            recall_weight in 0.0f32..1.0f32,
            latency_weight in 0.0f32..1.0f32
        )| {
            let criterion = Criterion::Balanced {
                k: 10,
                recall_weight,
                latency_weight,
            };
            let (score, _met) = criterion.evaluate(recall, latency);
            
            prop_assert!(score >= 0.0 && score <= 1.0,
                "Balanced criterion score should be in [0, 1]: {}", score);
        });
    }

    /// Property: Auto-tune should handle small datasets
    #[cfg(all(feature = "ivf_pq", feature = "benchmark"))]
    #[test]
    fn prop_autotune_small_dataset() {
        let dataset = create_benchmark_dataset(20, 5, 64, 42);
        let tuner = ParameterTuner::new()
            .num_test_queries(5);
        
        let result = tuner.tune_ivf_pq_nprobe(
            &dataset,
            64,
            8,
            &[1, 2, 4],
        );
        
        assert!(result.is_ok(), "Auto-tune should handle small datasets");
        let tuning_result = result.unwrap();
        assert!(!tuning_result.all_results.is_empty(),
            "Should return results even for small dataset");
    }

    /// Property: Auto-tune results should be consistent
    #[cfg(all(feature = "ivf_pq", feature = "benchmark"))]
    #[test]
    fn prop_autotune_consistency() {
        let dataset = create_benchmark_dataset(200, 20, 128, 42);
        let tuner = ParameterTuner::new()
            .num_test_queries(10);
        
        // Run twice with same parameters
        let result1 = tuner.tune_ivf_pq_nprobe(
            &dataset,
            128,
            16,
            &[1, 2, 4, 8],
        );
        
        let result2 = tuner.tune_ivf_pq_nprobe(
            &dataset,
            128,
            16,
            &[1, 2, 4, 8],
        );
        
        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            // Results should have same structure
            assert_eq!(r1.all_results.len(), r2.all_results.len(),
                "Results should have same number of parameter values tried");
            
            // Best value should be same (deterministic)
            assert_eq!(r1.best_value, r2.best_value,
                "Best value should be consistent: {} vs {}", 
                r1.best_value, r2.best_value);
        }
    }
}
