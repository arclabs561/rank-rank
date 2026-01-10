# Facets vs Filters: Comprehensive Analysis

This document provides a comprehensive analysis of facets versus filters in search systems, their technical differences, implementation patterns, and implications for `rank-retrieve`.

## Executive Summary

**Filters** and **Facets** are both mechanisms for narrowing search results, but they serve different purposes and have distinct implementation characteristics:

- **Filters**: Static, broad constraints applied to the search query (e.g., "category=Books", "in-stock=true")
- **Facets**: Dynamic, multi-dimensional attribute groupings with counts computed from results (e.g., "Brand=Sony (24)", "Price: $0-$50 (156)")

While both use similar underlying mechanisms (field indexing, term matching), they differ fundamentally in:
1. **When they're applied**: Filters before/during search, Facets after search
2. **What they return**: Filters narrow results, Facets provide navigation options
3. **How they're computed**: Filters are cached queries, Facets are aggregations over results
4. **User experience**: Filters are few and stable, Facets are many and dynamic

## Conceptual Differences

### Filters

**Definition**: High-level, fixed conditions that restrict the search space.

**Characteristics**:
- **Static**: Don't change with query or result set
- **Broad**: Represent global partitions (categories, availability, permissions)
- **Few**: Typically 3-10 filter options
- **Stable**: Same filters available across all queries
- **Hard boundaries**: Exclude non-matching documents

**Examples**:
- Site section (e.g., "Men's Clothing")
- Availability (e.g., "In Stock Only")
- Permission level (e.g., "Public Documents")
- Date range presets (e.g., "Last 30 Days")
- Language (e.g., "English Only")

**Use Cases**:
- Initial search scope definition
- Security/permission filtering
- Global content partitioning
- Pre-query narrowing

### Facets

**Definition**: Dynamic attribute dimensions that expose refinement options with counts.

**Characteristics**:
- **Dynamic**: Values and counts change with query and result set
- **Multi-dimensional**: Many attribute types (brand, size, color, price, author)
- **Many**: Typically 10-50+ facet dimensions
- **Contextual**: Only shows values present in current results
- **Navigational**: Help users explore when query is broad/underspecified

**Examples**:
- Brand (e.g., "Sony (24)", "Apple (156)")
- Size (e.g., "Small (12)", "Medium (45)", "Large (23)")
- Color (e.g., "Red (8)", "Blue (15)", "Green (3)")
- Price ranges (e.g., "$0-$50 (156)", "$50-$100 (89)")
- Author (e.g., "Smith (5)", "Jones (12)")

**Use Cases**:
- Product discovery and refinement
- Exploratory search
- Multi-attribute filtering
- Result set navigation

## Technical Implementation Differences

### Filters

**Implementation Pattern**:
```rust
// Filter: Applied before/during search
query: {
    bool: {
        must: [/* main query */],
        filter: [
            { term: { category: "Books" }},
            { term: { in_stock: true }}
        ]
    }
}
```

**Technical Characteristics**:
- **Query-level**: Applied as part of the search query
- **Cached**: Filter results are cached independently (Solr `fq`, ES `filter` clause)
- **Score-independent**: Don't affect relevance scoring
- **Fast**: Reused across queries with same filter
- **Pre-computed**: Can be pre-applied to index partitions

**Performance**:
- **Latency**: Minimal overhead (cached filter lookups)
- **Memory**: Filter cache per query pattern
- **Scalability**: Excellent (filters are reused)

### Facets

**Implementation Pattern**:
```rust
// Facets: Computed after search, from results
query: { /* main query */ },
aggs: {
    brands: {
        terms: { field: "brand" }
    },
    price_ranges: {
        range: {
            field: "price",
            ranges: [
                { from: 0, to: 50 },
                { from: 50, to: 100 }
            ]
        }
    }
}
```

**Technical Characteristics**:
- **Aggregation-level**: Computed from search results
- **Dynamic**: Recalculated for each query
- **Count-based**: Returns value + document count pairs
- **Post-query**: Applied after main search completes
- **Result-dependent**: Only shows values present in current results

