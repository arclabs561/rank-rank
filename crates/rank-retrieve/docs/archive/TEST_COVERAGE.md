# Test Coverage for rank-retrieve Optimizations

This document describes the comprehensive test suite added to verify correctness, properties, and edge cases for all optimizations in `rank-retrieve`.

## Test Files

### 1. `tests/property_tests.rs` (32 tests)

Property-based tests verifying mathematical and algorithmic invariants.

#### SIMD Dense Vector Properties
- **Commutativity**: `dot(a, b) == dot(b, a)`
- **Distributivity**: `dot(a, b + c) ≈ dot(a, b) + dot(a, c)`
- **Finite values**: All dot products return finite values
- **Zero vector handling**: Dot with zero vector is zero
- **Negative values**: Correct handling of negative vector components
- **Cosine range**: Cosine similarity in `[-1, 1]` (allowing floating-point error)
- **Identical vectors**: Cosine of identical vectors is 1.0
- **Orthogonal vectors**: Cosine of orthogonal vectors is 0.0
- **Opposite vectors**: Cosine of opposite vectors is -1.0
- **Norm properties**: Non-negative, zero vector has zero norm
- **Mismatched lengths**: Correct handling of different-length vectors

#### SIMD Sparse Vector Properties
- **Commutativity**: `sparse_dot(a, b) == sparse_dot(b, a)`
- **Finite values**: All sparse dot products return finite values
- **No overlap**: Sparse dot product of non-overlapping vectors is 0.0
- **Empty vectors**: Sparse dot product with empty vector is 0.0

#### BM25 Retrieval Properties
- **Sorted results**: Results sorted by score descending
- **No duplicates**: No duplicate document IDs in results
- **Finite scores**: All scores are finite
- **Non-negative scores**: BM25 scores are non-negative
- **k=0 handling**: k=0 returns empty results
- **k > num_docs**: Returns all matching documents when k exceeds matches
- **Early termination correctness**: Top-k results are correct
- **Precomputed IDF correctness**: Precomputed IDF matches on-the-fly calculation
- **Retrieval consistency**: Multiple retrievals produce identical results
- **Query term ordering**: Query term order doesn't affect results

#### Dense Retrieval Properties
- **Sorted results**: Results sorted by score descending
- **No duplicates**: No duplicate document IDs
- **Score range**: Scores (cosine similarity) in `[-1, 1]`

#### Sparse Retrieval Properties
- **Sorted results**: Results sorted by score descending
- **No duplicates**: No duplicate document IDs
- **Finite scores**: All scores are finite

### 2. `tests/edge_case_tests.rs` (32 tests)

Edge case tests for boundary conditions and extreme inputs.

#### SIMD Edge Cases
- Empty vectors
- Single-element vectors
- Very large vectors (10,000+ dimensions)
- Very small values (near subnormal range)
- Very large values (near overflow)
- Mixed positive/negative values
- Zero norm vectors
- Very small norm vectors (below epsilon)

#### BM25 Edge Cases
- Empty index
- Empty query
- Single document
- Single query term
- No matching documents
- Very long queries (50+ terms)
- Very large index (1000+ documents)
- Duplicate query terms
- Special characters in terms

#### Dense Retrieval Edge Cases
- Empty retriever
- Empty query
- Dimension mismatch
- Single document
- Very high dimension (10,000+)

#### Sparse Retrieval Edge Cases
- Empty retriever
- Empty query
- Single document
- No overlap between query and documents
- Very sparse vectors

### 3. `tests/invariant_tests.rs` (13 tests)

Invariant tests for mathematical properties and consistency.

### 4. `tests/mathematical_property_tests.rs` (19 tests)

Mathematical and theoretical property tests verifying formal mathematical properties, theoretical guarantees, and invariants.

#### BM25 Mathematical Properties
- **IDF Properties**: Non-negativity, monotonicity in document frequency, upper bounds
- **Term Frequency Saturation**: Monotonicity, saturation behavior as tf increases
- **Length Normalization Bounds**: Behavior at parameter boundaries (b=0, b=1)
- **Score Additivity**: Multi-term queries = sum of single-term contributions, commutativity
- **Parameter Bounds**: Theoretical bounds on k1 and b parameters
- **Score Continuity**: Small parameter changes produce small score changes

