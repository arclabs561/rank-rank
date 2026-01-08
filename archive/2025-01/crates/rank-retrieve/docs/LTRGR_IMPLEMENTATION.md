# LTRGR Implementation Plan: Learning to Rank in Generative Retrieval

**Paper**: "Learning to Rank in Generative Retrieval" (AAAI 2024)  
**Authors**: Yongqi Li, Nan Yang, Liang Wang, Furu Wei, Wenjie Li  
**arXiv**: 2306.15222v2  
**GitHub**: https://github.com/liyongqi67/LTRGR

## Overview

LTRGR (Learning-to-Rank for Generative Retrieval) bridges the gap between generative retrieval and passage ranking by adding a learning-to-rank training phase. Generative retrieval generates identifiers (titles, substrings, pseudo-queries) for passages, but optimizing for identifier generation doesn't directly optimize for passage ranking. LTRGR adds a margin-based rank loss to directly optimize passage ranking.

## Key Concepts

### Generative Retrieval

Instead of encoding queries and passages into vectors (dense retrieval) or using term matching (BM25), generative retrieval uses autoregressive language models to **generate identifiers** of relevant passages:

- **Input**: Query text
- **Output**: Identifier strings (e.g., "Prime Rate in Canada", passage substrings, pseudo-queries)
- **Mapping**: Identifiers map to passages (many-to-one for text-based identifiers)

### The Learning Gap

1. **Generation objective**: Maximize likelihood of correct identifiers
2. **Ranking objective**: Rank passages optimally
3. **Disconnect**: Good identifier generation ≠ optimal passage ranking

### LTRGR Solution

Two-phase training:
1. **Learning-to-generate**: Standard generative retrieval (identifier generation)
2. **Learning-to-rank**: Margin-based loss optimizing passage ranking directly

## Architecture

```
Query → Autoregressive Model → Predicted Identifiers → Heuristic Scoring → Passage Rank List
                                                                    ↓
                                                          Margin-based Rank Loss
                                                                    ↓
                                                          Optimize Model
```

## Implementation Structure

### Module Organization

```
rank-retrieve/
├── src/
│   ├── generative/              # NEW: Generative retrieval module
│   │   ├── mod.rs               # Module exports
│   │   ├── identifier.rs        # Multiview identifier generation
│   │   ├── fm_index.rs          # FM-index for constrained generation
│   │   ├── scorer.rs            # Heuristic scoring function
│   │   ├── ltrgr.rs             # LTR training integration
│   │   └── model.rs             # Autoregressive model wrapper
│   └── ...
└── docs/
    └── LTRGR_IMPLEMENTATION.md  # This file
```

## Component Details

### 1. Identifier Generation (`identifier.rs`)

**Purpose**: Generate multiview identifiers for passages.

**Types**:
- **Title**: Passage title (e.g., "Prime Rate in Canada")
- **Substring**: Random substring from passage body
- **Pseudo-query**: Generated query-like representation

**API**:
```rust
pub enum IdentifierType {
    Title,
    Substring,
    PseudoQuery,
}

pub struct MultiviewIdentifier {
    pub title: String,
    pub substring: String,
    pub pseudo_query: String,
}

pub trait IdentifierGenerator {
    fn generate(&self, passage: &str) -> MultiviewIdentifier;
}
```

**Implementation Notes**:
- Title extraction: Use metadata or extract from passage
- Substring: Random sampling from passage body
- Pseudo-query: Could use LLM or template-based generation

### 2. FM-Index (`fm_index.rs`)

**Purpose**: Constrain generation to valid identifiers only.

**Why FM-Index**:
- Autoregressive models can generate invalid identifiers
- FM-index provides valid token successors given a prefix
- Ensures all generated identifiers exist in corpus

**API**:
```rust
pub struct FMIndex {
    // Internal FM-index structure
}

impl FMIndex {
    pub fn new(identifiers: &[String]) -> Self;
    pub fn get_successors(&self, prefix: &str) -> Vec<String>;
    pub fn contains(&self, identifier: &str) -> bool;
}
```

**Dependencies**:
- Need Rust FM-index implementation (e.g., `fm-index` crate or custom)
- Or use existing library bindings

**Implementation Options**:
1. Use existing Rust FM-index crate (if available)
2. Implement basic FM-index (simplified version)
3. Use alternative constrained generation (e.g., trie-based)

### 3. Heuristic Scorer (`scorer.rs`)

**Purpose**: Convert predicted identifiers to passage scores.

**Formula** (from paper, Equation 3):
```
s(q, p) = Σ_{ip ∈ Ip} s_ip
```

Where:
- `Ip` = set of predicted identifiers that appear in passage `p`
- `s_ip` = language model score of identifier `ip`

