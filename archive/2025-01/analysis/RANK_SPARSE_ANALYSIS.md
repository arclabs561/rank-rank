# rank-sparse Analysis: Should It Exist?

## Question 1: Should `crates/rank-sparse` → `src/`?

**Answer: NO** - This doesn't make sense for a monorepo structure.

### Arguments Against Moving to Root `src/`:

1. **Monorepo Structure**: Each `rank-*` is a separate crate with its own:
   - `Cargo.toml` (package definition)
   - Versioning (independent semver)
   - Publishing (can publish separately to crates.io)
   - Dependencies (can have different dependency sets)

2. **Workspace Pattern**: The monorepo uses workspaces where:
   - Root `Cargo.toml` is a virtual manifest (no package)
   - Each `crates/rank-*/` is a workspace member
   - Moving to root `src/` would break this pattern

3. **Consistency**: All other crates follow `crates/rank-*/` pattern:
   - `crates/rank-eval/`
   - `crates/rank-fusion/`
   - `crates/rank-retrieve/`
   - `crates/rank-soft/`
   - Breaking this would be inconsistent

4. **Dependency Management**: Root `src/` would mean:
   - No separate `Cargo.toml` for rank-sparse
   - Can't publish independently
   - Can't version independently
   - Can't have separate dependencies

**Conclusion**: Keep in `crates/rank-sparse/` to maintain monorepo structure.

---

## Question 2: Is `rank-sparse` Worth a Separate Crate?

**Answer: PROBABLY NOT** - It should be merged into `rank-retrieve`.

### Arguments Against Keeping `rank-sparse` Separate:

#### 1. **Size and Complexity**
- **rank-sparse**: 98 lines (just `SparseVector` and `dot_product`)
- **rank-retrieve sparse module**: 128 lines (uses rank-sparse)
- **Total**: 226 lines - small enough to be a single module

#### 2. **Usage Pattern**
- **Only used by**: `rank-retrieve` (10 files reference it, all in rank-retrieve)
- **Not used by**: Any other `rank-*` crate
- **Single consumer**: Classic case for merging into consumer

#### 3. **Rust Best Practices**
According to research:
- **Modules**: For utilities used only internally by one crate
- **Separate crates**: For code shared across multiple crates/apps
- **rank-sparse**: Only used by `rank-retrieve` → should be a module

#### 4. **Overhead of Separate Crate**
- Extra `Cargo.toml` to maintain
- Extra workspace member
- Extra dependency resolution
- Extra Python bindings crate
- Extra documentation
- No benefit (not shared, not independently versioned)

#### 5. **Pipeline Logic**
- `rank-sparse` is specifically for **sparse retrieval**
- Sparse retrieval is part of `rank-retrieve`'s domain
- It's not a general-purpose utility used across the pipeline
- It's tightly coupled to retrieval operations

#### 6. **Comparison to Other Crates**
- `rank-eval`: Used by multiple crates (fusion, rerank, etc.) → **Worth separate**
- `rank-fusion`: Core algorithm used across pipeline → **Worth separate**
- `rank-soft`: Used by `rank-learn` and external users → **Worth separate**
- `rank-sparse`: Only used by `rank-retrieve` → **NOT worth separate**

### Recommendation: Merge into `rank-retrieve`

**Structure After Merge:**
```
crates/rank-retrieve/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── bm25.rs
│   ├── dense.rs
│   ├── sparse.rs          # Sparse retrieval (uses sparse_vector module)
│   └── sparse_vector.rs  # SparseVector + dot_product (merged from rank-sparse)
└── rank-retrieve-python/
```

**Benefits:**
1. ✅ Simpler structure (one less crate)
2. ✅ No cross-crate dependency overhead
3. ✅ Better encapsulation (sparse vectors are retrieval-specific)
4. ✅ Easier to maintain (one place for sparse retrieval code)
5. ✅ Matches Rust best practices (module for single-consumer utilities)

**Migration Steps:**
1. Move `rank-sparse/src/lib.rs` → `rank-retrieve/src/sparse_vector.rs`
2. Update `rank-retrieve/src/sparse.rs` to use local module
3. Remove `rank-sparse` directory
4. Update root `Cargo.toml` (remove rank-sparse)
5. Update `rank-retrieve/Cargo.toml` (remove rank-sparse dependency)

---

## Final Recommendation

1. **Keep structure**: `crates/rank-sparse/` → `crates/rank-retrieve/src/sparse_vector.rs` (merge, don't move to root)
2. **Merge into rank-retrieve**: `rank-sparse` is too small and too single-purpose to warrant a separate crate

