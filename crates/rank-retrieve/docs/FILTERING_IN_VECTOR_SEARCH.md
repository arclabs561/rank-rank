# Filtering in Vector Search: The Achilles' Heel of ANN

This document synthesizes insights from "The Achilles Heel of Vector Search: Filters" (Bits & Backprops, May 2025) and relates them to `rank-retrieve`'s architecture and potential improvements.

## Core Problem

Unlike traditional RDBMS where filters accelerate queries, **adding filters to vector search often slows it down**. This counterintuitive behavior stems from fundamental incompatibilities between ANN indexes (optimized for proximity) and boolean predicates (optimized for exact matches).

## Facets vs Filters

**Note**: This document focuses on **filters** (static constraints that narrow search). For information on **facets** (dynamic attribute groupings with counts for navigation), see [`FACETS_VS_FILTERS.md`](FACETS_VS_FILTERS.md).

**Quick distinction**:
- **Filters**: Static constraints applied to search (e.g., "category=Books", "in-stock=true")
- **Facets**: Dynamic attribute counts computed from results (e.g., "Brand=Sony (24)", "Price: $0-$50 (156)")

Both use similar underlying mechanisms but serve different purposes: filters narrow the search space, facets enable exploratory navigation.

## Three Filtering Strategies

### 1. Pre-filtering (Filter-Then-Search)

**Approach**: Apply metadata filter first (via inverted index), then run ANN on the filtered subset.

**Characteristics**:
- Guarantees correctness (100% recall on filtered subset)
- Performance: O(subset size) - degrades to brute force for large subsets
- Memory: Low overhead (metadata index only)
- Complexity: Low

**Implementation Status**: Not implemented in `rank-retrieve`. Current ANN implementations (HNSW, IVF-PQ, SCANN) operate on full dataset.

**When it works**: Tiny filtered subsets (<1% of data). For larger subsets, performance collapses to linear scan.

### 2. Post-filtering (Search-Then-Filter)

**Approach**: Run standard ANN search ignoring filters, then discard non-matching results.

**Characteristics**:
- Recall: Variable (<100%) - can miss results under strict filters
- Latency: O(ANN + oversample) - requires oversampling to maintain recall
- Memory: None (no extra structures)
- Complexity: Low (but requires tuning oversample parameters)

**Implementation Status**: Not implemented. Would require:
- Metadata storage per vector
- Oversampling logic (e.g., search 10×k candidates if only 10% match filter)
- Post-filtering pass

**When it works**: Loose filters (>50% selectivity). Strict filters require massive oversampling, hurting latency.

### 3. Integrated Filtering (Single-Stage)

**Approach**: Modify index structure or search algorithm to respect filters during traversal.

**Examples**:
- **Qdrant Filterable HNSW**: Adds extra intra-category graph links
- **Weaviate ACORN**: Two-hop graph expansions to skip filtered nodes
- **Pinecone Single-Stage**: Merges vector and metadata indexes
- **IVF Cluster Tagging**: Tag centroids with filter values, skip irrelevant clusters

**Characteristics**:
- Recall: ≈100% (preserves ANN accuracy)
- Latency: Near unfiltered performance (pruned search space)
- Memory: High (extra links, edges, tags)
- Complexity: High (algorithmic changes required)

**Implementation Status**: Not implemented. Would require:
- HNSW: Filter-aware graph construction (extra links per category)
- IVF-PQ: Cluster tagging with filter metadata
- SCANN: Partition-level filter tags

**When it works**: High-selectivity filters (10% of data) where pruning provides real benefit.

## Performance Implications

### Benchmark Findings (from article)

**Throughput (QPS)**:
- **Integrated systems** (Pinecone-p2, Zilliz-AUTOINDEX): 1.2×–1.5× throughput improvement with 10% filter
- **Post-filter systems** (LanceDB-IVF_PQ, OpenSearch-HNSW): QPS degradation under filtering
- **Qdrant-HNSW**: Smaller gain via augmented-graph strategy

**Latency (P95)**:
- **Integrated systems**: 10–30ms reduction (30–50ms at 16 threads)
- **Post-filter systems**: 200–300ms spikes under strict filters

**Key Insight**: Engines with in-algorithm filtering not only preserve recall—they actually get faster when filters prune the workload.

### Why Traditional Filtering Fails

1. **Index Mismatch**: ANN indexes optimize for proximity, not boolean predicates
2. **Graph Disconnects**: Removing nodes fragments HNSW graphs (B-trees never face this)
3. **No Composite Key**: Can't jointly index (vector, metadata) as easily as relational DBs
4. **Approximation vs Exactness**: ANN trades recall for speed; filters demand exact matches
5. **Join-Like Complexity**: Filtering + similarity is akin to joining vector and metadata indexes

## Filter Fusion: Encoding Filters into Embeddings

**Concept**: Encode metadata directly into vector embeddings so standard ANN search naturally respects filters without explicit filtering steps.

### Simple Approach: Vector Concatenation

For categorical filters, append weighted one-hot encodings to embeddings:

```rust
// Original embedding: x_i ∈ R^d
// Category label: k_i ∈ {1, ..., T}
// Metadata weight: α > 0

// One-hot encoding: m_i = e_{k_i} ∈ {0,1}^T
// Augmented vector: x̃_i = [x_i, α·m_i] ∈ R^{d+T}
// Augmented query: q̃ = [q, α·e_c] where c is desired category
```

**Distance computation**:
```
||x̃_i - q̃||² = ||x_i - q||² + α²·||m_i - e_c||²
```

Since `||m_i - e_c||² = 0` if `k_i = c`, else `2`, any non-matching category incurs penalty `2α²`. By choosing `α` large enough, top-k results naturally satisfy the filter.

### FAISS Implementation (Big-ANN Benchmark)

