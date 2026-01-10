# Research-Based Recommendations for rank-retrieve API Design

This document synthesizes research on Rust library API design patterns, analysis of the `rank-rank` ecosystem, and examination of successful Rust crates to provide evidence-based recommendations.

## Key Findings

### 1. rank-fusion Uses Concrete Functions + Enum Dispatch

**Pattern:**
```rust
// Concrete functions (primary API)
pub fn rrf(a: &[(I, f32)], b: &[(I, f32)]) -> Vec<(I, f32)>
pub fn combsum(a: &[(I, f32)], b: &[(I, f32)]) -> Vec<(I, f32)>

// Enum for unified dispatch (convenience)
pub enum FusionMethod {
    Rrf { k: u32 },
    CombSum,
    // ...
}

impl FusionMethod {
    pub fn fuse<I>(&self, a: &[(I, f32)], b: &[(I, f32)]) -> Vec<(I, f32)> {
        match self {
            Self::Rrf { k } => rrf(a, b),
            Self::CombSum => combsum(a, b),
            // ...
        }
    }
}
```

**Why it works:**
- All inputs are the same type: `&[(I, f32)]`
- All outputs are the same type: `Vec<(I, f32)>`
- Enum provides convenience without abstraction overhead
- Concrete functions are discoverable and simple

**Comparison to rank-retrieve:**
- rank-retrieve has incompatible input types (`Vec<String>` vs `&[f32]` vs `SparseVector`)
- Can't use enum dispatch because query types differ
- Trait doesn't help because users still need concrete types to construct queries

### 2. Real-World Usage: All Examples Use Concrete Types

**Evidence from codebase:**
- `examples/basic_retrieval.rs`: Direct method calls on concrete types
- `examples/hybrid_retrieval.rs`: `HybridRetriever` struct with concrete fields
- `examples/full_pipeline.rs`: Direct calls to `index.retrieve()` and `retriever.retrieve()`
- `tests/integration_tests.rs`: All tests use concrete types
- **Zero trait usage found in examples or tests**

**Implication:**
The trait exists but isn't used. Users naturally gravitate to concrete types because:
1. Query construction requires type knowledge anyway
2. Concrete types are simpler and more discoverable
3. No benefit from trait abstraction in practice

### 3. Research: Traits vs Concrete Functions

**When traits are valuable:**
- Extensibility: Users need to implement custom types
- Dependency injection: Testing with mocks
- Runtime polymorphism: Heterogeneous collections
- Shared behavior: Multiple types with compatible operations

**When concrete functions are better:**
- Incompatible input types: Different operations, not shared behavior
- No extensibility needed: Closed set of implementations
- Performance critical: Static dispatch preferred
- Simplicity: Easier to understand and use

**Research finding:**
> "The distinction between concrete functions and trait abstractions fundamentally shapes how libraries expose functionality. Successful libraries employ multiple strategies simultaneously, choosing appropriate mechanisms based on factors including the degree of type heterogeneity, performance requirements, the need for extensibility, and anticipated API evolution."

**For rank-retrieve:**
- Input types are incompatible (text vs embeddings vs sparse vectors)
- No runtime polymorphism needed (all types known at compile time)
- No extensibility requirement (closed set of retrieval methods)
- Performance matters (first-stage retrieval is hot path)

**Conclusion:** Concrete functions are the better fit.

### 4. The Facade Pattern: Multi-Level APIs

**Example from reqwest:**
```rust
// Simple, concrete API for common cases
pub fn get(url: &str) -> RequestBuilder

// Trait-based API for extensibility
pub trait Body { }
impl Body for Vec<u8> { }
impl Body for String { }
```

**Pattern:**
- Primary API: Concrete functions for common use cases
- Secondary API: Traits for extensibility when needed

**For rank-retrieve:**
- Primary API: Concrete functions (`retrieve_bm25()`, `retrieve_dense()`, etc.)
- Secondary API: `Backend` trait for external integrations (if needed)

### 5. Enum Dispatch: Middle Ground