**API**:
```rust
pub struct HeuristicScorer {
    // Configuration
}

impl HeuristicScorer {
    pub fn score_passage(
        &self,
        passage: &str,
        predicted_identifiers: &[(String, f32)], // (identifier, score)
    ) -> f32;
    
    pub fn score_batch(
        &self,
        passages: &[&str],
        predicted_identifiers: &[(String, f32)],
    ) -> Vec<f32>;
}
```

**Implementation**:
- For each passage, find matching identifiers
- Sum scores of matching identifiers
- Return ranked list of passages

### 4. LTR Training (`ltrgr.rs`)

**Purpose**: Learning-to-rank training phase.

**Loss Function** (from paper, Equation 4-5):
```rust
// Margin-based rank loss
L_rank = max(0, s(q, p_n) - s(q, p_p) + m)

// Multi-task loss
L = L_rank1 + L_rank2 + λ * L_gen
```

Where:
- `L_rank1`: Rank loss with highest-scoring positive/negative
- `L_rank2`: Rank loss with randomly sampled positive/negative
- `L_gen`: Generation loss (standard autoregressive loss)
- `m`: Margin (default: 500)
- `λ`: Weight (default: 1000)

**API**:
```rust
pub struct LTRGRTrainer {
    margin: f32,
    lambda: f32,
    // Model reference
}

impl LTRGRTrainer {
    pub fn new(margin: f32, lambda: f32) -> Self;
    
    pub fn compute_rank_loss(
        &self,
        positive_score: f32,
        negative_score: f32,
    ) -> f32;
    
    pub fn compute_total_loss(
        &self,
        rank_loss_1: f32,
        rank_loss_2: f32,
        gen_loss: f32,
    ) -> f32;
}
```

**Training Loop**:
1. Phase 1: Train on generation loss (standard generative retrieval)
2. Phase 2: For each query in training set:
   - Generate identifiers
   - Score passages using heuristic function
   - Sample positive/negative passages
   - Compute rank loss
   - Backpropagate

### 5. Model Integration (`model.rs`)

**Purpose**: Wrapper for autoregressive language models.

**Requirements**:
- Autoregressive generation (BART, T5, GPT-style)
- Beam search support
- Constrained generation (via FM-index)
- Gradient computation for training

**API**:
```rust
pub trait AutoregressiveModel {
    fn generate(
        &self,
        query: &str,
        prefix: &str,  // Identifier prefix ("title", "substring", "pseudo-query")
        beam_size: usize,
        fm_index: Option<&FMIndex>,  // For constrained generation
    ) -> Vec<(String, f32)>;  // (identifier, score)
    
    fn generate_batch(
        &self,
        queries: &[&str],
        prefix: &str,
        beam_size: usize,
        fm_index: Option<&FMIndex>,
    ) -> Vec<Vec<(String, f32)>>;
}
```

**Implementation Options**:
1. **External model**: Use `candle` or `burn` for model loading
2. **Python bridge**: Call Python models via PyO3
3. **Trait-based**: Define trait, users provide implementation

**Recommended**: Trait-based approach for flexibility.

## Training Workflow

### Phase 1: Learning to Generate

```rust
// Standard generative retrieval training
for (query, target_identifiers) in training_data {
    let loss = model.compute_generation_loss(query, target_identifiers);
    loss.backward();
    optimizer.step();
}
```

### Phase 2: Learning to Rank

```rust
// Retrieve passages for all training queries
let mut passage_rank_lists = Vec::new();
for query in training_queries {
    let identifiers = model.generate(query, beam_size=15, fm_index);
    let passages = scorer.score_passages(identifiers, corpus);
    passage_rank_lists.push(passages);
}

// LTR training
for (query, passage_list) in training_queries.zip(passage_rank_lists) {
    // Sample positive/negative passages
    let (pos, neg) = sample_passages(&passage_list);
    
    // Compute rank losses
    let rank_loss_1 = compute_rank_loss(
        highest_positive_score,
        highest_negative_score,
    );
    let rank_loss_2 = compute_rank_loss(
        random_positive_score,
        random_negative_score,
    );
    
    // Generation loss (keep identifier generation capability)
    let gen_loss = model.compute_generation_loss(query, target_identifiers);
    
    // Total loss
    let total_loss = rank_loss_1 + rank_loss_2 + lambda * gen_loss;
    total_loss.backward();
    optimizer.step();
}
```

## Dependencies

### Required

- **Autoregressive model**: BART, T5, or similar
  - Options: `candle`, `burn`, or Python bridge
- **FM-index**: For constrained generation
  - Need to find or implement Rust FM-index

### Optional

- **rank-learn**: For LTR loss functions (or implement self-contained)
- **rank-soft**: For differentiable ranking operations (optional enhancement)