**Performance**:
- **Latency**: Additional computation per request (aggregation overhead)
- **Memory**: Doc values or field cache for aggregation
- **Scalability**: Good (but requires computing counts for all result documents)

## Implementation in Major Search Systems

### Elasticsearch

**Filters**:
- Implemented via `filter` clause in `bool` query
- Cached independently from query
- Example: `{ "filter": { "term": { "category": "Books" }}}`

**Facets** (called "Aggregations"):
- Implemented via `aggs` parameter
- Types: `terms`, `range`, `histogram`, `date_histogram`, etc.
- Example: `{ "aggs": { "brands": { "terms": { "field": "brand" }}}}`

**Key Difference**:
- Filters narrow the search space
- Aggregations compute statistics over results
- `post_filter` applies filters after aggregations (for different counts vs results)

### Apache Solr

**Filters**:
- Implemented via `fq` (filter query) parameter
- Cached independently
- Example: `fq=category:Books&fq=in_stock:true`

**Facets**:
- Implemented via `facet=true` and `facet.field=brand`
- Types: field faceting, range faceting, query faceting, JSON faceting
- Example: `facet=true&facet.field=brand&facet.field=color`

**Key Difference**:
- `fq` restricts documents before faceting
- Facets computed from filtered result set
- Can use `facet.query` for custom facet queries

### Vector Databases

**Filters**:
- Metadata filters applied during vector search
- Examples: Qdrant `filter`, Pinecone `filter`, Weaviate `where`
- Performance: Often degrades search (see `FILTERING_IN_VECTOR_SEARCH.md`)

**Facets**:
- Less common in pure vector databases
- Some support aggregations (e.g., Qdrant aggregations API)
- Typically requires separate metadata index

**Key Challenge**:
- Vector search optimizes for similarity, not metadata
- Faceting requires metadata indexing separate from vectors
- Performance trade-offs similar to filtering

## When to Use Each

### Use Filters For:

1. **Global Constraints**
   - Site sections, collections, categories
   - Permission/security boundaries
   - Availability status
   - Language/region restrictions

2. **Stable Partitions**
   - Content types that don't change with query
   - Pre-defined ranges (date windows, price bands)
   - Boolean attributes (in-stock, featured)

3. **Performance-Critical Narrowing**
   - When you need fast, cached filtering
   - When filter selectivity is high (<10% of corpus)
   - When filters are reused across many queries

### Use Facets For:

1. **Multi-Attribute Refinement**
   - Product attributes (brand, size, color, material)
   - Content metadata (author, topic, tag)
   - Numeric ranges (price, date, rating)

2. **Exploratory Search**
   - When user intent is unclear
   - When query is broad ("shoes", "laptops")
   - When users need to discover available options

3. **Dynamic Navigation**
   - When available values depend on query
   - When counts help users understand result space
   - When multiple dimensions need simultaneous refinement

## Current State in rank-retrieve

### What We Have

- **Filtering Support**: Post-filtering, filter fusion, integrated filtering (HNSW, IVF-PQ)
- **Metadata Storage**: `MetadataStore` for document metadata (`HashMap<u32, DocumentMetadata>`)
- **Filter Predicates**: `FilterPredicate` enum (Equals, And, Or)
- **Selectivity Estimation**: `estimate_selectivity()` computes filter match counts (faceting-like!)

### What We're Missing

- **Value Enumeration**: No way to discover available filter values (users must know category IDs)
- **Value Counts**: No way to get (value, count) pairs for a field
- **Filtered Facets**: No way to compute facet counts from filtered result sets
- **Multi-Field Indexing**: Indexes only support a single `filter_field` (HNSW, IVF-PQ)
- **Dynamic Faceting**: No support for computing facets from search results

### Key Insight: `estimate_selectivity()` is Already Faceting

Our `estimate_selectivity()` method is actually doing faceting work - it counts documents matching a filter. The gap is that we don't expose:
1. **Value enumeration**: What values exist for a field?
2. **Value counts**: How many documents per value?
3. **Filtered counts**: What values exist in the filtered result set?

## Should We Add Faceting Support?

