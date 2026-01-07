# Rust Ecosystem Organization Research

## Current State Analysis

### Existing rank-* Repository Structure

**Observation**: The rank-* repositories are **separate git repositories**:
- `rank-fusion/` - separate `.git`
- `rank-rerank/` - separate `.git`
- `rank-soft/` - separate `.git`
- `rank-eval/` - separate `.git`

**Pattern**: Each repository uses a **Cargo workspace** with:
- Core Rust crate (e.g., `rank-fusion/`)
- Python bindings crate (e.g., `rank-fusion-python/`)
- Optional: fuzz, test-e2e-local, evals

**Dependencies**: Cross-repo dependencies use **path dependencies**:
```toml
rank-soft = { path = "../rank-soft", package = "rank-soft" }
rank-sparse = { path = "../../rank-sparse/rank-sparse" }
```

**Helm Pattern**: `rank-rank/` serves as a central control point for:
- Shared scripts
- Cursor rules templates
- Documentation
- Introspection tools

## Rust Ecosystem Best Practices

### 1. Workspace Organization (Cargo Workspaces)

**Best Practice**: Use Cargo workspaces for related crates developed together.

**Benefits**:
- ✅ Shared `target/` directory (faster builds, less disk space)
- ✅ Single `Cargo.lock` (consistent dependency versions)
- ✅ Unified testing (`cargo test` runs all crates)
- ✅ Atomic commits across related crates
- ✅ Easier refactoring across crate boundaries

**Structure**:
```
project/
├── Cargo.toml          # Workspace root
├── Cargo.lock
├── crate1/
│   ├── Cargo.toml
│   └── src/
├── crate2/
│   ├── Cargo.toml
│   └── src/
└── target/             # Shared build directory
```

### 2. Monorepo vs. Separate Repositories

**Monorepo (Single Repository)**:
- ✅ Atomic commits across crates
- ✅ Easier cross-crate refactoring
- ✅ Unified CI/CD
- ✅ Single place for documentation
- ✅ Easier dependency management
- ❌ Larger repository size
- ❌ All-or-nothing cloning
- ❌ Potential for tighter coupling

**Separate Repositories**:
- ✅ Independent versioning
- ✅ Independent release cycles
- ✅ Smaller, focused repositories
- ✅ Clearer boundaries
- ✅ Can be maintained by different teams
- ❌ Harder to coordinate changes
- ❌ More complex dependency management
- ❌ Cross-repo refactoring is harder

### 3. Real-World Examples

#### Tokio
- **Structure**: Monorepo with workspace
- **Organization**: Multiple crates in one repository
- **Publishing**: Individual crates published separately
- **Pattern**: `tokio/tokio`, `tokio/tokio-util`, `tokio/tokio-stream`, etc.

#### Serde
- **Structure**: Monorepo with workspace
- **Organization**: Core crate + derive macros + utilities
- **Publishing**: Individual crates published separately
- **Pattern**: `serde-rs/serde` contains multiple related crates

#### Clap
- **Structure**: Monorepo with workspace
- **Organization**: Core + derive macros + utilities
- **Publishing**: Individual crates published separately
- **Pattern**: `clap-rs/clap` contains multiple related crates

**Key Insight**: Major Rust projects use **monorepos with workspaces** for related crates, but publish them **independently**.

### 4. Publishing Strategy

**Independent Publishing**:
- Each crate has its own version number
- Can publish only changed crates
- Users can depend on specific versions
- Example: `tokio = "1.0"`, `tokio-util = "1.0"` (can be different versions)

**Coordinated Publishing**:
- All crates versioned together
- Simpler for users (all crates compatible)
- Example: `serde = "1.0"`, `serde_derive = "1.0"` (same version)

### 5. Dependency Management Patterns

**Path Dependencies (Development)**:
```toml
[dependencies]
rank-soft = { path = "../rank-soft" }
```
- Used during development
- Allows local changes
- Not published to crates.io

**Published Dependencies (Production)**:
```toml
[dependencies]
rank-soft = "0.1.0"
```
- Used when publishing
- Points to crates.io version
- Versioned independently

