//! Comprehensive error handling tests for rank-learn.
//!
//! Tests all error conditions and edge cases.

#[cfg(test)]
mod tests {
    use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k, LambdaRankParams};
    use rank_learn::LearnError;

    #[test]
    fn test_ndcg_empty_input_error() {
        let result = ndcg_at_k(&[], None, true);
        assert!(result.is_err());
        match result {
            Err(LearnError::EmptyInput) => {}
            _ => panic!("Expected EmptyInput error"),
        }
    }

    #[test]
    fn test_ndcg_invalid_k_error() {
        let relevance = vec![3.0, 2.0, 1.0];
        
        // k > length
        let result = ndcg_at_k(&relevance, Some(100), true);
        assert!(result.is_err());
        match result {
            Err(LearnError::InvalidNDCG { k, length }) => {
                assert_eq!(k, 100);
                assert_eq!(length, 3);
            }
            _ => panic!("Expected InvalidNDCG error"),
        }
    }

    #[test]
    fn test_ndcg_k_zero() {
        let relevance = vec![3.0, 2.0, 1.0];
        let result = ndcg_at_k(&relevance, Some(0), true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_lambdarank_empty_scores_error() {
        let trainer = LambdaRankTrainer::default();
        let result = trainer.compute_gradients(&[], &[1.0, 2.0], None);
        assert!(result.is_err());
        match result {
            Err(LearnError::EmptyInput) => {}
            _ => panic!("Expected EmptyInput error"),
        }
    }

    #[test]
    fn test_lambdarank_empty_relevance_error() {
        let trainer = LambdaRankTrainer::default();
        let result = trainer.compute_gradients(&[1.0, 2.0], &[], None);
        assert!(result.is_err());
        match result {
            Err(LearnError::EmptyInput) => {}
            _ => panic!("Expected EmptyInput error"),
        }
    }

    #[test]
    fn test_lambdarank_length_mismatch_error() {
        let trainer = LambdaRankTrainer::default();
        let result = trainer.compute_gradients(&[1.0, 2.0], &[3.0], None);
        assert!(result.is_err());
        match result {
            Err(LearnError::LengthMismatch { scores_len, relevance_len }) => {
                assert_eq!(scores_len, 2);
                assert_eq!(relevance_len, 1);
            }
            _ => panic!("Expected LengthMismatch error"),
        }
    }

    #[test]
    fn test_lambdarank_single_document() {
        let trainer = LambdaRankTrainer::default();
        let result = trainer.compute_gradients(&[0.5], &[3.0], None);
        assert!(result.is_ok());
        let lambdas = result.unwrap();
        assert_eq!(lambdas.len(), 1);
        assert!(lambdas[0].is_finite());
    }

    #[test]
    fn test_lambdarank_all_same_relevance() {
        let trainer = LambdaRankTrainer::default();
        // All documents have same relevance - should produce zero gradients
        let scores = vec![0.1, 0.5, 0.9];
        let relevance = vec![2.0, 2.0, 2.0];
        let result = trainer.compute_gradients(&scores, &relevance, None);
        assert!(result.is_ok());
        let lambdas = result.unwrap();
        // Gradients should be zero (no preference)
        assert!(lambdas.iter().all(|&l| l.abs() < 1e-6));
    }

    #[test]
    fn test_lambdarank_all_same_scores() {
        let trainer = LambdaRankTrainer::default();
        // All documents have same scores but different relevance
        let scores = vec![0.5, 0.5, 0.5];
        let relevance = vec![3.0, 2.0, 1.0];
        let result = trainer.compute_gradients(&scores, &relevance, None);
        assert!(result.is_ok());
        let lambdas = result.unwrap();
        // Should have non-zero gradients (need to differentiate)
        assert!(lambdas.iter().any(|&l| l.abs() > 1e-6));
    }

    #[test]
    fn test_lambdarank_extreme_sigma() {
        // Test with extreme sigma parameter
        let mut params = LambdaRankParams::default();
        params.sigma = 100.0;
        let trainer = LambdaRankTrainer::new(params);
        let scores = vec![0.1, 0.5, 0.9];
        let relevance = vec![3.0, 2.0, 1.0];
        let result = trainer.compute_gradients(&scores, &relevance, None);
        assert!(result.is_ok());
        let lambdas = result.unwrap();
        assert_eq!(lambdas.len(), scores.len());
        assert!(lambdas.iter().all(|&l| l.is_finite()));
    }

    #[test]
    fn test_ndcg_all_zero_relevance() {
        let relevance = vec![0.0, 0.0, 0.0];
        let result = ndcg_at_k(&relevance, None, true);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0.0);
    }

    #[test]
    fn test_ndcg_negative_relevance() {
        // NDCG should handle negative relevance (though unusual)
        let relevance = vec![-1.0, 0.0, 1.0];
        let result = ndcg_at_k(&relevance, None, true);
        assert!(result.is_ok());
        // Should compute (may be negative or zero depending on implementation)
        let ndcg = result.unwrap();
        assert!(ndcg.is_finite());
    }

    #[test]
    fn test_lambdarank_with_k_parameter() {
        let trainer = LambdaRankTrainer::default();
        let scores = vec![0.1, 0.5, 0.9, 0.3, 0.7];
        let relevance = vec![3.0, 2.0, 1.0, 2.5, 1.5];
        
        // Test with k=3 (only optimize top 3)
        let result = trainer.compute_gradients(&scores, &relevance, Some(3));
        assert!(result.is_ok());
        let lambdas = result.unwrap();
        assert_eq!(lambdas.len(), scores.len());
        assert!(lambdas.iter().all(|&l| l.is_finite()));
    }

    #[test]
    fn test_lambdarank_k_larger_than_length() {
        let trainer = LambdaRankTrainer::default();
        let scores = vec![0.1, 0.5, 0.9];
        let relevance = vec![3.0, 2.0, 1.0];
        
        // k > length should still work (treats as k=None)
        let result = trainer.compute_gradients(&scores, &relevance, Some(100));
        assert!(result.is_ok());
        let lambdas = result.unwrap();
        assert_eq!(lambdas.len(), scores.len());
    }
}

