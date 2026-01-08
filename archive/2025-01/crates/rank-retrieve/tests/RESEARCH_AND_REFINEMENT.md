# Test Helpers Research and Refinement

## Research Summary

Based on research into Rust testing best practices and analysis of patterns across rank-* crates, the test helpers have been enhanced with:

1. **Mock Embedding Generators** - Deterministic embedding generation for testing
2. **TREC Format Helpers** - Utilities for creating TREC-format test files
3. **Property-Based Testing Helpers** - Proptest strategies for generating test data
4. **Builder Patterns** - Fluent builders for complex test scenarios
5. **Test-Support Crate Pattern** - Documentation for future workspace-level sharing

## Patterns Found Across Crates

### rank-rerank Patterns

- `mock_dense_embed()` - Deterministic hash-based embeddings
- `mock_token_embed()` - Token-level embeddings for ColBERT
- `MockCrossEncoder` - Mock implementations for testing

### rank-fusion Patterns

- Property-based testing with `proptest`
- `arb_results()` - Strategies for generating test data
- Commutativity and invariance testing

### rank-eval Patterns

- TREC format file creation with `tempfile`
- `create_temp_trec_runs()` - Helper functions for TREC files
- `create_temp_trec_qrels()` - Qrel file creation

## Enhancements Added

### 1. Mock Embedding Generators

```rust
use test_helpers::{mock_dense_embed, mock_token_embed};

// Generate deterministic embeddings
let query_emb = mock_dense_embed("machine learning", 128);
let doc_emb = mock_dense_embed("deep learning algorithms", 128);

// Generate token embeddings for ColBERT
let query_tokens = mock_token_embed("machine learning", 128);
```

**Benefits:**
- Deterministic (same input → same output)
- No external dependencies
- Fast (no model loading)
- Works across rank-retrieve and rank-rerank

### 2. TREC Format Helpers

```rust
use test_helpers::{TrecRunEntry, TrecQrelEntry, results_to_trec_runs};

// Convert retrieval results to TREC format
let runs = results_to_trec_runs("q1", &results, "bm25");
for entry in &runs {
    writeln!(file, "{}", entry.to_trec_line()).unwrap();
}

// Create qrels
let qrel = TrecQrelEntry {
    query_id: "q1".to_string(),
    doc_id: "doc1".to_string(),
    relevance: 2,
};
writeln!(file, "{}", qrel.to_trec_line()).unwrap();
```

**Benefits:**
- Consistent TREC format across tests
- Works with rank-eval's TREC loading
- Reduces boilerplate

### 3. Property-Based Testing Helpers

```rust
#[cfg(feature = "proptest")]
use test_helpers::proptest_helpers::*;

proptest! {
    #[test]
    fn test_retrieval_invariants(
        results in arb_results(50, 100),
        query in arb_query(5)
    ) {
        // Test invariants
        prop_assert!(results.len() <= 50);
        prop_assert!(results.iter().all(|(_, score)| *score >= 0.0 && *score <= 1.0));
    }
}
```

**Benefits:**
- Reusable strategies across crates
- Consistent test data generation
- Works with rank-fusion's proptest patterns

### 4. Builder Patterns

```rust
use test_helpers::{ResultBuilder, TestCollectionBuilder};

// Build results
let results = ResultBuilder::new()
    .add(0, 0.9)
    .add(1, 0.8)
    .add(2, 0.7)
    .build();

// Build test collection
let collection = TestCollectionBuilder::new()
    .add_document("doc1", vec!["machine".to_string(), "learning".to_string()])
    .add_query("q1", vec!["machine".to_string(), "learning".to_string()])
    .add_qrel("q1", vec!["doc1"])
    .build();
```

**Benefits:**
- Fluent API for complex scenarios
- Type-safe construction
- Reduces setup code

## Test-Support Crate Pattern

Based on Rust best practices research, the recommended approach for sharing test utilities across a workspace is to create a dedicated test-support crate.

### Recommended Structure

```
workspace/
├── crates/
│   ├── rank-retrieve/
│   ├── rank-fusion/
│   ├── rank-rerank/
│   ├── rank-eval/
│   └── rank-test-helpers/  # NEW: Shared test utilities
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
```

### Implementation

**`rank-test-helpers/Cargo.toml`:**
```toml
[package]
name = "rank-test-helpers"
version = "0.1.0"
edition = "2021"

[dependencies]
rank-eval = { path = "../rank-eval" }
proptest = "1"
tempfile = "3"

[dev-dependencies]
# Test helpers crate itself is the "dev" piece
```

**Usage in other crates:**
```toml
[dev-dependencies]
rank-test-helpers = { path = "../rank-test-helpers" }
```

**Benefits:**
- Clean separation of test-only code
- No `#[path = "..."]` hacks
- Works with `cargo test` at workspace level
- Scales to many crates

## Current vs. Recommended Approach

### Current (rank-retrieve/tests/test_helpers.rs)

**Pros:**
- Immediate availability
- No workspace restructuring needed
- Works now

**Cons:**
- Requires copying to other crates
- Or using `#[path = "..."]` hacks
- Duplication risk

### Recommended (rank-test-helpers crate)

**Pros:**
- Single source of truth
- Clean dependencies
- Scales well
- Follows Rust best practices

**Cons:**
- Requires workspace restructuring
- Additional crate to maintain

## Migration Path

1. **Phase 1 (Current)**: Keep helpers in rank-retrieve, document usage
2. **Phase 2 (Future)**: Create rank-test-helpers crate
3. **Phase 3 (Future)**: Migrate all crates to use shared crate
4. **Phase 4 (Future)**: Remove duplicate helpers from individual crates

## Additional Patterns to Consider

### 1. Test Fixtures with rstest

```rust
use rstest::*;

#[fixture]
fn test_collection() -> TestCollection {
    TestCollection::machine_learning_collection()
}

#[rstest]
fn test_with_fixture(test_collection: TestCollection) {
    // Use fixture
}
```

### 2. Snapshot Testing

Consider adding snapshot testing for:
- TREC format output
- Evaluation results
- Fusion outputs

### 3. Performance Test Helpers

```rust
pub fn benchmark_retrieval<F>(f: F, iterations: usize) -> Duration
where
    F: Fn() -> Vec<(u32, f32)>,
{
    // Benchmark helper
}
```

## References

1. Rust Testing Best Practices - Test Support Crates
2. Proptest Book - Property-Based Testing Strategies
3. rank-rerank/tests/integration.rs - Mock embedding patterns
4. rank-fusion/src/proptests.rs - Property-based testing patterns
5. rank-eval/tests/integration_e2e.rs - TREC format patterns

