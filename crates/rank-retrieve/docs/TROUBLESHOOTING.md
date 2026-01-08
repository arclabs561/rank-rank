# Troubleshooting Guide

Common issues and solutions for rank-retrieve.

## Retrieval Issues

### No Results Returned

**Symptoms**: Retrieval returns empty results.

**Solutions**:

1. **Check index is built**:
```rust
// Ensure index is populated
let index = bm25::Index::new(&documents)?;
assert!(!index.is_empty(), "Index is empty");
```

2. **Verify query is not empty**:
```rust
if query.is_empty() {
    return Vec::new(); // or handle appropriately
}
```

3. **Check document corpus**:
   - Ensure documents are not empty
   - Verify documents contain content

### Low Recall

**Symptoms**: Missing relevant documents in results.

**Solutions**:

1. **Increase candidate count**:
   - Retrieve more candidates (e.g., 1000 instead of 100)
   - First-stage should prioritize recall over precision

2. **Check indexing**:
   - Verify all documents are indexed
   - Check for indexing errors

3. **Use multiple retrievers**:
   - Combine BM25 + dense + sparse
   - Use rank-fusion to combine results

---

## BM25 Issues

### BM25 Scores Are Zero

**Symptoms**: All BM25 scores are 0.0.

**Solutions**:

1. **Check query terms**:
   - Query terms must appear in indexed documents
   - Verify tokenization matches indexing

2. **Verify index**:
   - Ensure index is built correctly
   - Check for empty index

3. **Check term matching**:
```rust
// Debug: Check which terms match
for term in query_terms {
    if index.contains_term(term) {
        println!("Term '{}' found in index", term);
    }
}
```

### BM25 Performance Issues

**Symptoms**: BM25 retrieval is slow.

**Solutions**:

1. **Optimize index**:
   - Use efficient data structures
   - Consider inverted index optimization

2. **Limit candidate count**:
   - Retrieve only what you need
   - Don't retrieve entire corpus

3. **Cache frequent queries**:
   - Cache results for common queries
   - Invalidate cache when index updates

---

## Dense Retrieval Issues

### Embedding Dimension Mismatch

**Symptoms**: Error with embedding dimensions.

**Solutions**:

1. **Verify dimensions match**:
```rust
let query_dim = query_embedding.len();
let doc_dim = document_embeddings[0].len();
assert_eq!(query_dim, doc_dim, 
    "Query and document embeddings must have same dimension");
```

2. **Check embedding model**:
   - Use same model for query and documents
   - Verify embedding extraction is correct

### Low Similarity Scores

**Symptoms**: All similarity scores are very low.

**Solutions**:

1. **Normalize embeddings**:
```rust
// Normalize to unit length for cosine similarity
let normalized = normalize_embedding(&embedding);
```

2. **Check embedding quality**:
   - Verify embeddings are from trained model
   - Check for embedding extraction errors

3. **Use appropriate similarity**:
   - Cosine similarity for normalized embeddings
   - Dot product for unnormalized embeddings

---

## Integration Issues

### Combining with rank-rerank

**Symptoms**: Retrieval results don't work well with reranking.

**Solutions**:

1. **Typical pipeline**:
```rust
// 1. First-stage retrieval (this crate)
let candidates = bm25_index.search(query, 1000)?;

// 2. Rerank with rank-rerank
let reranked = rerank_with_maxsim(query, &candidates[..100]);

// 3. Return top-k
let top_k = &reranked[..10];
```

2. **Retrieve enough candidates**:
   - First-stage: 100-1000 candidates (prioritize recall)
   - Reranking: 10-100 candidates (prioritize precision)

### Combining with rank-fusion

**Symptoms**: Multiple retrievers don't fuse well.

**Solutions**:

1. **Use RRF for fusion**:
```rust
use rank_fusion::rrf;

let bm25_results = bm25_index.search(query, 100)?;
let dense_results = dense_index.search(query_embedding, 100)?;
let fused = rrf(&bm25_results, &dense_results);
```

2. **Retrieve similar counts**:
   - Retrieve similar number of candidates from each system
   - Typical: 100-1000 from each

---

## Performance Issues

### Slow Retrieval

**Symptoms**: Retrieval operations take too long.

**Solutions**:

1. **Optimize index**:
   - Use efficient data structures
   - Consider approximate methods for very large corpora

2. **Limit candidate count**:
   - Retrieve only what you need
   - Don't retrieve entire corpus

3. **Use approximate methods**:
   - For very large corpora (>10M docs)
   - Consider HNSW or other ANN methods

---

## Common Errors

### "Index not found"

**Symptoms**: Error loading index.

**Solutions**:

1. **Check file path**:
```rust
let path = "index.bm25";
if !std::path::Path::new(path).exists() {
    // Build index first
    let index = bm25::Index::new(&documents)?;
    index.save(path)?;
}
```

2. **Verify index format**:
   - Ensure index file is not corrupted
   - Check version compatibility

### "Empty corpus"

**Symptoms**: Error with empty document collection.

**Solutions**:

1. **Check before indexing**:
```rust
if documents.is_empty() {
    return Err("Cannot build index from empty corpus");
}
```

2. **Handle edge cases**:
   - Empty corpus returns empty index
   - Empty query returns empty results

---

## Getting Help

### Debug Information

When reporting issues, include:

1. **Rust version**: `rustc --version`
2. **Corpus size**: Number of documents
3. **Query**: Sample query that fails
4. **Index type**: BM25, dense, sparse
5. **Error messages**: Full output

### Resources

- **[Getting Started](GETTING_STARTED.md)** - Basic usage guide
- **[README](../README.md)** - Complete API reference
- **[Integration Guide](../INTEGRATION_GUIDE.md)** - Integration patterns

### Reporting Issues

1. Check existing issues on GitHub
2. Search documentation
3. Create minimal reproduction
4. Include debug information
5. File issue with details

---

## Common Patterns

### Pattern: "No results returned"

1. Check index is built and populated
2. Verify query is not empty
3. Check query terms appear in documents
4. Verify indexing process completed

### Pattern: "Low recall"

1. Increase candidate count
2. Use multiple retrievers
3. Check indexing completeness
4. Verify retrieval algorithm parameters

### Pattern: "Slow retrieval"

1. Optimize index structure
2. Limit candidate count
3. Use approximate methods for large corpora
4. Profile to find bottlenecks

