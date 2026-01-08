# Structure Refinement Plan

## Research Findings

Based on deep research of Rust monorepo best practices (tokio, serde, rust-analyzer):

### Key Finding: **Flat Structure is Superior**

**Evidence:**
- rust-analyzer: 200K lines, 32 crates, all flat in `crates/`
- Tokio, Serde, Clap: All use flat structures
- Flat structures avoid "ossification" problem of nested hierarchies
- Easier to discover (`ls crates/` shows everything)
- No mental model mismatch (directory structure = crate namespace)

### Current Structure Analysis

**Inconsistent patterns:**

1. **rank-eval**: ✅ Flat (top-level `src/`)
   ```
   crates/rank-eval/
   ├── Cargo.toml (workspace + package)
   ├── src/lib.rs
   └── rank-eval-python/
   ```

2. **rank-soft**: ✅ Flat (top-level `src/`)
   ```
   crates/rank-soft/
   ├── Cargo.toml (workspace + package)
   ├── src/lib.rs
   └── rank-soft-python/
   ```

3. **rank-retrieve**: ❌ Nested (double nesting)
   ```
   crates/rank-retrieve/
   ├── Cargo.toml (workspace)
   ├── rank-retrieve/
   │   ├── Cargo.toml (package)
   │   └── src/lib.rs
   └── rank-retrieve-python/
   ```

4. **rank-fusion**: ❌ Nested
   ```
   crates/rank-fusion/
   ├── Cargo.toml (workspace)
   ├── rank-fusion/
   │   ├── Cargo.toml (package)
   │   └── src/lib.rs
   └── rank-fusion-python/
   ```

5. **rank-rerank**: ❌ Nested (with different name)
   ```
   crates/rank-rerank/
   ├── Cargo.toml (workspace)
   ├── rank-rerank-core/
   │   ├── Cargo.toml (package)
   │   └── src/lib.rs
   └── rank-rerank-python/
   ```

6. **rank-learn**: ❌ Nested
   ```
   crates/rank-learn/
   ├── Cargo.toml (workspace)
   ├── rank-learn/
   │   ├── Cargo.toml (package)
   │   └── src/lib.rs
   └── rank-learn-python/
   ```

## Recommendation: Flatten All Crates

### Target Structure (Consistent)

All crates should follow the `rank-eval` / `rank-soft` pattern:

```
crates/rank-retrieve/
├── Cargo.toml          # Workspace + Package (combined)
├── src/lib.rs          # Top-level source
├── rank-retrieve-python/
└── README.md

crates/rank-fusion/
├── Cargo.toml          # Workspace + Package
├── src/lib.rs
├── rank-fusion-python/
└── README.md

crates/rank-rerank/
├── Cargo.toml          # Workspace + Package
├── src/lib.rs          # (rename from rank-rerank-core)
├── rank-rerank-python/
└── README.md

crates/rank-learn/
├── Cargo.toml          # Workspace + Package
├── src/lib.rs
├── rank-learn-python/
└── README.md
```

### Benefits

1. **Consistency**: All crates follow same pattern
2. **Simplicity**: One less directory level
3. **Matches best practices**: Flat structure (rust-analyzer pattern)
4. **Easier navigation**: `crates/rank-*/src/` is predictable
5. **No mental model mismatch**: Directory name = crate name

### Migration Steps

For each nested crate:

1. **Move source files**:
   ```bash
   mv crates/rank-retrieve/rank-retrieve/src/* crates/rank-retrieve/src/
   mv crates/rank-retrieve/rank-retrieve/Cargo.toml crates/rank-retrieve/Cargo.toml.new
   ```

2. **Merge Cargo.toml**:
   - Combine workspace and package definitions
   - Update paths in workspace members
   - Update path dependencies in other crates

3. **Update imports**:
   - No code changes needed (module paths unchanged)
   - Update documentation paths

4. **Remove nested directory**:
   ```bash
   rm -rf crates/rank-retrieve/rank-retrieve/
   ```

5. **Update root Cargo.toml**:
   - Path dependencies stay the same (`crates/rank-retrieve`)

### Special Case: rank-rerank

`rank-rerank-core` should become just `rank-rerank`:
- Move `rank-rerank-core/src/` → `rank-rerank/src/`
- Update package name in Cargo.toml
- Update all references

## Priority

**High Priority** (most nested, most confusing):
1. `rank-retrieve` (just created, easiest to fix)
2. `rank-learn` (just created, easiest to fix)

**Medium Priority**:
3. `rank-fusion` (well-established, but should be consistent)
4. `rank-rerank` (has `-core` suffix, needs renaming too)

## Verification

After flattening:
- ✅ All crates have `crates/rank-*/src/lib.rs`
- ✅ All crates have combined workspace+package Cargo.toml
- ✅ All path dependencies still work
- ✅ All tests pass
- ✅ All examples compile

