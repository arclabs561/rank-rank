# Rank-* Collection Organization Research

## Executive Summary

**Current Structure**: ✅ **Excellent** - Separate repositories with Cargo workspaces

**Recommendation**: **Keep current structure** - it aligns with Rust ecosystem best practices

**Key Finding**: Your current organization (separate repos + workspaces) matches patterns used by successful Rust projects, with the flexibility to publish independently.

## Research Findings

### 1. Rust Ecosystem Patterns

#### Major Projects Use Monorepos + Workspaces

**Tokio** (from CONTRIBUTING.md):
- **Structure**: Monorepo with multiple crates
- **Organization**: `tokio/`, `tokio-util/`, `tokio-stream/`, etc. in one repo
- **Publishing**: Independent publishing (each crate has own version)
- **Key Insight**: "When releasing a new version of a crate, ensure no path dependencies"

**Serde**:
- **Structure**: Monorepo with workspace
- **Organization**: `serde/`, `serde_derive/`, `serde_json/` in one repo
- **Publishing**: Independent publishing

**Clap**:
- **Structure**: Monorepo with workspace
- **Organization**: Core + derive macros + utilities
- **Publishing**: Independent publishing

**Pattern**: Monorepo for development, independent publishing for distribution

#### Why They Use Monorepos

1. **Atomic Commits**: Change multiple related crates in one commit
2. **Easier Refactoring**: Cross-crate changes are simpler
3. **Unified CI/CD**: Single repository, unified workflows
4. **Dependency Management**: Path dependencies work naturally

### 2. Your Current Structure Analysis

**Current Pattern**:
```
rank-retrieve/     (NEW) - Separate repo, workspace
rank-fusion/       (EXISTING) - Separate repo, workspace  
rank-rerank/       (EXISTING) - Separate repo, workspace
rank-soft/         (EXISTING) - Separate repo, workspace
rank-learn/        (NEW) - Separate repo, workspace
rank-eval/         (EXISTING) - Separate repo, workspace
rank-sparse/       (EXISTING) - Separate repo, workspace
rank-rank/         (HELM) - Central coordination
```

**Each Repository**:
- ✅ Independent git repository
- ✅ Cargo workspace (core crate + python bindings)
- ✅ Uses path dependencies for cross-repo deps
- ✅ Can be published independently

**Helm Pattern** (`rank-rank/`):
- Central coordination point
- Shared scripts and tools
- Cursor rules templates
- Introspection utilities

### 3. Comparison: Monorepo vs. Separate Repos

| Aspect | Monorepo (Tokio/Serde) | Separate Repos (Your Current) |
|--------|------------------------|------------------------------|
| **Atomic Commits** | ✅ Yes | ❌ No (requires coordination) |
| **Cross-Crate Refactoring** | ✅ Easy | ⚠️ Requires coordination |
| **Independent Versioning** | ⚠️ Possible but harder | ✅ Natural |
| **Independent Releases** | ⚠️ Possible but harder | ✅ Natural |
| **CI/CD** | ✅ Unified | ⚠️ Per-repo (but rank-rank helps) |
| **Repository Size** | ❌ Larger | ✅ Smaller, focused |
| **Cloning** | ❌ All-or-nothing | ✅ Pick what you need |
| **Maintenance** | ⚠️ Single repo | ✅ Can split teams |
| **Dependency Management** | ✅ Path deps natural | ⚠️ Path deps work but need coordination |
| **Publishing** | ✅ Independent | ✅ Independent |
| **User Flexibility** | ⚠️ Must clone all | ✅ Use only what needed |

### 4. Why Your Structure Works Well

**Advantages of Separate Repos**:

1. **Clear Boundaries**:
   - Retrieval ≠ Training ≠ Evaluation
   - Each repo has focused purpose
   - Easier to understand scope

2. **Independent Versioning**:
   - `rank-retrieve v0.1.0` can work with `rank-rerank v0.7.36`
   - Users can pick versions
   - Breaking changes isolated

3. **Flexible Maintenance**:
   - Different teams can maintain different repos
   - Release cycles can differ
   - Less coordination overhead

4. **User Experience**:
   - Users only clone what they need
   - Smaller repositories
   - Clearer dependencies