**rank-fusion pattern:**
```rust
pub enum FusionMethod {
    Rrf { k: u32 },
    CombSum,
    // ...
}

impl FusionMethod {
    pub fn fuse(&self, a: &[(I, f32)], b: &[(I, f32)]) -> Vec<(I, f32)> {
        match self { /* dispatch */ }
    }
}
```

**Why it works for fusion:**
- All inputs are `&[(I, f32)]` - same type
- Enum provides unified dispatch
- No trait overhead

**Why it doesn't work for retrieval:**
- Input types differ: `Vec<String>` vs `&[f32]` vs `SparseVector`
- Can't have unified enum because query types are incompatible
- Would need separate enums per query type (defeats purpose)

### 6. Workspace Consistency

**All other rank-* crates:**
- `default = []` (minimal defaults)
- Concrete functions for core operations
- Feature-gated implementations
- No trait abstraction for core API

**rank-retrieve (current):**
- `default = ["bm25", "dense", "sparse"]` (violates pattern)
- Trait abstraction for core operations (inconsistent)
- Feature-gated modules (inconsistent)

**Recommendation:** Align with workspace pattern.

## Recommended Path Forward

### Option 1: Concrete Functions (Recommended)

**Primary API:**
```rust
// Simple, direct, matches rank-fusion pattern
pub fn retrieve_bm25(
    index: &InvertedIndex,
    query: &[String],
    k: usize,
    params: Bm25Params,
) -> Result<Vec<(u32, f32)>, RetrieveError>

pub fn retrieve_dense(
    retriever: &DenseRetriever,
    query: &[f32],
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError>

pub fn retrieve_sparse(
    retriever: &SparseRetriever,
    query: &SparseVector,
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError>
```

**Benefits:**
- Consistent with `rank-fusion` and `rank-rerank`
- Simpler API (no trait complexity)
- More discoverable (concrete function names)
- Better performance (static dispatch)
- Fixes default features issue
- Matches actual usage patterns

**Keep:**
- `Backend` trait for external integrations (if actually needed)
- Feature-gated implementations
- Consistent output format: `Vec<(u32, f32)>`

