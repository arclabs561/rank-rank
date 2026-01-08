# rank-learn Examples

Complete examples demonstrating real-world LTR usage patterns.

## Basic LambdaRank Training

```rust
use rank_learn::lambdarank::{LambdaRankTrainer, LambdaRankParams};

// Create trainer
let trainer = LambdaRankTrainer::default();

// Model scores and ground truth relevance
let scores = vec![0.5, 0.8, 0.3, 0.9, 0.2];
let relevance = vec![3.0, 1.0, 2.0, 3.0, 1.0];

// Compute LambdaRank gradients
let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();

// Use lambdas as gradients to update your model
// In training loop:
//   model.update_weights(&lambdas);
```

## NDCG Calculation

```rust
use rank_learn::lambdarank::ndcg_at_k;

// Relevance scores (in ranked order)
let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];

// Compute NDCG@10
let ndcg_10 = ndcg_at_k(&relevance, Some(10)).unwrap();

// Compute NDCG@all
let ndcg_all = ndcg_at_k(&relevance, None).unwrap();

println!("NDCG@10: {}, NDCG@all: {}", ndcg_10, ndcg_all);
```

## Custom LambdaRank Parameters

```rust
use rank_learn::lambdarank::{LambdaRankTrainer, LambdaRankParams};

// Custom sigma parameter (controls gradient magnitude)
let params = LambdaRankParams { sigma: 2.0 };
let trainer = LambdaRankTrainer::new(params);

let scores = vec![0.5, 0.8, 0.3];
let relevance = vec![3.0, 2.0, 1.0];

let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();
```

## Training Loop Example

```rust
use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};

let trainer = LambdaRankTrainer::default();
let mut scores = vec![0.1, 0.5, 0.9, 0.3, 0.7];
let relevance = vec![3.0, 2.0, 1.0, 2.5, 1.5];

// Training loop
for epoch in 0..10 {
    // Compute gradients
    let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();
    
    // Update scores (simplified: just add gradients)
    let learning_rate = 0.1;
    for (score, lambda) in scores.iter_mut().zip(lambdas.iter()) {
        *score += learning_rate * lambda;
    }
    
    // Evaluate
    let ndcg = ndcg_at_k(&relevance, None).unwrap();
    println!("Epoch {}: NDCG = {}", epoch, ndcg);
}
```

## Optimizing for NDCG@k

```rust
use rank_learn::lambdarank::LambdaRankTrainer;

let trainer = LambdaRankTrainer::default();
let scores = vec![0.1, 0.5, 0.9, 0.3, 0.7];
let relevance = vec![3.0, 2.0, 1.0, 2.5, 1.5];

// Optimize for NDCG@3 (focus on top 3 positions)
let lambdas_k3 = trainer.compute_gradients(&scores, &relevance, Some(3)).unwrap();

// Optimize for all positions
let lambdas_all = trainer.compute_gradients(&scores, &relevance, None).unwrap();
```

## Python Usage

```python
import rank_learn

# LambdaRank training
trainer = rank_learn.LambdaRankTrainer()
scores = [0.5, 0.8, 0.3, 0.9, 0.2]
relevance = [3.0, 1.0, 2.0, 3.0, 1.0]

lambdas = trainer.compute_gradients(scores, relevance)
# Use lambdas to update your model

# NDCG calculation
ndcg = rank_learn.ndcg_at_k(relevance, k=10)
print(f"NDCG@10: {ndcg}")

# Custom parameters
params = rank_learn.LambdaRankParams(sigma=2.0)
trainer = rank_learn.LambdaRankTrainer(params=params)
```

## Integration with rank-retrieve

