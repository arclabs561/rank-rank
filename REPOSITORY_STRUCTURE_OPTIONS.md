# Repository Structure Options

## Current Structure Analysis

### Option A: Monorepo (rank-rank at top)

**Structure**:
```
rank-rank/                    (MAIN REPO - git repository)
├── .git/
├── rank-retrieve/            (subdirectory, no .git)
├── rank-fusion/              (subdirectory, no .git)
├── rank-rerank/              (subdirectory, no .git)
├── rank-soft/                (subdirectory, no .git)
├── rank-learn/               (subdirectory, no .git)
├── rank-eval/                (subdirectory, no .git)
├── rank-sparse/              (subdirectory, no .git)
├── rank-rank/                (helm tools)
├── scripts/                   (shared scripts)
└── README.md
```

**Pros**:
- ✅ Single repository to clone
- ✅ Atomic commits across all crates
- ✅ Unified CI/CD
- ✅ Easier cross-crate refactoring
- ✅ Single place for all documentation

**Cons**:
- ❌ Can't publish crates independently easily
- ❌ Larger repository size
- ❌ All-or-nothing cloning
- ❌ Harder to have different maintainers per crate

### Option B: Separate Repos (Current)

**Structure**:
```
rank-rank/                    (NO .git, just directory)
├── rank-retrieve/            (separate .git)
├── rank-fusion/              (separate .git)
├── rank-rerank/              (separate .git)
├── rank-soft/                (separate .git)
├── rank-learn/               (separate .git)
├── rank-eval/                (separate .git)
├── rank-sparse/              (separate .git)
├── rank-rank/                (helm tools, no .git)
└── README.md
```

**Pros**:
- ✅ Independent versioning
- ✅ Independent publishing
- ✅ Users can clone only what they need
- ✅ Different maintainers per repo
- ✅ Clear boundaries

**Cons**:
- ❌ No atomic commits across repos
- ❌ More complex CI/CD
- ❌ Harder cross-repo refactoring

## Recommendation: Hybrid Approach

### Option C: Monorepo with rank-rank at top (RECOMMENDED)

**Structure**:
```
rank-rank/                    (MAIN REPO - git repository)
├── .git/
├── crates/
│   ├── rank-retrieve/
│   ├── rank-fusion/
│   ├── rank-rerank/
│   ├── rank-soft/
│   ├── rank-learn/
│   ├── rank-eval/
│   └── rank-sparse/
├── rank-rank/                (helm tools)
├── scripts/                  (shared scripts)
├── Cargo.toml                (workspace root - optional)
└── README.md
```

**Benefits**:
- ✅ Single repository (easier to manage)
- ✅ Atomic commits across all crates
- ✅ Unified CI/CD
- ✅ Can still publish crates independently
- ✅ Helm pattern (rank-rank/) provides coordination
- ✅ Matches tokio/serde pattern more closely

**How Publishing Works**:
- Each crate in `crates/` can be published independently
- Use `cargo publish -p rank-retrieve` from workspace root
- Path dependencies work naturally
- Users still get independent crates on crates.io

## Comparison

| Aspect | Monorepo (rank-rank at top) | Separate Repos (Current) |
|--------|----------------------------|-------------------------|
| **Atomic Commits** | ✅ Yes | ❌ No |
| **Unified CI/CD** | ✅ Yes | ⚠️ Per-repo |
| **Independent Publishing** | ✅ Yes (cargo publish -p) | ✅ Yes |
| **Repository Size** | ❌ Larger | ✅ Smaller |
| **Cloning** | ❌ All-or-nothing | ✅ Pick what you need |
| **Cross-Crate Refactoring** | ✅ Easy | ⚠️ Requires coordination |
| **Maintenance** | ⚠️ Single repo | ✅ Can split teams |
| **Helm Pattern** | ✅ Natural | ✅ Works but separate |

## Recommendation

**Move to monorepo with rank-rank at top** because:

1. **Matches Rust Ecosystem**: Tokio, serde, clap all use monorepos
2. **Better Coordination**: Atomic commits, unified CI/CD
3. **Still Flexible**: Can publish independently
4. **Helm Pattern**: rank-rank/ naturally at top
5. **Easier Development**: Cross-crate changes are simpler

**Migration Path**:
1. Initialize git in rank-rank/ (if not already)
2. Move all rank-* repos to `crates/` subdirectory
3. Update path dependencies
4. Update CI/CD
5. Keep publishing independently

**Alternative**: Keep separate repos if:
- Release cycles are very different
- Different teams maintain different repos
- Want maximum flexibility

