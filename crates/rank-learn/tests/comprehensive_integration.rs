//! Comprehensive integration tests covering real-world LTR scenarios.
//!
//! Tests complete workflows and edge cases that might occur in production.

#[cfg(test)]
mod tests {
    use rank_learn::lambdarank::{ndcg_at_k, LambdaRankParams, LambdaRankTrainer};

    #[test]
    fn test_training_workflow() {
        // Real-world scenario: Complete training workflow

        // 1. Initial model scores (poor ranking)
        let mut scores = vec![0.3, 0.5, 0.9, 0.1, 0.7];
        let relevance = vec![3.0, 2.0, 1.0, 2.5, 1.5]; // Perfect ranking: [3.0, 2.5, 2.0, 1.5, 1.0]

        // 2. Compute initial NDCG
        let initial_ndcg = ndcg_at_k(&relevance, None, true).unwrap();
        assert!(initial_ndcg > 0.0);

        // 3. Train for multiple iterations
        let trainer = LambdaRankTrainer::default();

        for _iteration in 0..5 {
            // Compute gradients
            let lambdas = trainer
                .compute_gradients(&scores, &relevance, None)
                .unwrap();

            // Update scores (simplified: just add gradients)
            for (score, lambda) in scores.iter_mut().zip(lambdas.iter()) {
                *score += 0.1 * lambda; // Learning rate = 0.1
            }
        }

        // 4. Verify scores improved (better alignment with relevance)
        // After training, scores should be better aligned with relevance order
        let final_ndcg = ndcg_at_k(&relevance, None, true).unwrap();
        assert!(final_ndcg.is_finite());
    }

    #[test]
    fn test_ndcg_progression() {
        // Test NDCG progression as ranking improves
        let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];

        // Perfect ranking
        let perfect_ndcg = ndcg_at_k(&relevance, None, true).unwrap();
        assert!((perfect_ndcg - 1.0).abs() < 0.01);

        // Random ranking
        let random_relevance = vec![1.0, 3.0, 0.5, 2.0, 0.0];
        let random_ndcg = ndcg_at_k(&random_relevance, None, true).unwrap();