### Arguments For

1. **Completeness**: Major search systems (ES, Solr) support both filters and facets
2. **User Experience**: Facets enable exploratory search and discovery
3. **E-commerce Use Cases**: Product search requires faceted navigation
4. **Different Use Case**: Facets serve different purpose than filters

### Arguments Against

1. **Scope Creep**: `rank-retrieve` focuses on first-stage retrieval, not full search engine features
2. **Complexity**: Faceting requires aggregation infrastructure (doc values, field caches)
3. **Performance**: Aggregations add latency to every query
4. **Vector Search Focus**: Our primary use case is vector similarity, not metadata exploration

### Recommendation (Validated Against Reality)

**Add lightweight faceting helpers** to `MetadataStore` (not full aggregation infrastructure):

1. **Value Enumeration**: `get_all_values(field)` - discover available filter values
2. **Value Counts**: `get_value_counts(field)` - get (value, count) pairs for a field
3. **Filtered Counts**: `get_value_counts_filtered(field, filter)` - counts from filtered result set

**Why this makes sense (validated)**:
- **Appropriate for our scale**: O(n) iteration is acceptable for 10K-100K documents (our target users)
- **Fills a real gap**: Users building dynamic UIs need value enumeration
- **Leverages existing data**: We already have all metadata in `MetadataStore`
- **Enables UI building**: Users can build filter UIs that show available options
- **Low complexity**: Simple HashMap iteration, no new infrastructure

**What we're NOT adding (and why)**:
- **Full aggregation framework**: Production systems (Elasticsearch, Solr) use doc values, field caches, distributed aggregation - out of scope
- **Dynamic faceting from search results**: Would require tracking result sets, adds complexity - our static faceting is sufficient for most RAG use cases
- **Multi-dimensional faceting UI**: Out of scope - we provide data, not UI
- **Range faceting**: Would require numeric field support - users can pre-define ranges as categories

**Honest Limitations**:
- **Single-field integrated filtering**: Only one field gets optimized filtering (HNSW, IVF-PQ) - real constraint
- **O(n) performance**: Acceptable for 10K-100K documents, slow for 1M+ - use production backends for larger scales
- **Static faceting**: Counts from entire corpus, not from search results - gap, but may not matter for RAG
- **Categorical only**: No range faceting - users must pre-define ranges

**When to use our lightweight faceting**:
- ✅ RAG systems with 10K-100K documents
- ✅ Academic search with categorical metadata
- ✅ Research prototypes needing value enumeration
- ✅ Applications building dynamic filter UIs

**When to use production backends**:
- ❌ Very large corpora (1M+ documents)
- ❌ Need sub-10ms faceting performance
- ❌ Need multi-dimensional integrated filtering
- ❌ Need range faceting or numeric histograms
- ❌ Need dynamic faceting from search results

**Rationale (refined)**:
- Our users (RAG, academic search) are in the "beneficial but not essential" zone for faceting
- The data is already there (MetadataStore), so adding helpers is low-cost
- Value enumeration is a real need for dynamic UIs
- Simple counting is lightweight and appropriate for our target scale
- Full faceting (from search results, production-scale) is a search engine feature, not retrieval
- We acknowledge limitations honestly and recommend production backends when needed

**See `FACETS_VS_FILTERS_VALIDATION.md` for deep validation against real-world evidence.**

## Implementation Pattern

Add lightweight faceting helpers to `MetadataStore`:

