# Monorepo Architecture Critique: rank-rank

**Date:** January 2025  
**Status:** Comprehensive analysis after deep research  
**Updated:** Includes Python bindings (PyO3/maturin) analysis

## Executive Summary

The current `rank-rank` monorepo structure is **partially correct** but has significant architectural issues that limit its effectiveness. The structure resembles "separate repositories in one directory" rather than a true monorepo, missing key benefits while retaining coordination overhead.

**Python Bindings Assessment:** The Python bindings architecture is **correctly structured** - bindings are workspace members of their parent crates (not root workspace), which is the recommended pattern. However, version synchronization and publishing workflows need improvement.

**Verdict:** The monorepo approach is appropriate for this project, but the implementation needs substantial improvement. Python bindings structure is good; Rust workspace structure needs fixing.

---

## Current Architecture Analysis

### Structure

```
rank-rank/                    (Workspace root)
├── Cargo.toml                (Workspace with all crates as members)
├── crates/
│   ├── rank-retrieve/        (Workspace member)
│   ├── rank-fusion/          (Workspace member)
│   ├── rank-rerank/          (Workspace member)
│   ├── rank-soft/            (Workspace member - includes LTR algorithms)
│   └── rank-eval/            (Workspace member)
```

### Key Observations

1. **Root workspace includes all crates** - ✅ All crates are workspace members
2. **Python bindings are per-crate workspace members** - ✅ Correct structure
3. **Cross-crate dependencies properly specified** - ✅ With version constraints
4. **Version numbers are independent** - Each crate versions independently
5. **Publishing is per-crate** - Can use `cargo publish --workspace` for coordination

---

## Critical Issues

### 1. **Not a True Monorepo**

**Problem:** The root `Cargo.toml` doesn't declare workspace members. Each crate operates as an independent workspace.

**Evidence:**
```toml
# Root Cargo.toml
[workspace]
# Note: Each crate has its own workspace, so we don't include them here
# This is just for shared dependencies.
```

**Impact:**
- Cannot run `cargo test --workspace` from root
- Cannot use `cargo publish --workspace` (Cargo 1.90+ feature)
- No unified dependency resolution
- Missing benefits of workspace-level tooling

**Comparison to Industry Standards:**
- **tokio**: Single workspace with all crates as members
- **serde**: Single workspace with coordinated publishing
- **clap**: Single workspace with unified versioning

### 1a. **Python Bindings Architecture (PyO3/Maturin)**

**Current Structure:**
- Each Rust crate has its own workspace including Python bindings
- Example: `rank-rerank` workspace includes `rank-rerank-python` as a member
- Python bindings use path dependencies to parent Rust crate
- Versions are manually synchronized between `Cargo.toml` and `pyproject.toml`

**What's Working:**
- ✅ Python bindings are correctly structured as workspace members (per-crate)
- ✅ Path dependencies work for local development
- ✅ Maturin correctly builds from workspace contexts
- ✅ Version synchronization is enforced in CI

**What's Problematic:**
- ⚠️ **Manual version synchronization** - Versions must be updated in both `Cargo.toml` and `pyproject.toml`
- ⚠️ **No automatic Python publishing** - Maturin doesn't support workspace publishing like Cargo 1.90+
- ⚠️ **Dual publishing workflow** - Must publish Rust crate first, then Python package separately
- ⚠️ **Version drift risk** - Python and Rust versions can diverge if not careful

**Key Insight:** Python bindings should NOT be in the root workspace. They belong in each crate's workspace (current structure is correct for this). However, the root workspace should still include all Rust crates.

### 2. **Cross-Crate Dependency Management** ✅ RESOLVED

**Status:** All cross-crate dependencies properly specify versions.

**Current State:**
- All workspace dependencies include version specifications
- Dependencies are properly declared in `[workspace.dependencies]` with versions
- Individual crates reference workspace dependencies correctly

**Note:** `rank-learn` has been merged into `rank-soft` (January 2025), eliminating the previous cross-crate dependency issue.

**Python Bindings Consideration:**
- Python bindings depend on parent Rust crate via path (correct for local development)
- When publishing, maturin handles path→version conversion automatically
- No cross-crate Python dependencies currently needed

