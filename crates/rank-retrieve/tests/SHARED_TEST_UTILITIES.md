# Shared Test Utilities Across rank-* Crates

## Summary

The test helpers in `test_helpers.rs` are designed to work across all rank-* crates in the workspace. They provide:

1. **Generic ID Types**: Work with `u32`, `String`, `&str`, `u64` and any `Display` type
2. **Consistent Evaluation**: Same evaluation patterns across all crates
3. **Reduced Duplication**: Common test setup code shared
4. **Type Safety**: Generic implementations ensure correctness

## Available Crates

- `rank-retrieve`: First-stage retrieval (BM25, dense, sparse)
- `rank-fusion`: Result fusion (RRF, CombSUM, etc.)
- `rank-rerank`: Reranking (ColBERT, MaxSim, etc.)
- `rank-eval`: Evaluation metrics (nDCG, Precision, Recall, MRR, MAP)
- `rank-soft`: Differentiable ranking operations and Learning to Rank algorithms (LambdaRank, Ranking SVM, neural LTR)

## Result Type Compatibility

| Crate | Result Type | Compatible |
|-------|-------------|------------|
| `rank-retrieve` | `Vec<(u32, f32)>` | ✅ Via `ToRankedList` |
| `rank-fusion` | `Vec<(String, f32)>` | ✅ Via `ToRankedList` |
| `rank-fusion` | `Vec<(&str, f32)>` | ✅ Via `ToRankedList` |
| `rank-rerank` | `Vec<(String, f32)>` | ✅ Via `ToRankedList` |
| `rank-eval` | Any `Display` type | ✅ Via `ToRankedList` |

## Usage Patterns

### Pattern 1: Retrieve → Evaluate

```rust
// rank-retrieve
let results: Vec<(u32, f32)> = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

// Convert and evaluate
let ranked = results.to_ranked_list();
let relevant = relevant_set(&[0, 1]);
let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
```

### Pattern 2: Fuse → Evaluate

```rust
// rank-fusion
let fused: Vec<(String, f32)> = rrf(&bm25_results, &dense_results);

// Evaluate
let ranked = fused.to_ranked_list();
let relevant: HashSet<String> = ["doc1", "doc2"].iter().map(|s| s.to_string()).collect();
let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
```

### Pattern 3: Retrieve → Fuse → Rerank → Evaluate

```rust
// rank-retrieve
let bm25_results: Vec<(u32, f32)> = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();

// Convert for fusion (rank-fusion uses String IDs)
let bm25_string: Vec<(String, f32)> = bm25_results.iter()
    .map(|(id, score)| (id.to_string(), *score))
    .collect();

// rank-fusion
let fused: Vec<(String, f32)> = rrf(&bm25_string, &dense_string);

// rank-rerank
let reranked: Vec<(String, f32)> = /* ... */;

// rank-eval
let ranked = reranked.to_ranked_list();
let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
```

## Helper Types Available

1. **`TestCollection<ID>`**: Labeled test collections with documents, queries, and relevance judgments
2. **`TestScenario`**: Builder for consistent test setups (rank-retrieve specific)
3. **`MultiRetrieverFixture`**: BM25, dense, sparse retrievers with same documents (rank-retrieve specific)
4. **`TestQuery`**: Query representation with type classification
5. **`EvaluationResults`**: All IR evaluation metrics in one struct
6. **`ToRankedList` trait**: Convert any result type to `Vec<String>` for evaluation
7. **Helper functions**: `relevant_set()`, `relevant_set_str()`, `results_to_ranked_list()`

## Sharing Strategy

### Current Approach

The helpers are in `rank-retrieve/tests/test_helpers.rs` and can be:

1. **Copied** to other crates' `tests/` directories
2. **Referenced** using `#[path = "..."]` in other crates
3. **Used as reference** for creating crate-specific versions

### Future Enhancement

Consider creating a shared test utilities crate:

```toml
# Cargo.toml (workspace level)
[dev-dependencies]
rank-test-helpers = { path = "crates/rank-test-helpers" }
```

## Benefits

1. **Consistency**: Same test patterns across all crates
2. **Maintainability**: Changes in one place benefit all crates
3. **Type Safety**: Generic implementations ensure correctness
4. **Readability**: Tests focus on what they test, not setup
5. **Reusability**: Write once, use everywhere

## Examples

See:
- `example_with_helpers.rs` - rank-retrieve specific examples
- `CROSS_CRATE_TEST_HELPERS.md` - Cross-crate usage examples
- `TEST_HELPERS.md` - Detailed documentation

## Integration Points

The helpers integrate with:
- `rank-eval::binary::*` - All binary metrics
- `rank-eval::graded::*` - Graded relevance metrics
- `rank-fusion::*` - All fusion algorithms
- `rank-rerank::*` - All reranking methods
- `rank-retrieve::*` - All retrieval methods

