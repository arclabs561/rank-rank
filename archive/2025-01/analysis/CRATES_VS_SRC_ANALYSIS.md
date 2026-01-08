# crates/ vs src/ Analysis: Rust Monorepo Best Practices

## Current Structure

```
rank-rank/
├── Cargo.toml          # Virtual workspace (shared deps only)
├── crates/
│   ├── rank-retrieve/
│   │   ├── Cargo.toml  # Workspace: rank-retrieve + python
│   │   ├── rank-retrieve/
│   │   │   ├── Cargo.toml
│   │   │   └── src/    # ← Actual source code here
│   │   └── rank-retrieve-python/
│   │       ├── Cargo.toml
│   │       └── src/
│   ├── rank-fusion/
│   │   ├── Cargo.toml  # Workspace
│   │   ├── rank-fusion/
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   └── rank-fusion-python/
│   └── ...
```

## Rust Best Practices

### ✅ `crates/` is CORRECT for Monorepos

**Standard Pattern** (used by tokio, serde, clap):
```
monorepo/
├── Cargo.toml          # Workspace root
├── crates/              # ← Standard name for multi-crate repos
│   ├── crate1/
│   │   ├── Cargo.toml
│   │   └── src/
│   └── crate2/
│       ├── Cargo.toml
│       └── src/
```

**Why `crates/`?**
- ✅ Standard Rust convention (tokio, serde, clap all use this)
- ✅ Clear separation: each crate is a separate package
- ✅ Each crate has its own `src/` directory
- ✅ Works with `cargo publish -p crate-name`

### ❌ `src/` is for Single-Crate Projects

**Single-Crate Pattern**:
```
single-crate/
├── Cargo.toml
└── src/                 # ← Only for single-crate projects
    └── lib.rs
```

**Why NOT `src/` for monorepos?**
- ❌ `src/` implies a single package
- ❌ Can't have multiple `Cargo.toml` files in `src/`
- ❌ Breaks workspace structure
- ❌ Doesn't match Rust ecosystem patterns

## Your Current Structure Analysis

### ✅ Current Structure is CORRECT

**What you have:**
- `crates/` directory ✅ (standard for monorepos)
- Each crate has its own `src/` ✅ (required by Cargo)
- Each crate is a workspace ✅ (allows core + python bindings)

**This matches:**
- Tokio: `tokio/tokio/src/`, `tokio/tokio-util/src/`
- Serde: `serde/serde/src/`, `serde/serde_derive/src/`
- Clap: `clap/clap/src/`, `clap/clap_derive/src/`

### Potential Confusion: Nested Structure

**Current:**
```
crates/rank-retrieve/
├── Cargo.toml           # Workspace
├── rank-retrieve/       # Main crate
│   ├── Cargo.toml
│   └── src/
└── rank-retrieve-python/ # Python bindings
    ├── Cargo.toml
    └── src/
```

**Why nested?**
- Each `rank-*` directory is a **workspace** (core + python)
- This is correct! Workspaces can contain multiple crates
- The `rank-retrieve/` subdirectory is the actual crate

**Alternative (flatter, but less common):**
```
crates/
├── rank-retrieve/
│   ├── Cargo.toml
│   └── src/
└── rank-retrieve-python/
    ├── Cargo.toml
    └── src/
```

But this loses the workspace grouping (core + python together).

## Recommendation

### ✅ Keep `crates/` - It's Standard

**Your structure is correct:**
1. ✅ `crates/` is the standard name for Rust monorepos
2. ✅ Each crate has its own `src/` (required by Cargo)
3. ✅ Nested workspaces (core + python) are fine

**Don't change to `src/`:**
- ❌ `src/` is for single-crate projects
- ❌ Would break workspace structure
- ❌ Doesn't match Rust ecosystem patterns

## Examples from Rust Ecosystem

### Tokio
```
tokio/
├── Cargo.toml
├── tokio/
│   ├── Cargo.toml
│   └── src/
├── tokio-util/
│   ├── Cargo.toml
│   └── src/
└── tokio-stream/
    ├── Cargo.toml
    └── src/
```

### Serde
```
serde/
├── Cargo.toml
├── serde/
│   ├── Cargo.toml
│   └── src/
├── serde_derive/
│   ├── Cargo.toml
│   └── src/
└── serde_json/
    ├── Cargo.toml
    └── src/
```

**Pattern**: All use `crates/` or direct crate directories, NOT `src/` at root.

## Conclusion

✅ **Keep `crates/`** - Your structure matches Rust ecosystem best practices.

The `src/` directory belongs **inside each crate**, not at the monorepo root.