### 3. **No Publishing Coordination Strategy**

**Problem:** Each crate has independent publishing workflows with no coordination.

**Evidence:**
- `rank-rerank`: v0.7.36 (mature)
- `rank-fusion`: v0.1.20 (early)
- Others: v0.1.0 (very early)

**Issues:**
- No mechanism to publish interdependent crates together
- Manual ordering required for publishing
- Risk of inconsistent published states

**Missing:** Workspace publishing automation (Cargo 1.90+ `cargo publish --workspace`)

**Python Publishing Complexity:**
- **Dual publishing required**: Rust crate → crates.io, then Python package → PyPI
- **Sequential dependency**: Python package cannot publish until Rust crate is available on crates.io
- **No maturin workspace publishing**: Must publish each Python package individually
- **Version synchronization**: Manual updates in both `Cargo.toml` and `pyproject.toml`
- **Current workflow**: CI validates versions match, but publishing is separate jobs

### 4. **Version Number Chaos**

**Problem:** Independent versioning with no coordination strategy.

**Current State:**
- `rank-rerank`: 0.7.36 (Rust) / 0.7.36 (Python) ✅
- `rank-fusion`: 0.1.20 (Rust) / 0.1.20 (Python) ✅
- `rank-eval`: 0.1.0 (Rust) / 0.1.0 (Python) ✅
- `rank-retrieve`: 0.1.0 (Rust) / 0.1.0 (Python) ✅
- `rank-soft`: 0.1.0 (Rust) / 0.1.0 (Python) ✅
- `rank-soft`: 0.1.0 (Rust) / 0.1.0 (Python) - Includes LTR algorithms

**Good News:**
- ✅ Rust and Python versions are synchronized within each crate
- ✅ CI validates version consistency before publishing

**Issues:**
- Users cannot infer compatibility from version numbers across crates
- No clear signal about maturity/stability
- Breaking changes in dependencies require manual tracking
- Confusing for users trying to understand the ecosystem

**Python-Specific Versioning Challenges:**
- **Manual synchronization**: Versions in `pyproject.toml` must match `Cargo.toml`
- **No dynamic versioning**: Not using maturin's ability to read from `Cargo.toml`
- **Dual maintenance**: Two files to update per release
- **Risk of drift**: Easy to forget updating `pyproject.toml`

**Industry Patterns:**
- **Unified versioning**: All crates share major.minor (tokio ecosystem)
- **Independent versioning**: Each crate versions independently (serde ecosystem)
- **Hybrid**: Core crates unified, utilities independent

**Recommendation:** Choose a strategy and document it clearly. Consider using dynamic versioning in `pyproject.toml` to reduce duplication.

### 5. **Root Workspace Serves No Purpose**

**Problem:** Root `Cargo.toml` only provides shared dependencies but doesn't create a workspace.

**Current:**
```toml
[workspace]
# Note: Each crate has its own workspace, so we don't include them here
```

**Issues:**
- Cannot use workspace-level commands
- No unified dependency resolution
- Duplicated dependency specifications
- Missing workspace inheritance benefits

**What It Should Be:**
```toml
[workspace]
members = [
    "crates/rank-retrieve",
    "crates/rank-fusion",
    "crates/rank-rerank",
    "crates/rank-soft",
    "crates/rank-learn",
    "crates/rank-eval",
]
resolver = "2"
```

---

## Publishing Workflow Analysis

### Current State

Each crate has its own `.github/workflows/publish.yml` that:
1. Validates version consistency (within that crate)
2. Publishes Rust crate to crates.io
3. Publishes Python package to PyPI

**Problems:**
1. **No cross-crate coordination** - Publishing interdependent crates requires manual ordering
2. **Manual ordering required** - Must publish dependencies before dependents
3. **No atomic publishing** - Partial failures leave inconsistent state
4. **Version validation is per-crate** - Doesn't check cross-crate compatibility

**Note:** `rank-learn` has been merged into `rank-soft` (January 2025), reducing cross-crate dependencies.

### What's Missing

**Cargo 1.90+ Workspace Publishing:**
```bash
# Should be able to do:
cargo publish --workspace  # Publishes all crates in dependency order
```

