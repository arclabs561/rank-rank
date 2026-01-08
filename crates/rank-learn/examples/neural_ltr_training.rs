//! Neural LTR training example.
//!
//! Demonstrates how to use rank-soft with neural LTR models.

use rank_learn::neural::*;

fn main() {
    println!("=== Neural LTR Training Example ===\n");
    
    // Create neural LTR model
    let config = NeuralLTRConfig::default();
    let model = NeuralLTRModel::new(config);
    
    // Example: Model predictions and ground truth
    let predictions = vec![0.1, 0.9, 0.3, 0.7, 0.5];
    let targets = vec![0.0, 1.0, 0.2, 0.8, 0.4];
    
    println!("Model predictions: {:?}", predictions);
    println!("Ground truth: {:?}\n", targets);
    
    // Compute soft ranks using rank-soft
    let pred_ranks = model.soft_rank_scores(&predictions);
    println!("Soft ranks (predictions): {:?}", pred_ranks);
    
    // Compute loss using Spearman correlation
    let loss = model.compute_loss(&predictions, &targets);
    println!("Spearman loss: {:.6}", loss);
    println!("(Lower is better, 0.0 = perfect correlation)\n");
    
    println!("=== Training Loop (Pseudocode) ===");
    println!("for epoch in 0..num_epochs {{");
    println!("    for batch in batches {{");
    println!("        predictions = model.forward(query, documents);");
    println!("        loss = model.compute_loss(&predictions, &targets);");
    println!("        loss.backward();  // Gradients flow through rank-soft");
    println!("        optimizer.step();");
    println!("    }}");
    println!("}}");
    
    println!("\nKey advantage: Gradients flow through soft ranking operations!");
    println!("This enables end-to-end training of ranking models.");
}

