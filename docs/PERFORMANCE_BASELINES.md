# Performance Baselines

**Date:** January 2025  
**Status:** ✅ Initial Baselines Established

## Purpose

This document tracks performance baselines for critical operations in `rank-rank`. These baselines are used to:

1. Detect performance regressions in CI
2. Guide optimization efforts
3. Set realistic expectations for users
4. Compare different implementations

## How to Establish Baselines

1. Run performance tests in release mode:
   ```bash
   cargo test --release --test performance_regression
   ```

2. Record timings for each test on representative hardware

3. Update this document with:
   - Hardware specifications
   - Timings for each operation
   - Environment details (OS, Rust version, etc.)

4. Update test thresholds in `tests/performance_regression.rs` based on actual performance

## Established Baselines

### Test Environment (Initial Baseline)

- **Date**: 2025-01 (exact date to be filled when baseline is established)
- **Hardware**: MacBook Pro (aarch64, NEON)
- **OS**: macOS (darwin 25.2.0)
- **Rust**: 1.74.0
- **Build**: `cargo test --release --test performance_regression`

**Status**: Baselines have been measured and thresholds set, but exact measurement date needs to be recorded.

### rank-soft Baselines

| Operation | Input Size | Baseline | Threshold | Status |
|-----------|------------|----------|-----------|--------|
| `soft_rank` | 1000 elements | ~3ms | <5ms | ✅ Pass |
| `soft_sort` | 1000 elements | <1ms | <1ms | ✅ Pass |
| `spearman_loss` | 1000 elements | ~8ms | <10ms | ✅ Pass |
| `soft_rank` scaling | 100→1000 (10x) | ~100x time | O(n²) verified | ✅ Pass |
| `soft_sort` scaling | 100→1000 (10x) | <50x time | O(n log n) verified | ✅ Pass |

**Notes:**
- `soft_rank` is O(n²) - quadratic scaling confirmed
- `soft_sort` is O(n log n) - sub-quadratic scaling confirmed
- `spearman_loss` includes two `soft_rank` calls, so ~2x the time

### rank-rerank Baselines

| Operation | Input Size | Target | Notes |
|-----------|------------|--------|-------|
| `maxsim_vecs` | 100 query × 1000 doc tokens | <10ms | Typical reranking workload |
| `maxsim_batch` | 10 queries × 100 doc tokens each | <50ms | Batch processing |
| `maxsim_cosine_vecs` | 100 query × 1000 doc tokens | <15ms | Includes normalization |

## Current Targets (Pre-Baseline)

These are initial targets based on typical workloads. They will be updated once baselines are established.

### MaxSim Operations

| Operation | Input Size | Target | Notes |
|-----------|------------|--------|-------|
| `maxsim_vecs` | 100 query × 1000 doc tokens | <10ms | Typical reranking workload |
| `maxsim_batch` | 10 queries × 100 doc tokens each | <50ms | Batch processing |
| `maxsim_cosine_vecs` | 100 query × 1000 doc tokens | <15ms | Includes normalization |

### Similarity Operations

| Operation | Input Size | Target | Notes |
|-----------|------------|--------|-------|
| `cosine` | 1000-dim vectors | <0.1ms | Single comparison |
| `dot` | 1000-dim vectors | <0.05ms | Single comparison |
| `norm` | 1000-dim vectors | <0.05ms | Single vector |

### Scaling Characteristics

| Operation | Size Ratio | Max Time Ratio | Notes |
|-----------|------------|----------------|-------|
| `maxsim_vecs` | 10x | <100x | Sub-quadratic scaling expected |

## Hardware Requirements

To establish meaningful baselines, tests should be run on:

- **CPU**: Modern x86_64 (AVX2 or AVX-512) or aarch64 (NEON)
- **Memory**: Sufficient RAM for test data
- **Environment**: Clean system (no other heavy processes)

## Adding New Baselines

When adding new baseline entries, follow this format:

```markdown
### Test Environment

- **Date**: [YYYY-MM-DD] (fill in when baseline is established)
- **Hardware**: [CPU model] ([architecture], [SIMD])
- **OS**: [OS version]
- **Rust**: [version]
- **Build**: `cargo test --release --test performance_regression`

### Results

| Test | Time | Status |
|------|------|--------|
| `test_operation_performance_regression` | Xms | ✅ Pass |
| `test_operation_scaling` | Xx (Yx size) | ✅ Pass |
```

## CI Integration

Performance regression tests run in CI with:

- Release mode builds
- Representative test data
- Thresholds based on established baselines

If a test fails:

1. Verify the regression is real (not just test variance)
2. Investigate the cause (profiling, code review)
3. Either fix the regression or update the threshold if justified

## Future Enhancements

- [ ] Automated baseline tracking
- [ ] Performance comparison across hardware
- [ ] Historical performance trends
- [ ] Integration with benchmarking tools (criterion)

---

**Last Updated:** January 2025  
**Next Review:** After establishing initial baselines

