# Name Decision: "relax" vs "soft" - Comprehensive Analysis

## Research Findings

### Paper Terminology

**"Relaxation" is the primary technical term:**
- SoftSort paper: "A **Continuous Relaxation** for the argsort Operator"
- NeuralSort: "Stochastic Optimization of Sorting Networks via **Continuous Relaxations**"
- NeuralNDCG: "Differentiable **Relaxation** of Sorting"
- Papers consistently use "relaxation" when describing the mathematical operation

**"Soft" is used for naming and outputs:**
- SoftSort (operator name)
- "Soft permutation matrices"
- "Soft ranking"
- But the underlying concept is still "relaxation"

### Library Naming Patterns

| Library | Name | Terminology Used |
|---------|------|-------------------|
| diffsort | "diff" prefix | "Differentiable sorting" |
| softsort.pytorch | "soft" in name | "Continuous relaxation" (in docs) |
| fast-soft-sort | "soft" in name | "Differentiable sorting" |
| rank-relax (current) | "relax" | "Smooth relaxation" |

**Observation**: Libraries use "soft" in names, but papers use "relaxation" in technical descriptions.

### Current Codebase Usage

**Function names**: `soft_rank`, `soft_sort` (uses "soft")
**Crate name**: `rank-relax` (uses "relax")
**Documentation**: "Smooth relaxation" (uses "relaxation")

**Inconsistency**: The codebase mixes terminology.

## Mathematical Distinction

### "Relaxation" (Technical/Formal)
- **Focus**: The mathematical operation
- **Meaning**: Removing/weakening constraints (from optimization theory)
- **Context**: "Continuous relaxation of discrete operations"
- **Usage**: Papers, formal descriptions, mathematical theory

### "Soft" (Descriptive/Intuitive)
- **Focus**: The resulting output
- **Meaning**: Smooth, differentiable (vs hard, discrete)
- **Context**: "Soft permutation matrices", "soft ranking"
- **Usage**: Operator names, user-facing APIs, intuitive descriptions

## User Perspective

### Search Behavior
- Users searching for "differentiable ranking" → find both
- Users searching for "soft ranking" → more intuitive, common in ML
- Users searching for "relaxation ranking" → more technical, optimization-focused

### Discoverability
- **"soft"**: More intuitive, matches ML terminology (softmax, soft attention)
- **"relax"**: More technical, matches optimization terminology

### API Clarity
- `soft_rank()` → Clear: produces soft (smooth) ranks
- `relax_rank()` → Less clear: what does "relax" mean to users?

## Industry Standards

### Machine Learning Community
- **"Soft"** dominates in naming: softmax, soft attention, soft sorting
- **"Relaxation"** used in papers but not in API names
- Users expect "soft" in function/operator names

### Optimization Community
- **"Relaxation"** is the standard term
- More formal, mathematically precise
- Used in academic contexts

## Recommendation Analysis

### Option 1: "rank-soft" (Recommended)

**Pros:**
- ✅ Matches ML community expectations (softmax, soft attention)
- ✅ More intuitive for users ("soft ranking" is clear)
- ✅ Consistent with function names (`soft_rank`, `soft_sort`)
- ✅ Better discoverability (users search for "soft")
- ✅ Matches library naming patterns (softsort, fast-soft-sort)
- ✅ Clearer API (`soft_rank()` vs `relax_rank()`)

**Cons:**
- ⚠️ Less technically precise (papers use "relaxation")
- ⚠️ Doesn't emphasize the optimization theory connection

**Verdict**: **Better for users, matches industry patterns**

### Option 2: "rank-relax"

**Pros:**
- ✅ Matches paper terminology ("continuous relaxation")
- ✅ More technically precise
- ✅ Emphasizes optimization theory connection
- ✅ Current name (less breaking change)

**Cons:**
- ❌ Less intuitive ("relax" is ambiguous)
- ❌ Inconsistent with function names (`soft_rank` not `relax_rank`)
- ❌ Less discoverable (users don't search for "relax")
- ❌ Doesn't match library naming patterns

**Verdict**: **More technically accurate but less user-friendly**

### Option 3: Hybrid Approach

**Crate name**: `rank-soft`
**Documentation**: Use "relaxation" in technical descriptions
**Function names**: Keep `soft_rank`, `soft_sort`

**Example**:
```rust
// Crate: rank-soft
// Function: soft_rank()
// Docs: "This function implements a continuous relaxation of discrete ranking"
```

**Pros:**
- ✅ Best of both worlds
- ✅ User-friendly name
- ✅ Technically accurate documentation
- ✅ Matches industry patterns

**Verdict**: **Optimal balance**

## Final Recommendation: "rank-soft"

### Reasoning

1. **User Experience**: "Soft" is more intuitive and matches ML terminology
2. **Discoverability**: Users search for "soft ranking", not "relaxation ranking"
3. **Consistency**: Matches function names (`soft_rank`, `soft_sort`)
4. **Industry Patterns**: Libraries use "soft" in names (softsort, fast-soft-sort)
5. **API Clarity**: `soft_rank()` is clearer than `relax_rank()`

### Documentation Strategy

Use "relaxation" in technical documentation to maintain mathematical precision:

```rust
/// Computes soft ranks using a continuous relaxation of discrete ranking.
/// 
/// This implements a smooth relaxation that enables gradient flow while
/// preserving ranking semantics. The relaxation parameter controls the
/// sharpness of the approximation.
pub fn soft_rank(values: &[f32], regularization: f32) -> Vec<f32>
```

**Key Points**:
- Crate name: `rank-soft` (user-facing, intuitive)
- Function names: `soft_*` (consistent, clear)
- Documentation: Use "relaxation" in technical descriptions (mathematically precise)

## Conclusion

**Recommendation: Use "rank-soft"**

The name "soft" is:
- More intuitive for users
- Better for discoverability
- Consistent with function names
- Matches industry patterns

But maintain mathematical precision in documentation by using "relaxation" in technical descriptions. This gives us:
- User-friendly naming
- Technical accuracy
- Industry alignment
- Clear API

**The nuance**: "Soft" for naming (user-facing), "relaxation" for description (technical accuracy).

