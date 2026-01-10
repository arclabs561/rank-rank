# TF-IDF Implementation Research

This document synthesizes research on TF-IDF implementation for `rank-retrieve`, based on the high-priority recommendation in `RETRIEVAL_METHODS_RESEARCH.md`.

## Overview

TF-IDF (Term Frequency-Inverse Document Frequency) is the predecessor to BM25, providing a simpler baseline for lexical retrieval. It calculates relevance as the product of term frequency and inverse document frequency.

## Why Implement TF-IDF?

**High Priority** (from `RETRIEVAL_METHODS_RESEARCH.md`):
- Simple implementation (easier than BM25)
- Fast (similar computational complexity)
- Complementary (sometimes outperforms BM25 on specific datasets)
- Educational value (helps users understand BM25 evolution)
- Low overhead (can reuse existing inverted index structure)

## Key Differences from BM25

### 1. Term Frequency (TF)

**TF-IDF:**
- Linear or log-linear growth: `tf = f_{t,d}` or `tf = 1 + log(f_{t,d})`
- No saturation: Repeating a term always increases score
- Simple computation: Just count term occurrences

**BM25:**
- Saturating TF: `tf = (f_{t,d} * (k1 + 1)) / (f_{t,d} + k1 * (1 - b + b * |d|/avgdl))`
- Saturation: Extra repetitions have diminishing returns
- Parameters: `k1` controls saturation, `b` controls length normalization

### 2. Document Length Normalization

**TF-IDF:**
- **No explicit normalization** in basic form
- Longer documents naturally get higher scores (more terms)
- Can add custom normalization if needed

**BM25:**
- **Explicit length normalization** via `|d|/avgdl` in denominator
- Parameter `b` controls normalization strength
- Prevents bias toward longer documents

### 3. Inverse Document Frequency (IDF)

**TF-IDF:**
- Standard IDF: `idf = log(N / df_t)`
- Where `N` = total documents, `df_t` = documents containing term `t`
- Simple logarithmic scaling

**BM25:**
- Smoothed IDF: `idf = log(1 + (N - df_t + 0.5) / (df_t + 0.5))`
- More stable for rare and common terms
- Avoids negative weights when term appears in >50% of documents

### 4. Parameters

**TF-IDF:**
- Parameter-free (aside from choosing TF variant and log base)
- No tuning required once formula is fixed

**BM25:**
- Tunable parameters: `k1` (TF saturation), `b` (length normalization)
- Defaults: `k1 ≈ 1.2-2.0`, `b ≈ 0.75`
- Requires parameter tuning for optimal performance

## Implementation Plan

### 1. Reuse BM25 Infrastructure

**Shared Components:**
- Inverted index structure (`InvertedIndex`)
- Document posting lists
- Term frequency storage
- IDF computation (can share with BM25)

**TF-IDF-Specific:**
- Simpler scoring formula (no saturation, no length normalization)
- Different IDF formula (optional - can use same as BM25 for consistency)

### 2. API Design

**Concrete Function** (consistent with existing API):
```rust
pub fn retrieve_tfidf(
    index: &InvertedIndex,
    query_terms: &[String],
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError>
```

**Feature Flag:**
```toml
[features]
tfidf = []  # Separate from bm25 feature
```

**Parameters** (optional, for flexibility):
```rust
pub struct TfIdfParams {
    /// TF variant: linear or log-scaled
    pub tf_variant: TfVariant,
    /// IDF formula: standard or smoothed
    pub idf_variant: IdfVariant,
}

pub enum TfVariant {
    Linear,      // tf = f_{t,d}
    LogScaled,   // tf = 1 + log(f_{t,d})
}

pub enum IdfVariant {
    Standard,   // idf = log(N / df_t)
    Smoothed,   // idf = log(1 + (N - df_t + 0.5) / (df_t + 0.5))
}
```

### 3. Scoring Formula