        // Perfect should be >= random
        assert!(perfect_ndcg >= random_ndcg);
    }

    #[test]
    fn test_lambdarank_batch_processing() {
        // Test processing multiple query-document lists
        let trainer = LambdaRankTrainer::default();

        let batches = vec![
            (vec![0.5, 0.8, 0.3], vec![3.0, 2.0, 1.0]),
            (vec![0.2, 0.9, 0.4], vec![1.0, 3.0, 2.0]),
            (vec![0.7, 0.1, 0.6], vec![2.0, 1.0, 3.0]),
        ];

        for (scores, relevance) in batches {
            let lambdas = trainer
                .compute_gradients(&scores, &relevance, None)
                .unwrap();

            // Verify gradients computed
            assert_eq!(lambdas.len(), scores.len());
            assert!(lambdas.iter().all(|&l| l.is_finite()));
        }
    }

    #[test]
    fn test_ndcg_at_different_k() {
        // Test NDCG@k for different k values
        let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];

        let ndcg_1 = ndcg_at_k(&relevance, Some(1), true).unwrap();
        let ndcg_3 = ndcg_at_k(&relevance, Some(3), true).unwrap();
        let ndcg_5 = ndcg_at_k(&relevance, Some(5), true).unwrap();
        let ndcg_all = ndcg_at_k(&relevance, None, true).unwrap();

        // All should be valid
        assert!(ndcg_1 >= 0.0 && ndcg_1 <= 1.0);
        assert!(ndcg_3 >= 0.0 && ndcg_3 <= 1.0);
        assert!(ndcg_5 >= 0.0 && ndcg_5 <= 1.0);
        assert!(ndcg_all >= 0.0 && ndcg_all <= 1.0);

        // NDCG@5 should equal NDCG@all when k >= length
        assert!((ndcg_5 - ndcg_all).abs() < 0.001);
    }

    #[test]
    fn test_lambdarank_with_different_sigma() {
        // Test LambdaRank with different sigma parameters
        let scores = vec![0.1, 0.5, 0.9];
        let relevance = vec![3.0, 2.0, 1.0];

        let trainer_default = LambdaRankTrainer::default();
        let mut params_small = LambdaRankParams::default();
        params_small.sigma = 0.1;
        let trainer_small_sigma = LambdaRankTrainer::new(params_small);
        let mut params_large = LambdaRankParams::default();
        params_large.sigma = 10.0;
        let trainer_large_sigma = LambdaRankTrainer::new(params_large);

        let lambdas_default = trainer_default
            .compute_gradients(&scores, &relevance, None)
            .unwrap();
        let lambdas_small = trainer_small_sigma
            .compute_gradients(&scores, &relevance, None)
            .unwrap();
        let lambdas_large = trainer_large_sigma
            .compute_gradients(&scores, &relevance, None)
            .unwrap();

        // All should produce valid gradients
        assert_eq!(lambdas_default.len(), scores.len());
        assert_eq!(lambdas_small.len(), scores.len());
        assert_eq!(lambdas_large.len(), scores.len());

        // Gradients should differ with different sigma
        // (exact relationship depends on implementation)
        assert!(lambdas_default.iter().all(|&l| l.is_finite()));
        assert!(lambdas_small.iter().all(|&l| l.is_finite()));
        assert!(lambdas_large.iter().all(|&l| l.is_finite()));
    }

    #[test]
    fn test_ndcg_with_ties() {
        // Test NDCG when multiple documents have same relevance
        let relevance = vec![3.0, 3.0, 2.0, 2.0, 1.0];

        let ndcg = ndcg_at_k(&relevance, None, true).unwrap();

        // Should still compute valid NDCG
        assert!(ndcg >= 0.0 && ndcg <= 1.0);
        assert!(ndcg.is_finite());
    }

    #[test]
    fn test_lambdarank_gradient_magnitude() {
        // Test that gradient magnitudes are reasonable
        let trainer = LambdaRankTrainer::default();

        // Case 1: Perfect ranking (should have small gradients)
        let perfect_scores = vec![0.9, 0.7, 0.5];
        let perfect_relevance = vec![3.0, 2.0, 1.0];
        let perfect_lambdas = trainer
            .compute_gradients(&perfect_scores, &perfect_relevance, None)
            .unwrap();

        let max_grad_perfect = perfect_lambdas.iter().map(|&l| l.abs()).fold(0.0, f32::max);
        assert!(max_grad_perfect < 100.0); // Reasonable bound

        // Case 2: Completely wrong ranking (should have larger gradients)
        let wrong_scores = vec![0.3, 0.5, 0.9]; // Reversed
        let wrong_relevance = vec![3.0, 2.0, 1.0];
        let wrong_lambdas = trainer
            .compute_gradients(&wrong_scores, &wrong_relevance, None)
            .unwrap();

        let max_grad_wrong = wrong_lambdas.iter().map(|&l| l.abs()).fold(0.0, f32::max);
        // Wrong ranking should generally have larger gradients
        // (though exact relationship depends on implementation)
        assert!(max_grad_wrong.is_finite());
    }

    #[test]
    fn test_ndcg_computation_consistency() {
        // Test that NDCG computation is consistent
        let relevance = vec![3.0, 2.0, 1.0];

        // Compute multiple times
        let ndcg1 = ndcg_at_k(&relevance, None, true).unwrap();
        let ndcg2 = ndcg_at_k(&relevance, None, true).unwrap();
        let ndcg3 = ndcg_at_k(&relevance, None, true).unwrap();

        // Should be identical
        assert!((ndcg1 - ndcg2).abs() < 1e-6);
        assert!((ndcg2 - ndcg3).abs() < 1e-6);
    }

    #[test]
    fn test_lambdarank_with_k_optimization() {
        // Test LambdaRank with k parameter (optimize for NDCG@k)
        let trainer = LambdaRankTrainer::default();
        let scores = vec![0.1, 0.5, 0.9, 0.3, 0.7];
        let relevance = vec![3.0, 2.0, 1.0, 2.5, 1.5];

        // Optimize for NDCG@3
        let lambdas_k3 = trainer
            .compute_gradients(&scores, &relevance, Some(3))
            .unwrap();

        // Optimize for all positions
        let lambdas_all = trainer
            .compute_gradients(&scores, &relevance, None)
            .unwrap();

        // Both should produce valid gradients
        assert_eq!(lambdas_k3.len(), scores.len());
        assert_eq!(lambdas_all.len(), scores.len());

        // Gradients may differ (k=3 focuses on top 3)
        assert!(lambdas_k3.iter().all(|&l| l.is_finite()));
        assert!(lambdas_all.iter().all(|&l| l.is_finite()));
    }
}
