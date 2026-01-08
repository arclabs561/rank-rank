# Benchmarking Guide for rank-learn

## Overview

Performance benchmarks for LambdaRank gradient computation, NDCG calculation, and batch processing.

## Running Benchmarks

```bash
cd crates/rank-learn
cargo bench
```

## Benchmark Structure

### LambdaRank Benchmarks (`benches/lambdarank.rs`)

**Gradient Computation**:
- List sizes: 10, 50, 100, 500, 1000, 5000
- Measures: Time to compute LambdaRank gradients
- Complexity: O(n²) pairwise comparisons

**NDCG Computation**:
- List sizes: 10, 50, 100, 500, 1000
- Measures: Time to compute NDCG@k
- Complexity: O(n log n) for sorting

**Batch Processing**:
- Batch sizes: 10, 50, 100, 500
- List sizes: 50, 100, 200
- Measures: Time to process multiple query-document lists

## Performance Goals

### LambdaRank
- **Gradient computation**: < 10ms for n=100
- **Gradient computation**: < 100ms for n=500
- **Gradient computation**: < 1s for n=1000

### NDCG
- **NDCG computation**: < 0.1ms for n=100
- **NDCG computation**: < 1ms for n=1000

### Batch Processing
- **Batch throughput**: > 100 queries/second for n=100

## Comparison with Python

Python implementations (XGBoost, LightGBM) provide reference:
- XGBoost LambdaRank: Industry standard
- LightGBM LambdaRank: Optimized implementation
- Our goal: Match or exceed Python performance

## Optimization Opportunities

1. **SIMD**: Vectorize pairwise comparisons
2. **Parallelization**: Parallelize batch processing
3. **Caching**: Cache sorted relevance for NDCG
4. **Early termination**: Skip pairs with zero relevance difference

## Continuous Benchmarking

Benchmarks track:
- Performance regressions
- Optimization impact
- Comparison with Python implementations

