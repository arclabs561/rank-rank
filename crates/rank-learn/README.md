# rank-learn

Learning to Rank (LTR) frameworks: LambdaRank, LambdaMART, XGBoost integration, and neural LTR models.

[![CI](https://github.com/arclabs561/rank-learn/actions/workflows/ci.yml/badge.svg)](https://github.com/arclabs561/rank-learn/actions)
[![Crates.io](https://img.shields.io/crates/v/rank-learn.svg)](https://crates.io/crates/rank-learn)
[![Docs](https://docs.rs/rank-learn/badge.svg)](https://docs.rs/rank-learn)

```
cargo add rank-learn
```

## Purpose

Complete Learning to Rank algorithms for training ranking models. This crate provides full LTR frameworks, while `rank-soft` provides the differentiable operations used internally.

## Relationship to rank-soft

- **rank-soft**: Differentiable ranking operations (mathematical primitives)
  - Soft ranking, differentiable sorting
  - Loss functions (ListNet, ListMLE, Spearman)
  - Framework-agnostic building blocks

- **rank-learn**: Complete LTR frameworks (full ML systems)
  - LambdaRank, LambdaMART
  - XGBoost/LightGBM integration for ranking
  - Neural LTR models
  - Uses `rank-soft` for differentiable operations

**Boundary**: `rank-soft` provides building blocks, `rank-learn` provides complete solutions.

## Features

- **LambdaRank**: Pairwise LTR with metric-aware gradients
- **LambdaMART**: Gradient boosting for ranking (MART + LambdaRank)
- **Neural LTR**: Neural ranking models using rank-soft
- **XGBoost Integration**: (Planned - requires external bindings)
- **LightGBM Integration**: (Planned - requires external bindings)

## Quick Start

### LambdaRank

```rust
use rank_learn::prelude::*;

// Create trainer
let trainer = LambdaRankTrainer::default();

// Model scores and ground truth relevance
let scores = vec![0.5, 0.8, 0.3, 0.9, 0.2];
let relevance = vec![3.0, 1.0, 2.0, 3.0, 1.0];

// Compute LambdaRank gradients
let lambdas = trainer.compute_gradients(&scores, &relevance, None);

// Use lambdas as gradients to update your model
// In training loop:
//   model.update_weights(&lambdas);
```

### Neural LTR

```rust
use rank_learn::prelude::*;

// Create neural LTR model
let config = NeuralLTRConfig::default();
let model = NeuralLTRModel::new(config);

// Compute predictions
let predictions = vec![0.1, 0.9, 0.3];
let targets = vec![0.0, 1.0, 0.2];

// Compute loss (uses rank-soft internally)
let loss = model.compute_loss(&predictions, &targets);

// Gradients flow through soft ranking operations!
```

## LambdaRank Algorithm

LambdaRank optimizes ranking metrics (like NDCG) directly by computing gradients based on how swapping document pairs would change the metric.

For a pair of documents (i, j) where document i should rank higher than j:

```text
lambda_ij = -σ / (1 + exp(σ * (s_i - s_j))) * |ΔNDCG|
```

Where:
- `s_i`, `s_j` = scores for documents i and j
- `σ` = sigmoid parameter (typically 1.0)
- `ΔNDCG` = change in NDCG if documents i and j were swapped

The lambda for document i is the sum of all lambda_ij over pairs where i is involved.

## Examples

See the `examples/` directory:
- `lambdarank_training.rs` - LambdaRank training loop
- `neural_ltr_training.rs` - Neural LTR with rank-soft

Run examples:
```bash
cargo run --example lambdarank_training
cargo run --example neural_ltr_training
```

## Dependencies

- `rank-soft`: For differentiable ranking operations

## Status

 **Core functionality implemented**:
- LambdaRank with NDCG-aware gradients
- Neural LTR interface using rank-soft

⏳ **Planned** (requires external bindings):
- XGBoost integration
- LightGBM integration
- Full neural network architectures

## Integration with Other rank-* Crates

### Training Pipeline

```rust
use rank_learn::prelude::*;
use rank_retrieve::prelude::*;
use rank_rerank::simd;
use rank_eval::ndcg_at_k;

// 1. Retrieve candidates
let candidates = retriever.retrieve(&query, 1000);

// 2. Score with model
let scores: Vec<f32> = model.score(&query, &candidates);

// 3. Compute LambdaRank gradients
let trainer = LambdaRankTrainer::default();
let lambdas = trainer.compute_gradients(&scores, &relevance, None);

// 4. Update model
model.update_weights(&lambdas);

// 5. Evaluate
let ndcg = ndcg_at_k(&relevance, Some(10));
```
