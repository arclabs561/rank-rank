# LTRR (Learning to Rank Retrievers) Implementation Research

This document synthesizes research on LTRR (arXiv:2506.13743) for implementing query routing in `rank-retrieve`.

## Overview

LTRR (Learning to Rank Retrievers) is a framework that uses machine learning to dynamically select the best retriever(s) for each query, showing 10-20% improvement in retrieval quality over fixed routing.

**Key insight**: Different queries benefit from different retrieval methods. LTRR learns to route queries to the optimal retriever(s) based on query characteristics and expected utility.

## Current Status

The `routing` module in `rank-retrieve` provides:
- ✅ Basic query feature extraction (length, complexity, query type)
- ✅ Simple heuristic-based routing
- ❌ Trained XGBoost model (NOT IMPLEMENTED)
- ❌ Utility-aware training (NOT IMPLEMENTED)

## Implementation Requirements

### 1. Model Selection: XGBoost vs LightGBM

**Research Finding:**
- **LightGBM** is typically faster and often achieves equal or better NDCG@K for large-scale ranking
- **XGBoost** is more mature with better documentation and tooling
- Both support pairwise learning-to-rank objectives

**LightGBM Advantages:**
- **Faster training**: Up to 10x faster than XGBoost on large datasets
- **Better ranking objectives**: Native `lambdarank` and `xendcg` objectives
- **Faster convergence**: Reaches higher NDCG@K faster
- **Lower memory usage**: More efficient for large-scale ranking

**XGBoost Advantages:**
- **More mature**: Better documentation, tooling, examples
- **More conservative**: Less prone to overfitting
- **Better for small datasets**: Competitive or faster on small/medium data
- **Ecosystem**: More integrations and community support

**Recommendation:**
- **Primary**: LightGBM for large-scale ranking (better performance)
- **Fallback**: XGBoost for smaller datasets or when maturity is critical

**Model Type:**
- Use **gradient-boosted decision trees** (XGBoost or LightGBM) as the ranking model
- **LightGBM**: Configure with `lambdarank` or `xendcg` objective (native ranking objectives)
- **XGBoost**: Configure in **rank mode** with `rank:pairwise` objective
- Research shows **pairwise learning-to-rank** gives best routing performance

**Training Setup (XGBoost):**
```python
import xgboost as xgb

# Configure for pairwise ranking
params = {
    'objective': 'rank:pairwise',
    'eval_metric': 'ndcg@5',
    'tree_method': 'hist',
    'max_depth': 6,
    'learning_rate': 0.1,
}

# Training data must be grouped by query
dtrain = xgb.DMatrix(features, labels, group=query_groups)
model = xgb.train(params, dtrain, num_boost_round=100)
```

**Training Setup (LightGBM - Recommended for Large Scale):**
```python
import lightgbm as lgb

# Configure for ranking (lambdarank or xendcg)
params = {
    'objective': 'lambdarank',  # or 'xendcg'
    'metric': 'ndcg@5',
    'boosting_type': 'gbdt',
    'num_leaves': 31,
    'learning_rate': 0.1,
    'feature_fraction': 0.9,
    'bagging_fraction': 0.8,
    'bagging_freq': 5,
}

# Training data must be grouped by query
train_data = lgb.Dataset(features, label=labels, group=query_groups)
model = lgb.train(params, train_data, num_boost_round=100)
```

### 2. Feature Engineering

**Query Features** (already implemented):
- Query length (number of terms)
- Query complexity (term diversity)
- Query type (keyword, semantic, hybrid)
- Domain/context indicators (optional)

**Post-Retrieval Features** (NOT IMPLEMENTED):
- OverallSim: Average similarity across retrieved documents
- AvgSim: Mean similarity score
- MaxSim: Maximum similarity score
- VarSim: Variance of similarity scores
- Retrieval scores/entropies for each retriever

**Retriever-Specific Features** (NOT IMPLEMENTED):
- Retriever type (BM25, Dense, Sparse, Generative)
- Retriever configuration flags (e.g., BM25+reg, E5+stoch)
- Historical performance metrics per retriever

### 3. Training Data Format

**Input Representation:**
- Each training instance is a **(query, retriever)** pair
- Feature vector: `Φ(q, R_i)` capturing query and retriever characteristics
- For each query `q`, create one row per retriever in the pool

**Labels/Targets:**
- Supervised target: **utility gain** `δ_i` of retriever `R_i` over no-retrieval baseline
- Measured with downstream LLM metric: **Answer Correctness (AC)** or **Binary Exact Match (BEM)**
- Utility gain = `AC_with_retriever - AC_without_retriever`