```rust
impl MetadataStore {
    /// Get all unique values for a field.
    ///
    /// Returns a sorted list of all category IDs present in the field.
    /// Useful for discovering available filter values.
    pub fn get_all_values(&self, field: &str) -> Vec<u32> {
        let mut values: std::collections::HashSet<u32> = std::collections::HashSet::new();
        for metadata in self.metadata.values() {
            if let Some(&value) = metadata.get(field) {
                values.insert(value);
            }
        }
        let mut result: Vec<u32> = values.into_iter().collect();
        result.sort();
        result
    }

    /// Get value counts for a field.
    ///
    /// Returns (value, count) pairs sorted by count descending.
    /// This is basic faceting - shows how many documents have each value.
    pub fn get_value_counts(&self, field: &str) -> Vec<(u32, usize)> {
        let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for metadata in self.metadata.values() {
            if let Some(&value) = metadata.get(field) {
                *counts.entry(value).or_insert(0) += 1;
            }
        }
        let mut result: Vec<(u32, usize)> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        result
    }

    /// Get value counts for documents matching a filter.
    ///
    /// Computes facet counts from a filtered subset of documents.
    /// This enables "filtered faceting" - showing available values in current results.
    pub fn get_value_counts_filtered(
        &self,
        field: &str,
        filter: &FilterPredicate,
    ) -> Vec<(u32, usize)> {
        let mut counts: std::collections::HashMap<u32, usize> = std::collections::HashMap::new();
        for metadata in self.metadata.values() {
            if filter.matches(metadata) {
                if let Some(&value) = metadata.get(field) {
                    *counts.entry(value).or_insert(0) += 1;
                }
            }
        }
        let mut result: Vec<(u32, usize)> = counts.into_iter().collect();
        result.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending
        result
    }
}
```

**Usage Example**:
```rust
let store = MetadataStore::new();
// ... add metadata ...

// Discover available categories
let categories = store.get_all_values("category");
println!("Available categories: {:?}", categories);

// Get counts per category
let counts = store.get_value_counts("category");
for (category_id, count) in counts {
    println!("Category {}: {} documents", category_id, count);
}

// Get counts from filtered results
let filter = FilterPredicate::equals("region", 1);
let filtered_counts = store.get_value_counts_filtered("category", &filter);
println!("Categories in region 1: {:?}", filtered_counts);
```

**Performance**: O(n) where n = number of documents. Acceptable for in-memory use cases.

## References

- **Elasticsearch**: [Aggregations](https://www.elastic.co/guide/en/elasticsearch/reference/current/search-aggregations.html)
- **Solr**: [Faceting](https://solr.apache.org/guide/solr/latest/query-guide/faceting.html)
- **UX Research**: [Filters vs Facets (Nielsen Norman Group)](https://www.nngroup.com/articles/filters-vs-facets/)
- **Vector Search**: See `FILTERING_IN_VECTOR_SEARCH.md` for vector-specific filtering challenges

## Design Constraints in rank-retrieve

### Single Field Limitation

Our current implementation has a significant constraint: **indexes only support a single `filter_field`**:

- **HNSW**: `HNSWIndex::with_filtering(dimension, m, m_max, filter_field)` - one field only
- **IVF-PQ**: `IVFPQIndex::with_filtering(dimension, params, filter_field)` - one field only

**Why**: Integrated filtering (cluster tagging, intra-category edges) requires knowing the filter field at index construction time.

**Implications**:
- Users must choose one primary filter dimension per index
- Multi-field filtering requires separate indexes or post-filtering
- Faceting is naturally limited to the indexed field

**Workaround**: Use `FilterPredicate::And()` for multi-field filtering, but only one field benefits from integrated filtering performance.

### Metadata Structure

Our `DocumentMetadata` is `HashMap<String, u32>` - supports multiple fields, but:
- Values are `u32` category IDs (not strings) - requires external mapping
- No range support (numeric ranges would need different structure)
- No type information (all fields treated as categorical)

**For faceting**: This structure is sufficient for categorical faceting, but not for:
- Range faceting (price ranges, date ranges)
- Numeric histograms
- String-based faceting (would need string→u32 mapping)

## Related Documentation

- [`FILTERING_IN_VECTOR_SEARCH.md`](FILTERING_IN_VECTOR_SEARCH.md) - Vector search filtering strategies
- [`FACETS_VS_FILTERS_VALIDATION.md`](FACETS_VS_FILTERS_VALIDATION.md) - Deep validation against real-world evidence, production patterns, and user needs
- [`VECTOR_DATABASE_INTEGRATION.md`](VECTOR_DATABASE_INTEGRATION.md) - External system integration
- [`USE_CASES.md`](USE_CASES.md) - When to use rank-retrieve vs full search engines
