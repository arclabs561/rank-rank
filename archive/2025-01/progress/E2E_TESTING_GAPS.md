# E2E Testing Gaps Analysis

Comprehensive analysis of missing end-to-end tests that should be implemented for the rank-* crates.

## Current E2E Test Coverage

### ✅ Existing E2E Tests (in `crates/rank-fusion/test-e2e-local/`)

1. **`test-fusion-basic`** - Tests all rank-fusion algorithms
2. **`test-fusion-eval-integration`** - Tests rank-fusion + rank-eval integration
3. **`test-refine-basic`** - Tests basic rank-rerank functionality (MaxSim, ColBERT, token pooling)
4. **`test-eval-basic`** - Tests rank-eval functionality (binary/graded metrics, TREC parsing)
5. **`test-full-pipeline`** - Tests complete RAG pipeline (fusion → refine → eval)

**Status**: ✅ All passing, but limited scope

## Missing E2E Tests

### 1. New Feature Tests (High Priority)

#### 1.1 Contextual Relevance (TS-SetRank)
**Status**: ❌ Not tested
**Location**: `crates/rank-rerank/src/contextual.rs`
**What to test**:
- Beta-Bernoulli posterior updates
- Uniform exploration phase
- Thompson sampling adaptive phase
- Relative/Percentile/Adaptive modes
- Integration with MaxSim scoring
- Performance with large candidate sets

**Test file**: `test-e2e-local/src/bin/test_contextual_relevance.rs`

#### 1.2 Fine-Grained Scoring (ERANK)
**Status**: ⚠️ Partially tested (unit tests exist, no E2E)
**Location**: `crates/rank-rerank/src/explain.rs::rerank_fine_grained`
**What to test**:
- Integer scoring (0-10) mapping
- Probability weighting
- Integration with different reranking methods
- Threshold filtering
- Top-k truncation

**Test file**: `test-e2e-local/src/bin/test_fine_grained_scoring.rs`

#### 1.3 Query Routing (LTRR)
**Status**: ❌ Not implemented, no tests
**Location**: Should be in `crates/rank-retrieve` or `crates/rank-learn`
**What to test**:
- Query feature extraction
- Retriever selection
- Multi-retriever routing
- Utility-aware training
- Integration with rank-fusion

**Test file**: `test-e2e-local/src/bin/test_query_routing.rs` (after implementation)

### 2. Cross-Crate Integration Tests (High Priority)

#### 2.1 Complete Pipeline: Retrieve → Fusion → Rerank → Eval → Learn
**Status**: ⚠️ Partial (fusion → rerank → eval exists, missing retrieve and learn)
**Current**: `test-full-pipeline.rs` only tests fusion → rerank → eval
**Missing**:
- rank-retrieve integration (BM25, dense, sparse)
- rank-learn integration (LambdaRank training)
- Full pipeline with all 6 crates

**Test file**: `test-e2e-local/src/bin/test_complete_pipeline.rs`

#### 2.2 rank-retrieve E2E Tests
**Status**: ❌ Missing
**What to test**:
- BM25 retrieval with real queries
- Dense retrieval with embeddings
- Sparse retrieval
- Integration with rank-fusion
- Large-scale retrieval (1000+ candidates)

**Test file**: `test-e2e-local/src/bin/test_retrieve_basic.rs`

#### 2.3 rank-soft E2E Tests
**Status**: ❌ Missing
**What to test**:
- Soft ranking with different methods
- Differentiable sorting
- Spearman loss computation
- Integration with PyTorch/JAX (if bindings exist)
- Gradient flow validation

**Test file**: `test-e2e-local/src/bin/test_soft_ranking.rs`

#### 2.4 rank-learn E2E Tests
**Status**: ❌ Missing
**What to test**:
- LambdaRank training loop
- NDCG-aware gradient computation
- Integration with rank-soft
- Model training and inference
- Evaluation with rank-eval

**Test file**: `test-e2e-local/src/bin/test_learn_basic.rs`

### 3. Python Bindings E2E Tests (High Priority)

#### 3.1 rank-fusion Python E2E
**Status**: ⚠️ Partial (unit tests exist, no E2E)
**What to test**:
- All algorithms accessible from Python
- Configuration classes work correctly
- Explainability module accessible
- Error handling (exceptions)
- Performance (should match Rust)

**Test file**: `test-e2e-local/src/bin/test_python_fusion.rs` (calls Python)

#### 3.2 rank-rerank Python E2E
**Status**: ⚠️ Partial (unit tests exist, no E2E)
**What to test**:
- MaxSim from Python
- Fine-grained scoring from Python
- Contextual relevance from Python (when feature enabled)
- Batch operations
- Error handling

**Test file**: `test-e2e-local/src/bin/test_python_rerank.rs`

#### 3.3 rank-retrieve Python E2E
**Status**: ⚠️ Partial (unit tests exist, no E2E)
**What to test**:
- BM25 from Python
- Dense retrieval from Python
- Sparse retrieval from Python
- Integration with Python RAG stacks

**Test file**: `test-e2e-local/src/bin/test_python_retrieve.rs`

#### 3.4 rank-soft Python E2E
**Status**: ⚠️ Partial (PyTorch/JAX examples exist, not E2E tested)
**What to test**:
- Soft ranking from Python
- PyTorch autograd integration
- JAX primitive integration
- Training loop integration