**Hybrid Approach**:
```toml
[dependencies]
rank-soft = { path = "../rank-soft", package = "rank-soft" }
# When publishing, change to:
# rank-soft = "0.1.0"
```

## Recommendations for rank-* Collection

### Current Architecture Assessment

**Strengths**:
- ✅ Clear separation of concerns
- ✅ Independent versioning possible
- ✅ Each repo can be maintained independently
- ✅ Helm pattern (rank-rank) provides central coordination

**Weaknesses**:
- ⚠️ Cross-repo dependencies require path dependencies
- ⚠️ Harder to coordinate changes across repos
- ⚠️ More complex CI/CD setup
- ⚠️ Cross-repo refactoring is difficult

### Recommended Approach: Hybrid

**Option 1: Keep Separate Repos (Current)**
- **Best for**: Independent release cycles, different maintainers
- **Works well when**: Crates are loosely coupled
- **Requires**: Good coordination, clear interfaces

**Option 2: Monorepo with Workspaces**
- **Best for**: Tightly coupled crates, coordinated releases
- **Works well when**: Crates change together frequently
- **Requires**: Single repository, unified CI/CD

**Option 3: Hybrid (Recommended)**
- **Core pipeline crates**: Monorepo (retrieve, rerank, fusion, eval)
- **Supporting crates**: Separate repos (sparse, soft, learn)
- **Rationale**: Pipeline crates are tightly coupled, supporting crates are independent

### Specific Recommendations for rank-retrieve and rank-learn

#### rank-retrieve

**Current**: Separate repository with workspace
**Recommendation**: **Keep separate** OR **Move to monorepo with pipeline crates**

**Reasoning**:
- Tightly coupled with rank-rerank (pipeline stage 1 → stage 2)
- Often changed together
- Benefits from atomic commits
- But: Can be independent if release cycles differ

**If keeping separate**:
- Use path dependencies for development
- Publish independently
- Coordinate releases with rank-rerank

**If moving to monorepo**:
- Group with: rank-retrieve, rank-rerank, rank-fusion, rank-eval
- Keep: rank-soft, rank-learn, rank-sparse separate (different concerns)

#### rank-learn

**Current**: Separate repository with workspace
**Recommendation**: **Keep separate**

**Reasoning**:
- Different concern (training vs. inference)
- Different release cycle (ML frameworks change independently)
- Can be used independently
- Depends on rank-soft but not tightly coupled to pipeline

**Structure**:
- Keep as separate repository
- Use path dependency for rank-soft during development
- Publish independently
- Clear interface with rank-soft

## Best Practices Summary

### For rank-retrieve

1. **Workspace Structure** (✅ Current):
   ```toml
   [workspace]
   members = ["rank-retrieve", "rank-retrieve-python"]
   ```

2. **Dependencies**:
   ```toml
   # Development
   rank-sparse = { path = "../../rank-sparse/rank-sparse" }
   
   # When publishing, users will use:
   # rank-sparse = "0.1.0"
   ```

3. **Publishing**:
   - Publish independently
   - Coordinate with rank-rerank for pipeline compatibility
   - Version independently

### For rank-learn

1. **Workspace Structure** (✅ Current):
   ```toml
   [workspace]
   members = ["rank-learn", "rank-learn-python"]
   ```

2. **Dependencies**:
   ```toml
   # Development
   rank-soft = { path = "../rank-soft", package = "rank-soft" }
   
   # When publishing, users will use:
   # rank-soft = "0.1.0"
   ```

3. **Publishing**:
   - Publish independently
   - Version independently from rank-soft
   - Clear semver for breaking changes

## Conclusion

**Current structure is good** for:
- Independent versioning
- Clear boundaries
- Flexible maintenance

**Consider monorepo** if:
- Pipeline crates (retrieve, rerank, fusion, eval) change together frequently
- Want atomic commits across pipeline
- Unified CI/CD is important

**Keep separate** if:
- Release cycles differ significantly
- Different maintainers
- Want maximum flexibility

**Recommendation**: **Keep current structure** (separate repos) but:
1. Improve coordination (shared CI/CD, release notes)
2. Use path dependencies for development
3. Publish independently
4. Consider grouping pipeline crates in future if coordination becomes difficult

