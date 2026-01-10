# Query Likelihood / Language Models Implementation Research

This document synthesizes research on query likelihood language models for `rank-retrieve`.

## Overview

Query likelihood models rank documents by the probability that the document's language model generated the query: `P(Q|D)`. This inverts the traditional relevance question: instead of asking "how relevant is document D to query Q?", we ask "how likely is query Q to be generated from document D's language model?"

## Theoretical Foundation

### Basic Query Likelihood Model

The ranking score is:
```
score(Q, D) = P(Q|D) = ∏ P(q_i|D)
```

Where:
- `Q = {q_1, q_2, ..., q_n}` is the query
- `D` is a document
- `P(q_i|D)` is the probability of term `q_i` in document D's language model

### Zero-Probability Problem

The basic model assigns zero probability to queries containing terms not in the document. This is solved through **smoothing**: assigning non-zero probabilities to unseen terms.

## Smoothing Techniques

### 1. Jelinek-Mercer Smoothing

Interpolates between document language model and corpus language model:

```
P(q_i|D) = λ * P(q_i|D) + (1 - λ) * P(q_i|C)
```

Where:
- `λ` (lambda) is the interpolation parameter (typically 0.1-0.7)
- `P(q_i|D)` is the term probability in document D
- `P(q_i|C)` is the term probability in corpus C

**Characteristics:**
- Simple and effective
- `λ` controls the balance between document-specific and corpus-wide probabilities
- Lower `λ` (e.g., 0.1-0.3): More weight to corpus, better for short documents
- Higher `λ` (e.g., 0.5-0.7): More weight to document, better for long documents

### 2. Dirichlet Smoothing

Uses a Bayesian approach with a Dirichlet prior:

```
P(q_i|D) = (c(q_i, D) + μ * P(q_i|C)) / (|D| + μ)
```

Where:
- `c(q_i, D)` is the count of term `q_i` in document D
- `|D|` is the document length
- `μ` (mu) is the smoothing parameter (typically 50-2000)
- `P(q_i|C)` is the corpus probability

**Characteristics:**
- More sophisticated than Jelinek-Mercer
- Automatically adapts to document length (longer docs → less smoothing)
- `μ` controls smoothing strength (higher μ = more smoothing)
- Often performs better than Jelinek-Mercer in practice

## Implementation Plan

### API Design

```rust
pub enum SmoothingMethod {
    JelinekMercer { lambda: f32 },  // Default: 0.5
    Dirichlet { mu: f32 },  // Default: 1000
}

pub struct QueryLikelihoodParams {
    pub smoothing: SmoothingMethod,
}

pub fn retrieve_query_likelihood(
    index: &InvertedIndex,
    query: &[String],
    k: usize,
    params: QueryLikelihoodParams,
) -> Result<Vec<(u32, f32)>, RetrieveError>
```

### Implementation Steps

1. Extend `InvertedIndex` to compute corpus probabilities `P(q_i|C)`
2. Implement Jelinek-Mercer smoothing
3. Implement Dirichlet smoothing
4. Add query likelihood scoring function
5. Add tests
6. Add example

### Index Extensions

Query likelihood requires:
- **Corpus term frequencies**: Total count of each term across all documents
- **Corpus size**: Total number of terms in corpus (for normalization)

These can be computed from existing `InvertedIndex`:
- Corpus term frequency: Sum of term frequencies across all documents
- Corpus size: Sum of all document lengths

### Performance Considerations

- Precompute corpus probabilities during indexing (lazy computation)
- Use log probabilities to avoid underflow: `log P(Q|D) = Σ log P(q_i|D)`
- Early termination for top-k retrieval
- Reuse existing inverted index structure

## Research Evidence

- Query likelihood with Dirichlet smoothing is competitive with BM25
- Often performs better on short queries
- Better theoretical foundation than BM25 (probabilistic vs. heuristic)
- Jelinek-Mercer is simpler but Dirichlet often performs better

## When to Use

- Research/prototyping scenarios
- Queries where probabilistic approach helps
- When theoretical grounding is important
- As a baseline for comparison with BM25/TF-IDF

## References

- "Using Query Likelihood Language Models in IR" (Stanford)
- "Language Models for Information Retrieval" (Jelinek-Mercer, Dirichlet)
- Recent LLM-QL research (2024-2025): Integration with large language models
