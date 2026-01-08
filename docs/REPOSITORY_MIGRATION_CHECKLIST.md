# Repository Migration Checklist

**Date:** January 2025  
**Status:** ✅ Complete - All repository fields updated, repos archived

## ✅ Completed

- [x] Updated all `Cargo.toml` repository fields to point to monorepo (8 files)
- [x] Updated all `pyproject.toml` repository/homepage URLs to point to monorepo (5 files)
- [x] Updated CI badges in README files (6 files)
- [x] Updated code comments referencing old repos (3 files: matryoshka.rs, colbert.rs, scoring.rs)
- [x] Updated documentation references (3 files: AVX512_TESTING.md, DOCUMENTATION_INDEX.md, GETTING_STARTED.md)
- [x] Fixed all workflow references (`rank-refine` → `rank-rerank`, `rank-relax` → `rank-soft`)
- [x] Fixed all example code references (`rank-refine` → `rank-rerank`, `rank-relax` → `rank-soft`)
- [x] Created migration decision document (`docs/SEPARATE_REPOS_DECISION.md`)

## 📋 Next Steps

### Phase 1: Update Documentation and Badges (Before Archiving)

- [x] **Update CI badges in README files** ✅
  - ✅ `crates/rank-rerank/README.md`
  - ✅ `crates/rank-fusion/README.md`
  - ✅ `crates/rank-eval/README.md` (no badge found)
  - ✅ `crates/rank-soft/README.md`
  - ✅ `crates/rank-retrieve/README.md`
  - ✅ `crates/rank-learn/README.md`

- [x] **Update repository references in documentation** ✅
  - ✅ Updated code comments in `crates/rank-rerank/src/*.rs`
  - ✅ Updated `crates/rank-rerank/docs/AVX512_TESTING.md`
  - ✅ Updated `crates/rank-soft/docs/DOCUMENTATION_INDEX.md`
  - ✅ Updated `crates/rank-soft/docs/GETTING_STARTED.md`

- [x] **Check for any remaining references** ✅
  - ✅ All code comments updated
  - ✅ All documentation updated
  - ✅ All README badges updated
  - Note: Only references found are in `.venv` (Python cache, safe to ignore)

- [x] **Update GitHub Actions workflows** ✅
  - ✅ Workflows use `actions/checkout@v4` which checks out current repo (monorepo)
  - ✅ Fixed all `rank-refine` → `rank-rerank` references in rank-rerank workflows:
    - `publish.yml` (version checks, cargo publish, working-directory)
    - `ci.yml` (cargo doc command)
    - `e2e-published.yml` (all package names: Rust, Python, WASM)
    - `publish-wasm.yml` (package.json path, working-directory)
  - ✅ Fixed all `rank-relax` → `rank-soft` references in rank-soft workflows:
    - `publish.yml` (working-directory, pip install)
    - `e2e-published.yml` (all package names: Rust, Python)
    - `test.yml` (working-directory)
  - ✅ No hardcoded repository references found in workflows
  - ✅ Workflows will work correctly with monorepo structure

### Phase 2: Archive Separate Repositories

**Only proceed after Phase 1 is complete!**

- [x] **Archive `rank-fusion` repository** ✅
  - Archived via `gh repo archive arclabs561/rank-fusion --yes`

- [x] **Archive `rank-eval` repository** ✅
  - Archived via `gh repo archive arclabs561/rank-eval --yes`

- [x] **Archive `rank-relax` repository** ✅ (old name for rank-soft)
  - Archived via `gh repo archive arclabs561/rank-relax --yes`

- [x] **Archive `rank-refine` repository** ✅ (old name for rank-rerank)
  - Archived via `gh repo archive arclabs561/rank-refine --yes`

- [x] **Check for other repos** ✅
  - `rank-rerank`, `rank-soft`, `rank-retrieve`, `rank-learn` don't exist as separate repos
  - They were never created separately or were already deleted

### Phase 3: Add Redirect Notices to Archived Repos

- [x] **Redirect notices** ✅
  - Temporarily unarchived repos via GitHub API (`PATCH /repos/{owner}/{repo}` with `archived=false`)
  - Added redirect notices to all archived repos via GitHub API:
    - `rank-fusion`: Points to monorepo with source link
    - `rank-eval`: Points to monorepo with source link
    - `rank-relax`: Points to monorepo with note about rename to `rank-soft`
    - `rank-refine`: Points to monorepo with note about rename to `rank-rerank`
  - Re-archived all repos via GitHub API (`PATCH /repos/{owner}/{repo}` with `archived=true`)
  - All archived repos now have clear redirect notices in their READMEs