**Test file**: `test-e2e-local/src/bin/test_python_soft.rs`

#### 3.5 rank-learn Python E2E
**Status**: ⚠️ Partial (unit tests exist, no E2E)
**What to test**:
- LambdaRank from Python
- Training loop from Python
- Integration with rank-eval from Python

**Test file**: `test-e2e-local/src/bin/test_python_learn.rs`

#### 3.6 rank-eval Python E2E
**Status**: ⚠️ Partial (unit tests exist, no E2E)
**What to test**:
- All metrics from Python
- TREC parsing from Python
- Integration with other crates from Python

**Test file**: `test-e2e-local/src/bin/test_python_eval.rs`

### 4. WASM Bindings E2E Tests (Medium Priority)

#### 4.1 rank-fusion WASM
**Status**: ❌ Missing
**What to test**:
- RRF from JavaScript/Node.js
- All algorithms accessible
- Error handling
- Performance

**Test file**: `test-e2e-local/src/bin/test_wasm_fusion.rs` (calls Node.js)

#### 4.2 rank-rerank WASM
**Status**: ❌ Missing
**What to test**:
- MaxSim from JavaScript
- Cosine similarity from JavaScript
- Batch operations

**Test file**: `test-e2e-local/src/bin/test_wasm_rerank.rs`

### 5. Real Dataset Tests (Medium Priority)

#### 5.1 TREC Dataset Integration
**Status**: ❌ Missing
**What to test**:
- Load TREC qrels
- Load TREC runs
- Evaluate with real data
- Compare results with published baselines

**Test file**: `test-e2e-local/src/bin/test_trec_datasets.rs`

#### 5.2 BEIR Dataset Integration
**Status**: ❌ Missing
**What to test**:
- Load BEIR datasets
- Run retrieval → fusion → rerank → eval
- Compare with published results

**Test file**: `test-e2e-local/src/bin/test_beir_datasets.rs`

### 6. Error Handling E2E Tests (Medium Priority)

#### 6.1 Error Propagation
**Status**: ❌ Missing
**What to test**:
- Errors propagate correctly across crates
- Error messages are helpful
- Graceful degradation
- Invalid input handling

**Test file**: `test-e2e-local/src/bin/test_error_handling.rs`

#### 6.2 Edge Cases
**Status**: ❌ Missing
**What to test**:
- Empty inputs
- Single document
- All documents identical
- Very large inputs
- Very small inputs
- NaN/Inf handling

**Test file**: `test-e2e-local/src/bin/test_edge_cases.rs`

### 7. Performance E2E Tests (Low Priority)

#### 7.1 Performance Benchmarks
**Status**: ⚠️ Partial (unit benchmarks exist, no E2E)
**What to test**:
- Full pipeline performance
- Memory usage
- Scalability (10 → 100 → 1000 → 10000 documents)
- Comparison with baselines

**Test file**: `test-e2e-local/src/bin/test_performance.rs`

#### 7.2 Published Version Performance
**Status**: ❌ Missing
**What to test**:
- Performance with published crates.io versions
- No regression from path dependencies
- Binary size validation

**Test file**: `test-e2e-local/src/bin/test_published_performance.rs`

### 8. CI/CD Integration Tests (Low Priority)

#### 8.1 Published Crate Tests
**Status**: ⚠️ Partial (workflows exist, not fully tested)
**What to test**:
- Install from crates.io
- Install from PyPI
- Install from npm
- Verify all features work

**Test file**: `.github/workflows/e2e-published.yml` (enhance existing)

## Priority Order

### Critical (Implement First)
1. ✅ Contextual relevance E2E test
2. ✅ Fine-grained scoring E2E test
3. ✅ Complete pipeline test (retrieve → fusion → rerank → eval → learn)
4. ✅ rank-retrieve E2E test
5. ✅ rank-soft E2E test
6. ✅ rank-learn E2E test

### High Priority
7. Python bindings E2E tests (all crates)
8. Error handling E2E tests
9. Edge cases E2E tests

### Medium Priority
10. WASM bindings E2E tests
11. Real dataset tests (TREC, BEIR)
12. Performance E2E tests

### Low Priority
13. Published version tests
14. CI/CD integration enhancements

## Implementation Plan

### Phase 1: New Features (Immediate)
- [ ] `test_contextual_relevance.rs` - Test TS-SetRank implementation
- [ ] `test_fine_grained_scoring.rs` - Test ERANK-style scoring

### Phase 2: Missing Crate Tests (Week 1)
- [ ] `test_retrieve_basic.rs` - rank-retrieve E2E
- [ ] `test_soft_ranking.rs` - rank-soft E2E
- [ ] `test_learn_basic.rs` - rank-learn E2E
- [ ] `test_complete_pipeline.rs` - All 6 crates together

### Phase 3: Python Bindings (Week 2)
- [ ] Python E2E tests for all crates
- [ ] Integration with pytest
- [ ] CI integration

### Phase 4: Advanced Tests (Week 3+)
- [ ] WASM tests
- [ ] Real dataset tests
- [ ] Performance tests
- [ ] Error handling tests

## Notes

- All E2E tests should use path dependencies (simulating published versions)
- Tests should be runnable independently and in CI
- Tests should validate both correctness and performance
- Python/WASM tests may require external tooling (pytest, Node.js)