#### Vector Space Mathematical Properties
- **Cauchy-Schwarz Inequality**: `|a · b| <= ||a|| * ||b||`, cosine bounds
- **Dot Product Bilinearity**: `dot(αa + βb, c) = α*dot(a,c) + β*dot(b,c)`
- **Triangle Inequality**: `||a + b|| <= ||a|| + ||b||`
- **Norm Homogeneity**: `||αa|| = |α| * ||a||`
- **Cosine Angle Properties**: Parallel (cos=1), orthogonal (cos=0), opposite (cos=-1)
- **Dot Product Symmetry**: `dot(a, b) = dot(b, a)`
- **Norm Positive Definiteness**: `||a|| >= 0`, `||a|| = 0` iff `a = 0`

#### Sparse Vector Mathematical Properties
- **Sparse Dot Product Linearity**: `sparse_dot(αa, c) = α * sparse_dot(a, c)`
- **Sparse Dot Product Symmetry**: `sparse_dot(a, b) = sparse_dot(b, a)`

#### Retrieval Algorithm Theoretical Properties
- **Retrieval Monotonicity**: More relevant documents score higher
- **Early Termination Optimality**: Correct top-k retrieval with min-heap
- **Score Continuity**: Continuous in parameters (important for tuning)

#### Information-Theoretic Properties
- **IDF Information Content**: Rare terms (high IDF) carry more information

#### Convergence and Stability Properties
- **Score Convergence**: BM25 scores converge as document count increases

#### Score Ordering Invariants
- **Monotonicity with k**: Increasing k doesn't decrease scores of existing results
- **Top-k consistency**: Top-k from larger k matches top-k from smaller k

#### Consistency Invariants
- **Idempotency**: Multiple retrievals with same query produce identical results
- **Relative ordering preserved**: Adding unrelated documents preserves relative ordering

#### Bounds Invariants
- **Cosine bounds**: Cosine similarity in `[-1, 1]` across many test cases
- **Norm non-negative**: Norm always non-negative across many test cases
- **BM25 scores non-negative**: BM25 scores always non-negative

#### Mathematical Properties
- **Dot product linearity**: `dot(alpha * a, b) = alpha * dot(a, b)`
- **Cosine symmetry**: `cosine(a, b) == cosine(b, a)`

#### Early Termination Correctness
- **Top-k preservation**: Early termination produces correct top-k (not just any k results)

#### Precomputed IDF Correctness
- **IDF consistency**: Precomputed IDF values are consistent with expectations

## Test Statistics

- **Total test files**: 4
- **Total tests**: 96+ (property: 32, edge case: 32, invariant: 13, mathematical: 19)
- **Coverage areas**:
  - SIMD dense operations: 15+ tests
  - SIMD sparse operations: 5+ tests
  - BM25 retrieval: 25+ tests
  - Dense retrieval: 10+ tests
  - Sparse retrieval: 8+ tests

## Running Tests

```bash
# Run all property and behavior tests
cargo test --features bm25,dense,sparse --test property_tests --test edge_case_tests --test invariant_tests --test mathematical_property_tests

# Run specific test suite
cargo test --features bm25,dense,sparse --test property_tests
cargo test --features bm25,dense,sparse --test edge_case_tests
cargo test --features bm25,dense,sparse --test invariant_tests
cargo test --features bm25,dense,sparse --test mathematical_property_tests

# Run all tests (including unit tests)
cargo test --features bm25,dense,sparse --workspace
```

## Test Principles

1. **Correctness**: SIMD implementations match scalar/portable implementations
2. **Properties**: Mathematical properties (commutativity, linearity, etc.) are preserved
3. **Bounds**: All outputs are within expected ranges (finite, non-negative where appropriate)
4. **Consistency**: Multiple calls with same inputs produce same outputs
5. **Edge cases**: Boundary conditions (empty, single element, very large) are handled correctly
6. **Invariants**: Algorithmic invariants (sorted results, no duplicates) are maintained
7. **Mathematical Theory**: Formal mathematical properties (inequalities, identities, convergence) are verified
8. **Theoretical Guarantees**: Algorithmic correctness properties (optimality, monotonicity, stability) are tested

## Future Test Additions

Potential areas for additional testing:
- Property-based testing with `proptest` for randomized inputs
- Performance regression tests (ensure optimizations don't regress)
- Cross-platform testing (verify SIMD works on different architectures)
- Numerical stability tests (very small/large values, subnormal handling)
- Concurrency tests (if multi-threaded access is added)
