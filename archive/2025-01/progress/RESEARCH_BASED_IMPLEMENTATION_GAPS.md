# Research-Based Implementation Gaps Analysis

Comprehensive analysis of what each rank-* crate should implement based on latest research (2024-2025) but doesn't yet.

## rank-retrieve

### Current State
- ✅ BM25 retrieval (basic inverted index)
- ✅ Dense retrieval (cosine similarity)
- ✅ Sparse retrieval (sparse vectors)
- ✅ Python bindings
- ✅ Property-based testing

### Research Gaps (2024-2025)

#### 1. Query Routing Framework (LTRR, arXiv:2506.13743)
**Status**: Not implemented
**Impact**: High - 10-20% improvement in retrieval quality
**What**: Learning to Rank Retrievers - dynamically select from pool of retrievers based on query characteristics
**Implementation**:
- Query feature extraction (length, complexity, domain)
- Retriever selection model (pairwise XGBoost)
- Multi-retriever fusion with routing
- Utility-aware training (BEM, AC metrics)

#### 2. Production ANN Integration
**Status**: Planned but not implemented
**Impact**: High - required for scale
**What**: HNSW/FAISS integration for dense retrieval at scale
**Implementation**:
- Optional `hnsw` feature for HNSW indexing
- Optional `faiss` feature for FAISS backend
- Tantivy integration for production BM25

#### 3. ColBERT Late Interaction Retrieval
**Status**: Not implemented
**Impact**: Medium - better quality than dense-only
**What**: Token-level retrieval using MaxSim (delegates to rank-rerank for scoring)
**Implementation**:
- Token-level indexing
- MaxSim-based retrieval (uses rank-rerank)
- Integration with rank-rerank for scoring

#### 4. Batch Retrieval Operations
**Status**: Not implemented
**Impact**: Medium - performance optimization
**What**: Process multiple queries efficiently
**Implementation**:
- Batch query processing
- Streaming/chunked processing
- Parallel retrieval

#### 5. Generative Retrieval with Learning-to-Rank (LTRGR, arXiv:2306.15222)
**Status**: Not implemented
**Impact**: Medium - novel retrieval paradigm, 2-36% improvement over baseline generative retrieval
**What**: Generative retrieval that generates passage identifiers (titles, substrings, pseudo-queries) with LTR training phase
**Implementation**:
- Identifier generation module (multiview: title, substring, pseudo-query)
- FM-index integration for constrained generation
- Heuristic scoring function (identifier → passage scores)
- Margin-based rank loss for LTR training phase
- Two-phase training framework (generate → rank)
- Integration with autoregressive language models (BART, T5)

**Key Components**:
- `generative/identifier.rs` - Multiview identifier generation
- `generative/fm_index.rs` - FM-index for constrained generation
- `generative/scorer.rs` - Heuristic scoring (sum of identifier scores)
- `generative/ltrgr.rs` - LTR training integration
- Training: margin loss `max(0, s(q, p_n) - s(q, p_p) + m)`
- Multi-task: `L = L_rank1 + L_rank2 + λL_gen`

**Dependencies**:
- Autoregressive model (BART/T5) - external dependency
- FM-index library (Rust implementation needed)
- rank-learn or self-contained LTR training

**Results from Paper**:
- NQ: 68.8 hits@5 (vs 65.8 MINDER baseline)
- TriviaQA: 70.2 hits@5 (vs 68.4 MINDER)
- MS MARCO: 40.2 R@5, 25.5 MRR@10 (vs 29.5 R@5, 18.6 MRR@10 MINDER)

**Priority**: Medium - novel approach but requires significant infrastructure (autoregressive models, FM-index)

### Priority Order
1. Query routing framework (LTRR) - highest research impact
2. Production ANN integration - required for scale
3. Batch operations - performance optimization
4. ColBERT integration - quality improvement
5. Generative retrieval (LTRGR) - novel paradigm, medium priority

---

## rank-fusion

### Current State
- ✅ RRF, CombSum, CombMNZ, Borda, DBSF, ISR
- ✅ Additive multi-task fusion (ResFlow)
- ✅ Standardized fusion (ERANK-style)
- ✅ Python bindings (partial - only 4 functions)
- ✅ Comprehensive testing

### Research Gaps (2024-2025)

#### 1. Complete Python Bindings
**Status**: Only 4/20+ functions exposed
**Impact**: High - limits adoption
**What**: Expose all algorithms (`combsum`, `combmnz`, `weighted`, etc.), `explain` module, configuration classes
**Implementation**:
- Expose all `_multi` variants
- Add `explain` module to Python
- Add configuration classes for all algorithms