The FAISS team used **ID-based signatures** for filter fusion:

- Low 24 bits: Vector index (log₂(10⁷) ≈ 23.25 bits)
- High 39 bits: Bitwise signature for metadata terms
- Each metadata term `j` gets random 39-bit mask `S_j`
- Vector signature: `sig_i = ⋁_{j∈W_i} S_j` (OR of term masks)
- Query signature: `sig_q = S_{w1} ∨ S_{w2}` (OR of query term masks)
- Filter check: `if (¬sig_i ∧ sig_q) ≠ 0 continue` (skip if any query term missing)

This eliminates 80% of negatives before distance computation, running entirely inside the ANN engine's tight loop.

### Limitations

1. **Dimensional Explosion**: High-cardinality fields bloat vectors (thousands of categories → thousands of extra dimensions)
2. **Multi-Column Filters**: Concatenating multiple one-hot encodings multiplies dimensionality
3. **Range/Temporal Queries**: One-hot encoding infeasible for continuous values (dates, numeric ranges)
4. **Weight Tuning**: Choosing `α` is a trade-off between strict filtering and semantic relevance
5. **Dynamic Updates**: Metadata changes require embedding regeneration
6. **Interpretability**: Mixed metadata/content dimensions harder to debug

## Current State in rank-retrieve

### What Exists

- **ANN Algorithms**: HNSW, IVF-PQ, SCANN, NSW, LSH, etc. (no filtering support)
- **Awareness**: `CRITICAL_PERSPECTIVES_AND_LIMITATIONS.md` documents filtering challenges
- **Integration Guide**: Mentions metadata filtering as vector database feature

### What's Missing

1. **No Filtering API**: Search methods don't accept filter predicates
2. **No Metadata Storage**: Vectors stored without associated metadata
3. **No Filter Fusion**: Embeddings are pure content (no metadata encoding)
4. **No Integrated Filtering**: Index structures don't encode filter information

## Implementation Opportunities

### Short-Term (Low Complexity)

1. **Post-Filtering Support**
   - Add metadata storage alongside vectors
   - Implement oversampling logic (search `k * (1/selectivity)` candidates)
   - Post-filter results before returning top-k
   - **API**: `search_with_filter(query, k, filter_predicate)`

2. **Filter Fusion (Simple)**
   - Add `augment_with_metadata()` helper function
   - Concatenate one-hot category encodings to embeddings
   - Document weight tuning guidelines
   - **API**: `augment_embedding(embedding, category, weight) -> augmented_embedding`

### Medium-Term (Medium Complexity)

3. **IVF-PQ Cluster Tagging**
   - Tag each cluster with filter value bitmasks
   - Skip clusters with no matching vectors during search
   - **API**: `build_with_metadata(vectors, metadata)` → filterable index

4. **HNSW Filterable Graph**
   - Add extra intra-category edges during construction
   - Modify search to prefer same-category neighbors
   - **API**: `build_filterable_hnsw(vectors, categories)` → filterable graph

### Long-Term (High Complexity)

5. **FAISS-Style ID Signatures**
   - Use high bits of vector IDs for metadata signatures
   - Implement bitwise filter checks in tight loops
   - Requires careful ID management (24-bit vector index, 39-bit signatures)

6. **ACORN-Style Two-Hop Expansion**
   - Maintain unpruned graph edges
   - Use two-hop jumps to skip filtered nodes
   - More complex than simple filterable HNSW

## Design Considerations

### API Design

```rust
// Option 1: Explicit filter parameter
pub fn search_with_filter(
    &self,
    query: &[f32],
    k: usize,
    filter: &FilterPredicate,
) -> Result<Vec<(u32, f32)>, RetrieveError>;

// Option 2: Metadata-augmented embeddings (filter fusion)
pub fn search_augmented(
    &self,
    query: &[f32],
    metadata: &[u32],  // Category IDs
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError>;

// Option 3: Filter-aware index construction
pub struct FilterableHNSW {
    // Index with filter metadata baked in
}
```

### Performance Trade-offs

| Approach | Recall | Latency | Memory | Complexity | Best For |
|----------|--------|---------|--------|------------|---------|
| Pre-filter | 100% | O(subset) | Low | Low | Tiny subsets |
| Post-filter | Variable | O(ANN+oversample) | None | Low | Loose filters |
| Integrated | ≈100% | Near unfiltered | High | High | High-selectivity |
| Filter Fusion | ≈100% | Unfiltered | Medium | Medium | Categorical only |

### When to Use Each

- **Pre-filter**: <1% selectivity, can afford linear scan
- **Post-filter**: >50% selectivity, acceptable recall loss
- **Integrated**: 10-50% selectivity, need high recall
- **Filter Fusion**: Categorical filters, low cardinality, static metadata

## References

- **Bits & Backprops**: "The Achilles Heel of Vector Search: Filters" (May 2025)
- **FAISS Paper**: Big-ANN Benchmark (NeurIPS 2023)
- **Qdrant**: Filterable HNSW implementation
- **Weaviate**: ACORN two-hop expansion
- **Pinecone**: Single-stage filtering architecture

## Related Documentation

- [`FACETS_VS_FILTERS.md`](FACETS_VS_FILTERS.md) - Comprehensive analysis of facets vs filters
- [`CRITICAL_PERSPECTIVES_AND_LIMITATIONS.md`](CRITICAL_PERSPECTIVES_AND_LIMITATIONS.md) - Existing filtering analysis
- [`VECTOR_DATABASE_INTEGRATION.md`](VECTOR_DATABASE_INTEGRATION.md) - External filtering via vector DBs
- [`ANN_ALGORITHMS_PLAN.md`](ANN_ALGORITHMS_PLAN.md) - ANN implementation status
