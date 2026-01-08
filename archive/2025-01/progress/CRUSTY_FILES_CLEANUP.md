# Crusty/Noisy Files Cleanup

**Date**: 2025-01-XX  
**Purpose**: Remove auto-generated, binary artifacts, and duplicate/unreferenced documentation

## Additional Files Archived: 17

### Auto-Generated/Binary Files (6)
- crates/rank-eval/INTROSPECTION_REPORT.json - Auto-generated introspection report
- crates/rank-fusion/eval_report.html - Generated evaluation report
- crates/rank-fusion/eval_results.json - Generated evaluation results
- crates/rank-soft/rank-soft-python/pytestdebug.log - Debug log file
- crates/rank-fusion/liblib.rlib - Binary artifact (Rust library)
- crates/rank-soft/test_sigmoid_stability - Binary artifact (executable)

### Research Documents (2)
- crates/rank-retrieve/ANN_BENCHMARK_RESEARCH.md - Internal research document
- crates/rank-retrieve/RESEARCH_FINDINGS.md - Internal research findings

### Duplicate/Unreferenced Guides (6)
- crates/rank-retrieve/EXAMPLES.md - Duplicates examples/ directory, not referenced from README
- crates/rank-retrieve/BENCHMARKING.md - Not referenced, duplicates benchmarks/ directory
- crates/rank-learn/EXAMPLES.md - Not referenced from README
- crates/rank-learn/BENCHMARKING.md - Not referenced from README
- crates/rank-soft/USAGE_EXAMPLES.md - Not referenced from README
- crates/rank-eval/COMPREHENSIVE_FEATURES.md - Not referenced from README

### Generated Reports (3)
- crates/rank-rerank/rank-rerank-python/eval_results.json - Generated evaluation results
- crates/rank-rerank/rank-rerank-python/examples/benchmark_results.json - Generated benchmark results
- crates/rank-rerank/rank-rerank-python/examples/benchmark_report.html - Generated benchmark report

## Files Kept (Referenced)

### User Guides (Referenced from README/docs)
- crates/rank-retrieve/INTEGRATION_GUIDE.md - Referenced from docs/GETTING_STARTED.md
- crates/rank-learn/QUICK_START.md - Referenced from docs/README.md

## Decision Framework

**Archive if:**
- Auto-generated files (JSON reports, HTML reports, introspection)
- Binary artifacts (executables, .rlib files)
- Debug/log files
- Internal research documents
- Duplicate of examples/ or benchmarks/ directories
- Not referenced from README or docs/

**Keep if:**
- Referenced from README or docs/
- User-facing guides
- Active documentation
- Not duplicated elsewhere

## Impact

### Before
- Auto-generated files in source tree
- Binary artifacts committed
- Duplicate documentation
- Unreferenced guides

### After
- Only user-facing, referenced documentation
- Auto-generated files in archive
- Binary artifacts removed
- Clean structure

## Grand Total Archived

**~235+ files** across all cleanup phases:
- Initial cleanup: 75 files
- Deep cleanup: 30 files
- Top-level cleanup: 10 files
- Crusty files cleanup: 17 files
- Analysis/refinement: 6 files (already archived)
- Additional markdown cleanup: ~97 files

Repository is now significantly cleaner with only essential, user-facing documentation remaining.