#### 2. QPP-Based Routing (Reranker-Guided Search)
**Status**: Not implemented
**Impact**: Medium - reduces manual tuning
**What**: Automatically detect hard queries and route to specialized fusion strategies
**Implementation**:
- Query performance prediction (QPP)
- Strategy routing based on query difficulty
- Adaptive fusion parameters

#### 3. Batch Fusion Operations
**Status**: Not implemented
**Impact**: Medium - performance optimization
**What**: Process multiple fusion operations efficiently
**Implementation**:
- Batch fusion API
- Parallelization hints
- Streaming/chunked processing

#### 4. Enhanced Explainability
**Status**: Partial (Rust only)
**Impact**: Medium - debugging and trust
**What**: Complete explainability in Python/WASM
**Implementation**:
- Python bindings for `explain` module
- WASM bindings for explainability
- Token-level alignment information

### Priority Order
1. Complete Python bindings - highest adoption impact
2. QPP-based routing - research-backed improvement
3. Batch operations - performance optimization
4. Enhanced explainability - UX improvement

---

## rank-rerank

### Current State
- ✅ MaxSim (ColBERT/ColPali) with SIMD acceleration
- ✅ Cosine similarity
- ✅ Diversity selection (MMR, DPP)
- ✅ Token pooling and alignment
- ✅ Matryoshka refinement
- ✅ Cross-encoder trait (implementation disabled)
- ✅ Python bindings
- ✅ Comprehensive testing

### Research Gaps (2024-2025)

#### 1. Fine-Grained Scoring (ERANK, arXiv:2509.00520)
**Status**: Not implemented
**Impact**: High - 3-7% nDCG@10 improvement
**What**: Integer scoring (0-10) instead of binary classification
**Formula**: `s_i × Pr(token = s_i)` fully utilizes LLM generative power
**Implementation**:
- Fine-grained scoring trait
- Integer score mapping (0-10)
- Probability-weighted scoring
- Threshold-based filtering

#### 2. Contextual Relevance (TS-SetRank, arXiv:2511.01208)
**Status**: Not implemented
**Impact**: High - 15-25% nDCG@10 improvement on BRIGHT
**What**: Models relevance as context-dependent (varies by batch composition)
**Implementation**:
- Beta-Bernoulli posteriors
- Thompson sampling for exploration
- Context-aware scoring
- Batch-dependent relevance modeling

#### 3. Reasoning Explanations (ERANK, Reranker-Guided Search)
**Status**: Partial (token alignment exists, reasoning traces missing)
**Impact**: Medium - UX enhancement
**What**: Chain-of-thought reasoning traces for interpretability
**Implementation**:
- Reasoning step generation
- Confidence estimation
- Explanation struct extension
- Integration with existing `explain` module

#### 4. Cross-Encoder Implementation
**Status**: Trait exists, implementation disabled
**Impact**: Medium - highest quality reranking
**What**: Full cross-encoder implementation (ONNX or native)
**Implementation**:
- Enable ONNX cross-encoder (feature exists)
- Native Rust implementation option
- Tokenization support
- Batch processing

#### 5. PE-Rank: Passage Embedding Reranking (2025)
**Status**: Not implemented
**Impact**: Medium - efficient listwise reranking
**What**: Leverage single passage embedding as context compression for efficient listwise reranking
**Implementation**:
- Passage embedding compression
- Listwise reranking with embeddings
- Context-aware scoring

### Priority Order
1. Fine-grained scoring (ERANK) - highest validated impact (3-7%)
2. Contextual relevance (TS-SetRank) - highest absolute impact (15-25%)
3. Cross-encoder implementation - quality improvement
4. Reasoning explanations - UX enhancement
5. PE-Rank - efficiency improvement

---

## rank-soft

### Current State
- ✅ Sigmoid-based soft ranking (default)
- ✅ NeuralSort-style, Probabilistic (SoftRank), SmoothI
- ✅ Gumbel-Softmax Top-k
- ✅ Analytical gradients
- ✅ Batch processing
- ✅ Python bindings with PyTorch/JAX examples
- ✅ Candle/Burn integration (partial)

### Research Gaps (2024-2025)

#### 1. Admissible Rank-Based Operators (arXiv:2512.22587)
**Status**: Not implemented
**Impact**: High - addresses fundamental limitations
**What**: Rank-level invariance, pointwise definition, structural consistency
**Key Issue**: Current methods rely on value gaps and batch-level statistics, violating admissibility criteria
**Implementation**:
- QNorm-style operators (strictly increasing functions)
- Rank-level invariance under feature-wise transformations
- Pointwise definition independent of batch composition
- Structural consistency conditions