- [x] **Note**: `rank-rerank`, `rank-soft`, `rank-retrieve`, `rank-learn` don't exist as separate repos, so no redirect needed

### Phase 4: Update External References

- [ ] **Update crates.io metadata** (if crates are already published)
  - Note: Repository field in `Cargo.toml` will be used in next publish
  - May need to update manually if crates.io has cached old URLs

- [ ] **Update docs.rs links** (if applicable)
  - docs.rs automatically uses repository field from `Cargo.toml`
  - Will update on next publish

- [ ] **Check for external links**
  - Search GitHub for references to old repos
  - Update any documentation sites
  - Update any blog posts or articles

### Phase 5: Verify and Test

- [ ] **Verify all repository fields updated**
  ```bash
  cd /Users/arc/Documents/dev/_rank-rank
  grep -r "github.com/arclabs561/rank-rerank" crates/ --exclude-dir=target
  grep -r "github.com/arclabs561/rank-fusion" crates/ --exclude-dir=target
  grep -r "github.com/arclabs561/rank-eval" crates/ --exclude-dir=target
  # Should find no matches (except in this checklist/doc)
  ```

- [ ] **Test that monorepo links work**
  - Click through all updated repository URLs
  - Verify they point to correct crate directories

- [ ] **Test CI badges** (if updated)
  - Verify badges display correctly
  - Check that they link to correct workflows

- [ ] **Run tests to ensure nothing broke**
  ```bash
  cargo test --workspace
  ```

## Files Updated

### Cargo.toml Files
- ✅ `crates/rank-rerank/Cargo.toml`
- ✅ `crates/rank-fusion/Cargo.toml`
- ✅ `crates/rank-eval/Cargo.toml`
- ✅ `crates/rank-soft/Cargo.toml`
- ✅ `crates/rank-retrieve/Cargo.toml`
- ✅ `crates/rank-learn/Cargo.toml`
- ✅ `crates/rank-rerank/rank-rerank-python/Cargo.toml`
- ✅ `crates/rank-fusion/rank-fusion-python/Cargo.toml`

### pyproject.toml Files
- ✅ `crates/rank-rerank/rank-rerank-python/pyproject.toml`
- ✅ `crates/rank-fusion/rank-fusion-python/pyproject.toml`
- ✅ `crates/rank-eval/rank-eval-python/pyproject.toml`
- ✅ `crates/rank-soft/rank-soft-python/pyproject.toml`
- ✅ `crates/rank-retrieve/rank-retrieve-python/pyproject.toml`
- ⚠️ `crates/rank-learn/rank-learn-python/pyproject.toml` (check if it has URLs)

## Notes

- **Do NOT archive repos yet** - wait until all documentation is updated
- **Archived repos are still accessible** - they just show as archived
- **Can unarchive if needed** - decision is reversible
- **External links will still work** - archived repos remain accessible

## Verification Commands

```bash
# Check for any remaining old repo references
cd /Users/arc/Documents/dev/_rank-rank
grep -r "github.com/arclabs561/rank-rerank" crates/ --exclude-dir=target
grep -r "github.com/arclabs561/rank-fusion" crates/ --exclude-dir=target
grep -r "github.com/arclabs561/rank-eval" crates/ --exclude-dir=target
grep -r "github.com/arclabs561/rank-soft" crates/ --exclude-dir=target
grep -r "github.com/arclabs561/rank-retrieve" crates/ --exclude-dir=target
grep -r "github.com/arclabs561/rank-learn" crates/ --exclude-dir=target

# Verify new repo references
grep -r "github.com/arclabs561/rank-rank/tree/main/crates" crates/ --exclude-dir=target | wc -l
# Should show all updated files
```

## Timeline

- **Week 1**: Complete Phase 1 (documentation updates)
- **Week 2**: Complete Phase 2 (archive repos) and Phase 3 (redirect notices)
- **Week 3**: Complete Phase 4 (external references) and Phase 5 (verification)

## Questions to Resolve

- [ ] Should CI badges point to monorepo workflows or crate-specific workflows?
- [ ] Do any external sites/documentation need manual updates?
- [ ] Are there any GitHub Actions that depend on separate repos?
