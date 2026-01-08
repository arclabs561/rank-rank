# Archived Duplicate Scripts

This directory contains duplicate scripts that were archived from `crates/rank-fusion/scripts/`.

## Files Archived

These scripts are duplicates of the canonical versions in `rank-rank/scripts/`:

- `verify_readme_viz.py` - Duplicate of `rank-rank/scripts/verify_readme_viz.py` (identical, 323 lines)
- `screenshot_readme.js` - Duplicate of `rank-rank/scripts/screenshot_readme.js` (identical, 30 lines)
- `vlm_inspect_readme.py` - Duplicate of `rank-rank/scripts/vlm_inspect_readme.py` (minor differences: parent directory fallback)
- `README.md` - Duplicate documentation (root `scripts/README.md` is canonical)
- `README_VERIFICATION.md` - Outdated verification documentation
- `README_VLM.md` - Outdated VLM documentation
- `VLM_SETUP.md` - Outdated setup documentation

## Rationale

The root `rank-rank/scripts/` directory is the canonical location for shared scripts across all rank-* repositories. The rank-fusion-specific scripts (`verify_readme.sh`, `verify_all_readmes.sh`, `vlm_inspect_all_readmes.sh`) were kept as they are rank-fusion-specific implementations.

## Archive Date

2025-01-XX

