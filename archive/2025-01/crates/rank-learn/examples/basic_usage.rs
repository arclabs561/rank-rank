//! Basic usage examples for rank-learn.
//!
//! Demonstrates LambdaRank training and NDCG calculation.

use rank_learn::lambdarank::{ndcg_at_k, LambdaRankParams, LambdaRankTrainer};

fn main() {
    println!("=== Basic rank-learn Examples ===\n");

    // Example 1: NDCG calculation
    println!("1. NDCG Calculation:");
    let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];
    let ndcg = ndcg_at_k(&relevance, None, true).unwrap();
    println!("   NDCG@all: {:.4}", ndcg);

    // NDCG@10 when list has 5 items - use k=5 or less
    let ndcg_5 = ndcg_at_k(&relevance, Some(5), true).unwrap();
    println!("   NDCG@5: {:.4}", ndcg_5);

    let ndcg_3 = ndcg_at_k(&relevance, Some(3), true).unwrap();
    println!("   NDCG@3: {:.4}", ndcg_3);

    // Example 2: LambdaRank gradient computation
    println!("\n2. LambdaRank Gradient Computation:");
    let trainer = LambdaRankTrainer::default();
    let scores = vec![0.5, 0.8, 0.3, 0.9, 0.2];
    let relevance = vec![3.0, 1.0, 2.0, 3.0, 1.0];

    let lambdas = trainer
        .compute_gradients(&scores, &relevance, None)
        .unwrap();
    println!("   Gradients computed:");
    for (i, lambda) in lambdas.iter().enumerate() {
        println!("      Document {}: {:.4}", i, lambda);
    }

    // Example 3: Custom parameters
    println!("\n3. Custom LambdaRank Parameters:");
    let mut params = LambdaRankParams::default();
    params.sigma = 2.0;
    let trainer_custom = LambdaRankTrainer::new(params);
    let lambdas_custom = trainer_custom
        .compute_gradients(&scores, &relevance, None)
        .unwrap();
    println!("   Gradients with sigma=2.0:");
    for (i, lambda) in lambdas_custom.iter().enumerate() {
        println!("      Document {}: {:.4}", i, lambda);
    }

    // Example 4: Optimizing for NDCG@k
    println!("\n4. Optimizing for NDCG@k:");
    let trainer = LambdaRankTrainer::default();

    // Optimize for top 3 positions
    let lambdas_k3 = trainer
        .compute_gradients(&scores, &relevance, Some(3))
        .unwrap();
    println!("   Gradients (NDCG@3):");
    for (i, lambda) in lambdas_k3.iter().enumerate() {
        println!("      Document {}: {:.4}", i, lambda);
    }

    // Optimize for all positions
    let lambdas_all = trainer
        .compute_gradients(&scores, &relevance, None)
        .unwrap();
    println!("   Gradients (NDCG@all):");
    for (i, lambda) in lambdas_all.iter().enumerate() {
        println!("      Document {}: {:.4}", i, lambda);
    }

    // Example 5: Training loop simulation
    println!("\n5. Training Loop Simulation:");
    let trainer = LambdaRankTrainer::default();
    let mut scores = vec![0.1, 0.5, 0.9, 0.3, 0.7];
    let relevance = vec![3.0, 2.0, 1.0, 2.5, 1.5];
    let learning_rate = 0.1;

    println!("   Initial scores: {:?}", scores);
    for epoch in 0..3 {
        let lambdas = trainer
            .compute_gradients(&scores, &relevance, None)
            .unwrap();

        // Update scores
        for (score, lambda) in scores.iter_mut().zip(lambdas.iter()) {
            *score += learning_rate * lambda;
        }

        let ndcg = ndcg_at_k(&relevance, None, true).unwrap();
        println!(
            "   Epoch {}: NDCG = {:.4}, scores = {:?}",
            epoch + 1,
            ndcg,
            scores
        );
    }

    println!("\n=== Basic Examples Complete ===");
}
