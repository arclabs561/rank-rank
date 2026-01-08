# Additional Reasoning Errors Found

## Critical: Normalization::Rank Assumes Sorted Input

**Location**: `crates/rank-fusion/src/lib.rs:1767-1774`

**Issue**: The `Normalization::Rank` implementation uses `enumerate()` to assign ranks based on position in the input slice, but it **does not sort the input first**. This means if the input is not sorted by score (descending), the ranks will be incorrect.

**Current Code**:
```rust
Normalization::Rank => {
    // Convert to ranks (0-indexed), then normalize by list length
    let n = results.len() as f32;
    results
        .iter()
        .enumerate()
        .map(|(rank, (id, _))| (id.clone(), 1.0 - (rank as f32 / n)))
        .collect()
}
```

**Problem**: 
- `enumerate()` gives positions 0, 1, 2, ... based on input order
- If input is `[("doc1", 0.1), ("doc2", 0.9)]` (unsorted), it assigns:
  - doc1 gets rank 0 (best rank) even though it has lower score
  - doc2 gets rank 1 (worse rank) even though it has higher score
- This is backwards! Higher scores should get better (lower) ranks

**Where it's called**: `additive_multi_task_multi` at line 1651:
```rust
.map(|(list, _)| normalize_scores(list.as_ref(), config.normalization))
```

The input `list.as_ref()` may not be sorted, so this will produce incorrect ranks.

**Fix Options**:
1. **Sort before ranking** (recommended):
   ```rust
   Normalization::Rank => {
       let mut sorted: Vec<_> = results.to_vec();
       sorted.sort_by(|a, b| b.1.total_cmp(&a.1)); // Sort descending
       let n = sorted.len() as f32;
       sorted
           .iter()
           .enumerate()
           .map(|(rank, (id, _))| (id.clone(), 1.0 - (rank as f32 / n)))
           .collect()
   }
   ```

2. **Document requirement** (if sorting is expensive and callers guarantee sorted input):
   - Add clear documentation that input must be sorted by score (descending)
   - Add debug assertion to verify sortedness

**Recommendation**: Fix by sorting, as it's safer and the performance cost is minimal (O(n log n) for normalization which is already called per-list).

---

## Medium: CombANZ Division Safety

**Location**: `crates/rank-fusion/src/lib.rs:2663`

**Issue**: The code does `sum / count as f32` where `count` is guaranteed to be >= 1 (we only increment when we see an item), so this is actually safe. However, the logic could be clearer.

**Current Code**:
```rust
.map(|(id, (sum, count))| (id, sum / count as f32))
```

**Status**: Actually safe - `count` is always >= 1 because we only add entries when we see items. But could add a comment or assertion for clarity.

**Recommendation**: Add a comment explaining why division is safe, or add a debug assertion:
```rust
.map(|(id, (sum, count))| {
    debug_assert!(count > 0, "Count should always be > 0");
    (id, sum / count as f32)
})
```

---

## Low: Documentation Gap for Normalization::Rank

**Location**: `crates/rank-fusion/src/lib.rs:1717-1720`

**Issue**: Documentation doesn't mention that input should be sorted, or that the function will sort it.

**Current Documentation**:
```rust
/// Rank-based: convert scores to ranks, then normalize
///
/// Ignores score magnitudes entirely. Most robust but loses information.
Rank,
```

**Recommendation**: Update documentation to clarify behavior:
```rust
/// Rank-based: convert scores to ranks, then normalize
///
/// Sorts input by score (descending), assigns ranks 0..n-1, then normalizes
/// to [0, 1] range where rank 0 (best) → 1.0, rank n-1 (worst) → 1/n.
/// Ignores score magnitudes entirely. Most robust but loses information.
Rank,
```

---

## Summary

| Error | Severity | Location | Status |
|-------|----------|----------|--------|
| Normalization::Rank doesn't sort input | **Critical** | `lib.rs:1767-1774` | ✅ **FIXED** |
| CombANZ division clarity | Medium | `lib.rs:2663` | ✅ **FIXED** |
| Documentation gap | Low | `lib.rs:1717-1720` | ✅ **FIXED** |

## Implementation Status

All fixes have been implemented:

1. ✅ **Fixed `Normalization::Rank`**: Now sorts input by score (descending) before assigning ranks
2. ✅ **Added debug assertion**: CombANZ now has assertion explaining why division is safe
3. ✅ **Updated documentation**: Rank normalization now documents sorting behavior
4. ✅ **Added comprehensive tests**: 7 new tests in `tests/normalization_rank_bug.rs` covering:
   - Unsorted input handling
   - Sorted input preservation
   - Duplicate scores
   - Single element
   - Empty input
   - NaN scores
   - Integration with additive_multi_task

