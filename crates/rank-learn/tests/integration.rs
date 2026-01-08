//! Integration tests for rank-learn.

use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};
use rank_learn::LearnError;

#[test]
fn lambdarank_training_workflow() {
    let trainer = LambdaRankTrainer::default();
    
    let scores = vec![0.3, 0.8, 0.5, 0.2];
    let relevance = vec![1.0, 3.0, 2.0, 0.0];
    
    let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();
    
    assert_eq!(lambdas.len(), 4);
    assert!(lambdas.iter().all(|&l| l.is_finite()));
}

#[test]
fn ndcg_computation_workflow() {
    let relevance = vec![3.0, 2.0, 1.0, 0.0];
    
    let ndcg_all = ndcg_at_k(&relevance, None, true).unwrap();
    let ndcg_at_2 = ndcg_at_k(&relevance, Some(2), true).unwrap();
    
    assert!(ndcg_all >= 0.0 && ndcg_all <= 1.0);
    assert!(ndcg_at_2 >= 0.0 && ndcg_at_2 <= 1.0);
    assert!(ndcg_all >= ndcg_at_2);
}

#[test]
fn error_handling_empty_input() {
    let trainer = LambdaRankTrainer::default();
    let result = trainer.compute_gradients(&[], &[], None);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LearnError::EmptyInput));
}

#[test]
fn error_handling_length_mismatch() {
    let trainer = LambdaRankTrainer::default();
    let result = trainer.compute_gradients(&[0.5, 0.8], &[1.0, 2.0, 3.0], None);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LearnError::LengthMismatch { .. }));
}

#[test]
fn error_handling_invalid_ndcg_k() {
    let relevance = vec![3.0, 2.0, 1.0];
    let result = ndcg_at_k(&relevance, Some(100), true);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), LearnError::InvalidNDCG { k: 100, length: 3 }));
}

