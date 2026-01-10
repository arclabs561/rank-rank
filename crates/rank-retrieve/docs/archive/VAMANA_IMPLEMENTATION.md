# Vamana Implementation

## Overview

Vamana is a graph-based approximate nearest neighbor (ANN) search algorithm that uses two-pass construction with RRND (Relaxed Relative Neighborhood Diversification) and RND (Relative Neighborhood Diversification) strategies.

**Status**: ✅ Implemented

## Algorithm

Vamana constructs a proximity graph using:

1. **Random graph initialization**: Each node connects to ≥ log(n) random neighbors
2. **First pass**: Refine using RRND with α ≥ 1.5
3. **Second pass**: Further refine using RND

The two-pass construction ensures better graph quality compared to single-pass methods.

## Performance Characteristics

Based on 2025 research (Azizi et al.):

- **Indexing time**: Third fastest (after ELPIS and HNSW)
- **Query performance**: Competitive with HNSW on large datasets
- **Memory**: 30% more than ELPIS, but less than HNSW
- **SSD serving**: Better than HNSW (5-10× more points/node ratio)
- **Scalability**: Scales to billion-scale datasets

## Usage

```rust
use rank_retrieve::dense::vamana::{VamanaIndex, VamanaParams};

let params = VamanaParams {
    max_degree: 64,        // Maximum out-degree per node
    alpha: 1.3,            // Relaxation factor for RRND (typically 1.3-1.5)
    ef_construction: 200,   // Search width during construction
    ef_search: 50,         // Default search width during query
};

let mut index = VamanaIndex::new(128, params)?;

// Add vectors
for i in 0..10_000 {
    let vector: Vec<f32> = /* ... */;
    index.add(i as u32, vector)?;
}

// Build index (two-pass construction)
index.build()?;

// Search
let results = index.search(&query, 10, 50)?;
```

## Parameters

### `max_degree`

Maximum out-degree per node. Typical values:
- **64**: Standard configuration
- **128**: Higher quality, larger memory footprint
- **256+**: For SSD-based serving (5-10× more points/node than HNSW)

### `alpha`

Relaxation factor for RRND (first pass). Typical values:
- **1.3**: Balanced (default)
- **1.5**: More pruning, smaller graphs
- **1.2**: Less pruning, larger graphs

### `ef_construction`

Search width during construction. Typical values:
- **200**: Standard
- **400**: Higher quality, slower construction

### `ef_search`

Search width during query. Typical values:
- **50**: Fast queries, lower recall
- **100-200**: Higher recall, slower queries

## Comparison with HNSW

| Aspect | Vamana | HNSW |
|--------|--------|------|
| Construction | Two-pass (RRND + RND) | Single-pass (RND) |
| Indexing time | Slower (two passes) | Faster (single pass) |
| Graph quality | Higher (two-pass refinement) | Good (single-pass) |
| Query performance | Competitive | Best overall |
| SSD serving | Better (higher points/node) | Good |
| Memory | Moderate | Higher (hierarchical layers) |

## When to Use Vamana

**Use Vamana when**:
- You need competitive performance with HNSW
- SSD-based serving is important (higher points/node ratio)
- You can tolerate higher indexing time for better graph quality
- You want a flat graph structure (no hierarchy)

**Use HNSW when**:
- You need fastest indexing time
- Query performance is critical
- You want hierarchical structure for very large datasets

## Implementation Details

### Two-Pass Construction

1. **Initialize random graph**: Each node connects to log(n) random neighbors
2. **First pass (RRND)**: For each node, refine neighbors using relaxed RND formula:
   - `dist(X_q, X_j) < α · dist(X_i, X_j)` with α ≥ 1.5
   - Less aggressive pruning, creates larger initial graph
3. **Second pass (RND)**: Further refine using strict RND formula:
   - `dist(X_q, X_j) < dist(X_i, X_j)` for all neighbors X_i
   - More aggressive pruning, creates final optimized graph

### Search Algorithm

Uses beam search (similar to HNSW but without hierarchy):
1. Start from random entry point (KS seed selection)
2. Maintain candidate queue (min-heap by distance)
3. Explore neighbors of candidates
4. Stop when ef candidates visited
5. Return top-k results

## References

- Subramanya et al. (2019): "DiskANN: Fast accurate billion-point nearest neighbor search"
- Azizi et al. (2025): "Graph-Based Vector Search: An Experimental Evaluation of the State-of-the-Art"

## See Also

- [Seed Selection and ND Guide](SEED_SELECTION_AND_ND_GUIDE.md) for details on RRND and RND strategies
- [Graph-Based Vector Search Survey](GRAPH_BASED_VECTOR_SEARCH_SURVEY.md) for comprehensive research synthesis
- `examples/vamana_basic.rs` for usage example
