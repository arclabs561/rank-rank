# Shared Visualization Utilities

Shared visualization generation patterns and utilities for all rank-* repositories.

## Purpose

This directory contains:
- Template scripts for visualization generation
- Shared statistical analysis functions
- Common visualization patterns
- Documentation on quality standards

## Quality Standards

All visualizations should match pre-AI quality (games/tenzi):
- ✅ Real data from actual code execution
- ✅ Statistical depth (distribution fitting, hypothesis testing)
- ✅ Large sample sizes (1000+)
- ✅ Code-driven and reproducible
- ✅ Professional appearance

## Template Pattern

Each rank-* repo should have its own `hack/viz/` directory with:
- `generate_*_real_data.py` - Real data visualization script
- `*_VISUALIZATIONS.md` - Documentation with embedded images
- Generated PNG files

## Shared Utilities

### Statistical Functions

Common statistical analysis patterns:
- Gamma distribution fitting (for error distributions)
- Beta distribution fitting (for bounded [0,1] metrics)
- Hypothesis testing (t-tests, ANOVA)
- Effect size calculations (Cohen's d)

### Visualization Patterns

- 4-panel comprehensive analysis
- Box plots with statistical comparison
- Confidence intervals on all comparisons
- Distribution fitting overlays

## Integration

### From rank-* Repository

```bash
# Reference shared patterns
cp ../../rank-rank/hack/viz/template.py hack/viz/generate_my_viz.py
# Customize for your repo
```

### Standalone

Use as reference for creating new visualizations.

## See Also

- [rank-fusion/hack/viz/](../rank-fusion/hack/viz/) - RRF visualizations
- [rank-refine/hack/viz/](../rank-refine/hack/viz/) - MaxSim visualizations
- [rank-relax/hack/viz/](../rank-relax/hack/viz/) - Soft ranking visualizations
- [rank-eval/hack/viz/](../rank-eval/hack/viz/) - NDCG visualizations

