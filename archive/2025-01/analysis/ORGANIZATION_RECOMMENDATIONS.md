# Organization Recommendations for rank-* Collection

## Executive Summary

**Current Structure**: ✅ **Good** - Separate repositories with workspaces

**Recommendation**: **Keep separate repos** but improve coordination

**Rationale**: 
- Independent versioning and release cycles
- Clear boundaries between concerns
- Flexible maintenance
- Follows Rust ecosystem patterns (like tokio, serde)

## Detailed Analysis

### Current Architecture

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

**Pattern**: Each repo is:
- Independent git repository
- Cargo workspace (core + python bindings)
- Uses path dependencies for cross-repo deps
- Can be published independently

### Rust Ecosystem Patterns

**Major Projects Use**:
- **Monorepo + Workspace**: tokio, serde, clap
- **Publish Independently**: Each crate has own version
- **Path Dependencies**: For development, published deps for users

**Why They Use Monorepos**:
- Atomic commits across related crates
- Easier cross-crate refactoring
- Unified CI/CD
- Coordinated releases

**Why rank-* Uses Separate Repos**:
- Different release cycles
- Different concerns (retrieval vs. training vs. evaluation)
- Independent maintenance possible
- Clear boundaries

### Recommendations

#### 1. Keep Separate Repositories ✅

**Reasoning**:
- ✅ Independent versioning (rank-retrieve v0.1, rank-rerank v0.7)
- ✅ Independent release cycles
- ✅ Clear boundaries (retrieval ≠ training ≠ evaluation)
- ✅ Can be maintained by different teams
- ✅ Follows current successful pattern

**When to Consider Monorepo**:
- If pipeline crates (retrieve, rerank, fusion, eval) change together frequently
- If atomic commits across pipeline become critical
- If unified CI/CD becomes important

#### 2. Improve Cross-Repo Coordination

**Current**: Path dependencies work but require coordination

**Improvements**:
1. **Shared CI/CD**: Use rank-rank for unified workflows
2. **Release Coordination**: Document compatibility matrix
3. **Dependency Management**: Clear versioning strategy
4. **Documentation**: Cross-repo integration guides

#### 3. Dependency Management Strategy

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

**Best Practice**:
- Use path dependencies during development
- Publish independently
- Document compatibility in READMEs
- Use semver for breaking changes

#### 4. Workspace Structure (Already Good)

**Current Pattern** (✅ Correct):
```toml
[workspace]
members = ["rank-retrieve", "rank-retrieve-python"]
resolver = "2"
default-members = ["rank-retrieve"]

[workspace.package]
version = "0.1.0"
edition = "2021"
rust-version = "1.74"
license = "MIT OR Apache-2.0"
authors = ["Arc <attobop@gmail.com>"]

[workspace.dependencies]
# Shared dependencies
rank-sparse = { path = "../../rank-sparse/rank-sparse" }
```

**Why This Works**:
- ✅ Core crate + Python bindings in same workspace
- ✅ Shared dependencies via workspace.dependencies
- ✅ Single Cargo.lock for consistency
- ✅ Can build/test together

#### 5. Publishing Strategy

**Independent Publishing** (Recommended):
- Each crate publishes independently
- Users can pick versions
- Example: `rank-retrieve = "0.1.0"`, `rank-rerank = "0.7.36"`

**Versioning Strategy**:
- **Major.Minor.Patch** (semver)
- **Breaking changes**: Major version bump
- **New features**: Minor version bump
- **Bug fixes**: Patch version bump

**Compatibility Matrix**:
Document which versions work together:
```
rank-retrieve 0.1.x  →  rank-sparse 0.1.x
rank-learn 0.1.x     →  rank-soft 0.1.x
```

## Specific Recommendations

### For rank-retrieve

**Structure**: ✅ Keep current (separate repo, workspace)

**Dependencies**:
```toml
# Development
rank-sparse = { workspace = true }

# When publishing, users use:
# rank-sparse = "0.1.0"
```

**Publishing**:
- Publish independently
- Coordinate with rank-rerank for pipeline compatibility
- Document compatibility in README

**Integration**:
- Clear interface with rank-rerank (1000 candidates → rerank)
- Document expected input/output formats
- Provide examples showing integration

### For rank-learn

**Structure**: ✅ Keep current (separate repo, workspace)

**Dependencies**:
```toml
# Development
rank-soft = { workspace = true }

# When publishing, users use:
# rank-soft = "0.1.0"
```

**Publishing**:
- Publish independently
- Version independently from rank-soft
- Document rank-soft version requirements

**Integration**:
- Clear interface with rank-soft (uses differentiable operations)
- Document which rank-soft features are required
- Provide examples showing training workflows

## Comparison: Monorepo vs. Separate Repos

| Aspect | Monorepo | Separate Repos (Current) |
|--------|----------|-------------------------|
| **Atomic Commits** | ✅ Yes | ❌ No |
| **Cross-Crate Refactoring** | ✅ Easy | ⚠️ Requires coordination |
| **Independent Versioning** | ⚠️ Possible but harder | ✅ Natural |
| **Independent Releases** | ⚠️ Possible but harder | ✅ Natural |
| **CI/CD Complexity** | ✅ Unified | ⚠️ Per-repo |
| **Repository Size** | ❌ Larger | ✅ Smaller |
| **Cloning** | ❌ All-or-nothing | ✅ Pick what you need |
| **Maintenance** | ⚠️ Single repo | ✅ Can split teams |
| **Dependency Management** | ✅ Easier | ⚠️ Path deps needed |

## Final Recommendation

**Keep Separate Repositories** because:

1. **Different Concerns**: 
   - Retrieval (rank-retrieve) ≠ Training (rank-learn) ≠ Evaluation (rank-eval)
   - Different release cycles make sense

2. **Flexibility**:
   - Users can depend on only what they need
   - Independent versioning
   - Can be maintained by different teams

3. **Rust Ecosystem Pattern**:
   - Many successful projects use separate repos
   - Works well with path dependencies for development
   - Publishing independently is standard

4. **Current Structure Works**:
   - Already established pattern
   - Workspaces handle internal organization
   - Path dependencies work for development

**Improvements to Make**:

1. **Better Coordination**:
   - Use rank-rank for shared CI/CD
   - Document compatibility matrix
   - Coordinate releases when needed

2. **Clear Interfaces**:
   - Document expected formats between crates
   - Provide integration examples
   - Version compatibility docs

3. **Publishing Strategy**:
   - Publish independently
   - Use semver
   - Document dependencies clearly

## Conclusion

**Current structure is good** ✅

**Keep separate repos** but:
- Improve coordination via rank-rank
- Document compatibility
- Use path deps for dev, published deps for users
- Publish independently with clear versioning

**Consider monorepo** only if:
- Pipeline crates change together very frequently
- Atomic commits become critical
- Unified CI/CD becomes important

The separate repository approach aligns with Rust ecosystem best practices and provides the flexibility needed for independent development and versioning.