**Current limitation:** Cannot use this because root workspace has no members.

**Required:** Restructure to true workspace, then use workspace publishing.

---

## Comparison to Industry Standards

### Tokio Pattern (Recommended)

**Structure:**
- Single workspace with all crates as members
- Unified versioning (major.minor synchronized)
- Workspace dependency inheritance
- Coordinated releases

**Benefits:**
- `cargo test --workspace` works
- `cargo publish --workspace` works
- Unified dependency resolution
- Clear compatibility signals

### Serde Pattern (Alternative)

**Structure:**
- Single workspace
- Independent versioning
- Coordinated but independent releases
- Clear API boundaries

**Benefits:**
- Each crate can evolve independently
- More accurate version history
- Requires careful dependency management

### Your Current Pattern (Not Recommended)

**Structure:**
- Virtual root workspace
- Independent workspaces per crate
- No coordination mechanism

**Problems:**
- Missing workspace benefits
- Retaining coordination overhead
- Worst of both worlds

---

## Recommendations

### Immediate Fixes (High Priority)

1. **Convert to True Workspace**
   ```toml
   # Root Cargo.toml
   [workspace]
   members = [
       "crates/rank-retrieve",
       "crates/rank-retrieve/rank-retrieve-python",
       "crates/rank-fusion",
       "crates/rank-fusion/rank-fusion-python",
       "crates/rank-rerank",
       "crates/rank-rerank/rank-rerank-python",
       "crates/rank-soft",
       "crates/rank-soft/rank-soft-python",
       "crates/rank-eval",
       "crates/rank-eval/rank-eval-python",
   ]
   resolver = "2"
   ```

2. **Fix Cross-Crate Dependencies**
   ```toml
   # rank-learn/Cargo.toml
   [dependencies]
   rank-soft = { path = "../rank-soft", version = "0.1", package = "rank-soft" }
   ```

3. **Add Workspace Dependency Inheritance**
   ```toml
   # Root Cargo.toml
   [workspace.dependencies]
   rank-soft = { path = "crates/rank-soft", version = "0.1", package = "rank-soft" }
   
   # rank-learn/Cargo.toml
   [dependencies]
   rank-soft = { workspace = true }
   ```

4. **Improve Python Version Management**
   ```toml
   # Option A: Use dynamic versioning (recommended)
   # pyproject.toml
   [project]
   dynamic = ["version"]  # Maturin reads from Cargo.toml
   
   # Option B: Keep manual but add validation script
   # scripts/check_versions.sh - validates all versions match
   ```

### Medium-Term Improvements

4. **Adopt Workspace Publishing for Rust**
   - Use `cargo publish --workspace` for coordinated Rust releases
   - Implement registry overlay verification
   - Add dry-run validation
   - **Note**: Maturin doesn't support workspace publishing yet - Python packages remain per-crate

5. **Choose Versioning Strategy**
   - **Option A (Recommended)**: Unified major.minor (e.g., all at 0.1.x)
   - **Option B**: Independent with compatibility matrix
   - Document the choice clearly
   - **Python consideration**: Use dynamic versioning to reduce duplication

6. **Implement Release Automation**
   - Use `release-plz` or `cargo-release` for Rust automation
   - Integrate `cargo-semver-checks` for breaking change detection
   - Add CI validation before publishing
   - **Python workflow**: After Rust publish, wait for crates.io availability, then publish Python packages
   - Consider custom script to orchestrate: Rust publish → wait → Python publish

### Long-Term Considerations

7. **Evaluate Crate Boundaries**
   - Are all crates truly independent?
   - Should some be merged?
   - Are boundaries clear to users?

8. **Document Publishing Strategy**
   - When to publish together vs. independently
   - How breaking changes propagate
   - Version compatibility guarantees

---

## Specific Code Issues Found

### Issue 1: rank-learn Merged into rank-soft ✅ RESOLVED

**Status:** `rank-learn` functionality has been merged into `rank-soft` (January 2025).

**Resolution:**
- LambdaRank, Ranking SVM, and Neural LTR are now in `rank-soft`
- All LTR algorithms available through `rank-soft` API
- Unified training crate for all ranking training needs
- Matches industry patterns (LightGBM/XGBoost integrate ranking objectives)

