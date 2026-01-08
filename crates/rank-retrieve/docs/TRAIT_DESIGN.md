# Trait-Based Design

**Note (2025-01):** The `Retriever` trait is **deprecated** in favor of concrete functions (`retrieve_bm25()`, `retrieve_dense()`, etc.). This document is kept for historical context and for users who need to understand the trait interface for backward compatibility or custom implementations.

**Current API:** Use concrete functions as the primary interface. See [README.md](../README.md) for examples.

## Core Design

`rank-retrieve` is built around a **trait interface** that is always available, with **feature-gated implementations**.

### Always Available

- `Retriever` trait: Core interface for all retrieval methods
- `RetrieverBuilder` trait: Interface for adding documents
- `RetrieveError`: Error types
- `integration::Backend`: Trait for external backend integration

### Feature-Gated Implementations

- `bm25`: BM25 retrieval (`InvertedIndex`)
- `dense`: Dense retrieval (`DenseRetriever`)
- `sparse`: Sparse retrieval (`SparseRetriever`)
- `generative`: Generative retrieval (`GenerativeRetriever`)

## Usage Patterns

### Minimal Usage (Trait Only)

```rust
use rank_retrieve::retriever::Retriever;

// Trait is available even without implementations
fn search<R: Retriever>(retriever: &R, query: &R::Query, k: usize) {
    // Works with any retriever
}
```

### With Specific Implementation

```rust
use rank_retrieve::prelude::*;

// Requires bm25 feature
let mut index = InvertedIndex::new();
index.add_document(0, vec!["test".to_string()]).unwrap();
let results = index.retrieve(&vec!["test".to_string()], 10).unwrap();
```

### Polymorphic Code

```rust
use rank_retrieve::retriever::Retriever;

fn hybrid_search<R1: Retriever, R2: Retriever>(
    retriever1: &R1,
    retriever2: &R2,
    query1: &R1::Query,
    query2: &R2::Query,
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    let results1 = retriever1.retrieve(query1, k)?;
    let results2 = retriever2.retrieve(query2, k)?;
    // Combine results...
    Ok(results1)
}
```

## Feature Flags

### Default Features

**Current (2025-01):** By default, **no features are enabled** (`default = []`) to match workspace pattern:

```toml
rank-retrieve = { path = "..." }  # No features enabled by default
```

### Minimal Build

For trait-only usage or custom implementations, no special configuration needed:

```toml
rank-retrieve = { path = "..." }  # Already minimal by default
```

### Enable Specific Features

Enable only what you need:

```toml
rank-retrieve = { path = "...", features = ["bm25"] }
```

### All Features

```toml
rank-retrieve = { path = "...", features = ["all"] }  # Includes generative
```

## Implementing Custom Retrievers

You can implement the `Retriever` trait for your own backends:

```rust
use rank_retrieve::retriever::{Retriever, RetrieverBuilder};
use rank_retrieve::RetrieveError;

struct MyCustomBackend {
    // Your implementation
}

impl Retriever for MyCustomBackend {
    type Query = &'static str;  // Or Vec<f32>, SparseVector, etc.

    fn retrieve(&self, query: &Self::Query, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        // Your retrieval logic
        Ok(vec![])
    }
}

impl RetrieverBuilder for MyCustomBackend {
    type Content = String;

    fn add_document(&mut self, doc_id: u32, content: Self::Content) -> Result<(), RetrieveError> {
        // Your indexing logic
        Ok(())
    }
}
```

## Benefits

1. **Lightweight**: Use trait only without pulling in implementations
2. **Polymorphic**: Write code that works with any retriever
3. **Flexible**: Choose only the implementations you need
4. **Extensible**: Easy to add custom retrievers
5. **Ecosystem integration**: Works seamlessly with `rank-fusion`, `rank-rerank`

## Testing

Tests are feature-gated. When testing with specific implementations:

```rust
#[cfg(feature = "bm25")]
#[test]
fn test_bm25() {
    // BM25-specific test
}
```

The trait interface can be tested without any features:

```rust
#[test]
fn test_trait_available() {
    use rank_retrieve::retriever::Retriever;
    // Trait is always available
}
```

