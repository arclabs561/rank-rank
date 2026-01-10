# Troubleshooting Guide

Common issues and solutions for rank-learn.

## Training Issues

### Model Not Converging

**Symptoms**: Loss doesn't decrease during training.

**Solutions**:

1. **Check learning rate**:
   - Too high: Loss oscillates or diverges
   - Too low: Loss decreases very slowly
   - Typical range: 0.001 to 0.1

2. **Verify data quality**:
   - Check relevance labels are correct
   - Ensure query-document pairs are valid
   - Verify features are normalized

3. **Check gradient flow**:
```rust
// Verify gradients are not zero or NaN
for param in model.parameters() {
    if param.grad().is_none() || param.grad().unwrap().is_nan().any() {
        println!("Warning: Gradient issue detected");
    }
}
```

### Overfitting

**Symptoms**: Training loss decreases but validation loss increases.

**Solutions**:

1. **Add regularization**:
   - L2 regularization on model parameters
   - Early stopping based on validation loss

2. **Reduce model complexity**:
   - Fewer layers or parameters
   - Simpler feature set

3. **Increase training data**:
   - More diverse queries
   - More query-document pairs

---

## LambdaRank Issues

### Lambda Values Are Zero

**Symptoms**: All lambda values (gradients) are zero.

**Solutions**:

1. **Check relevance labels**:
   - Must have at least two documents with different relevance
   - Binary: 0 and 1 (or higher)
   - Graded: Multiple distinct levels

2. **Verify pairwise comparisons**:
   - LambdaRank needs pairs of documents with different relevance
   - If all documents have same relevance, lambdas will be zero

3. **Check NDCG calculation**:
   - Verify NDCG is computed correctly
   - Ensure ranking order affects NDCG

### Training Is Slow

**Symptoms**: LambdaRank training takes too long.

**Solutions**:

1. **Reduce number of pairs**:
   - Sample pairs instead of using all pairs
   - Focus on pairs with large relevance difference

2. **Optimize NDCG computation**:
   - Cache ideal DCG
   - Use efficient sorting

3. **Use mini-batches**:
   - Process queries in batches
   - Parallelize across queries

---

## Integration Issues

### rank-soft Integration

**Symptoms**: Errors when using rank-soft differentiable operations.

**Solutions**:

1. **Check feature flags**:
```toml
[dependencies]
rank-learn = { version = "0.1", features = [] }
rank-soft = { version = "0.1" }  # Required dependency
```

2. **Verify tensor shapes**:
   - Input features must match expected dimensions
   - Batch dimensions must be consistent

3. **Check regularization strength**:
   - rank-soft operations require appropriate regularization
   - Too high: gradients vanish
   - Too low: not differentiable enough

**See**: rank-soft [TROUBLESHOOTING.md](../rank-soft/docs/TROUBLESHOOTING.md) for details.

### XGBoost/LightGBM Integration

**Symptoms**: External library integration fails.

**Solutions**:

1. **Check feature flags**:
```toml
[dependencies]
rank-learn = { version = "0.1", features = ["xgboost"] }
```

2. **Verify library installation**:
   - XGBoost/LightGBM must be installed
   - Check system dependencies

3. **Check API compatibility**:
   - External libraries may have version requirements
   - Verify API matches expected interface

**Note**: XGBoost/LightGBM integration is planned but may not be fully implemented.

---

## Data Issues

### Invalid Relevance Labels

**Symptoms**: Error with relevance labels.

**Solutions**:

1. **Check label range**:
   - Labels should be non-negative integers
   - Typical: 0 (not relevant) to 4 (highly relevant)

2. **Normalize if needed**:
```rust
// Ensure labels are in valid range
let normalized_labels: Vec<u32> = labels.iter()
    .map(|&l| l.max(0) as u32)
    .collect();
```

3. **Handle missing labels**:
   - Use default relevance (0) for missing labels
   - Or exclude documents without labels

### Feature Mismatch

**Symptoms**: Error with feature dimensions.

**Solutions**:

1. **Verify feature dimensions**:
```rust
let expected_dim = model.feature_dim();
let actual_dim = features.len();
assert_eq!(expected_dim, actual_dim, 
    "Feature dimension mismatch: expected {}, got {}", expected_dim, actual_dim);
```

2. **Normalize features**:
   - Features should be normalized (mean 0, std 1)
   - Or use appropriate normalization for your model

3. **Handle missing features**:
   - Use default values (0.0) for missing features
   - Or impute missing values

---

## Performance Issues

### Slow Training

**Symptoms**: Training iterations take too long.

**Solutions**:

1. **Reduce data size**:
   - Sample queries for faster iteration
   - Use smaller feature sets during development

2. **Optimize NDCG computation**:
   - Cache ideal DCG values
   - Use efficient sorting algorithms

3. **Use GPU acceleration**:
   - If using neural models, enable GPU
   - Use appropriate backend (Candle, Burn)

4. **Profile to find bottlenecks**:
```bash
cargo install cargo-flamegraph
cargo flamegraph --bench lambdarank
```

---

## Common Errors

### "No valid pairs"

**Symptoms**: Error when computing lambdas.

**Solutions**:

1. **Check relevance diversity**:
   - Need at least two documents with different relevance
   - All documents having same relevance = no pairs

2. **Verify query structure**:
   - Each query should have multiple documents
   - Documents should have relevance labels

### "Dimension mismatch"

**Symptoms**: Error with feature dimensions.

**Solutions**:

1. **Verify all features have same dimension**:
```rust
let dim = features[0].len();
for (i, feat) in features.iter().enumerate() {
    assert_eq!(feat.len(), dim, 
        "Feature {} has wrong dimension: expected {}, got {}", i, dim, feat.len());
}
```

2. **Check model expects correct dimension**:
   - Model must be initialized with correct feature dimension
   - Verify model configuration

---

## Getting Help

### Debug Information

When reporting issues, include:

1. **Rust version**: `rustc --version`
2. **Training configuration**: Learning rate, batch size, etc.
3. **Data characteristics**: Number of queries, documents per query, feature dimensions
4. **Error messages**: Full output
5. **Minimal reproduction**: Small code example

### Resources

- **[QUICK_START.md](../QUICK_START.md)** - Basic usage guide
- **[README](../README.md)** - Complete API reference
- **[EXAMPLES.md](../EXAMPLES.md)** - Code examples
- **rank-soft docs** - Differentiable operations documentation

### Reporting Issues

1. Check existing issues on GitHub
2. Search documentation
3. Create minimal reproduction
4. Include debug information
5. File issue with details

---

## Common Patterns

### Pattern: "Model doesn't learn"

1. Check learning rate (too high or too low)
2. Verify data quality (relevance labels, features)
3. Check gradient flow (not zero, not NaN)
4. Review model architecture

### Pattern: "Training is too slow"

1. Reduce data size for development
2. Optimize NDCG computation
3. Use GPU acceleration if available
4. Profile to find bottlenecks

### Pattern: "Lambda values are zero"

1. Check relevance labels (need diversity)
2. Verify pairwise comparisons exist
3. Check NDCG calculation
4. Review LambdaRank implementation

