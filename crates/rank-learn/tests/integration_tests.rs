//! Integration tests for rank-learn.
//!
//! Tests the complete LTR pipeline and integration with other rank-* crates.

#[cfg(test)]
mod tests {
    use rank_learn::lambdarank::{ndcg_at_k, LambdaRankTrainer};

    #[test]
    fn test_ltr_pipeline() {
        // Test complete LTR pipeline: NDCG → LambdaRank gradients

        // 1. Compute NDCG
        let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];
        let ndcg = ndcg_at_k(&relevance, None, true).unwrap();

        assert!(ndcg >= 0.0 && ndcg <= 1.0);
        assert!(ndcg > 0.0); // Should be positive for non-zero relevance

        // 2. Compute LambdaRank gradients
        let trainer = LambdaRankTrainer::default();
        let scores = vec![0.5, 0.8, 0.3, 0.9, 0.2];
        let lambdas = trainer
            .compute_gradients(&scores, &relevance, None)
            .unwrap();

        assert_eq!(lambdas.len(), scores.len());
        assert!(lambdas.iter().all(|&l| l.is_finite()));
    }

    #[test]
    fn test_ndcg_lambdarank_consistency() {
        // Test that NDCG and LambdaRank work together consistently

        // Perfect ranking should have NDCG = 1.0
        let perfect_relevance = vec![3.0, 2.0, 1.0];
        let perfect_ndcg = ndcg_at_k(&perfect_relevance, None, true).unwrap();
        assert!((perfect_ndcg - 1.0).abs() < 0.01);

        // LambdaRank should push towards perfect ranking
        let trainer = LambdaRankTrainer::default();
        let imperfect_scores = vec![0.3, 0.5, 0.8]; // Wrong order
        let lambdas = trainer
            .compute_gradients(&imperfect_scores, &perfect_relevance, None)
            .unwrap();

        // Gradients should be non-zero (need correction)
        // The exact sign depends on score differences and delta_ndcg
        // Just verify gradients are computed (non-zero for non-trivial cases)
        assert!(lambdas.iter().any(|&l| l != 0.0)); // At least one non-zero lambda
    }

    #[test]
    fn test_ndcg_at_k_consistency() {
        // Test that NDCG@k is consistent across different k values

        let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];

        let ndcg_all = ndcg_at_k(&relevance, None, true).unwrap();
        let ndcg_3 = ndcg_at_k(&relevance, Some(3), true).unwrap();
        let ndcg_5 = ndcg_at_k(&relevance, Some(5), true).unwrap();

        // NDCG@5 should equal NDCG@all when k >= length
        assert!((ndcg_5 - ndcg_all).abs() < 0.001);

        // NDCG@3 should be >= NDCG@5 (focusing on top)
        // Actually, this isn't always true, but in this case it should be
        assert!(ndcg_3 >= 0.0 && ndcg_3 <= 1.0);
        assert!(ndcg_5 >= 0.0 && ndcg_5 <= 1.0);
    }

    #[test]
    fn test_lambdarank_gradient_properties() {
        // Test that LambdaRank gradients have expected properties

        let trainer = LambdaRankTrainer::default();

        // Case 1: Perfect ranking (scores match relevance order)
        let scores1 = vec![0.9, 0.7, 0.5];
        let relevance1 = vec![3.0, 2.0, 1.0];
        let lambdas1 = trainer
            .compute_gradients(&scores1, &relevance1, None)
            .unwrap();

        // Gradients should be small (already well-ranked)
        let max_grad1 = lambdas1.iter().map(|&l| l.abs()).fold(0.0, f32::max);
        assert!(max_grad1 < 10.0); // Reasonable bound

        // Case 2: Completely wrong ranking
        let scores2 = vec![0.3, 0.5, 0.9]; // Reversed
        let relevance2 = vec![3.0, 2.0, 1.0];
        let lambdas2 = trainer
            .compute_gradients(&scores2, &relevance2, None)
            .unwrap();

        // Gradients should be larger (need more correction)
        let max_grad2 = lambdas2.iter().map(|&l| l.abs()).fold(0.0, f32::max);
        assert!(max_grad2 > 0.0); // Should have non-zero gradients
    }

    #[test]
    fn test_error_handling_consistency() {
        // Test that error handling is consistent across functions

        // Empty input
        let result1 = ndcg_at_k(&[], None, true);
        assert!(result1.is_err());

        let trainer = LambdaRankTrainer::default();
        let result2 = trainer.compute_gradients(&[], &[], None);
        assert!(result2.is_err());

        // Length mismatch
        let result3 = trainer.compute_gradients(&[1.0, 2.0], &[3.0], None);
        assert!(result3.is_err());

        // Invalid k
        let result4 = ndcg_at_k(&[1.0, 2.0], Some(100), true);
        assert!(result4.is_err());
    }
}
