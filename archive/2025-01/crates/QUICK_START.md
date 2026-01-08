# Quick Start Guide

Get started with rank-learn in 5 minutes.

## Installation

### Rust

```bash
cargo add rank-learn
```

### Python

```bash
pip install rank-learn
# Or for development:
cd rank-learn-python
maturin develop
```

## Basic Usage

### LambdaRank Training

```rust
use rank_learn::lambdarank::LambdaRankTrainer;

let trainer = LambdaRankTrainer::default();
let scores = vec![0.5, 0.8, 0.3];
let relevance = vec![3.0, 2.0, 1.0];
let lambdas = trainer.compute_gradients(&scores, &relevance, None)?;
// Use lambdas to update your model
```

### NDCG Calculation

```rust
use rank_learn::lambdarank::ndcg_at_k;

let relevance = vec![3.0, 2.0, 1.0, 0.5, 0.0];
let ndcg = ndcg_at_k(&relevance, Some(10))?;
```

## Python Quick Start

```python
import rank_learn

# LambdaRank
trainer = rank_learn.LambdaRankTrainer()
scores = [0.5, 0.8, 0.3]
relevance = [3.0, 2.0, 1.0]
lambdas = trainer.compute_gradients(scores, relevance)

# NDCG
ndcg = rank_learn.ndcg_at_k(relevance, k=10)
```

## Next Steps

- See `EXAMPLES.md` for complete examples
- See `README.md` for full documentation

