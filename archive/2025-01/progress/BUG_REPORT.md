# Bug Report: Crate Review Findings

## Critical Issues

### 1. CI Workflow Bug: Wrong Path in rank-rerank
**Location**: `crates/rank-rerank/.github/workflows/ci.yml:157, 161, 170, 174, 181`

**Issue**: CI checks for unsafe code and unwrap/expect in `rank-refine/src/` instead of `rank-rerank/src/`. The crate was renamed but CI wasn't updated.

**Impact**: CI quality checks don't actually run on the correct code.

**Fix**:
```yaml
# Change from:
unsafe_count=$(grep -r "unsafe" --include="*.rs" rank-refine/src/ 2>/dev/null | wc -l || echo "0")
# To:
unsafe_count=$(grep -r "unsafe" --include="*.rs" src/ 2>/dev/null | wc -l || echo "0")
```

Also update cargo doc command from `-p rank-refine` to `-p rank-rerank`.

### 2. Fuzz Target Uses Wrong Crate Name
**Location**: `crates/rank-rerank/fuzz/fuzz_targets/fuzz_pool.rs:4`

**Issue**: Uses `rank_refine::colbert` but crate is `rank-rerank`.

**Fix**:
```rust
// Change from:
use rank_refine::colbert;
// To:
use rank_rerank::colbert;
```

### 3. Old Crate Name References Throughout Codebase
**Location**: Multiple files reference `rank-refine` and `rank-relax` (old names)

**Files affected**:
- `crates/rank-fusion/test-e2e-local/Cargo.toml:14` - dependency name `rank-refine`
- `crates/rank-rerank/fuzz/fuzz_targets/fuzz_pool.rs:4` - import `rank_refine::colbert`
- `crates/rank-rerank/README.md:196` - documentation link to `rank-relax`
- `crates/rank-rerank/docs/main.typ:8,21,84` - documentation uses `rank-refine` name
- `crates/rank-rerank/rank-rerank-python/rank_refine.pyi:1` - file name and content
- `crates/rank-rerank/examples/python_integration.rs:1` - comment references `rank-refine`
- `crates/rank-rerank/rank-rerank-python/examples/benchmark_reranking.py:5` - commented import

**Impact**: Confusion, potential build failures, incorrect documentation, broken fuzz tests

## Medium Priority Issues

### 4. Incomplete Implementation: CrossEncoder ONNX
**Location**: `crates/rank-rerank/src/crossencoder/ort.rs` and `crates/rank-rerank/src/crossencoder_ort.rs`

**Issue**: Both files have placeholder implementations with TODOs:
- `encode_pair()` returns dummy token IDs (all 1s)
- `score_batch()` returns `doc.len() / 1000.0` as placeholder score

**Impact**: Feature doesn't actually work, misleading API

**Note**: This appears intentional (feature gated, documented as placeholder), but should be clearly marked or removed if not ready.

### 5. Placeholder Neural LTR Implementation
**Location**: `crates/rank-learn/src/neural.rs:61-64`

**Issue**: `NeuralLTRModel::score()` always returns `vec![0.5; _documents.len()]` - constant scores for all documents.

**Impact**: Neural LTR training won't work correctly

**Note**: Documented as placeholder in comments, but should be marked more clearly or feature-gated.

### 6. Potential Division by Zero (False Positive - Already Handled)
**Location**: `crates/rank-fusion/src/lib.rs:1769`

**Issue**: `Normalization::Rank` divides by `n` without explicit check.

**Status**: Actually safe - there's an early return at line 1731 if `results.is_empty()`, so `n` is always >= 1 when we reach line 1769.

## Code Quality Observations

### 7. Unsafe SIMD Code
**Location**: `crates/rank-rerank/src/simd.rs:63, 71, 1330, 1337`

**Status**: Has proper safety comments explaining why unsafe is safe:
- Runtime feature detection before use
- Length checks via `min(a.len(), b.len())`
- Platform-specific availability checks

**Recommendation**: Looks correct, but worth reviewing safety invariants are maintained.

### 8. Many Unwrap Calls in Tests
**Location**: Throughout test files

**Status**: Acceptable - tests can use unwrap for cleaner test code. Production code appears to use Result types appropriately.

## Recommendations

1. **Fix CI workflow immediately** - Quality checks aren't running
2. **Update all crate name references** - Search and replace old names
3. **Mark incomplete features clearly** - Either remove or add `#[cfg(feature = "unstable")]` gates
4. **Consider deprecating crossencoder/ort** - If not ready, mark as experimental or remove

## Files to Review

- `crates/rank-rerank/.github/workflows/ci.yml` - Fix paths
- `crates/rank-rerank/fuzz/fuzz_targets/fuzz_pool.rs` - Fix import
- `crates/rank-fusion/test-e2e-local/Cargo.toml` - Fix dependency name
- All documentation files - Update crate name references

