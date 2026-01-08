# LTRGR Integration Summary

**Paper**: "Learning to Rank in Generative Retrieval" (AAAI 2024, arXiv:2306.15222v2)  
**Status**: Documented and planned for implementation

## Where LTRGR Fits

LTRGR (Learning-to-Rank for Generative Retrieval) is a novel retrieval paradigm that combines:
1. **Generative retrieval**: Autoregressive models generate passage identifiers
2. **Learning-to-rank**: Margin-based loss optimizes passage ranking directly

### Primary Location: `rank-retrieve`

LTRGR is primarily a **retrieval method**, so it belongs in `rank-retrieve` alongside:
- BM25 retrieval (`bm25` module)
- Dense retrieval (`dense` module)
- Sparse retrieval (`sparse` module)
- **Generative retrieval** (`generative` module) ← NEW

### Cross-Crate Dependencies

LTRGR spans multiple crates:

| Crate | Role | Components |
|-------|------|------------|
| **rank-retrieve** | Primary location | Identifier generation, FM-index, heuristic scoring, retrieval API |
| **rank-learn** | Training support | Margin-based rank loss, multi-task training (optional) |
| **rank-soft** | Optional enhancement | Differentiable ranking operations for rank loss |

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     rank-retrieve                            │
│                                                               │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │    BM25      │  │    Dense     │  │   Sparse     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│                                                               │
│  ┌──────────────────────────────────────────────────────┐    │
│  │         Generative Retrieval (LTRGR)                │    │
│  │                                                       │    │
│  │  ┌──────────────┐  ┌──────────────┐                │    │
│  │  │ Identifier   │  │   FM-Index   │                │    │
│  │  │  Generation  │  │  (constrained)│                │    │
│  │  └──────────────┘  └──────────────┘                │    │
│  │                                                       │    │
│  │  ┌──────────────┐  ┌──────────────┐                │    │
│  │  │   Heuristic  │  │   LTRGR      │                │    │
│  │  │    Scorer    │  │   Trainer    │                │    │
│  │  └──────────────┘  └──────────────┘                │    │
│  └──────────────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│ rank-learn   │    │ rank-soft    │    │ rank-rerank  │
│              │    │              │    │              │
│ Margin-based │    │ Differentiable│   │ (downstream) │
│ rank loss    │    │ operations   │    │              │
│ (optional)   │    │ (optional)   │    │              │
└──────────────┘    └──────────────┘    └──────────────┘
```

## Key Components

### 1. Identifier Generation (`rank-retrieve/src/generative/identifier.rs`)

Generates multiview identifiers for passages:
- **Title**: Passage title (e.g., "Prime Rate in Canada")
- **Substring**: Random substring from passage body
- **Pseudo-query**: Query-like representation

### 2. FM-Index (`rank-retrieve/src/generative/fm_index.rs`)

Constrains generation to valid identifiers:
- Prevents invalid identifier generation
- Provides valid token successors given prefix
- Ensures all generated identifiers exist in corpus

### 3. Heuristic Scorer (`rank-retrieve/src/generative/scorer.rs`)

Converts predicted identifiers to passage scores:
```
s(q, p) = Σ_{ip ∈ Ip} s_ip
```
Where `Ip` = identifiers matching passage `p`, `s_ip` = identifier score

### 4. LTR Trainer (`rank-retrieve/src/generative/ltrgr.rs`)

Learning-to-rank training phase:
- Margin-based rank loss: `max(0, s(q, p_n) - s(q, p_p) + m)`
- Multi-task loss: `L = L_rank1 + L_rank2 + λ * L_gen`
- Two-phase training: generate → rank

## Training Workflow

### Phase 1: Learning to Generate
Standard generative retrieval training:
- Input: Query + target identifiers
- Loss: Generation loss (autoregressive)
- Output: Model that generates identifiers

### Phase 2: Learning to Rank
LTR training phase:
1. Generate identifiers for all training queries
2. Score passages using heuristic function
3. Sample positive/negative passages
4. Compute margin-based rank loss
5. Combine with generation loss
6. Backpropagate

## Integration with Existing Pipeline

LTRGR fits into the standard retrieval pipeline:

```
Query
  ↓
[rank-retrieve: Generative Retrieval (LTRGR)]
  ↓
10M docs → 1000 candidates
  ↓
[rank-fusion: Combine with BM25/Dense results]
  ↓
[rank-rerank: Rerank top candidates]
  ↓
100 candidates
  ↓
[rank-eval: Evaluate results]
```

## Results from Paper

| Dataset | Metric | LTRGR | MINDER (baseline) | Improvement |
|---------|--------|-------|-------------------|-------------|
| Natural Questions | hits@5 | 68.8 | 65.8 | +4.56% |
| TriviaQA | hits@5 | 70.2 | 68.4 | +2.63% |
| MS MARCO | R@5 | 40.2 | 29.5 | +36.3% |
| MS MARCO | MRR@10 | 25.5 | 18.6 | +28.8% |

## Implementation Status

- ✅ **Documented**: Added to `RESEARCH_CONNECTIONS.md`
- ✅ **Gap Analysis**: Added to `RESEARCH_BASED_IMPLEMENTATION_GAPS.md`
- ✅ **Implementation Plan**: Created `LTRGR_IMPLEMENTATION.md`
- ⏳ **Implementation**: Not yet started

## Priority & Effort

- **Priority**: Medium
  - Novel approach with validated improvements
  - Requires significant infrastructure (autoregressive models, FM-index)
  - Lower priority than LTRR query routing (higher impact)

- **Estimated Effort**: 2-3 months
  - Phase 1 (Core): 3-4 weeks
  - Phase 2 (Generation): 4-6 weeks
  - Phase 3 (LTR Training): 4-6 weeks

## Dependencies

### Required
- Autoregressive model (BART, T5) - external dependency
- FM-index implementation - need to find or implement
- Training infrastructure - autograd support

### Optional
- `rank-learn`: For LTR loss functions (or self-contained)
- `rank-soft`: For differentiable operations (enhancement)

## Next Steps

1. **Research**: Find or implement FM-index in Rust
2. **Design**: Finalize model integration approach (trait-based vs concrete)
3. **Prototype**: Start with identifier generation and heuristic scoring
4. **Iterate**: Add FM-index and constrained generation
5. **Train**: Implement LTR training phase
6. **Evaluate**: Benchmark on NQ, TriviaQA, MS MARCO

## Related Documents

- **Research Connection**: `docs/analysis/RESEARCH_CONNECTIONS.md` (LTRGR entry)
- **Implementation Gaps**: `RESEARCH_BASED_IMPLEMENTATION_GAPS.md` (rank-retrieve section)
- **Detailed Plan**: `crates/rank-retrieve/docs/LTRGR_IMPLEMENTATION.md`

## Key Insight

Generative retrieval learns to generate identifiers as an intermediate goal, but the disconnect between the generation objective and the ranking target creates a learning gap. LTRGR bridges this gap by directly optimizing for passage ranking while maintaining identifier generation capability.