**Grouping:**
- Training data must be **grouped by query** (XGBoost's `group`/`qid` mechanism)
- Ranker only compares retrievers within the same query

### 4. Inference

**Routing Process:**
1. Extract query features: `QueryFeatures::from_terms(&query_terms)`
2. For each retriever `R_i` in pool:
   - Compute feature vector `Φ(q, R_i)`
   - Run through trained XGBoost model
3. Sort retrievers by predicted score (descending)
4. Select top-k retrievers (or single best)

**Example:**
```rust
let features = QueryFeatures::from_terms(&query_terms);
let retriever_scores: Vec<(RetrieverId, f32)> = retrievers
    .iter()
    .map(|retriever| {
        let feature_vector = extract_features(&features, retriever);
        let score = xgboost_model.predict(&feature_vector)?;
        (retriever.id, score)
    })
    .collect();

retriever_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
let top_retriever = retriever_scores[0].0;
```

## Integration with rank-retrieve

### Current Architecture

The `routing` module provides:
- `QueryFeatures`: Query feature extraction
- `QueryRouter`: Routing logic (currently heuristic-based)
- `RetrieverId`: Retriever identifiers
- `UtilityMetrics`: Utility metrics (BEM, AC)

### Required Extensions

1. **Model Integration (XGBoost or LightGBM)**:
   - **Option A (Recommended)**: Separate crates (`rank-retrieve-xgboost`, `rank-retrieve-lightgbm`)
     - Avoids dependency bloat in core crate
     - Independent versioning
     - Users choose based on needs
   - **Option B**: Optional features in core crate
     - `xgboost` feature: `["dep:xgboost-rs"]`
     - `lightgbm` feature: `["dep:lightgbm-rs"]`
     - Simpler for users, but adds dependencies
   - Load trained model at runtime
   - Implement feature extraction for post-retrieval features

2. **Post-Retrieval Feature Extraction**:
   ```rust
   pub struct PostRetrievalFeatures {
       pub overall_sim: f32,
       pub avg_sim: f32,
       pub max_sim: f32,
       pub var_sim: f32,
       pub retrieval_scores: Vec<f32>,
   }
   
   impl PostRetrievalFeatures {
       pub fn from_results(results: &[(u32, f32)]) -> Self {
           // Extract features from retrieval results
       }
   }
   ```

3. **Training Pipeline**:
   - Collect query-retriever pairs with utility labels
   - Extract features (query + post-retrieval)
   - Train XGBoost model with pairwise objective
   - Serialize model for runtime loading

4. **Runtime Integration**:
   ```rust
   impl QueryRouter {
       pub fn with_trained_model(model_path: &str) -> Result<Self, Error> {
           let model = XGBoostModel::load(model_path)?;
           Ok(Self { model: Some(model), .. })
       }
       
       pub fn route(&self, features: &QueryFeatures, 
                    post_features: &[PostRetrievalFeatures]) -> Vec<RetrieverId> {
           if let Some(model) = &self.model {
               // Use trained model
               self.route_with_model(features, post_features, model)
           } else {
               // Fallback to heuristics
               self.route_heuristic(features)
           }
       }
   }
   ```

## Research Findings

### Performance Gains

- **10-20% improvement** in retrieval quality over fixed routing
- **Pairwise XGBoost** with AC labels gives best routing performance
- Generalizes well across different query types and domains

### Training Requirements

- Requires labeled data: queries with utility gains per retriever
- Utility measured with downstream LLM metrics (AC, BEM)
- Training data grouped by query for pairwise ranking

### Practical Considerations

- **No retriever fine-tuning**: XGBoost only routes over fixed retrievers
- **Low-dimensional features**: Default XGBoost hyperparameters often sufficient
- **Query grouping**: Critical for pairwise ranking objective

## Implementation Priority

**Phase 1: Feature Extraction** (Current)
- ✅ Query features (implemented)
- ⏳ Post-retrieval features (needs implementation)

**Phase 2: Model Integration** (Next)
- **Recommended**: Create separate crates (`rank-retrieve-xgboost`, `rank-retrieve-lightgbm`)
- **Alternative**: Add optional features to core crate
- Implement model loading (XGBoost and/or LightGBM)
- Implement feature vector construction
- Implement routing with trained model
- Support both XGBoost and LightGBM (users choose)

**Phase 3: Training Pipeline** (Future)
- Collect training data
- Implement training script
- Validate model performance

## References

- **LTRR Paper**: arXiv:2506.13743 - "Learning to Rank Retrievers"
- **XGBoost Ranking**: XGBoost documentation on ranking objectives
- **Pairwise LTR**: Research on pairwise learning-to-rank for retrieval

## See Also

- [Routing Module](../src/routing.rs) - Current implementation
- [RETRIEVAL_METHODS_RESEARCH.md](RETRIEVAL_METHODS_RESEARCH.md) - Additional retrieval methods
- [USE_CASES.md](USE_CASES.md) - When to use query routing
