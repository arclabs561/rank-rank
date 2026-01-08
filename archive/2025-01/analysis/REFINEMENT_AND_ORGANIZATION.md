# Structure Refinement & Documentation Organization

## Part 1: Structure Refinement

### Research Finding: Flat Structure is Superior

Based on deep research of Rust monorepo best practices:
- **rust-analyzer**: 200K lines, 32 crates, all flat in `crates/`
- **Tokio, Serde, Clap**: All use flat structures
- **Key insight**: Flat structures avoid "ossification" problem, easier discovery, no mental model mismatch

### Current Inconsistency

**Flat (correct):**
- `rank-eval`: `Cargo.toml` has `default-members = ["."]` → package at root
- `rank-soft`: `Cargo.toml` has `default-members = ["."]` → package at root

**Nested (should flatten):**
- `rank-retrieve`: `default-members = ["rank-retrieve"]` → nested package
- `rank-fusion`: `default-members = ["rank-fusion"]` → nested package  
- `rank-rerank`: `default-members = ["rank-rerank-core"]` → nested + wrong name
- `rank-learn`: `default-members = ["rank-learn"]` → nested package

### Target: All Flat

All crates should match `rank-eval` pattern:

```
crates/rank-retrieve/
├── Cargo.toml          # Workspace + Package (default-members = ["."])
├── src/lib.rs          # Top-level source
├── rank-retrieve-python/
└── README.md
```

### Migration Steps

1. Move `rank-retrieve/rank-retrieve/src/*` → `rank-retrieve/src/`
2. Merge package definition into workspace Cargo.toml
3. Set `default-members = ["."]`
4. Remove nested directory
5. Repeat for other nested crates

## Part 2: Documentation Organization

### Current State: 266 MD Files

**Root-level clutter:**
- 57 status/complete files (temporary progress tracking)
- Multiple duplicate analysis files
- Historical decision documents

### Organization Plan

**Archive Structure:**
```
archive/
├── 2025-01/
│   ├── status/          # Status/complete files
│   ├── analysis/        # Analysis/decision files  
│   └── renames/         # Rename-related files
└── README.md            # Archive index
```

**Keep at Root:**
- `README.md` (main entry)
- `CURSOR_CONFIG.md`
- `SECURITY_AUDIT.md`
- `USAGE.md`
- `SETUP.md`
- `LTR_ANALYSIS.md` (active)
- `REFINEMENT_PRIORITIES.md` (active)
- `CRATES_VS_SRC_ANALYSIS.md` (just created)

**Move to docs/:**
- `RELAX_MATHEMATICAL_THEORY.md` → `docs/theory/`
- `SOFT_MATHEMATICAL_THEORY.md` → `docs/theory/`

### Files to Archive

**Status Files (27):**
- ALL_COMPLETE.md, BENCHMARKING_COMPLETE.md, COMPLETE_STATUS.md
- FINAL_STATUS.md, FINAL_TIDY_COMPLETE.md, MERGE_COMPLETE.md
- MONOREPO_FINAL_STATUS.md, MONOREPO_MIGRATION_COMPLETE.md
- REFINEMENT_COMPLETE.md, RENAME_COMPLETE.md, SETUP_COMPLETE.md
- TIDY_COMPLETE.md, TEST_RESULTS.md, etc.

**Analysis Files (30+):**
- CRATE_ORGANIZATION_ANALYSIS.md, DECISION_REVIEW.md
- FINAL_NAME_RECOMMENDATION.md, FINAL_ORGANIZATION_ANALYSIS.md
- RANK_SPARSE_ANALYSIS.md, RANK_SPARSE_MERGE_COMPLETE.md (multiple)
- MONOREPO_RECOMMENDATION.md, REPOSITORY_STRUCTURE_OPTIONS.md
- etc.

## Action Plan

1. ✅ Create archive structure
2. ⏳ Flatten nested crates (start with rank-retrieve, rank-learn)
3. ⏳ Archive status/analysis files
4. ⏳ Move theory docs to docs/theory/
5. ⏳ Update root README to reference active docs only
6. ⏳ Create archive README

