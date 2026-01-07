//! Edge case tests for rank-learn.

use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};

#[test]
fn ndcg_empty_relevance() {
    let relevance = vec![];
    let ndcg = ndcg_at_k(&relevance, None);
    assert_eq!(ndcg, 0.0);
}

#[test]
fn ndcg_all_zeros() {
    let relevance = vec![0.0, 0.0, 0.0];
    let ndcg = ndcg_at_k(&relevance, None);
    assert_eq!(ndcg, 0.0);
}

#[test]
fn ndcg_single_element() {
    let relevance = vec![1.0];
    let ndcg = ndcg_at_k(&relevance, None);
    assert!((ndcg - 1.0).abs() < 0.01);
}

#[test]
fn ndcg_at_k_zero() {
    let relevance = vec![3.0, 2.0, 1.0];
    let ndcg = ndcg_at_k(&relevance, Some(0));
    assert_eq!(ndcg, 0.0);
}

#[test]
fn ndcg_at_k_larger_than_length() {
    let relevance = vec![3.0, 2.0, 1.0];
    let ndcg = ndcg_at_k(&relevance, Some(100));
    let ndcg_all = ndcg_at_k(&relevance, None);
    assert!((ndcg - ndcg_all).abs() < 0.01);
}

#[test]
fn lambdarank_empty_scores() {
    let trainer = LambdaRankTrainer::default();
    let scores = vec![];
    let relevance = vec![];
    let lambdas = trainer.compute_gradients(&scores, &relevance, None);
    assert_eq!(lambdas.len(), 0);
}

#[test]
fn lambdarank_mismatched_lengths() {
    let trainer = LambdaRankTrainer::default();
    let scores = vec![0.5, 0.8];
    let relevance = vec![1.0, 2.0, 3.0];
    let lambdas = trainer.compute_gradients(&scores, &relevance, None);
    assert_eq!(lambdas.len(), scores.len());
    assert!(lambdas.iter().all(|&l| l == 0.0));
}

#[test]
fn lambdarank_all_same_relevance() {
    let trainer = LambdaRankTrainer::default();
    let scores = vec![0.5, 0.8, 0.3];
    let relevance = vec![1.0, 1.0, 1.0];
    let lambdas = trainer.compute_gradients(&scores, &relevance, None);
    assert_eq!(lambdas.len(), 3);
    assert!(lambdas.iter().all(|&l| l == 0.0));
}

#[test]
fn lambdarank_all_same_scores() {
    let trainer = LambdaRankTrainer::default();
    let scores = vec![0.5, 0.5, 0.5];
    let relevance = vec![3.0, 2.0, 1.0];
    let lambdas = trainer.compute_gradients(&scores, &relevance, None);
    assert_eq!(lambdas.len(), 3);
    assert!(lambdas.iter().any(|&l| l != 0.0));
}

#[test]
fn lambdarank_single_element() {
    let trainer = LambdaRankTrainer::default();
    let scores = vec![0.5];
    let relevance = vec![1.0];
    let lambdas = trainer.compute_gradients(&scores, &relevance, None);
    assert_eq!(lambdas.len(), 1);
    assert_eq!(lambdas[0], 0.0);
}

