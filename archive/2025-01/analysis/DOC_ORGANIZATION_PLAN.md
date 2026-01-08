# Documentation Organization Plan

## Current State

**266 MD files** found across the repository. Many are:
- Status/complete files (temporary progress tracking)
- Duplicate analysis files
- Outdated decision documents
- Archive-worthy historical records

## Categorization

### 1. Root-Level Status Files (Archive)

These are temporary progress tracking files that should be archived:

**Status/Complete Files:**
- `ALL_COMPLETE.md`
- `BENCHMARKING_COMPLETE.md`
- `BENCHMARKING_READY.md`
- `COMPLETE_STATUS.md`
- `FINAL_STATUS.md`
- `FINAL_TIDY_COMPLETE.md`
- `MERGE_COMPLETE.md`
- `MONOREPO_FINAL_STATUS.md`
- `MONOREPO_MIGRATION_COMPLETE.md`
- `MONOREPO_SETUP_COMPLETE.md`
- `REFINEMENT_COMPLETE.md`
- `RENAME_COMPLETE.md`
- `SETUP_COMPLETE.md`
- `TIDY_COMPLETE.md`
- `TEST_RESULTS.md`

**Analysis/Decision Files (Historical):**
- `CRATE_ORGANIZATION_ANALYSIS.md`
- `DECISION_REVIEW.md`
- `FINAL_NAME_RECOMMENDATION.md`
- `FINAL_ORGANIZATION_ANALYSIS.md`
- `FINAL_STRUCTURE.md`
- `IMPLEMENTATION_SUMMARY.md`
- `MONOREPO_RECOMMENDATION.md`
- `MONOREPO_VS_SEPARATE_ANALYSIS.md`
- `NAME_DECISION_ANALYSIS.md`
- `NAME_DECISION_WITH_PUBLISHED_CONSTRAINT.md`
- `ORGANIZATION_RECOMMENDATIONS.md`
- `RANK_COLLECTION_ORGANIZATION_RESEARCH.md`
- `RANK_SPARSE_ANALYSIS.md`
- `RANK_SPARSE_COMPLETE.md`
- `RANK_SPARSE_FINAL.md`
- `RANK_SPARSE_FINAL_STATUS.md`
- `RANK_SPARSE_MERGE_COMPLETE.md`
- `RANK_SPARSE_MERGE_COMPLETE_FINAL.md`
- `RANK_SPARSE_MERGE_FINAL.md`
- `RANK_SPARSE_RESTRUCTURE_COMPLETE.md`
- `REPOSITORY_STRUCTURE_OPTIONS.md`
- `RESEARCH_ANALYSIS.md`
- `RESEARCH_FINDINGS.md`
- `REVIEW_ANALYSIS.md`
- `RENAME_INSTRUCTIONS.md`
- `RENAME_PLAN.md`
- `RENAME_PROGRESS.md`
- `RENAME_SUMMARY.md`
- `RUST_ECOSYSTEM_ORGANIZATION_RESEARCH.md`
- `STRUCTURE_SUMMARY.md`

**Mathematical Theory (Keep, but organize):**
- `RELAX_MATHEMATICAL_THEORY.md` → Move to `docs/theory/`
- `SOFT_MATHEMATICAL_THEORY.md` → Move to `docs/theory/`

### 2. Keep at Root (Active Documentation)

**Core Documentation:**
- `README.md` (main entry point)
- `README_TYPST.md` (Typst documentation guide)
- `CURSOR_CONFIG.md` (Cursor setup)
- `SECURITY_AUDIT.md` (security info)
- `USAGE.md` (usage guide)
- `SETUP.md` (setup instructions)

**Active Analysis:**
- `CRATES_VS_SRC_ANALYSIS.md` (just created, relevant)
- `LTR_ANALYSIS.md` (Learning to Rank analysis)
- `REFINEMENT_PRIORITIES.md` (active priorities)

### 3. Crate-Specific Docs (Keep in place)

These are fine where they are:
- `crates/*/README.md` (crate documentation)
- `crates/*/docs/*.md` (crate-specific docs)
- `crates/*/CHANGELOG.md` (changelogs)

### 4. Archive Structure

Create dated archive directories:

```
archive/
├── 2025-01/
│   ├── status/          # Status/complete files
│   ├── analysis/        # Analysis/decision files
│   └── renames/         # Rename-related files
└── README.md            # Archive index
```

## Action Plan

1. **Create archive structure**
2. **Move status files** to `archive/2025-01/status/`
3. **Move analysis files** to `archive/2025-01/analysis/`
4. **Move rename files** to `archive/2025-01/renames/`
5. **Move theory files** to `docs/theory/`
6. **Update root README** to reference active docs only
7. **Create archive README** explaining what's archived and why

## Benefits

1. **Cleaner root**: Only active documentation visible
2. **Better navigation**: Clear what's current vs historical
3. **Preserved history**: Nothing deleted, just organized
4. **Easier discovery**: Active docs are obvious

