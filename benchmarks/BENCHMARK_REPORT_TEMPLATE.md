# Benchmark Report Template

## Overview

This template documents benchmark results comparing `rank-*` crates against similar tools.

## Test Environment

- **Date**: [Date of benchmark]
- **Hardware**: [CPU, RAM, OS]
- **Rust Version**: [rustc version]
- **Criterion Version**: [criterion version]

## Results Summary

| Crate | Operation | Throughput | Latency (p50) | Latency (p95) | Memory |
|-------|-----------|------------|---------------|----------------|--------|
| rank-retrieve | BM25 indexing | [ops/s] | [ms] | [ms] | [MB] |
| rank-retrieve | BM25 retrieval | [ops/s] | [ms] | [ms] | [MB] |
| rank-fusion | RRF | [ops/s] | [ms] | [ms] | [MB] |
| rank-rerank | MaxSim | [ops/s] | [ms] | [ms] | [MB] |
| rank-learn | LambdaRank | [ops/s] | [ms] | [ms] | [MB] |

## Detailed Results

### rank-retrieve: BM25

#### Indexing Performance

| Documents | Terms/Doc | rank-retrieve | bm25 crate | tantivy |
|-----------|-----------|---------------|------------|---------|
| 1K | 100 | [ms] | [ms] | [ms] |
| 10K | 200 | [ms] | [ms] | [ms] |
| 100K | 300 | [ms] | [ms] | [ms] |

#### Retrieval Performance

| Documents | Query Length | k | rank-retrieve | bm25 crate | tantivy |
|-----------|-------------|---|---------------|------------|---------|
| 1K | 10 | 20 | [ms] | [ms] | [ms] |
| 10K | 10 | 20 | [ms] | [ms] | [ms] |
| 100K | 10 | 20 | [ms] | [ms] | [ms] |

### rank-retrieve: Dense Retrieval

#### Indexing Performance

| Documents | Dimension | rank-retrieve | hnsw | usearch |
|-----------|-----------|---------------|------|---------|
| 1K | 128 | [ms] | [ms] | [ms] |
| 10K | 256 | [ms] | [ms] | [ms] |
| 100K | 384 | [ms] | [ms] | [ms] |

#### Retrieval Performance

| Documents | Dimension | k | rank-retrieve | hnsw | usearch |
|-----------|-----------|---|---------------|------|---------|
| 1K | 128 | 20 | [ms] | [ms] | [ms] |
| 10K | 256 | 20 | [ms] | [ms] | [ms] |
| 100K | 384 | 20 | [ms] | [ms] | [ms] |

### rank-fusion: Rank Fusion Algorithms

| Algorithm | Lists | List Size | rank-fusion | Python (reference) |
|-----------|-------|----------|------------|-------------------|
| RRF | 2 | 100 | [ms] | [ms] |
| RRF | 5 | 100 | [ms] | [ms] |
| ISR | 2 | 100 | [ms] | [ms] |
| CombMNZ | 2 | 100 | [ms] | [ms] |

### rank-rerank: MaxSim and Cosine Similarity

| Operation | Query Tokens | Doc Tokens | Dimension | rank-rerank | Python (reference) |
|-----------|--------------|------------|-----------|-------------|-------------------|
| MaxSim | 10 | 50 | 128 | [ms] | [ms] |
| MaxSim | 20 | 100 | 256 | [ms] | [ms] |
| Cosine | - | - | 768 | [ms] | [ms] |

### rank-learn: LambdaRank

| Operation | List Size | rank-learn | XGBoost (Python) | LightGBM (Python) |
|-----------|-----------|------------|------------------|-------------------|
| Compute Lambdas | 100 | [ms] | [ms] | [ms] |
| Compute Lambdas | 1000 | [ms] | [ms] | [ms] |
| NDCG Computation | 100 | [ms] | [ms] | [ms] |

## Analysis

### Strengths

- [List strengths of rank-* implementations]

### Areas for Improvement

- [List areas where rank-* could be improved]

### Comparison Notes

- [Notes on comparison methodology, limitations, etc.]

## Recommendations

1. [Recommendation 1]
2. [Recommendation 2]
3. [Recommendation 3]

## Next Steps

- [ ] Run benchmarks on larger datasets
- [ ] Compare accuracy (not just speed)
- [ ] Add memory profiling
- [ ] Benchmark against more tools

