# Cross-Crate Test Helpers

## Overview

The test helpers in `test_helpers.rs` are designed to work across all rank-* crates:
- `rank-retrieve`: BM25, dense, sparse retrieval
- `rank-fusion`: Result fusion (RRF, CombSUM, etc.)
- `rank-rerank`: Reranking with ColBERT, MaxSim, etc.
- `rank-eval`: Evaluation metrics (nDCG, Precision, Recall, MRR, MAP)

## Generic Design

The helpers are generic over ID types to work with different result formats:

- `rank-retrieve`: Uses `u32` IDs → `Vec<(u32, f32)>`
- `rank-fusion`: Uses `String` or `&str` IDs → `Vec<(String, f32)>` or `Vec<(&str, f32)>`
- `rank-rerank`: Uses `String` IDs → `Vec<(String, f32)>`
- `rank-eval`: Works with any ID type that implements `Display`

## Usage Across Crates

### rank-retrieve

```rust
use rank_retrieve::test_helpers::*;
use rank_retrieve::{retrieve_bm25, retrieve_dense};
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};

let collection = TestCollection::machine_learning_collection();
let scenario = TestScenario::new(collection);

let query = scenario.get_query("q1").unwrap();
let results = retrieve_bm25(&scenario.index, &query, 10, Bm25Params::default()).unwrap();

let ranked = results.to_ranked_list(); // Vec<(u32, f32)> → Vec<String>
let relevant = scenario.get_relevant("q1").unwrap();
let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
```

### rank-fusion

```rust
// In rank-fusion tests, you can use the helpers for evaluation
use rank_fusion::rrf;
use rank_eval::binary::ndcg_at_k;

let bm25_results: Vec<(String, f32)> = vec![
    ("doc1".to_string(), 12.5),
    ("doc2".to_string(), 11.0),
];
let dense_results: Vec<(String, f32)> = vec![
    ("doc2".to_string(), 0.95),
    ("doc1".to_string(), 0.88),
];

let fused = rrf(&bm25_results, &dense_results);

// Use ToRankedList trait
let ranked = fused.to_ranked_list(); // Vec<(String, f32)> → Vec<String>
let relevant: HashSet<String> = ["doc1", "doc2"].iter().map(|s| s.to_string()).collect();
let ndcg = ndcg_at_k(&ranked, &relevant, 10);
```

### rank-rerank

```rust
use rank_rerank::explain::{rerank_batch, Candidate, RerankerInput};
use rank_eval::binary::ndcg_at_k;

// After reranking, convert to ranked list
let reranked: Vec<(String, f32)> = /* ... */;

let ranked = reranked.to_ranked_list();
let relevant: HashSet<String> = relevant_set_str(&["doc1", "doc2"]);
let ndcg = ndcg_at_k(&ranked, &relevant, 10);
```

### rank-eval

```rust
use rank_eval::binary::*;

// Works with any ID type via ToRankedList
let results: Vec<(u64, f32)> = vec![(1001, 0.9), (1002, 0.8)];
let ranked = results.to_ranked_list();
let relevant: HashSet<String> = ["1001", "1002"].iter().map(|s| s.to_string()).collect();
let precision = precision_at_k(&ranked, &relevant, 10);
```

## Shared Patterns

### Test Collection Pattern

All crates can use `TestCollection` for consistent test data:

```rust
let collection = TestCollection::machine_learning_collection();
// Contains:
// - documents: Vec<(ID, Vec<String>)>
// - queries: Vec<(String, Vec<String>)>
// - qrels: Vec<(String, HashSet<String>)>
```

### Evaluation Pattern

All crates can use `EvaluationResults` for comprehensive metrics:

```rust
let results: Vec<(String, f32)> = /* ... */;
let ranked = results.to_ranked_list();
let relevant: HashSet<String> = /* ... */;

let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
// Contains: precision@1, precision@5, precision@10, recall@5, recall@10,
//           ndcg@5, ndcg@10, mrr, map

assert!(eval.meets_thresholds(0.1, 0.1, 0.1));
```

### Result Conversion Pattern

All crates can use `ToRankedList` trait for consistent conversion:

```rust
// Works with any result type
let results_u32: Vec<(u32, f32)> = /* ... */;
let results_string: Vec<(String, f32)> = /* ... */;
let results_str: Vec<(&str, f32)> = /* ... */;
let results_u64: Vec<(u64, f32)> = /* ... */;

let ranked_u32 = results_u32.to_ranked_list();
let ranked_string = results_string.to_ranked_list();
let ranked_str = results_str.to_ranked_list();
let ranked_u64 = results_u64.to_ranked_list();
```

## Sharing Helpers Across Crates

### Option 1: Copy to Each Crate

Copy `test_helpers.rs` to each crate's `tests/` directory:

```bash
# In rank-fusion
cp ../rank-retrieve/tests/test_helpers.rs tests/

# In rank-rerank
cp ../rank-retrieve/tests/test_helpers.rs tests/

# In rank-eval
cp ../rank-retrieve/tests/test_helpers.rs tests/
```

### Option 2: Workspace-Level Shared Module

Create a shared test utilities crate (future enhancement):

```toml
# Cargo.toml
[dev-dependencies]
rank-test-helpers = { path = "../rank-test-helpers" }
```

### Option 3: Reference from rank-retrieve

Reference the helpers from rank-retrieve in other crates:

```rust
// In rank-fusion tests
#[path = "../../rank-retrieve/tests/test_helpers.rs"]
mod test_helpers;
```

## Common Test Scenarios

### Scenario 1: Retrieve → Fuse → Evaluate

```rust
// rank-retrieve
let bm25_results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
let dense_results = retrieve_dense(&retriever, &query_emb, 10).unwrap();

// rank-fusion
let fused = rrf(&bm25_results.to_ranked_list(), &dense_results.to_ranked_list());
// Note: Need to convert u32 to String for rank-fusion

// rank-eval
let ranked = fused.to_ranked_list();
let relevant = relevant_set(&[0, 1]);
let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
```

### Scenario 2: Fuse → Rerank → Evaluate

```rust
// rank-fusion
let fused: Vec<(String, f32)> = rrf(&bm25_results, &dense_results);

// rank-rerank
let candidates: Vec<Candidate<String>> = /* ... */;
let reranked = rerank_batch(&input).unwrap();

// rank-eval
let ranked = reranked.to_ranked_list();
let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
```

## Benefits

1. **Consistency**: Same test data and evaluation patterns across all crates
2. **Reduced Duplication**: Common setup code shared
3. **Type Safety**: Generic over ID types, works with all crates
4. **Maintainability**: Changes to test patterns in one place
5. **Readability**: Tests focus on what they're testing, not setup

## See Also

- `test_helpers.rs` - Full implementation
- `TEST_HELPERS.md` - Detailed documentation
- `example_with_helpers.rs` - Usage examples