#### 2. O(n log n) Methods
**Status**: Documented but not implemented
**Impact**: High - scalability
**What**: Permutahedron projection, Optimal Transport, LapSum
**Current**: All methods are O(n²)
**Implementation**:
- Permutahedron projection via isotonic regression
- Optimal transport-based ranking
- LapSum unified method (Struski et al., 2025)
- DFTopK linear-time top-k selection

#### 3. Full SoftSort with Sinkhorn
**Status**: Simplified version exists
**Impact**: Medium - better gradient quality
**What**: Complete optimal transport-based soft sorting
**Implementation**:
- Sinkhorn iterations for optimal transport
- Full SoftSort algorithm
- Better gradient profiles than simplified version

#### 4. LambdaRank-Style Metric-Aware Gradients
**Status**: Not implemented
**Impact**: Medium - direct metric optimization
**What**: Gradients that directly optimize NDCG/other metrics
**Implementation**:
- NDCG-aware gradient computation
- Metric-specific gradient formulas
- Integration with rank-learn

#### 5. Complete Framework Integration
**Status**: Partial (PyTorch/JAX examples exist, not fully tested)
**Impact**: Medium - adoption
**What**: Complete, tested integration with PyTorch, JAX, Julia, Candle, Burn
**Implementation**:
- Complete PyTorch autograd testing
- Complete JAX primitive testing
- Julia C FFI bindings
- Complete Candle/Burn tensor integration

### Priority Order
1. Admissible rank-based operators - addresses fundamental limitations
2. O(n log n) methods - scalability requirement
3. Full SoftSort - gradient quality improvement
4. LambdaRank-style gradients - metric optimization
5. Complete framework integration - adoption

---

## rank-learn

### Current State
- ✅ LambdaRank with NDCG-aware gradients
- ✅ Neural LTR interface (placeholder)
- ✅ Python bindings
- ✅ Property-based testing

### Research Gaps (2024-2025)

#### 1. Query Routing for Retrievers (LTRR, arXiv:2506.13743)
**Status**: Not implemented
**Impact**: High - 10-20% improvement
**What**: Learning to Rank Retrievers - pairwise XGBoost for retriever selection
**Implementation**:
- Query feature extraction
- Pairwise XGBoost model for retriever ranking
- Integration with rank-retrieve
- Utility-aware training (BEM, AC metrics)

#### 2. Full NeuralLTRModel Implementation
**Status**: Placeholder exists
**Impact**: High - core functionality
**What**: Complete neural ranking model using rank-soft
**Implementation**:
- Neural network architecture
- Training loop integration
- Loss function integration (ListNet, ListMLE from rank-soft)
- Batch processing

#### 3. XGBoost/LightGBM Integration
**Status**: Planned but not implemented
**Impact**: High - industry standard
**What**: External bindings for gradient boosting with ranking objectives
**Implementation**:
- XGBoost Rust bindings
- LightGBM Rust bindings
- Ranking objective support
- Integration with LambdaRank

#### 4. Utility-Aware Metrics (BEM, AC)
**Status**: Not implemented
**Impact**: Medium - RAG optimization
**What**: Support training on downstream utility metrics (BEM, AC) for RAG systems
**Implementation**:
- BEM (Binary Evaluation Metric) support
- AC (Answer Correctness) support
- Integration with training loop
- RAG-specific optimization

#### 5. Temporal Information Retrieval (Re3, arXiv:2509.01306)
**Status**: Not implemented
**Impact**: Medium - specialized use case
**What**: Learning to balance relevance and recency for temporal queries
**Implementation**:
- Temporal feature extraction
- Recency-aware ranking
- Temporal constraint handling

### Priority Order
1. Query routing for retrievers (LTRR) - highest research impact
2. Full NeuralLTRModel - core functionality
3. XGBoost/LightGBM integration - industry standard
4. Utility-aware metrics - RAG optimization
5. Temporal IR - specialized use case

---

## rank-eval

### Current State
- ✅ 13 binary metrics (NDCG, MAP, MRR, ERR, RBP, etc.)
- ✅ 2 graded metrics (NDCG, MAP)
- ✅ TREC format parsing
- ✅ Batch evaluation
- ✅ Statistical testing
- ✅ Export utilities (CSV, JSON)
- ✅ Enhanced dataset support (MTEB, HotpotQA, Natural Questions)
- ✅ Python bindings

### Research Gaps (2024-2025)

