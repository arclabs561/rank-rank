# Final Name Recommendation: Accounting for Published Status

## Constraint

**`rank-relax` is already published** (either to crates.io, PyPI, or both)

This means renaming would be a **breaking change** affecting existing users.

## Decision: Keep `rank-relax`, Emphasize "Soft" in Documentation

### Recommendation

**Keep the crate name as `rank-relax`** but:

1. **Emphasize "soft" in user-facing documentation**
   - README title: "Soft ranking and sorting operations"
   - Examples: Show `soft_rank()` prominently
   - Keywords: Include "soft" in keywords

2. **Keep function names as `soft_*`** (already correct)
   - `soft_rank()`, `soft_sort()` - users see "soft" in API
   - This is where users interact most

3. **Use "relaxation" in technical descriptions**
   - Mathematical documentation
   - Technical papers/references
   - Maintains precision

### Why This Works

**Users see "soft" where it matters:**
- Function names: `soft_rank()`, `soft_sort()`
- Documentation: "Soft ranking operations"
- Examples: "Soft ranking example"

**Technical accuracy maintained:**
- Crate name: `rank-relax` (technically accurate)
- Technical docs: "Continuous relaxation"
- Mathematical descriptions: Use "relaxation"

**No breaking changes:**
- Existing users unaffected
- No migration required
- Ecosystem stability

### Implementation

**README.md Structure:**
```markdown
# rank-relax

**Soft ranking and sorting operations for machine learning**

This crate provides smooth relaxations of discrete ranking operations...

## Quick Start

```rust
use rank_relax::soft_rank;  // "soft" in function name

let ranks = soft_rank(&values, 1.0);
```

## Technical Details

The crate implements continuous relaxations of discrete operations...
```

**Cargo.toml:**
```toml
keywords = ["ranking", "sorting", "differentiable", "relaxation", "soft", "soft-ranking", "machine-learning"]
```

**Function Documentation:**
```rust
/// Computes soft ranks using a continuous relaxation of discrete ranking.
/// 
/// This function provides a smooth, differentiable approximation to discrete
/// ranking operations, enabling gradient-based optimization.
pub fn soft_rank(values: &[f32], regularization: f32) -> Vec<f32>
```

## Alternative: If Breaking Change is Acceptable

If the user base is small and breaking changes are acceptable:

### Option: Publish `rank-soft`, Deprecate `rank-relax`

**Migration Path:**
1. Publish `rank-soft` v0.1.0 (new crate)
2. Update `rank-relax` to v0.2.0 with:
   - Deprecation notice
   - Re-export from `rank-soft`
   - Migration guide in README
3. Mark `rank-relax` as deprecated on crates.io/PyPI
4. After 6-12 months, archive `rank-relax`

**Migration Guide:**
```toml
# Old
[dependencies]
rank-relax = "0.1"

# New  
[dependencies]
rank-soft = "0.1"
```

**Pros:**
- Better name long-term
- Consistent with function names
- Better discoverability

**Cons:**
- Breaking change for users
- Requires maintaining two crates
- Ecosystem fragmentation

## Final Verdict

**Keep `rank-relax`** because:
1. ✅ No breaking changes
2. ✅ Users see "soft" in function names (`soft_rank`)
3. ✅ Documentation can emphasize "soft"
4. ✅ "Relaxation" is technically accurate
5. ✅ Maintains ecosystem stability

**The name is fine** - users interact with `soft_rank()`, not the crate name directly. Documentation can bridge any gap.

