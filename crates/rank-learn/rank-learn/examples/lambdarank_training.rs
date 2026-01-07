//! LambdaRank training example.
//!
//! Demonstrates how to use LambdaRank for training ranking models.

use rank_learn::prelude::*;

fn main() {
    println!("=== LambdaRank Training Example ===\n");
    
    // Create trainer
    let trainer = LambdaRankTrainer::default();
    
    // Example: Documents with model scores and ground truth relevance
    // In practice, these would come from your ranking model and labeled data
    
    // Model scores (what the model currently predicts)
    let scores = vec![0.5, 0.8, 0.3, 0.9, 0.2];
    
    // Ground truth relevance (higher = more relevant)
    let relevance = vec![3.0, 1.0, 2.0, 3.0, 1.0];
    
    println!("Model scores: {:?}", scores);
    println!("Ground truth relevance: {:?}\n", relevance);
    
    // Compute LambdaRank gradients
    let lambdas = trainer.compute_gradients(&scores, &relevance, None);
    
    println!("LambdaRank gradients (lambdas):");
    for (i, lambda) in lambdas.iter().enumerate() {
        println!("  Document {}: {:.6}", i, lambda);
    }
    
    println!("\nInterpretation:");
    println!("- Positive lambda: document should rank higher");
    println!("- Negative lambda: document should rank lower");
    println!("- Magnitude: strength of the gradient signal");
    
    // Compute NDCG to see current quality
    let ndcg = ndcg_at_k(&relevance, None);
    println!("\nCurrent NDCG: {:.4}", ndcg);
    
    // In a real training loop, you would:
    // 1. Compute lambdas (done above)
    // 2. Use lambdas as gradients to update model
    // 3. Re-compute scores with updated model
    // 4. Repeat until convergence
    
    println!("\n=== Training Loop (Pseudocode) ===");
    println!("for epoch in 0..num_epochs {{");
    println!("    for query in queries {{");
    println!("        scores = model.predict(query, documents);");
    println!("        lambdas = trainer.compute_gradients(&scores, &relevance, None);");
    println!("        model.update_weights(&lambdas);  // Gradient descent");
    println!("    }}");
    println!("}}");
}

