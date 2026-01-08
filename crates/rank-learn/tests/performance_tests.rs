//! Performance and stress tests for rank-learn.
//!
//! Tests behavior under load and with large datasets.

#[cfg(test)]
mod tests {
    use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};
    use std::time::Instant;

    #[test]
    fn test_large_list_ndcg_performance() {
        // Test NDCG computation with large lists
        // Use small relevance values (0-4) to avoid overflow with exponential gain
        let n = 10000;
        let relevance: Vec<f32> = (0..n).map(|i| ((n - i) % 5) as f32).collect();
        
        let start = Instant::now();
        let ndcg = ndcg_at_k(&relevance, None, true).unwrap();
        let elapsed = start.elapsed();
        
        // Should complete quickly (< 100ms for 10K items)
        assert!(elapsed.as_millis() < 100, "NDCG computation took too long: {:?}", elapsed);
        assert!(ndcg >= 0.0 && ndcg <= 1.0);
    }

    #[test]
    fn test_large_list_lambdarank_performance() {
        // Test LambdaRank with large lists
        // Note: LambdaRank is O(n²), so we test with smaller n for performance tests
        let trainer = LambdaRankTrainer::default();
        let n = 500; // Reduced from 1000 to keep test reasonable
        let scores: Vec<f32> = (0..n).map(|i| (i as f32) * 0.001).collect();
        let relevance: Vec<f32> = (0..n).map(|i| ((n - i) as f32) * 0.1).collect();
        
        let start = Instant::now();
        let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();
        let elapsed = start.elapsed();
        
        // Should complete in reasonable time (< 30 seconds for 500 items in debug mode)
        // Note: LambdaRank is O(n²), so 500 items = 250K operations
        // Debug builds are slower, so use generous threshold
        assert!(elapsed.as_secs_f64() < 30.0, "LambdaRank computation took too long: {:?}", elapsed);
        assert_eq!(lambdas.len(), n);
    }

    #[test]
    fn test_batch_processing_performance() {
        // Test batch processing performance
        let trainer = LambdaRankTrainer::default();
        let batch_size = 100;
        let list_size = 50;
        
        let batches: Vec<(Vec<f32>, Vec<f32>)> = (0..batch_size)
            .map(|_| {
                let scores: Vec<f32> = (0..list_size).map(|i| (i as f32) * 0.1).collect();
                let relevance: Vec<f32> = (0..list_size).map(|i| ((list_size - i) as f32) * 0.1).collect();
                (scores, relevance)
            })
            .collect();
        
        let start = Instant::now();
        for (scores, relevance) in &batches {
            let _lambdas = trainer.compute_gradients(scores, relevance, None).unwrap();
        }
        let elapsed = start.elapsed();
        
        // Should process 100 batches in reasonable time (< 5 seconds)
        assert!(elapsed.as_secs_f64() < 5.0, "Batch processing took too long: {:?}", elapsed);
    }

    #[test]
    fn test_ndcg_repeated_computation() {
        // Test that repeated NDCG computations are consistent
        let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];
        let mut times = Vec::new();
        
        // Run multiple times
        for _ in 0..100 {
            let start = Instant::now();
            let _ndcg = ndcg_at_k(&relevance, None, true).unwrap();
            times.push(start.elapsed());
        }
        
        // All computations should complete
        assert_eq!(times.len(), 100);
        
        // Performance should be consistent
        let avg_time: f64 = times.iter().map(|t| t.as_nanos() as f64).sum::<f64>() / times.len() as f64;
        let max_time = times.iter().map(|t| t.as_nanos()).max().unwrap();
        
        // Max should not be more than 20x average (allowing for system variance)
        // NDCG computation is very fast, so variance can be high relative to average
        assert!((max_time as f64) < avg_time * 20.0 || avg_time < 0.001, 
            "Performance inconsistency detected: max={}ns, avg={:.2}ns", max_time, avg_time);
    }

    #[test]
    fn test_lambdarank_with_different_list_sizes() {
        // Test LambdaRank performance scales reasonably
        let trainer = LambdaRankTrainer::default();
        
        for n in [10, 50, 100, 500] {
            let scores: Vec<f32> = (0..n).map(|i| (i as f32) * 0.1).collect();
            let relevance: Vec<f32> = (0..n).map(|i| ((n - i) as f32) * 0.1).collect();
            
            let start = Instant::now();
            let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();
            let elapsed = start.elapsed();
            
            // Verify results
            assert_eq!(lambdas.len(), n);
            assert!(lambdas.iter().all(|&l| l.is_finite()));
            
            // Performance should scale roughly as O(n²)
            // For n=500, should complete in < 10 seconds (allowing for system variance)
            if n == 500 {
                assert!(elapsed.as_secs_f64() < 10.0, "LambdaRank for n={} took too long: {:?}", n, elapsed);
            }
        }
    }
}

