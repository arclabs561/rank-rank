//! Property-based tests for rank-learn.

use proptest::prelude::*;
use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};

proptest! {
    #[test]
    fn ndcg_is_bounded(
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
    ) {
        let ndcg = ndcg_at_k(&relevance, None);
        prop_assert!(
            ndcg >= 0.0 && ndcg <= 1.0,
            "NDCG must be in [0, 1]"
        );
    }

    #[test]
    fn ndcg_perfect_ranking_is_one(
        mut relevance in prop::collection::vec(0.0f32..10.0, 1..100),
    ) {
        relevance.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let ndcg = ndcg_at_k(&relevance, None);
        prop_assert!(
            (ndcg - 1.0).abs() < 0.01,
            "Perfect ranking should have NDCG = 1.0"
        );
    }

    #[test]
    fn ndcg_at_k_is_bounded(
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
        k in 1usize..100,
    ) {
        let ndcg = ndcg_at_k(&relevance, Some(k));
        prop_assert!(
            ndcg >= 0.0 && ndcg <= 1.0,
            "NDCG@k must be in [0, 1]"
        );
    }

    #[test]
    fn lambdarank_gradients_match_scores_length(
        scores in prop::collection::vec(-10.0f32..10.0, 1..100),
        relevance in prop::collection::vec(0.0f32..10.0, 1..100),
    ) {
        if scores.len() == relevance.len() {
            let trainer = LambdaRankTrainer::default();
            let lambdas = trainer.compute_gradients(&scores, &relevance, None);
            
            prop_assert_eq!(
                lambdas.len(),
                scores.len(),
                "Lambdas must match scores length"
            );
        }
    }

    #[test]
    fn lambdarank_gradients_are_finite(
        scores in prop::collection::vec(-10.0f32..10.0, 1..50),
        relevance in prop::collection::vec(0.0f32..10.0, 1..50),
    ) {
        if scores.len() == relevance.len() {
            let trainer = LambdaRankTrainer::default();
            let lambdas = trainer.compute_gradients(&scores, &relevance, None);
            
            for lambda in lambdas {
                prop_assert!(
                    lambda.is_finite(),
                    "All lambdas must be finite"
                );
            }
        }
    }
}

