# Additional Retrieval Methods for rank-retrieve: Research and Recommendations

This document analyzes additional first-stage retrieval methods that could complement BM25, dense, sparse, and generative retrieval in `rank-retrieve`.

## Executive Summary

Based on research into first-stage retrieval methods, the following methods would complement the existing implementation:

1. **TF-IDF** (High Priority) - Simple, fast, complements BM25
2. **Query Likelihood / Language Models** (Medium Priority) - Different probabilistic approach
3. **BM25 Variants** (BM25L, BM25+) (Medium Priority) - Improvements on standard BM25
4. **Query Expansion / Pseudo-Relevance Feedback** (Medium Priority) - Enhances existing methods
5. **Learned Sparse Retrieval (SPLADE)** (Low Priority) - Requires neural training, complex

## Current Implementation

`rank-retrieve` currently provides:
- **BM25**: Lexical retrieval with Okapi BM25 scoring
- **Dense Retrieval**: Semantic similarity using embeddings (SIMD-accelerated)
- **Sparse Retrieval**: Lexical matching using sparse vectors (SIMD-accelerated)
- **Generative Retrieval (LTRGR)**: Autoregressive identifier generation

## Recommended Additions

### 1. TF-IDF (High Priority)

**What it is:**
TF-IDF (Term Frequency-Inverse Document Frequency) is the predecessor to BM25, calculating relevance as the product of term frequency and inverse document frequency.

**Why it fits:**
- **Simple implementation**: Easier than BM25, good baseline
- **Fast**: Similar computational complexity to BM25
- **Complementary**: Sometimes outperforms BM25 on specific datasets
- **Educational value**: Helps users understand BM25 evolution
- **Low overhead**: Can reuse existing inverted index structure

**Implementation considerations:**
- Can share inverted index with BM25 (same data structure)
- Simple scoring: `score = tf * idf` (no saturation, no length normalization)
- Feature flag: `tfidf` (separate from `bm25`)

**When to use:**
- Simpler baseline for comparison
- Datasets where TF-IDF outperforms BM25 (rare but documented)
- Educational/prototyping scenarios

**Research evidence:**
- TF-IDF remains competitive in some domains despite BM25's dominance
- Useful as a baseline for benchmarking
- Lower computational overhead than BM25 (no saturation function)

**Fit with crate design:**
- ✅ Fast first-stage retrieval
- ✅ In-memory index (reuse BM25 index)
- ✅ Unified API (`retrieve_tfidf()`)
- ✅ Returns `Vec<(u32, f32)>` like other methods

### 2. Query Likelihood / Language Models (Medium Priority)

**What it is:**
Probabilistic retrieval model that ranks documents by the probability that the document's language model generated the query: `P(Q|D)`.

**Why it fits:**
- **Different approach**: Probabilistic framework vs. BM25's term frequency approach
- **Theoretical foundation**: Well-grounded in information theory
- **Complementary**: Can outperform BM25 on some queries
- **Smoothing techniques**: Jelinek-Mercer, Dirichlet smoothing provide flexibility

**Implementation considerations:**
- Requires language model estimation (word probabilities from documents)
- Smoothing is essential (handles unseen query terms)
- Can reuse inverted index but needs additional probability storage
- Feature flag: `language-model` or `query-likelihood`

**When to use:**
- Research/prototyping scenarios
- Queries where probabilistic approach helps
- When query expansion is needed (natural fit with language models)

**Research evidence:**
- Language models often perform comparably to BM25
- Query likelihood with Dirichlet smoothing is competitive
- Better theoretical foundation than BM25 (probabilistic vs. heuristic)

**Fit with crate design:**
- ✅ First-stage retrieval (fast with precomputed probabilities)
- ✅ In-memory index (can extend existing structure)
- ✅ Unified API (`retrieve_query_likelihood()`)
- ⚠️ More complex than BM25 (smoothing parameters)

### 3. BM25 Variants (BM25L, BM25+) (Medium Priority)