5. **Helm Pattern**:
   - `rank-rank/` provides coordination
   - Shared tools and scripts
   - Introspection capabilities

**Your Structure Matches**:
- ✅ Workspace pattern (like tokio/serde)
- ✅ Independent publishing (like tokio/serde)
- ✅ Path dependencies for development (standard)
- ✅ Clear boundaries (better than monorepo for your use case)

### 5. Specific Recommendations

#### For rank-retrieve

**Current Structure** ✅:
```toml
[workspace]
members = ["rank-retrieve", "rank-retrieve-python"]
resolver = "2"
default-members = ["rank-retrieve"]

[workspace.dependencies]
rank-sparse = { path = "../rank-sparse/rank-sparse" }
```

**Why This Works**:
- ✅ Core crate + Python bindings in same workspace
- ✅ Path dependency for development
- ✅ Can publish independently
- ✅ Users will use: `rank-sparse = "0.1.0"`

**Recommendation**: **Keep as-is** ✅

#### For rank-learn

**Current Structure** ✅:
```toml
[workspace]
members = ["rank-learn", "rank-learn-python"]
resolver = "2"
default-members = ["rank-learn"]

[workspace.dependencies]
rank-soft = { path = "../rank-soft", package = "rank-soft" }
```

**Why This Works**:
- ✅ Core crate + Python bindings in same workspace
- ✅ Path dependency for development
- ✅ Can publish independently
- ✅ Users will use: `rank-soft = "0.1.0"`

**Recommendation**: **Keep as-is** ✅

### 6. Publishing Strategy

**Development** (Current):
```toml
# rank-retrieve/Cargo.toml
rank-sparse = { path = "../../rank-sparse/rank-sparse" }

# rank-learn/Cargo.toml
rank-soft = { path = "../rank-soft", package = "rank-soft" }
```

**Publishing** (Users):
```toml
# Users will use published versions
rank-sparse = "0.1.0"
rank-soft = "0.1.0"
```

**Best Practice** (from Tokio):
1. **Before publishing**: Ensure no path dependencies
2. **Publish dependencies first**: Depth-first traversal
3. **Then publish dependents**: After dependencies are published

**Your Workflow**:
1. Publish `rank-sparse` first
2. Update `rank-retrieve` to use published version
3. Publish `rank-retrieve`
4. Similar for `rank-soft` → `rank-learn`

### 7. When to Consider Monorepo

**Consider Monorepo If**:
- Pipeline crates (retrieve, rerank, fusion, eval) change together frequently
- Atomic commits across pipeline become critical
- Unified CI/CD becomes important
- Cross-crate refactoring becomes difficult

**Keep Separate If** (Your Current Situation):
- ✅ Release cycles differ (retrieval vs. training vs. evaluation)
- ✅ Different concerns (clear boundaries)
- ✅ Want maximum flexibility
- ✅ Users benefit from picking only what they need

### 8. Improvements to Make

**1. Better Coordination** (via rank-rank):
- ✅ Shared CI/CD workflows (already have scripts)
- ✅ Compatibility matrix documentation
- ✅ Coordinated release notes

**2. Dependency Management**:
- ✅ Use path dependencies for development (current)
- ✅ Document published versions in READMEs
- ✅ Use semver for breaking changes

**3. Documentation**:
- ✅ Cross-repo integration examples (add more)
- ✅ Compatibility documentation
- ✅ Version requirements in READMEs

## Conclusion

**Your current structure is excellent** ✅

**Why**:
1. **Matches Rust ecosystem patterns**: Workspaces + independent publishing
2. **Better than monorepo for your use case**: Different concerns, release cycles
3. **Flexible**: Users can pick what they need
4. **Maintainable**: Clear boundaries, independent versioning
5. **Helm pattern**: `rank-rank/` provides coordination

**Recommendation**: **Keep separate repositories** but:
- ✅ Continue using workspaces (current pattern is correct)
- ✅ Use path dependencies for development (current pattern is correct)
- ✅ Publish independently (standard practice)
- ✅ Improve coordination via rank-rank (already have infrastructure)
- ✅ Document compatibility (add to READMEs)

**Your structure is production-ready and follows Rust ecosystem best practices!** 🎉

