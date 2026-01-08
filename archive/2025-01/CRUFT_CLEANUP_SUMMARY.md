# Cruft Cleanup Summary

**Date**: 2025-01-XX  
**Status**: Comprehensive cruft removal completed

## Summary

Removed and archived cruft from the repository, focusing on:
1. Duplicate documentation files
2. Backup files
3. Status/progress documents in active directories
4. Duplicate scripts
5. Outdated documentation

## Files Deleted

### Backup Files (2)
- `crates/rank-retrieve/docs/QUICK_START.md.bak`
- `crates/rank-rerank/src/crossencoder_old.rs.bak`

### Duplicate Typst Documentation (3)
- `docs/TYPST_DEBUGGING.md` (consolidated to `docs/typst/DEBUGGING.md`)
- `docs/TYPST_GUIDE.md` (consolidated to `docs/typst/GUIDE.md`)
- `README_TYPST.md` (consolidated to `docs/typst/README.md`)

### Temporary Analysis Document (1)
- `CRUFT_REPORT.md` (the report itself)

## Files Archived

### Status/Progress Documents from Test Directories (14 files)
**Location**: `archive/2025-01/crates/rank-retrieve/tests/`

- `REFINEMENT_STATUS.md`
- `VALIDATION_COMPLETE.md`
- `REFINEMENT_COMPLETE.md`
- `COMPLETE_REFINEMENT.md`
- `IMPLEMENTATION_COMPLETE.md`
- `FINAL_TEST_SUMMARY.md`
- `LATE_INTERACTION_TEST_SUMMARY.md`
- `CROSS_CRATE_SUMMARY.md`
- `E2E_TEST_SUMMARY.md`
- `REFINEMENT_SUMMARY.md`
- `TEST_REFINEMENT_SUMMARY.md`
- `TEST_REVIEW.md`
- `DOCUMENTATION_VALIDATION.md`
- `RESEARCH_AND_REFINEMENT.md`

### Status Documents from Docs (2 files)
- `crates/rank-retrieve/docs/IMPLEMENTATION_COMPLETE.md` → `archive/2025-01/crates/rank-retrieve/docs/`
- `crates/rank-soft/docs/ALL_FIXED.md` → `archive/2025-01/crates/rank-soft/docs/`

### Duplicate Scripts (7 files)
**Location**: `archive/2025-01/crates/rank-fusion/scripts/`

- `verify_readme_viz.py` (duplicate of `rank-rank/scripts/verify_readme_viz.py`)
- `screenshot_readme.js` (duplicate of `rank-rank/scripts/screenshot_readme.js`)
- `vlm_inspect_readme.py` (duplicate of `rank-rank/scripts/vlm_inspect_readme.py`)
- `README.md` (duplicate documentation)
- `README_VERIFICATION.md` (outdated)
- `README_VLM.md` (outdated)
- `VLM_SETUP.md` (outdated)

### Empty Directories
- `archive/2025-01/reports/` (removed)

## Total Files Cleaned

- **Deleted**: 6 files
- **Archived**: 23+ files
- **Total**: 29+ files removed from active codebase

## Files Kept (Decision Rationale)

### Test Helper Documentation
- `crates/rank-retrieve/tests/SHARED_TEST_UTILITIES.md` - Useful documentation
- `crates/rank-retrieve/tests/TEST_HELPERS.md` - Useful documentation
- `crates/rank-retrieve/tests/CROSS_CRATE_TEST_HELPERS.md` - Useful documentation

### Rank-Fusion Specific Scripts
- `crates/rank-fusion/scripts/verify_readme.sh` - Rank-fusion specific
- `crates/rank-fusion/scripts/verify_all_readmes.sh` - Rank-fusion specific
- `crates/rank-fusion/scripts/vlm_inspect_all_readmes.sh` - Rank-fusion specific

### Deprecated Code
- `crates/rank-retrieve/src/retriever.rs` - Kept for backward compatibility with clear deprecation notice

## Remaining Items (Not Cruft)

### Intentional Placeholders
- TODO comments in `crates/rank-rerank/src/crossencoder/ort.rs` - Documented placeholders for ONNX Runtime
- TODO comments in `crates/rank-soft/src/burn.rs` - Documented placeholders for Burn integration
- Commented test code in `crates/rank-rerank/tests/integration_onnx_runtime.rs` - Intentional placeholders

### Build Artifacts (Gitignored)
- `__pycache__/` directories - Already gitignored, normal Python artifacts
- `.venv/` directories - Already gitignored
- `target/` directories - Already gitignored

### Old Name References
- Some documentation still references `rank-refine` and `rank-relax` - These are historical references in docs, not cruft

## Impact

### Before
- Status documents cluttering test directories
- Duplicate scripts in multiple locations
- Duplicate Typst documentation
- Backup files in repository
- Confusing documentation structure

### After
- Clean test directories (only useful documentation and test code)
- Single canonical location for shared scripts (`rank-rank/scripts/`)
- Consolidated Typst documentation in `docs/typst/`
- No backup files
- Clear documentation structure

## Archive Location

All archived files are in `archive/2025-01/` organized by:
- `crates/rank-retrieve/tests/` - Test status documents
- `crates/rank-retrieve/docs/` - Docs status documents
- `crates/rank-soft/docs/` - Docs status documents
- `crates/rank-fusion/scripts/` - Duplicate scripts and outdated docs

