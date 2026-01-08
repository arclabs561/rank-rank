# Visualization Generation Workflow

Complete guide for generating statistical visualizations for rank-learn.

## Overview

Visualizations use **REAL DATA** from actual code execution with statistical depth matching pre-AI quality standards (games/tenzi).

## Quick Start

```bash
cd crates/rank-learn/hack/viz
uv run generate_ltr_real_data.py
```

This generates:
- `ltr_statistical_analysis.png` - 4-panel comprehensive analysis
- `ltr_ndcg_analysis.png` - NDCG-specific analysis

## Workflow Steps

### 1. Generate Real Data

The script automatically generates real data by:
- Creating actual `LambdaRankTrainer` instances
- Running real NDCG computations (1000+ queries)
- Computing real LambdaRank gradients

**Data Sources**:
- NDCG: Real NDCG computations from realistic relevance scores
- LambdaRank: Real gradient computations from model scores and relevance

### 2. Statistical Analysis

The script performs:
- **Distribution fitting**: Beta for NDCG (bounded [0,1]), Gamma for gradients
- **Confidence intervals**: 95% CI on all statistics
- **Box plots**: Median, quartiles, outliers
- **Correlation analysis**: NDCG vs gradient relationships

### 3. Generate Visualizations

Output files:
- `ltr_statistical_analysis.png` (4-panel)
- `ltr_ndcg_analysis.png` (2-panel)

## Quality Standards

All visualizations follow pre-AI quality standards:

| Aspect | Standard | Our Implementation |
|--------|----------|-------------------|
| Real data | ✅ Actual code execution | ✅ 1000+ real queries |
| Statistical depth | ✅ Distribution fitting | ✅ Beta/Gamma fitting |
| Sample size | ✅ 1000+ samples | ✅ 1000 samples |
| Reproducibility | ✅ Fixed random seed | ✅ seed=42 |
| Code-driven | ✅ Python scripts | ✅ PEP 723 inline deps |

## Troubleshooting

### Python Bindings Not Available

If `rank_learn` module is not available, the script falls back to mock data:
- NDCG: Beta distribution (bounded [0, 1])
- LambdaRank: Gamma distribution (gradient magnitudes)

**To use real data**: Build Python bindings first:
```bash
cd crates/rank-learn/rank-learn-python
maturin develop
```

### Missing Dependencies

The script uses PEP 723 inline dependencies. If `uv` is not available:
```bash
pip install matplotlib numpy scipy
python generate_ltr_real_data.py
```

## Integration with Documentation

Visualizations are referenced in:
- `README.md` - Main documentation
- `IMPLEMENTATION_STATUS.md` - Status tracking

## Comparison with Other rank-* Visualizations

| Crate | Real Data | Distribution | Sample Size | Status |
|-------|-----------|--------------|-------------|--------|
| rank-learn | ✅ 1000 queries | Beta/Gamma | 1000 | ✅ |
| rank-retrieve | ✅ 1000 queries | Gamma/Beta | 1000 | ✅ |
| rank-fusion | ✅ 25 scenarios | Gamma | 1000+ | ✅ |
| rank-eval | ✅ 1000 queries | Beta | 1000 | ✅ |

## References

- **rank-retrieve/hack/viz/VISUALIZATION_WORKFLOW.md**: Similar workflow
- **rank-eval/hack/viz/generate_ndcg_real_data.py**: NDCG visualization example
- **Pre-AI Quality Standards**: games/tenzi statistical rigor

