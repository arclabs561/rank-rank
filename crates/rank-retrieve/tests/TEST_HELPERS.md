# Test Helper Types

## Overview

The `test_helpers.rs` module provides types and utilities to reduce duplication and improve test organization. These helpers make tests more maintainable and easier to read.

**These helpers are designed to work across all rank-* crates:**
- `rank-retrieve`: BM25, dense, sparse retrieval
- `rank-fusion`: Result fusion
- `rank-rerank`: Reranking
- `rank-eval`: Evaluation metrics

See `CROSS_CRATE_TEST_HELPERS.md` for usage across all crates.

## Available Types

### `TestCollection`

Encapsulates a labeled test collection with documents, queries, and relevance judgments.

```rust
let collection = TestCollection::machine_learning_collection();
// Contains:
// - documents: Vec<(u32, Vec<String>)>
// - queries: Vec<(String, Vec<String>)>
// - qrels: Vec<(String, HashSet<String>)>
```

**Methods:**
- `machine_learning_collection()` - Pre-built collection for ML queries
- `head_tail_collection()` - Collection with head (common) and tail (rare) queries

**Benefits:**
- Eliminates repeated document/query setup
- Provides consistent test data across tests
- Includes ground truth relevance judgments

### `TestScenario`

Builder for creating consistent test setups from collections.

```rust
let collection = TestCollection::machine_learning_collection();
let scenario = TestScenario::new(collection);

// Get query and relevant documents
let query = scenario.get_query("q1").unwrap();
let relevant = scenario.get_relevant("q1").unwrap();
```

**Benefits:**
- Automatically builds index from collection
- Provides convenient accessors for queries and relevance
- Ensures consistency between setup and evaluation

### `MultiRetrieverFixture`

Encapsulates BM25, dense, and sparse retrievers with the same documents.

```rust
let fixture = MultiRetrieverFixture::machine_learning();
// Contains:
// - bm25_index: InvertedIndex
// - dense_retriever: DenseRetriever
// - sparse_retriever: SparseRetriever
// - collection: TestCollection
```

**Methods:**
- `machine_learning()` - Pre-built fixture with ML test collection

**Benefits:**
- Single setup for multi-retriever tests
- Ensures all retrievers use same documents
- Reduces boilerplate in comparison tests

### `TestQuery`

Query representation with type classification.

```rust
let lexical_query = TestQuery::lexical(vec!["machine".to_string(), "learning".to_string()]);
let short_query = TestQuery::short(vec!["learning".to_string()]);
let long_query = TestQuery::long(vec!["machine".to_string(), "learning".to_string(), ...]);
```

**Query Types:**
- `Lexical` - Exact keyword matches
- `Semantic` - Semantic similarity
- `Short` - 1-2 terms
- `Long` - 5+ terms
- `Head` - Common terms
- `Tail` - Rare terms

**Benefits:**
- Type-safe query classification
- Consistent query creation
- Easy query type filtering in tests

### `EvaluationResults`

Encapsulates all IR evaluation metrics for a query.

```rust
let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
let ranked = results.to_ranked_list();
let relevant = relevant_set(&[0, 1]);

let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
// Contains: precision@1, precision@5, precision@10, recall@5, recall@10,
//           ndcg@5, ndcg@10, mrr, map

// Check thresholds
assert!(eval.meets_thresholds(0.1, 0.1, 0.1));
```

**Benefits:**
- Calculates all metrics at once
- Provides threshold checking
- Reduces repeated metric calculation code

### `ToRankedList` Trait

Converts retrieval results to ranked list for evaluation.

```rust
let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
let ranked = results.to_ranked_list(); // Vec<String>
```

**Benefits:**
- Consistent conversion across all retrieval methods
- Reduces boilerplate conversion code

### Helper Functions

- `relevant_set(ids: &[u32]) -> HashSet<String>` - Create relevance set from document IDs

## Usage Examples

### Before: Manual Setup

```rust
#[test]
fn test_bm25_precision() {
    let mut index = InvertedIndex::new();
    index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    index.add_document(1, &["deep".to_string(), "learning".to_string()]);
    index.add_document(2, &["python".to_string(), "programming".to_string()]);
    
    let query = vec!["machine".to_string(), "learning".to_string()];
    let results = retrieve_bm25(&index, &query, 10, Bm25Params::default()).unwrap();
    
    let relevant: HashSet<String> = ["0", "1"].iter().map(|s| s.to_string()).collect();
    let ranked: Vec<String> = results.iter().map(|(id, _)| id.to_string()).collect();
    
    let precision = precision_at_k(&ranked, &relevant, 10);
    assert!(precision > 0.0);
}
```

### After: Using Helpers

```rust
#[test]
fn test_bm25_precision() {
    let collection = TestCollection::machine_learning_collection();
    let scenario = TestScenario::new(collection);
    
    let query = scenario.get_query("q1").unwrap();
    let relevant = scenario.get_relevant("q1").unwrap();
    
    let results = retrieve_bm25(&scenario.index, &query, 10, Bm25Params::default()).unwrap();
    let ranked = results.to_ranked_list();
    
    let eval = EvaluationResults::from_ranked("q1".to_string(), &ranked, &relevant);
    assert!(eval.precision_at_10 > 0.0);
}
```

## Benefits

1. **Reduced Duplication**: Common setup code is centralized
2. **Consistency**: All tests use same test data structure
3. **Maintainability**: Changes to test data only need to be made in one place
4. **Readability**: Tests focus on what they're testing, not setup
5. **Type Safety**: Query types and evaluation results are type-safe

## Migration Guide

To migrate existing tests to use helpers:

1. Replace manual document/query setup with `TestCollection`
2. Replace manual index creation with `TestScenario::new()`
3. Replace manual metric calculation with `EvaluationResults`
4. Replace manual result conversion with `ToRankedList` trait
5. Use `MultiRetrieverFixture` for multi-retriever tests

## See Also

- `example_with_helpers.rs` - Complete examples using all helper types
- `test_helpers.rs` - Full implementation and documentation