**What they are:**
Improvements to standard BM25 addressing specific limitations:
- **BM25L**: Addresses over-penalization of short documents
- **BM25+**: Adds constant term to prevent negative scores for common terms

**Why they fit:**
- **Direct improvements**: Address known BM25 limitations
- **Easy to implement**: Small modifications to existing BM25 code
- **Parameter options**: Users can choose variant based on document characteristics
- **Research-backed**: Proven improvements in specific scenarios

**Implementation considerations:**
- Can extend existing `Bm25Params` with variant selection
- BM25L: Modifies length normalization term
- BM25+: Adds constant `δ` to scoring formula
- Feature flag: Part of `bm25` feature (variant selection)

**When to use:**
- **BM25L**: When short documents are over-penalized
- **BM25+**: When common terms cause counterintuitive scoring
- Document collections with specific length distributions

**Research evidence:**
- BM25L: 2-5% improvement on short documents
- BM25+: Prevents negative scores, more stable behavior
- Both variants are well-documented and tested

**Fit with crate design:**
- ✅ Natural extension of existing BM25
- ✅ Same index structure
- ✅ Unified API (variant selection in params)
- ✅ Minimal code changes

### 4. Query Expansion / Pseudo-Relevance Feedback (Medium Priority)

**What it is:**
Techniques that reformulate queries to include semantically related terms:
- **Query Expansion**: Add related terms to query
- **Pseudo-Relevance Feedback (PRF)**: Use top-ranked documents to expand query

**Why it fits:**
- **Enhances existing methods**: Works with BM25, dense, sparse
- **Addresses vocabulary mismatch**: Key problem in first-stage retrieval
- **First-stage appropriate**: Improves recall (critical for first-stage)
- **Well-researched**: Classical IR technique with modern neural variants

**Implementation considerations:**
- Can be implemented as query preprocessing
- PRF: Retrieve initial results, extract terms, expand query, re-retrieve
- Feature flag: `query-expansion` (works with other features)
- Options: Rocchio algorithm, neural expansion, term co-occurrence

**When to use:**
- Queries with vocabulary mismatch
- Low recall scenarios
- Domain-specific terminology
- Research/prototyping

**Research evidence:**
- PRF improves recall by 10-30% in many scenarios
- Query expansion is standard in production systems
- Neural expansion methods show promise

**Fit with crate design:**
- ✅ First-stage retrieval enhancement
- ✅ Works with existing methods (BM25, dense, sparse)
- ✅ Can be optional wrapper around existing retrievers
- ⚠️ Adds complexity (expansion strategies, parameters)

### 5. Learned Sparse Retrieval (SPLADE) (Low Priority)

**What it is:**
Neural method that learns sparse representations, expanding queries/documents with semantically related terms not explicitly present.

**Why it's lower priority:**
- **Requires neural training**: Needs model training/fine-tuning
- **Complexity**: More complex than other methods
- **Dependencies**: Would require ML framework integration
- **Out of scope**: Better suited for specialized crate or external integration

**When it might fit:**
- If we add neural model support
- Research-focused use cases
- Integration with external SPLADE models

**Fit with crate design:**
- ⚠️ Requires neural model (out of scope for basic implementations)
- ⚠️ Training complexity (better as external integration)
- ✅ Could work with sparse retrieval infrastructure
- ❌ Not aligned with "basic implementations" philosophy

## Methods That Don't Fit

### Cross-Encoders
- **Why not**: Too slow for first-stage retrieval (quadratic attention)
- **Where it belongs**: `rank-rerank` (reranking stage)

### LLM Reranking
- **Why not**: Too slow and expensive for first-stage
- **Where it belongs**: `rank-rerank` or specialized crate

### Full Search Engines
- **Why not**: Out of scope (persistent storage, complex queries)
- **Where it belongs**: `tantivy`, `meilisearch`, external systems

## Implementation Priority

