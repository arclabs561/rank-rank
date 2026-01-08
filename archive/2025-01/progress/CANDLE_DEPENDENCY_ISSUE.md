# Candle Dependency Issue

**Status:** ⚠️ **Blocked**  
**Date:** January 2025  
**Impact:** Candle feature cannot be compiled until resolved

## Issue Summary

The `candle-core` crate (version 0.3.3) has a dependency conflict with `rand 0.9.2` and the `half` crate, preventing compilation when the `candle` feature is enabled.

## Error Details

```
error[E0277]: the trait bound `StandardNormal: rand_distr::Distribution<half::f16>` is not satisfied
             which is required by `rand_distr::Normal<half::f16>: rand_distr::Distribution<half::f16>`
```

This error occurs because:
1. `candle-core` depends on `half` for FP16 support
2. `rand_distr` doesn't support `half::f16` in the version used by `candle-core`
3. This is a known issue in `candle-core` itself, not our code

## Affected Code

- `crates/rank-rerank/src/candle.rs` - GPU-accelerated MaxSim operations
- `crates/rank-soft/src/candle.rs` - GPU-accelerated soft ranking
- `crates/rank-rerank/examples/candle_maxsim.rs` - Example code
- `crates/rank-soft/examples/candle_training.rs` - Example code

## Workarounds

### Option 1: Pin candle-core Version (Recommended)

Pin `candle-core` to version `0.3.2` if it doesn't have this issue:

```toml
[dependencies]
candle-core = { version = "0.3.2", optional = true }
```

**Status:** ⏳ Not tested yet

### Option 2: Wait for candle-core Fix

Monitor `candle-core` releases for a fix. The issue is tracked in the candle repository.

**Status:** ⏳ Waiting for upstream fix

### Option 3: Disable Candle Feature Temporarily

The code compiles and works without the `candle` feature. All core functionality (SIMD-accelerated CPU operations) remains available.

**Status:** ✅ Current workaround

## Impact Assessment

### What Works
- ✅ All CPU SIMD operations (MaxSim, cosine similarity)
- ✅ All Python bindings
- ✅ ONNX Runtime integration
- ✅ All other features

### What's Blocked
- ❌ GPU-accelerated MaxSim (Candle tensors)
- ❌ GPU-accelerated soft ranking
- ❌ Candle-based examples

### Performance Impact

**Current State:**
- CPU SIMD is already very fast (~1ms for 100 documents)
- GPU acceleration would help with large batches (1000+ documents)
- For typical workloads (10-100 documents), CPU is sufficient

**Conclusion:** The dependency issue doesn't block production use, but limits GPU acceleration for large-scale workloads.

## Tracking

- **Upstream Issue:** Monitor candle-core releases
- **Workaround Status:** Code compiles without `candle` feature
- **Priority:** Medium (GPU acceleration is nice-to-have, not critical)

## Next Steps

1. ✅ Document the issue (this file)
2. ⏳ Test `candle-core = "0.3.2"` if available
3. ⏳ Monitor candle-core releases for fix
4. ⏳ Consider alternative tensor libraries if issue persists

## References

- [Candle GitHub](https://github.com/huggingface/candle)
- [Candle Issues](https://github.com/huggingface/candle/issues)
- Related: `docs/GPU_ACCELERATION_PLAN.md` for GPU acceleration strategy

---

**Last Updated:** January 2025  
**Next Review:** After testing `candle-core = "0.3.2"` or when upstream fix is available