**Remove:**
- `Retriever` trait (not used, doesn't help)
- `RetrieverBuilder` trait (not needed with concrete functions)

### Option 2: Hybrid Approach (Alternative)

**Primary API: Concrete functions**
```rust
pub fn retrieve_bm25(...) -> Result<Vec<(u32, f32)>, RetrieveError>
pub fn retrieve_dense(...) -> Result<Vec<(u32, f32)>, RetrieveError>
```

**Secondary API: Trait for extensibility**
```rust
pub trait Retriever {
    type Query: ?Sized;
    fn retrieve(&self, query: &Self::Query, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError>;
}
```

**Why this might work:**
- Concrete functions for common cases (primary)
- Trait for custom implementations (secondary)
- Matches facade pattern from reqwest

**Why it might not:**
- Still has trait complexity
- Query types are incompatible (trait doesn't help)
- No evidence users need custom implementations

### Option 3: Keep Trait, Fix Issues

If keeping the trait:

1. **Fix defaults:** `default = []` (match workspace)
2. **Make modules public:** Gate implementations, not modules
3. **Document limitations:** Query types incompatible, trait for custom impls only
4. **Remove `Backend` trait:** Redundant with `Retriever` for dense
5. **Accept reality:** Trait doesn't enable polymorphism

**Why this is suboptimal:**
- Trait adds complexity without benefit
- Still violates workspace pattern
- Doesn't match actual usage

## Implementation Plan (Option 1)

### Phase 1: Add Concrete Functions

```rust
// src/lib.rs or src/retrieve.rs
pub fn retrieve_bm25(
    index: &InvertedIndex,
    query: &[String],
    k: usize,
    params: Bm25Params,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    index.retrieve(query, k, params)
}

pub fn retrieve_dense(
    retriever: &DenseRetriever,
    query: &[f32],
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    retriever.retrieve(query, k)
}

pub fn retrieve_sparse(
    retriever: &SparseRetriever,
    query: &SparseVector,
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    retriever.retrieve(query, k)
}
```

### Phase 2: Update Examples

Update examples to use concrete functions:
```rust
// Before
let results = index.retrieve(&query, 10, Bm25Params::default())?;

// After
let results = retrieve_bm25(&index, &query, 10, Bm25Params::default())?;
```

### Phase 3: Fix Defaults

```toml
[features]
default = []  # Match workspace pattern
```

### Phase 4: Make Modules Public

```rust
// src/lib.rs
pub mod bm25;  // Always available
pub mod dense;  // Always available
pub mod sparse;  // Always available

// Gate implementations, not modules
#[cfg(feature = "bm25")]
impl InvertedIndex { /* ... */ }
```

### Phase 5: Deprecate Trait (Optional)

Keep trait for backward compatibility but mark as deprecated:
```rust
#[deprecated(note = "Use concrete functions instead: retrieve_bm25(), retrieve_dense(), etc.")]
pub trait Retriever { /* ... */ }
```

## Comparison with Successful Crates

### reqwest
- **Core API:** Concrete functions (`get()`, `post()`, etc.)
- **Extensibility:** `Body` trait for custom request bodies
- **Pattern:** Facade (simple API, extensible when needed)

### serde
- **Core API:** Traits (`Serialize`, `Deserialize`)
- **Why traits work:** All types serialize/deserialize the same way
- **Pattern:** Trait-based (shared behavior across types)

### image
- **Core API:** Concrete functions + enums (`DynamicImage`)
- **Extensibility:** Traits for format encoders/decoders
- **Pattern:** Hybrid (concrete for common cases, traits for extensibility)

### rank-fusion
- **Core API:** Concrete functions (`rrf()`, `combsum()`, etc.)
- **Convenience:** Enum for unified dispatch
- **Pattern:** Concrete functions (all inputs same type)

### rank-retrieve (recommended)
- **Core API:** Concrete functions (`retrieve_bm25()`, `retrieve_dense()`, etc.)
- **Extensibility:** `Backend` trait for external integrations (if needed)
- **Pattern:** Concrete functions (incompatible input types)

## Decision Matrix

| Factor | Concrete Functions | Trait-Based | Hybrid |
|-------|-------------------|-------------|--------|
| **Simplicity** | ✅ High | ❌ Low | ⚠️ Medium |
| **Discoverability** | ✅ High | ❌ Low | ⚠️ Medium |
| **Performance** | ✅ Static dispatch | ⚠️ Depends | ✅ Static dispatch |
| **Extensibility** | ❌ None | ✅ High | ✅ High |
| **Workspace consistency** | ✅ Matches | ❌ Inconsistent | ⚠️ Partial |
| **Usage patterns** | ✅ Matches | ❌ Doesn't match | ⚠️ Partial |
| **Query type compatibility** | ✅ Handles | ❌ Doesn't help | ❌ Doesn't help |

## Final Recommendation

**Choose Option 1: Concrete Functions**

**Rationale:**
1. **Matches ecosystem:** Consistent with `rank-fusion` and `rank-rerank`
2. **Matches usage:** All examples use concrete types
3. **Handles incompatibility:** Different functions for different query types
4. **Simpler:** No trait complexity
5. **Better performance:** Static dispatch
6. **Fixes issues:** Defaults, module gating, workspace consistency

**The value proposition:**
- "Multiple retrieval methods in one crate" (concrete functions)
- "Consistent output format" (all return `Vec<(u32, f32)>`)
- "Easy integration with rank-fusion" (same output format)
- Not: "Unified trait interface" (query types are incompatible)

**Implementation:**
1. Add concrete functions as primary API
2. Fix defaults to `[]`
3. Make modules public (gate implementations)
4. Update examples and documentation
5. Keep `Backend` trait for external integrations (if needed)
6. Deprecate `Retriever` trait (optional, for backward compatibility)

This approach aligns with research findings, matches successful patterns from other crates, and addresses all identified issues while maintaining simplicity and performance.