**Note:** The `rank-learn` directory may still exist for historical reference but is deprecated.

### Issue 2: Root Workspace Configuration

**File:** `Cargo.toml` (root)

**Current:** Virtual workspace with no members.

**Fix:** Add members array (see Recommendation #1).

### Issue 3: Publishing Workflows

**Files:** `crates/*/.github/workflows/publish.yml`

**Problem:** No cross-crate coordination.

**Fix:** Add workspace-level publishing workflow or use `cargo publish --workspace`.

---

## Benefits of Fixing

### Immediate Benefits

1. **Unified Testing**
   ```bash
   cargo test --workspace  # Works from root
   cargo clippy --workspace  # Lints all crates
   cargo fmt --all  # Formats everything
   ```

2. **Coordinated Publishing**
   ```bash
   cargo publish --workspace  # Publishes in dependency order
   ```

3. **Dependency Resolution**
   - Single `Cargo.lock` at root
   - Unified version resolution
   - Shared build artifacts

### Long-Term Benefits

4. **Easier Refactoring**
   - Atomic commits across crates
   - Cross-crate refactoring tools work
   - Better IDE support

5. **Clearer User Experience**
   - Unified versioning signals compatibility
   - Single repository to clone
   - Coordinated releases

6. **Reduced Maintenance**
   - Less duplication
   - Centralized dependency management
   - Automated release coordination

---

## Migration Path

### Phase 1: Restructure Workspace (1-2 hours)

1. Update root `Cargo.toml` to include members
2. Remove individual workspace declarations from crates
3. Test: `cargo check --workspace`

### Phase 2: Fix Dependencies (1 hour)

1. Add version specs to cross-crate dependencies
2. Use workspace dependency inheritance
3. Test: `cargo publish --dry-run --workspace`

### Phase 3: Update CI/CD (2-3 hours)

1. Add workspace-level publishing workflow
2. Update per-crate workflows (or remove if redundant)
3. Test publishing process

### Phase 4: Version Strategy (Ongoing)

1. Choose unified vs. independent versioning
2. Document strategy
3. Implement automation

---

## Python Bindings Deep Dive

### Current Architecture (Correct Pattern)

**Structure:**
```
rank-rerank/
├── Cargo.toml              (workspace root for this crate)
│   └── members = ["rank-rerank-python", "fuzz", "test-e2e-local"]
├── src/                    (Rust library)
├── rank-rerank-python/     (Python bindings workspace member)
│   ├── Cargo.toml         (depends on parent via path)
│   ├── pyproject.toml     (Python package metadata)
│   └── src/               (PyO3 bindings)
```

**This is correct!** Python bindings should be workspace members of their parent crate, not the root workspace.

### How Maturin Works with Workspaces

1. **Maturin respects Cargo workspaces** - Can build from workspace context
2. **Path dependencies work** - `rank-rerank-python` depends on `rank-rerank` via `{ path = ".." }`
3. **Version handling** - Maturin can read version from `Cargo.toml` or `pyproject.toml`
4. **No workspace publishing** - Must publish each Python package individually

### Version Synchronization

**Current Approach (Manual):**
```toml
# Cargo.toml
[workspace.package]
version = "0.7.36"

# pyproject.toml
[project]
version = "0.7.36"  # Must match manually
```

**Better Approach (Dynamic):**
```toml
# pyproject.toml
[project]
dynamic = ["version"]  # Maturin reads from Cargo.toml automatically
```

**Benefits:**
- Single source of truth (Cargo.toml)
- No manual synchronization
- Less error-prone

### Publishing Workflow

**Current (Sequential):**
1. Publish Rust crate to crates.io
2. Wait for availability (seconds to minutes)
3. Publish Python package to PyPI

**Why Sequential:**
- Python package's sdist includes `Cargo.toml` with version constraints
- Maturin verifies dependencies during sdist build
- If Rust crate isn't on crates.io yet, verification fails

**Improvement Opportunity:**
- Use Cargo 1.90+ workspace publishing for Rust (handles ordering)
- Then publish Python packages (maturin doesn't support workspace publishing yet)
- Consider custom script to orchestrate both

### Python Bindings as Workspace Members

**Why This Works:**
- Each crate's workspace includes its Python bindings
- Enables `cargo test --workspace` within each crate
- Shared dependency resolution for Rust and Python build
- Clear ownership: Python bindings belong to their parent crate

**Why NOT in Root Workspace:**
- Root workspace should contain only Rust crates
- Python packages are published separately (PyPI vs crates.io)
- Different versioning/lifecycle concerns
- Maturin builds from crate context, not workspace root

## Conclusion

The monorepo approach is **correct and appropriate** for the rank-* ecosystem. The crates are tightly coupled (retrieve → rerank → fusion → eval), benefit from atomic commits, and should be developed together.

However, the **current implementation is suboptimal**. The structure is more like "separate repos in one directory" than a true monorepo, missing key benefits while retaining coordination overhead.

**Python Bindings Assessment:**
- ✅ **Correctly structured** - Python bindings are workspace members of parent crates
- ✅ **Version synchronization** - Currently working, but could be improved with dynamic versioning
- ⚠️ **Publishing workflow** - Manual coordination required (Rust first, then Python)
- ⚠️ **No automation** - No unified publishing for Python packages (maturin limitation)

**Priority Actions:**
1. ✅ Convert to true workspace (add Rust crates as members)
2. ✅ Fix cross-crate dependencies (add version specs)
3. ✅ Adopt workspace publishing for Rust (Cargo 1.90+)
4. ✅ Improve Python versioning (use dynamic versioning)
5. ✅ Choose versioning strategy (unified recommended)
6. ⚠️ **Python publishing**: Keep sequential workflow (maturin limitation) but automate it

**Estimated Effort:** 4-6 hours for complete migration.

**Risk:** Low - changes are structural, not functional. All existing functionality preserved. Python bindings structure is already correct.

---

## Python Bindings Specific Recommendations

### 1. Use Dynamic Versioning

**Current:**
```toml
# pyproject.toml
[project]
version = "0.7.36"  # Manual sync required
```

**Recommended:**
```toml
# pyproject.toml
[project]
dynamic = ["version"]  # Maturin reads from Cargo.toml
```

**Benefits:**
- Single source of truth
- Automatic synchronization
- Less error-prone

### 2. Automate Python Publishing Sequence

**Current:** Manual coordination between Rust and Python publishing

**Recommended:** Custom script or CI workflow:
```bash
# 1. Publish Rust crates (workspace-aware)
cargo publish --workspace

# 2. Wait for crates.io availability
./scripts/wait_for_crates_io.sh rank-rerank 0.7.36

# 3. Publish Python packages (sequential, maturin limitation)
cd crates/rank-rerank/rank-rerank-python && maturin publish
cd crates/rank-fusion/rank-fusion-python && maturin publish
# ... etc
```

### 3. Validate Version Consistency

**Add to CI:**
```bash
# Check all Rust/Python version pairs match
for crate in crates/*/; do
  rust_ver=$(grep '^version' "$crate/Cargo.toml" | head -1)
  if [ -f "$crate"*-python/pyproject.toml ]; then
    py_ver=$(grep '^version' "$crate"*-python/pyproject.toml | head -1)
    # Validate match
  fi
done
```

### 4. Consider Maturin Workspace Support

**Current Limitation:** Maturin doesn't support workspace publishing

**Workaround:** 
- Use `cargo publish --workspace` for Rust
- Then iterate over Python packages manually
- Future: Watch for maturin workspace publishing feature

## References

- [Cargo Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- [Workspace Publishing (Cargo 1.90)](https://tweag.io/blog/2025-07-10-cargo-package-workspace/)
- [Semantic Versioning in Rust](https://doc.rust-lang.org/cargo/reference/semver.html)
- [Maturin Documentation](https://www.maturin.rs)
- [PyO3 Building and Distribution](https://pyo3.rs/latest/building-and-distribution.html)
- [Maturin Workspace Issues](https://github.com/PyO3/maturin/issues/291)
- [Tokio Workspace Structure](https://github.com/tokio-rs/tokio)
- [Serde Workspace Structure](https://github.com/serde-rs/serde)
