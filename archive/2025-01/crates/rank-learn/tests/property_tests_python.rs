//! Property-based tests for rank-learn Python bindings.
//!
//! These tests verify invariants that should always hold for LTR operations
//! when accessed through Python bindings.

use proptest::prelude::*;
use rank_learn::lambdarank::{ndcg_at_k, LambdaRankTrainer};

proptest! {
    #[test]
    fn ndcg_python_bounds(
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
    ) {
        let ndcg = ndcg_at_k(&relevance, None, true).unwrap();
        prop_assert!(
            ndcg >= 0.0 && ndcg <= 1.0,
            "NDCG must be in [0, 1] for Python bindings"
        );
    }

    #[test]
    fn lambdarank_gradients_python_length(
        scores in prop::collection::vec(-10.0f32..10.0, 1..100),
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
    ) {
        if scores.len() == relevance.len() && !scores.is_empty() {
            let trainer = LambdaRankTrainer::default();
            let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();

            prop_assert_eq!(
                lambdas.len(),
                scores.len(),
                "Lambdas must match scores length for Python bindings"
            );
        }
    }

    #[test]
    fn lambdarank_gradients_python_finite(
        scores in prop::collection::vec(-10.0f32..10.0, 1..50),
        relevance in prop::collection::vec(0.0f32..10.0, 1..50),
    ) {
        if scores.len() == relevance.len() && !scores.is_empty() {
            let trainer = LambdaRankTrainer::default();
            let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();

            for lambda in lambdas {
                prop_assert!(
                    lambda.is_finite(),
                    "All lambdas must be finite for Python bindings"
                );
            }
        }
    }

    #[test]
    fn ndcg_at_k_python_bounded(
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
        k in 1usize..100,
    ) {
        if k <= relevance.len() {
            let ndcg = ndcg_at_k(&relevance, Some(k), true).unwrap();
            prop_assert!(
                ndcg >= 0.0 && ndcg <= 1.0,
                "NDCG@k must be in [0, 1] for Python bindings"
            );
        }
    }

    #[test]
    fn lambdarank_gradients_python_with_k(
        scores in prop::collection::vec(-10.0f32..10.0, 1..100),
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
        k in 1usize..50,
    ) {
        if scores.len() == relevance.len() && !scores.is_empty() && k <= scores.len() {
            let trainer = LambdaRankTrainer::default();
            let lambdas = trainer.compute_gradients(&scores, &relevance, Some(k)).unwrap();

            prop_assert_eq!(
                lambdas.len(),
                scores.len(),
                "Lambdas must match scores length even with k parameter"
            );
        }
    }
}
