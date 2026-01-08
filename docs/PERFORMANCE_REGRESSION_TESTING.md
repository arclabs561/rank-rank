# Performance Regression Testing Guide

**Date:** January 2025  
**Status:** Framework Established

---

## Overview

Performance regression tests ensure that optimizations and refactorings don't degrade performance. This guide documents the framework for tracking performance over time.

---

## Current Benchmark Infrastructure

### Existing Benchmarks

#### `rank-rerank`
- **Location**: `crates/rank-rerank/benches/`
- **Files**:
  - `comprehensive.rs` - Comprehensive benchmarks
  - `realistic_workloads.rs` - Real-world workload simulation
  - `refine.rs` - Refinement operations

#### `rank-retrieve`
- **Location**: `crates/rank-retrieve/benches/`
- **Files**:
  - `bm25.rs` - BM25 retrieval benchmarks
  - `dense.rs` - Dense retrieval benchmarks
  - `sparse.rs` - Sparse retrieval benchmarks
  - `generative.rs` - Generative retrieval benchmarks

#### `rank-soft`
- **Location**: `crates/rank-soft/benches/`
- **Files**: (check if exists)

#### `rank-fusion`
- **Location**: `crates/rank-fusion/benches/`
- **Files**:
  - `fusion.rs` - Fusion algorithm benchmarks

---

## Performance Regression Test Framework

### 1. Baseline Establishment

**Goal**: Establish performance baselines for critical operations

**Critical Paths**:
1. **MaxSim scoring** (`rank-rerank`)
   - Target: <1ms for 100 query tokens × 1000 doc tokens
   - Baseline: Measure current performance

2. **BM25 retrieval** (`rank-retrieve`)
   - Target: <10ms for 1M document index, top-100 retrieval
   - Baseline: Measure current performance

3. **Rank fusion** (`rank-fusion`)
   - Target: <5ms for 10 input lists, 1000 candidates each
   - Baseline: Measure current performance

4. **Soft ranking** (`rank-soft`)
   - Target: <1ms for 1000 elements
   - Baseline: Measure current performance

### 2. Regression Test Structure

```rust
// Example: crates/rank-rerank/tests/performance_regression.rs
#[cfg(test)]
mod performance_regression {
    use criterion::{black_box, Criterion};
    use rank_rerank::simd::maxsim_vecs;
    
    #[test]
    fn test_maxsim_performance_regression() {
        let query = vec![vec![1.0f32; 128]; 100];
        let doc = vec![vec![0.5f32; 128]; 1000];
        
        let start = std::time::Instant::now();
        let _score = maxsim_vecs(&query, &doc);
        let elapsed = start.elapsed();
        
        // Regression threshold: should complete in <10ms
        assert!(
            elapsed.as_millis() < 10,
            "Performance regression: maxsim took {}ms (threshold: 10ms)",
            elapsed.as_millis()
        );
    }
}
```

### 3. CI Integration

**GitHub Actions Workflow**:
```yaml
name: Performance Regression Tests

on: [push, pull_request]

jobs:
  performance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Run performance regression tests
        run: |
          cargo test --release --test performance_regression
          
      - name: Run benchmarks (track over time)
        run: |
          cargo bench --bench comprehensive
        continue-on-error: true
```

---

## Benchmarking Best Practices

### 1. Use Criterion for Benchmarks

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_maxsim(c: &mut Criterion) {
    let query = vec![vec![1.0f32; 128]; 100];
    let doc = vec![vec![0.5f32; 128]; 1000];
    
    c.bench_function("maxsim_100x1000", |b| {
        b.iter(|| maxsim_vecs(black_box(&query), black_box(&doc)))
    });
}

criterion_group!(benches, bench_maxsim);
criterion_main!(benches);
```

### 2. Track Performance Over Time

**Options**:
1. **Criterion HTML Reports**: Automatically generated, track manually
2. **GitHub Actions Artifacts**: Store benchmark results as artifacts
3. **External Service**: Use services like `cargo-benchcmp` or custom tracking

### 3. Performance Targets

| Operation | Target | Current | Status |
|-----------|--------|---------|--------|
| MaxSim (100×1000) | <1ms | TBD | ⏳ |
| BM25 (1M docs) | <10ms | TBD | ⏳ |
| RRF (10 lists) | <5ms | TBD | ⏳ |
| Soft Rank (1000) | <1ms | TBD | ⏳ |

---

## Running Benchmarks

### Local Development

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench comprehensive

# Compare with previous run
cargo bench --bench comprehensive -- --save-baseline current
cargo bench --bench comprehensive -- --baseline current
```

### CI/CD

```bash
# Run benchmarks in CI (may be slow)
cargo bench --bench comprehensive -- --output-format json > benchmarks.json
```

---

## Performance Monitoring

### 1. Track Key Metrics

- **Latency**: P50, P95, P99 percentiles
- **Throughput**: Operations per second
- **Memory**: Peak memory usage
- **CPU**: CPU utilization

### 2. Alert on Regressions

**Thresholds**:
- **Critical**: >20% performance degradation
- **Warning**: >10% performance degradation
- **Info**: >5% performance degradation

### 3. Performance Budgets

Set performance budgets for each operation:
- **MaxSim**: Must complete in <1ms for typical workloads
- **BM25**: Must complete in <10ms for 1M document index
- **Fusion**: Must complete in <5ms for 10 input lists

---

## Next Steps

1. ✅ **Framework Documented**: This guide
2. ⏳ **Establish Baselines**: Run benchmarks to establish current performance
3. ⏳ **Add Regression Tests**: Create `performance_regression.rs` test files
4. ⏳ **CI Integration**: Add performance regression tests to CI
5. ⏳ **Tracking**: Set up performance tracking over time

---

## References

- [Criterion.rs Documentation](https://docs.rs/criterion/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- Existing benchmarks in `benches/` directories

---

**Last Updated**: January 2025

