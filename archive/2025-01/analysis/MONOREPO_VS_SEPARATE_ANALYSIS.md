# Monorepo vs. Separate Repos: Analysis for rank-* Collection

## Current State

**Observation**: 
- `_rank-rank/` directory is **NOT a git repository**
- Some rank-* directories **ARE separate git repos** (rank-eval, rank-fusion, rank-rerank, rank-soft)
- Some rank-* directories **are NOT git repos** (rank-learn, rank-retrieve, rank-sparse, rank-rank)

**Current Structure**:
```
_rank-rank/                    (Directory, NOT a git repo)
├── rank-eval/                (✅ Separate git repo)
├── rank-fusion/              (✅ Separate git repo)
├── rank-rerank/              (✅ Separate git repo)
├── rank-soft/                (✅ Separate git repo)
├── rank-learn/               (❌ No git repo yet)
├── rank-retrieve/            (❌ No git repo yet)
├── rank-sparse/              (❌ No git repo yet)
├── rank-rank/                (❌ No git repo - helm tools)
└── README.md
```

## Option A: Monorepo (rank-rank at top) ✅ RECOMMENDED

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
├── rank-rank/                (helm tools - in main repo)
├── scripts/                  (shared scripts)
├── Cargo.toml                (optional workspace root)
└── README.md
```

**Benefits**:
- ✅ **Atomic commits** across all crates
- ✅ **Unified CI/CD** (single repository)
- ✅ **Easier cross-crate refactoring**
- ✅ **Single place for all documentation**
- ✅ **Matches tokio/serde pattern** (monorepo)
- ✅ **Can still publish independently** (`cargo publish -p rank-retrieve`)
- ✅ **Helm pattern natural** (rank-rank/ at top)

**How It Works**:
- Single git repository
- Each crate in `crates/` subdirectory
- Each crate can be published independently
- Path dependencies work naturally
- Users still get independent crates on crates.io

**Migration**:
1. Initialize git in `_rank-rank/` → rename to `rank-rank/`
2. Move all rank-* repos to `crates/` subdirectory
3. Update path dependencies
4. Keep publishing independently

## Option B: Keep Separate Repos (Current)

**Structure**:
```
_rank-rank/                    (Directory, NOT a git repo)
├── rank-eval/                (✅ Separate git repo)
├── rank-fusion/              (✅ Separate git repo)
├── rank-rerank/              (✅ Separate git repo)
├── rank-soft/                (✅ Separate git repo)
├── rank-learn/               (❌ Need to create git repo)
├── rank-retrieve/            (❌ Need to create git repo)
├── rank-sparse/              (❌ Need to create git repo)
└── rank-rank/                (❌ No git - just tools)
```

**Benefits**:
- ✅ Independent versioning
- ✅ Independent publishing
- ✅ Users can clone only what they need
- ✅ Different maintainers per repo

**Drawbacks**:
- ❌ No atomic commits across repos
- ❌ More complex CI/CD (per-repo)
- ❌ Harder cross-repo refactoring
- ❌ Need to create git repos for new crates

## Recommendation: Monorepo (Option A)

**Why Monorepo is Better for rank-* Collection**:

1. **Tightly Coupled Pipeline**:
   - retrieve → rerank → fusion → eval
   - Changes often span multiple crates
   - Atomic commits are valuable

2. **Matches Rust Ecosystem**:
   - Tokio, serde, clap all use monorepos
   - Standard pattern for related crates
   - Still publish independently

3. **Helm Pattern Natural**:
   - `rank-rank/` at top makes sense
   - Central coordination point
   - Shared tools in main repo

4. **Easier Development**:
   - Cross-crate refactoring is simpler
   - Unified CI/CD
   - Single place for documentation

5. **Still Flexible**:
   - Can publish crates independently
   - Users get independent crates on crates.io
   - Version independently

## Implementation Plan

### Step 1: Initialize Monorepo

```bash
cd /Users/arc/Documents/dev/_rank-rank
git init
git add .
git commit -m "Initial monorepo structure"
```

### Step 2: Reorganize Structure

```bash
mkdir crates
mv rank-retrieve crates/
mv rank-fusion crates/
mv rank-rerank crates/
mv rank-soft crates/
mv rank-learn crates/
mv rank-eval crates/
mv rank-sparse crates/
# rank-rank/ stays at top (helm tools)
```

### Step 3: Update Dependencies

Update path dependencies in `Cargo.toml` files:
```toml
# Before
rank-sparse = { path = "../../rank-sparse/rank-sparse" }

# After
rank-sparse = { path = "../rank-sparse" }
```

### Step 4: Optional: Workspace Root

Create `Cargo.toml` at root (optional):
```toml
[workspace]
members = [
    "crates/rank-retrieve",
    "crates/rank-fusion",
    "crates/rank-rerank",
    # ... etc
]
resolver = "2"
```

### Step 5: Update CI/CD

- Single repository = unified CI/CD
- Can still test/publish individual crates
- Use `cargo publish -p rank-retrieve` for publishing

## Comparison

| Aspect | Monorepo (rank-rank) | Separate Repos |
|--------|---------------------|----------------|
| **Atomic Commits** | ✅ Yes | ❌ No |
| **Unified CI/CD** | ✅ Yes | ❌ Per-repo |
| **Cross-Crate Refactoring** | ✅ Easy | ⚠️ Hard |
| **Independent Publishing** | ✅ Yes | ✅ Yes |
| **Helm Pattern** | ✅ Natural | ⚠️ Separate |
| **Repository Size** | ⚠️ Larger | ✅ Smaller |
| **Cloning** | ⚠️ All-or-nothing | ✅ Pick what you need |

## Final Recommendation

**✅ Use Monorepo with rank-rank at top**

**Reasons**:
1. Matches Rust ecosystem (tokio, serde pattern)
2. Better for tightly coupled pipeline
3. Helm pattern natural at top
4. Still flexible (independent publishing)
5. Easier development and maintenance

**Action**: Initialize git in `_rank-rank/`, reorganize to monorepo structure.