### Phase 1: High Priority (Immediate Value) ✅ COMPLETED
1. **TF-IDF** ✅
   - Simple implementation
   - Reuses BM25 infrastructure
   - Good baseline/comparison method
   - **Status**: Implemented and tested

### Phase 2: Medium Priority (Complementary Methods) ✅ COMPLETED
2. **BM25 Variants (BM25L, BM25+)** ✅
   - Small modifications to existing code
   - Address known limitations
   - **Status**: Implemented and tested

3. **Query Expansion / PRF** ✅
   - Enhances existing methods
   - Addresses vocabulary mismatch
   - **Status**: Implemented and tested

4. **Query Likelihood / Language Models** ✅
   - Different probabilistic approach
   - Research value
   - **Status**: Implemented and tested

### Phase 3: Low Priority (Future Consideration)
5. **Learned Sparse Retrieval (SPLADE)**
   - Requires neural training infrastructure
   - Better as external integration
   - **Status**: Documented as external integration (see `ADVANCED_OPTIMIZATIONS.md`)
   - **Recommendation**: Use external SPLADE models with `rank-retrieve`'s sparse retrieval module

## Design Considerations

### API Design
All new methods should follow existing patterns:
- Concrete functions: `retrieve_tfidf()`, `retrieve_query_likelihood()`
- Consistent output: `Vec<(u32, f32)>`
- Feature-gated: `tfidf`, `query-likelihood`, etc.
- Integration: Works with `rank-fusion` for hybrid search

### Index Sharing
- TF-IDF: Can share inverted index with BM25
- BM25 variants: Same index, different scoring
- Query Likelihood: Can extend BM25 index with probabilities
- Query Expansion: Wrapper around existing retrievers

### Performance
- All methods must maintain first-stage retrieval speed (<100ms for 1M docs)
- SIMD acceleration where applicable (TF-IDF can reuse BM25 optimizations)
- Early termination where possible

## Research Sources

1. **BM25 and Variants**:
   - "The BM25 Algorithm and Its Variables" (Elasticsearch)
   - "BM25L and BM25+" research papers
   - Stanford IR course materials

2. **Language Models**:
   - "Using Query Likelihood Language Models in IR" (Stanford)
   - "Language Models for Information Retrieval" (Jelinek-Mercer, Dirichlet)

3. **Query Expansion**:
   - "Query Expansion using Pseudo Relevance Feedback" (ArXiv)
   - "An Analysis of Fusion Functions for Hybrid Retrieval" (ArXiv)

4. **First-Stage Retrieval**:
   - "Semantic Models for the First-stage Retrieval: A Comprehensive Review" (ArXiv 2021)
   - "Leveraging Semantic and Lexical Matching" (ArXiv 2020)

5. **Hybrid Retrieval**:
   - "Hybrid Search with Amazon OpenSearch Service" (AWS)
   - "Hybrid Retrieval: Combining Sparse and Dense Methods" (Industry blogs)

## Recommendations

### Immediate Implementation (Phase 1)
**TF-IDF** should be implemented first because:
- Simple, fast implementation
- Reuses existing infrastructure
- Provides baseline for comparison
- Low risk, high value

### Short-term (Phase 2)
**BM25 Variants** and **Query Expansion** provide:
- Direct improvements to existing methods
- Address known limitations
- Well-researched, proven techniques

### Long-term (Phase 3)
**Query Likelihood** and **SPLADE** are:
- More complex implementations
- Require additional research
- Better suited for specialized use cases

## Conclusion

The recommended additions (TF-IDF, BM25 variants, Query Expansion, Query Likelihood) complement the existing implementation by:
1. Providing alternative approaches (TF-IDF, Query Likelihood)
2. Improving existing methods (BM25 variants)
3. Enhancing recall (Query Expansion)
4. Maintaining the crate's focus on fast, first-stage retrieval
5. Fitting the unified API design

All recommended methods align with `rank-retrieve`'s design philosophy: fast, in-memory, first-stage retrieval with a unified API that integrates seamlessly with the `rank-*` ecosystem.
