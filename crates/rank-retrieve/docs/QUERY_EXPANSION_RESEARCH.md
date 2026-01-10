# Query Expansion / Pseudo-Relevance Feedback (PRF) Implementation Research

This document synthesizes research on query expansion and PRF for `rank-retrieve`.

## Overview

Query expansion and PRF reformulate queries to include semantically related terms, addressing vocabulary mismatch - a key problem in first-stage retrieval.

## Research Findings (2024 Best Practices)

### 1. Control PRF Depth and Noise
- **Small PRF depth**: Top-3 to top-10 feedback docs typically give best trade-off
- **Deeper PRF**: Often introduces noise and query drift
- **Short, high-salience signals**: Keywords, entities, short facts over full passages

### 2. Use Structured Features
- Extract and weight **structured features** (named entities, keyphrases) from feedback docs
- More stable gains than unstructured text expansion
- Knowledge/concept-based expansion reduces spurious high-frequency terms

### 3. Tune Expansion Strength
- **Limit added terms**: Over-expansion degrades precision
- **Separate weighting**: Original query should dominate (Rocchio-style or interpolation)
- **Term selection thresholds**: Validate via held-out queries

### 4. Model-Aware PRF

**For Lexical (BM25/TF-IDF):**
- Use standard term-scoring (Robertson selection value, KL-divergence)
- Filter via semantic/knowledge constraints when possible
- Down-weight very frequent/generic terms (stop-like behavior)
- Prioritize rare, discriminative terms

**For Dense Retrieval:**
- Avoid concatenating raw feedback text directly
- Build refined query representations from encoded feedback documents
- Moderate PRF depth (3-10 docs) and limited feature types (entities/keywords)

### 5. Implementation Strategy

**Two-Stage Retrieval:**
1. Initial fast retrieval (get top-k candidates)
2. PRF expansion + rerun over full or narrowed candidate set

**Term Extraction Methods:**
- **Robertson Selection Value (RSV)**: Score terms by their contribution to relevance
- **KL-Divergence**: Select terms that distinguish relevant from non-relevant docs
- **Term Frequency**: Simple frequency-based selection
- **IDF-weighted**: Prioritize rare, discriminative terms

## Implementation Plan

### API Design

```rust
pub struct QueryExpander {
    prf_depth: usize,  // Top-k docs for feedback (default: 5)
    max_expansion_terms: usize,  // Max terms to add (default: 5)
    expansion_weight: f32,  // Weight for expansion terms (default: 0.5)
    method: ExpansionMethod,
}

pub enum ExpansionMethod {
    RobertsonSelection,  // RSV-based term selection
    KLDivergence,  // KL-divergence based
    TermFrequency,  // Simple frequency-based
    IDFWeighted,  // IDF-weighted selection
}

pub fn expand_query_with_prf<R>(
    retriever: &R,
    query: &[String],
    initial_k: usize,
    final_k: usize,
    expander: &QueryExpander,
) -> Result<Vec<(u32, f32)>, RetrieveError>
where
    R: Retriever<Query = Vec<String>>,
```

### Implementation Steps

1. Create `query_expansion` module
2. Implement term extraction from feedback documents
3. Implement expansion methods (Robertson, KL-divergence, etc.)
4. Implement PRF wrapper for retrievers
5. Add tests
6. Add example

### Integration Points

- Works with BM25, TF-IDF (via `InvertedIndex`)
- Works with dense retrieval (via `DenseRetriever`)
- Works with sparse retrieval (via `SparseRetriever`)
- Can be used as wrapper or preprocessing step

## References

- "Query Expansion using Pseudo Relevance Feedback" (ArXiv)
- "An Analysis of Fusion Functions for Hybrid Retrieval" (ArXiv)
- Recent PRF research (2024): Controlled feedback, model-aware expansion