```rust
use rank_retrieve::prelude::*;
use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};

// 1. Retrieve candidates
let mut index = InvertedIndex::new();
// ... add documents ...
let candidates = index.retrieve(&query, 100, Bm25Params::default()).unwrap();

// 2. Score candidates with your model
let scores: Vec<f32> = candidates.iter()
    .map(|(doc_id, _)| model.score(doc_id, &query))
    .collect();

// 3. Get ground truth relevance
let relevance: Vec<f32> = candidates.iter()
    .map(|(doc_id, _)| get_relevance(*doc_id))
    .collect();

// 4. Compute gradients for training
let trainer = LambdaRankTrainer::default();
let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();

// 5. Update model
model.update_weights(&lambdas);
```

## Batch Processing

```rust
use rank_learn::lambdarank::LambdaRankTrainer;

let trainer = LambdaRankTrainer::default();

// Process multiple queries
let queries = vec![
    (vec![0.5, 0.8, 0.3], vec![3.0, 2.0, 1.0]),
    (vec![0.2, 0.9, 0.4], vec![1.0, 3.0, 2.0]),
    (vec![0.7, 0.1, 0.6], vec![2.0, 1.0, 3.0]),
];

for (scores, relevance) in queries {
    let lambdas = trainer.compute_gradients(&scores, &relevance, None).unwrap();
    // Update model with lambdas
}
```

## Error Handling

```rust
use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};
use rank_learn::LearnError;

// NDCG with error handling
match ndcg_at_k(&relevance, Some(100)) {
    Ok(ndcg) => println!("NDCG: {}", ndcg),
    Err(LearnError::EmptyInput) => println!("Empty input"),
    Err(LearnError::InvalidNDCG { k, length }) => {
        println!("Invalid k={} for length={}", k, length);
    }
    Err(e) => println!("Error: {}", e),
}

// LambdaRank with error handling
let trainer = LambdaRankTrainer::default();
match trainer.compute_gradients(&scores, &relevance, None) {
    Ok(lambdas) => {
        // Use lambdas
    }
    Err(LearnError::LengthMismatch { scores_len, relevance_len }) => {
        println!("Length mismatch: scores={}, relevance={}", scores_len, relevance_len);
    }
    Err(e) => println!("Error: {}", e),
}
```

## Integration with rank-eval

```rust
use rank_learn::lambdarank::ndcg_at_k;
use rank_eval::binary::ndcg_at_k as eval_ndcg;

// rank-learn NDCG (graded relevance)
let graded_relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];
let ndcg_learn = ndcg_at_k(&graded_relevance, None).unwrap();

// rank-eval NDCG (binary relevance)
use std::collections::HashSet;
let ranked = vec!["doc1", "doc2", "doc3"];
let relevant: HashSet<_> = ["doc1", "doc3"].into_iter().collect();
let ndcg_eval = eval_ndcg(&ranked, &relevant, 10);

// Both compute NDCG, but with different input formats
```

## Production Training Pipeline

```rust
use rank_learn::lambdarank::{LambdaRankTrainer, ndcg_at_k};

// Load training data
let training_queries = load_training_data();

let trainer = LambdaRankTrainer::default();
let mut model = MyRankingModel::new();

// Training loop
for epoch in 0..100 {
    let mut total_ndcg = 0.0;
    let mut query_count = 0;
    
    for (query, documents, qrels) in &training_queries {
        // Score documents
        let scores: Vec<f32> = documents.iter()
            .map(|doc| model.score(query, doc))
            .collect();
        
        // Get relevance
        let relevance: Vec<f32> = documents.iter()
            .map(|doc| qrels.get(doc).copied().unwrap_or(0.0))
            .collect();
        
        // Compute gradients
        let lambdas = trainer.compute_gradients(&scores, &relevance, Some(10)).unwrap();
        
        // Update model
        model.update_weights(&lambdas);
        
        // Evaluate
        let ndcg = ndcg_at_k(&relevance, Some(10)).unwrap();
        total_ndcg += ndcg;
        query_count += 1;
    }
    
    let avg_ndcg = total_ndcg / query_count as f32;
    println!("Epoch {}: Average NDCG@10 = {}", epoch, avg_ndcg);
}
```

