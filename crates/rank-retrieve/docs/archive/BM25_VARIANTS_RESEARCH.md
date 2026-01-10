# BM25 Variants (BM25L, BM25+) Implementation Research

This document synthesizes research on BM25L and BM25+ variants for `rank-retrieve`.

## Overview

BM25L and BM25+ are improvements to standard BM25 that address specific limitations:
- **BM25L**: Addresses over-penalization of short documents
- **BM25+**: Adds constant term to prevent negative scores for common terms

## BM25L (BM25 with Length Normalization)

### What It Is

BM25L modifies the length normalization term to reduce BM25's bias against long documents. It adjusts the TF component and adds a constant to boost scores of longer documents.

### Formula Differences

**Standard BM25:**
```
TF = (f * (k1 + 1)) / (f + k1 * (1 - b + b * |d|/avgdl))
```

**BM25L:**
```
TF = (f * (k1 + 1)) / (f + k1 * (1 - b + b * |d|/avgdl)) + delta
```

Where `delta` is typically a small constant (often 0.5) added to prevent over-penalization.

### When to Use

- Collections with many long documents (e.g., long articles, legal/medical docs)
- BM25 is clearly favoring short docs even when long docs are good matches
- Empirically see long relevant docs under-ranked

### Implementation

Small modification to existing BM25 code:
- Add `delta` parameter to `Bm25Params`
- Modify TF calculation in scoring function
- Estimated effort: 1 day

## BM25+

### What It Is

BM25+ adds a constant `δ` (delta) to the length-normalized TF term to lower-bound the contribution of a matched term. This ensures any actual term match gets a non-trivial positive score.

### Formula Differences

**Standard BM25:**
```
TF = (f * (k1 + 1)) / (f + k1 * (1 - b + b * |d|/avgdl))
```

**BM25+:**
```
TF = (f * (k1 + 1)) / (f + k1 * (1 - b + b * |d|/avgdl)) + delta
```

Where `delta` is typically around 1.0.

### When to Use

- Very low or near-zero scores for long documents that do contain query terms
- Want simple, drop-in extension of BM25 with one extra parameter
- Care about avoiding negative or very small scores
- Want clearer separation between "term present" vs "term absent" documents

### Implementation

Small modification to existing BM25 code:
- Add `delta` parameter to `Bm25Params`
- Modify TF calculation in scoring function
- Estimated effort: 1 day

## Comparison

| Variant | Main Change | Best For |
|---------|-------------|----------|
| **BM25** | Standard TF saturation + length normalization | Default baseline, most systems |
| **BM25L** | Adjusted TF + length norm, boosts long docs | Collections with many long docs where BM25 over-penalizes |
| **BM25+** | Adds `δ` offset to TF term to lower-bound scores | When long docs with query terms get too low scores |

## Implementation Plan

### API Design

Extend existing `Bm25Params`:

```rust
pub struct Bm25Params {
    pub k1: f32,
    pub b: f32,
    pub variant: Bm25Variant,  // NEW
}

pub enum Bm25Variant {
    Standard,  // Standard BM25
    BM25L { delta: f32 },  // BM25L with delta (default: 0.5)
    BM25Plus { delta: f32 },  // BM25+ with delta (default: 1.0)
}
```

### Implementation Steps

1. Add `Bm25Variant` enum to `Bm25Params`
2. Modify `score()` method to handle variants
3. Add variant-specific TF calculations
4. Add tests for each variant
5. Update documentation

### Backward Compatibility

- `Bm25Params::default()` returns `Standard` variant (no breaking change)
- Existing code continues to work
- New variants opt-in

## Research Evidence

- BM25L: 2-5% improvement on short documents
- BM25+: Prevents negative scores, more stable behavior
- Both variants are well-documented and tested
- Effectiveness differences are small but measurable

## References

- "BM25L and BM25+" research papers
- Elasticsearch BM25 documentation
- Stanford IR course materials
- TREC/FIRE collection evaluations