**Basic TF-IDF:**
```rust
fn score_tfidf(term: &str, doc_id: u32, index: &InvertedIndex) -> f32 {
    let tf = index.term_frequency(term, doc_id);
    let idf = index.inverse_document_frequency(term);
    tf * idf
}

fn retrieve_tfidf(
    index: &InvertedIndex,
    query_terms: &[String],
    k: usize,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    // For each document containing query terms:
    //   score = sum over terms: tf(term, doc) * idf(term)
    // Return top-k by score
}
```

**TF Variants:**
- **Linear**: `tf = f_{t,d}` (raw count)
- **Log-scaled**: `tf = 1 + log(f_{t,d})` (reduces impact of very high frequencies)

**IDF Variants:**
- **Standard**: `idf = log(N / df_t)`
- **Smoothed**: `idf = log(1 + (N - df_t + 0.5) / (df_t + 0.5))` (BM25-style, more stable)

### 4. Implementation Steps

1. **Add TF-IDF module** (`src/tfidf.rs`):
   - `TfIdfParams` struct
   - `retrieve_tfidf()` function
   - Scoring logic

2. **Reuse `InvertedIndex`**:
   - Same data structure as BM25
   - Can share IDF computation (or use different formula)

3. **Add feature flag**:
   - `tfidf` feature in `Cargo.toml`
   - Feature-gate TF-IDF code

4. **Add tests**:
   - Unit tests for scoring formula
   - Integration tests with `InvertedIndex`
   - Comparison tests with BM25

5. **Add example**:
   - `examples/tfidf_retrieval.rs`
   - Show TF-IDF vs BM25 comparison

## Performance Considerations

**Computational Complexity:**
- Same as BM25: O(|Q| * |D|) where Q = query terms, D = documents
- Can reuse BM25 optimizations (early termination, SIMD if applicable)

**Memory:**
- Same as BM25: Reuses `InvertedIndex` structure
- No additional memory overhead

**Speed:**
- Slightly faster than BM25 (simpler formula, no length normalization)
- Estimated: 5-10% faster than BM25

## When to Use TF-IDF vs BM25

**Use TF-IDF when:**
- Need simpler baseline for comparison
- Datasets where TF-IDF outperforms BM25 (rare but documented)
- Educational/prototyping scenarios
- Want parameter-free retrieval

**Use BM25 when:**
- Need better handling of document length
- Want saturation to prevent term repetition abuse
- Production systems (generally better performance)
- Need tunable parameters

## Research Evidence

- TF-IDF remains competitive in some domains despite BM25's dominance
- Useful as a baseline for benchmarking
- Lower computational overhead than BM25 (no saturation function)
- Sometimes outperforms BM25 on specific datasets (documented in research)

## Implementation Checklist

- [ ] Add `tfidf` feature flag
- [ ] Create `src/tfidf.rs` module
- [ ] Implement `TfIdfParams` struct
- [ ] Implement `retrieve_tfidf()` function
- [ ] Add unit tests
- [ ] Add integration tests
- [ ] Add example (`examples/tfidf_retrieval.rs`)
- [ ] Update README with TF-IDF documentation
- [ ] Benchmark TF-IDF vs BM25 performance

## Estimated Effort

**1-2 days** (from `RETRIEVAL_METHODS_RESEARCH.md`):
- Simple implementation
- Reuses BM25 infrastructure
- Good baseline/comparison method

## References

- `RETRIEVAL_METHODS_RESEARCH.md` - Original research and recommendations
- BM25 implementation in `src/bm25.rs` - Reference for shared infrastructure
- Inverted index in `src/bm25.rs` - Data structure to reuse

## See Also

- [RETRIEVAL_METHODS_RESEARCH.md](RETRIEVAL_METHODS_RESEARCH.md) - Full research on additional retrieval methods
- [BM25 Implementation](../src/bm25.rs) - Reference implementation
- [USE_CASES.md](USE_CASES.md) - When to use different retrieval methods