#### 1. Temporal Evaluation Metrics
**Status**: Not implemented
**Impact**: Medium - temporal IR evaluation
**What**: Metrics for evaluating temporal information retrieval (Re3 paper)
**Implementation**:
- Recency-aware NDCG
- Temporal relevance metrics
- Time-decay functions

#### 2. Generative Search Metrics
**Status**: Not implemented
**Impact**: Medium - generative IR evaluation
**What**: Metrics for evaluating generative search and recommendation systems
**Implementation**:
- Generation quality metrics
- Factual accuracy metrics
- Citation quality metrics

#### 3. Interactive Evaluation Metrics
**Status**: Not implemented
**Impact**: Low - specialized use case
**What**: Metrics for interactive IR systems (TREC iKAT 2025)
**Implementation**:
- User interaction modeling
- Session-based metrics
- Conversational search metrics

#### 4. Performance Optimizations
**Status**: Partial
**Impact**: Medium - scalability
**What**: SIMD optimizations, parallel evaluation
**Implementation**:
- SIMD vectorization for metric computation
- Parallel query evaluation
- Memory-efficient batch processing

#### 5. Advanced Statistical Analysis
**Status**: Basic t-tests exist
**Impact**: Low - nice to have
**What**: Comprehensive statistical testing suite
**Implementation**:
- ANOVA for multiple methods
- Effect size computation
- Confidence intervals
- Bootstrap methods

### Priority Order
1. Temporal evaluation metrics - research-backed need
2. Performance optimizations - scalability
3. Generative search metrics - emerging use case
4. Advanced statistical analysis - research rigor
5. Interactive evaluation metrics - specialized

---

## Cross-Crate Integration Gaps

### 1. End-to-End Pipeline Examples
**Status**: Partial
**Impact**: High - adoption
**What**: Complete examples showing retrieve → fuse → rerank → learn → eval
**Implementation**:
- Full pipeline examples
- Integration testing
- Performance benchmarks

### 2. Shared Utilities
**Status**: Some duplication exists
**Impact**: Medium - maintenance
**What**: Shared utilities for common operations
**Implementation**:
- Common data structures
- Shared validation
- Common error types

### 3. Unified Configuration
**Status**: Not implemented
**Impact**: Low - convenience
**What**: Unified configuration system across crates
**Implementation**:
- Shared config format
- Environment variable support
- Configuration validation

---

## Summary by Priority

### Critical (Implement First)
1. **rank-rerank**: Fine-grained scoring (ERANK) - 3-7% improvement
2. **rank-rerank**: Contextual relevance (TS-SetRank) - 15-25% improvement
3. **rank-retrieve**: Query routing framework (LTRR) - 10-20% improvement
4. **rank-fusion**: Complete Python bindings - adoption blocker
5. **rank-learn**: Query routing for retrievers (LTRR) - 10-20% improvement

### High Priority (Implement Soon)
6. **rank-soft**: Admissible rank-based operators - fundamental fix
7. **rank-soft**: O(n log n) methods - scalability
8. **rank-retrieve**: Production ANN integration - scale requirement
9. **rank-learn**: Full NeuralLTRModel - core functionality
10. **rank-rerank**: Cross-encoder implementation - quality improvement

### Medium Priority (Consider)
11. **rank-fusion**: QPP-based routing - automation
12. **rank-soft**: Full SoftSort - gradient quality
13. **rank-learn**: XGBoost/LightGBM integration - industry standard
14. **rank-eval**: Temporal evaluation metrics - research need
15. **rank-rerank**: Reasoning explanations - UX enhancement

### Lower Priority (Future)
16. Batch operations across all crates
17. Enhanced explainability
18. Advanced statistical analysis
19. Interactive evaluation metrics
20. Unified configuration system

---

## Research Papers Referenced

1. **ERANK** (arXiv:2509.00520) - Fine-grained scoring, standardization-based fusion
2. **TS-SetRank** (arXiv:2511.01208) - Contextual relevance modeling
3. **LTRR** (arXiv:2506.13743) - Learning to Rank Retrievers
4. **ResFlow** (arXiv:2411.09705) - Additive multi-task fusion
5. **Reranker-Guided Search** (arXiv:2509.07163) - QPP-based routing
6. **PE-Rank** (2025) - Passage embedding reranking
7. **Re3** (arXiv:2509.01306) - Temporal information retrieval
8. **Admissible Rank-Based Operators** (arXiv:2512.22587) - Fundamental limitations
9. **Evolution of Reranking Models** (arXiv:2512.16236) - Comprehensive survey

