# Name Decision: Accounting for Published Crate

## Critical Constraint

**`rank-relax` is already published to crates.io**

This significantly changes the decision calculus. We must consider:
1. **Existing users** who depend on `rank-relax`
2. **Breaking changes** from renaming
3. **Migration path** if we change
4. **Ecosystem impact** of deprecation

## Options Analysis

### Option 1: Keep `rank-relax` (No Breaking Change)

**Pros:**
- ✅ No breaking change for existing users
- ✅ No migration required
- ✅ Maintains ecosystem stability
- ✅ No need to deprecate or maintain two crates

**Cons:**
- ⚠️ Less intuitive name ("relax" is ambiguous)
- ⚠️ Inconsistent with function names (`soft_rank`, not `relax_rank`)
- ⚠️ Less discoverable (users search for "soft ranking")

**Verdict**: **Safest option, maintains stability**

### Option 2: Publish `rank-soft`, Deprecate `rank-relax`

**Migration Strategy:**
1. Publish `rank-soft` as new crate
2. Add deprecation notice to `rank-relax` pointing to `rank-soft`
3. Maintain `rank-relax` for compatibility (re-export from `rank-soft`)
4. Eventually archive `rank-relax` after migration period

**Pros:**
- ✅ Better name (`rank-soft` is more intuitive)
- ✅ Consistent with function names
- ✅ Better discoverability
- ✅ Provides migration path

**Cons:**
- ❌ Breaking change for existing users
- ❌ Requires maintaining two crates during transition
- ❌ Ecosystem fragmentation during migration
- ❌ More complex release process

**Verdict**: **Better long-term but disruptive**

### Option 3: Keep `rank-relax`, Improve Documentation

**Strategy:**
- Keep crate name as `rank-relax`
- Emphasize "soft" in documentation and examples
- Use "soft" terminology in user-facing docs
- Keep "relaxation" in technical/mathematical descriptions

**Example Documentation:**
```rust
//! # rank-relax
//!
//! **Soft ranking and sorting operations** for machine learning.
//!
//! This crate provides smooth relaxations (continuous approximations) of
//! discrete ranking operations, enabling gradient-based optimization.
//!
//! ## Quick Start
//!
//! ```rust
//! use rank_relax::soft_rank;  // Note: function is "soft_rank"
//! ```
```

**Pros:**
- ✅ No breaking change
- ✅ Can emphasize "soft" in user-facing content
- ✅ Maintains technical accuracy ("relaxation" in docs)
- ✅ Best of both worlds

**Cons:**
- ⚠️ Crate name still less intuitive
- ⚠️ Some inconsistency (crate "relax", functions "soft")

**Verdict**: **Pragmatic compromise**

## Recommendation: Option 3 (Keep `rank-relax`, Improve Documentation)

### Reasoning

1. **Published Constraint**: Breaking changes are costly for users
2. **Function Names Already Use "Soft"**: `soft_rank()`, `soft_sort()` - users see "soft" in API
3. **Documentation Can Bridge Gap**: Emphasize "soft" in user-facing docs
4. **Technical Accuracy**: "Relaxation" is mathematically precise
5. **Ecosystem Stability**: Avoid disrupting existing users

### Implementation Strategy

**Crate Name**: `rank-relax` (keep as-is)

**Documentation Strategy**:
- **Title/Subtitle**: "Soft ranking and sorting operations"
- **Function names**: Keep `soft_rank()`, `soft_sort()` (already correct)
- **User-facing docs**: Use "soft" terminology
- **Technical docs**: Use "relaxation" for mathematical precision

**Example README Structure**:
```markdown
# rank-relax

**Soft ranking and sorting operations for machine learning**

This crate provides smooth relaxations of discrete ranking operations...

## Quick Start

```rust
use rank_relax::soft_rank;  // "soft" in function name
```

## Technical Details

The crate implements continuous relaxations of discrete operations...
```

**Key Points**:
- Users see "soft" in function names (`soft_rank`)
- Users see "soft" in documentation ("Soft ranking")
- Technical accuracy maintained ("relaxation" in mathematical descriptions)
- No breaking changes

## Alternative: If Breaking Change is Acceptable

If the user base is small and breaking changes are acceptable:

**Option 2B: Clean Migration**
1. Publish `rank-soft` v0.1.0 (new crate)
2. Update `rank-relax` to v0.2.0 with deprecation notice
3. `rank-relax` v0.2.0 re-exports from `rank-soft`
4. Set `rank-relax` as deprecated on crates.io
5. After 6-12 months, archive `rank-relax`

**Migration Guide**:
```toml
# Old
[dependencies]
rank-relax = "0.1"

# New
[dependencies]
rank-soft = "0.1"
```

## Final Recommendation

**Keep `rank-relax`** but:
1. Emphasize "soft" in user-facing documentation
2. Keep function names as `soft_*` (already correct)
3. Use "relaxation" in technical/mathematical descriptions
4. Add clear examples showing "soft ranking" terminology

**Rationale**: 
- No breaking changes
- Users see "soft" where it matters (function names, docs)
- Maintains technical accuracy
- Preserves ecosystem stability

**The Name "relax" is Fine** because:
- Function names use "soft" (`soft_rank`)
- Documentation can emphasize "soft"
- "Relaxation" is technically accurate
- No ecosystem disruption