## Integration Points

### With rank-retrieve

- **New retrieval method**: `GenerativeRetriever` alongside `Bm25Retriever`, `DenseRetriever`
- **Unified API**: Same `retrieve()` interface
- **Pipeline integration**: Works in hybrid retrieval scenarios

### With rank-learn

- **LTR losses**: Could use `rank-learn` for margin-based loss
- **Training utilities**: Reuse training infrastructure
- **Or self-contained**: Keep LTR training in `rank-retrieve` for simplicity

### With rank-soft

- **Differentiable operations**: Use soft ranking for passage scoring (optional)
- **Loss functions**: Could enhance rank loss with differentiable operations

## Implementation Phases

### Phase 1: Core Infrastructure (Foundation)

1. **Identifier generation** (`identifier.rs`)
   - Basic multiview identifier extraction
   - Title, substring, pseudo-query generation
   - Tests with sample passages

2. **Heuristic scorer** (`scorer.rs`)
   - Basic scoring function
   - Identifier-to-passage matching
   - Batch scoring

### Phase 2: Generation Infrastructure

3. **FM-index** (`fm_index.rs`)
   - Basic FM-index implementation or wrapper
   - Constrained generation support
   - Integration with model generation

4. **Model wrapper** (`model.rs`)
   - Trait definition
   - Basic implementation (e.g., using `candle`)
   - Beam search integration

### Phase 3: LTR Training

5. **LTR trainer** (`ltrgr.rs`)
   - Margin-based rank loss
   - Multi-task loss combination
   - Training loop implementation

6. **End-to-end integration**
   - Two-phase training workflow
   - Inference pipeline
   - Evaluation integration

## Testing Strategy

### Unit Tests

- Identifier generation: Test title/substring/pseudo-query extraction
- FM-index: Test constrained generation
- Scorer: Test heuristic scoring function
- Rank loss: Test margin-based loss computation

### Integration Tests

- Full retrieval pipeline: Query → Identifiers → Passages
- Training loop: Two-phase training
- Evaluation: Compare with baseline generative retrieval

### Benchmarks

- **Datasets**: Natural Questions, TriviaQA, MS MARCO
- **Metrics**: hits@5, hits@20, hits@100, MRR@10
- **Baselines**: MINDER, SEAL, DPR

## Challenges & Solutions

### Challenge 1: Autoregressive Model Integration

**Problem**: Need to integrate with BART/T5 models.

**Solutions**:
- Use `candle` for model loading (Rust-native)
- Python bridge via PyO3 (easier, but adds Python dependency)
- Trait-based design (users provide model implementation)

**Recommendation**: Start with trait-based, provide `candle` example.

### Challenge 2: FM-Index Implementation

**Problem**: Need FM-index for constrained generation.

**Solutions**:
- Find existing Rust FM-index crate
- Implement simplified version (trie-based alternative)
- Use Python FM-index via PyO3

**Recommendation**: Start with simplified trie-based constrained generation, upgrade to FM-index later.

### Challenge 3: Training Infrastructure

**Problem**: Need training loop with gradient computation.

**Solutions**:
- Use `candle` or `burn` for autograd
- Python bridge for training (use existing PyTorch code)
- Self-contained Rust training (more work)

**Recommendation**: Start with Python bridge for training, migrate to Rust later if needed.

## Performance Considerations

### Inference Speed

- **Beam search**: Configurable beam size (default: 15)
- **FM-index lookup**: Should be fast (O(m) where m = identifier length)
- **Scoring**: O(n * k) where n = passages, k = identifiers per passage

### Training Speed

- **Phase 1**: Standard generative retrieval (fast)
- **Phase 2**: Requires retrieving passages for all training queries (slower)
- **Optimization**: Batch processing, parallel retrieval

## Future Enhancements

1. **Differentiable scoring**: Use `rank-soft` for differentiable passage scoring
2. **Advanced identifiers**: Learn optimal identifier representations
3. **Hybrid retrieval**: Combine generative + dense + BM25
4. **Efficient FM-index**: Optimize constrained generation
5. **Multi-GPU training**: Scale to larger models

## References

- **Paper**: [arXiv:2306.15222v2](https://arxiv.org/abs/2306.15222)
- **GitHub**: https://github.com/liyongqi67/LTRGR
- **MINDER**: Multiview Identifiers Enhanced Generative Retrieval (baseline)
- **SEAL**: Autoregressive search engines (generative retrieval baseline)

## Status

- **Current**: 📋 Documented, not implemented
- **Priority**: Medium (novel approach, requires significant infrastructure)
- **Estimated Effort**: 2-3 months for full implementation
- **Dependencies**: Autoregressive model integration, FM-index, training infrastructure

