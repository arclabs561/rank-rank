# Separate GitHub Repos: Keep or Delete?

**Date:** January 2025  
**Context:** Monorepo migration complete, separate repos still exist

## Current Situation

You have:
- ✅ **Monorepo**: `arclabs561/rank-rank` (private) - contains all crates
- ⚠️ **Separate repos**: 
  - `arclabs561/rank-rerank` (public, 1 star)
  - `arclabs561/rank-fusion` (public)
  - `arclabs561/rank-eval` (public)
  - `arclabs561/rank-relax` (public, now renamed to rank-soft)
  - `arclabs561/rank-refine` (public, now renamed to rank-rerank)

**Problem:** All `Cargo.toml` files reference separate repos in `repository` field, but development happens in monorepo.

## Recommendation: **Archive, Don't Delete**

### Why Archive Instead of Delete

1. **Preserve History**: Separate repos may have commit history, issues, PRs, releases
2. **External Links**: crates.io, docs.rs, and other sites may link to separate repos
3. **Reversibility**: Can unarchive if you change your mind
4. **No Breaking Changes**: Existing links continue to work (redirect to monorepo)

### Migration Strategy

#### Option A: Archive with Redirect (Recommended)

1. **Archive all separate repos** (GitHub setting: Archive repository)
2. **Update README in each archived repo** to point to monorepo:
   ```markdown
   # ⚠️ This repository has been archived
   
   This crate has moved to the [rank-rank monorepo](https://github.com/arclabs561/rank-rank).
   
   - **Source**: https://github.com/arclabs561/rank-rank/tree/main/crates/rank-rerank
   - **Issues**: https://github.com/arclabs561/rank-rank/issues
   - **Releases**: https://github.com/arclabs561/rank-rank/releases
   ```

3. **Update Cargo.toml repository fields** to point to monorepo:
   ```toml
   repository = "https://github.com/arclabs561/rank-rank/tree/main/crates/rank-rerank"
   ```

4. **Update pyproject.toml** repository URLs similarly

#### Option B: Keep Separate Repos as Mirrors

**If you want better discoverability:**

1. Keep separate repos but make them **read-only mirrors**
2. Use GitHub Actions to sync from monorepo to separate repos
3. Update READMEs to indicate monorepo is source of truth
4. Issues/PRs go to monorepo

**Trade-off:** More maintenance, but better discoverability

#### Option C: Delete (Not Recommended)

**Only if:**
- Repos have no meaningful history
- No external links exist
- You're certain you'll never need them

**Risks:**
- Broken links from crates.io/docs.rs
- Lost commit history
- Can't reverse decision

## Industry Patterns

### Tokio Pattern (Monorepo Only)
- Single `tokio-rs/tokio` repository
- No separate repos per crate
- All crates in `tokio/` subdirectory
- **Result**: Excellent coordination, but less discoverability per crate

### Serde Pattern (Monorepo + Some Separate)
- Main `serde-rs/serde` monorepo
- Some crates have separate repos for historical reasons
- **Result**: Mixed approach, some confusion

### Your Situation
- Small ecosystem (6 crates)
- Tightly coupled (retrieve → rerank → fusion → eval)
- Early stage (most at 0.1.x)
- **Recommendation**: Archive separate repos, use monorepo only

## Action Plan

### Immediate (This Week)

1. **Archive separate repos**:
   ```bash
   # For each repo:
   # 1. Go to Settings → Archive repository
   # 2. Add redirect notice to README
   ```

2. **Update repository fields in monorepo**:
   ```bash
   # Update all Cargo.toml files
   repository = "https://github.com/arclabs561/rank-rank/tree/main/crates/rank-rerank"
   ```

3. **Update pyproject.toml files** similarly

### Short Term (This Month)

4. **Update crates.io metadata** (if already published):
   - Update repository links in crate metadata
   - Add note about monorepo migration

5. **Update documentation**:
   - Point all docs to monorepo
   - Update badges to use monorepo CI

### Long Term

6. **Monitor for broken links**
7. **Consider unarchiving** if discoverability becomes an issue

## Benefits of Archiving

✅ **Single source of truth** - monorepo only  
✅ **No broken links** - archived repos still accessible  
✅ **Preserved history** - can reference old repos if needed  
✅ **Reversible** - can unarchive if needed  
✅ **Cleaner workflow** - no confusion about where to contribute  

## Risks of Keeping Separate Repos

❌ **Confusion** - where do issues go?  
❌ **Maintenance burden** - keeping repos in sync  
❌ **Stale information** - separate repos get outdated  
❌ **Split community** - contributors don't know where to look  

## Decision Matrix

| Factor | Archive | Keep Separate | Delete |
|--------|---------|---------------|--------|
| Discoverability | Medium | High | Low |
| Maintenance | Low | High | Low |
| History Preservation | ✅ | ✅ | ❌ |
| Reversibility | ✅ | ✅ | ❌ |
| Simplicity | ✅ | ❌ | ✅ |
| External Links | ✅ | ✅ | ❌ |

**Verdict: Archive** - Best balance of simplicity and safety.

## Implementation Checklist

- [ ] Archive `rank-rerank` repo
- [ ] Archive `rank-fusion` repo  
- [ ] Archive `rank-eval` repo
- [ ] Archive `rank-relax` repo (if still exists)
- [ ] Archive `rank-refine` repo (if still exists)
- [ ] Add redirect notices to all archived repos
- [ ] Update all `Cargo.toml` repository fields
- [ ] Update all `pyproject.toml` repository fields
- [ ] Update monorepo README with links to archived repos
- [ ] Test that crates.io/docs.rs links still work

## References

- [GitHub: Archiving Repositories](https://docs.github.com/en/repositories/archiving-a-github-repository)
- [Cargo: Package Metadata](https://doc.rust-lang.org/cargo/reference/manifest.html#the-repository-field)
- [Tokio Repository Structure](https://github.com/tokio-rs/tokio)
