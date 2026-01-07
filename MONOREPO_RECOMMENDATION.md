# Monorepo Recommendation: rank-rank at Top

## Current Situation

**Observation**:
- `_rank-rank/` is **NOT a git repository** (just a directory)
- Some rank-* are **separate git repos** (rank-eval, rank-fusion, rank-rerank, rank-soft)
- Some rank-* are **NOT git repos** (rank-learn, rank-retrieve, rank-sparse, rank-rank)

**Question**: Should `rank-rank` be the top-level git repository containing all rank-* crates?

## Recommendation: ✅ YES - Make rank-rank a Monorepo

### Proposed Structure

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
├── rank-rank/                (helm tools - stays at top)
├── scripts/                  (shared scripts)
├── Cargo.toml                (optional workspace root)
└── README.md
```

### Why This Makes Sense

1. **Helm Pattern Natural**:
   - `rank-rank/` is the "helm" - central control
   - Makes sense as the main repository
   - All other crates are "ships" it coordinates

2. **Matches Rust Ecosystem**:
   - Tokio, serde, clap all use monorepos
   - Standard pattern for related crates
   - Still publish independently

3. **Better for Pipeline**:
   - retrieve → rerank → fusion → eval are tightly coupled
   - Atomic commits across pipeline stages
   - Easier cross-crate refactoring

4. **Unified Development**:
   - Single repository to clone
   - Unified CI/CD
   - Single place for documentation
   - Easier coordination

5. **Still Flexible**:
   - Can publish crates independently (`cargo publish -p rank-retrieve`)
   - Users get independent crates on crates.io
   - Version independently

### Migration Steps

1. **Initialize git in rank-rank**:
   ```bash
   cd /Users/arc/Documents/dev/_rank-rank
   git init
   git add .
   git commit -m "Initial monorepo structure"
   ```

2. **Reorganize to crates/**:
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

3. **Update path dependencies**:
   ```toml
   # Before
   rank-sparse = { path = "../../rank-sparse/rank-sparse" }
   
   # After
   rank-sparse = { path = "../rank-sparse" }
   ```

4. **Optional: Workspace root**:
   ```toml
   # Cargo.toml at root
   [workspace]
   members = [
       "crates/rank-retrieve",
       "crates/rank-fusion",
       # ... etc
   ]
   resolver = "2"
   ```

### Benefits

- ✅ Atomic commits across all crates
- ✅ Unified CI/CD
- ✅ Easier cross-crate refactoring
- ✅ Helm pattern natural (rank-rank at top)
- ✅ Still publish independently
- ✅ Matches Rust ecosystem patterns

### Publishing Strategy

**Development**:
- Path dependencies work naturally
- Single `Cargo.lock` for consistency

**Publishing**:
- `cargo publish -p rank-retrieve` from workspace root
- Each crate publishes independently
- Users get independent crates on crates.io

## Answer: YES - Make rank-rank the monorepo

**This is the right structure** for the rank-* collection:
- Helm pattern makes sense (rank-rank coordinates all)
- Matches Rust ecosystem (tokio, serde pattern)
- Better for tightly coupled pipeline
- Still flexible (independent publishing)

