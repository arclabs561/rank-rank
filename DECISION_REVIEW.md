# Decision Review: Crate Organization & Naming

## Decisions to Review

1. **rank-refine → rank-rerank**: Is this the right rename?
2. **rank-relax → rank-soft**: Is this the right rename?
3. **rank-learn for LTR**: Is this the right separation?
4. **rank-retrieve**: Is this the right scope?

## Analysis

### 1. rank-refine → rank-rerank

**Evidence for rename:**
- ✅ "Rerank" is standard IR term (used in papers, libraries, industry)
- ✅ Pipeline docs already say "rerank" not "refine"
- ✅ More discoverable (people search for "rerank", not "refine")
- ✅ Matches industry standard (Elasticsearch, OpenSearch use "rerank")

**Evidence against:**
- ⚠️ Breaking change (published to crates.io/PyPI)
- ⚠️ "Refine" could mean "improve quality" which is accurate
- ⚠️ Already established name

**Verdict: ✅ RENAME** - "Rerank" is the standard term, worth the breaking change for clarity.

### 2. rank-relax → rank-soft

**Evidence for rename:**
- ✅ "Soft ranking" is common in papers
- ✅ More descriptive (soft = smooth approximation)
- ✅ Clearer purpose

**Evidence against:**
- ⚠️ Breaking change
- ⚠️ "Relax" is also used in papers (relaxation methods)
- ⚠️ Less critical than refine→rerank

**Verdict: ✅ RENAME** - "Soft" is clearer and more common, worth the breaking change.

### 3. rank-learn for LTR

**Evidence for separate crate:**
- ✅ Different concerns: differentiable ops vs full ML systems
- ✅ Different dependencies: lightweight math vs heavy XGBoost/LightGBM
- ✅ Different users: custom neural models vs standard LTR
- ✅ Industry pattern: Libraries separate these (allRank separates LTR from ops)

**Evidence against:**
- ⚠️ Some overlap (ListNet/ListMLE are in rank-soft but used in LTR)
- ⚠️ Another crate to maintain

**Verdict: ✅ SEPARATE CRATE** - Clear separation of concerns, matches industry patterns.

### 4. rank-retrieve scope

**Evidence for current scope:**
- ✅ Matches pipeline stage 1
- ✅ Clear boundary (retrieval, not reranking)
- ✅ Can delegate to existing libraries (tantivy, hnsw)

**Evidence against:**
- ⚠️ Some overlap with rank-rerank (both do scoring)
- ⚠️ Might be too narrow (just retrieval, no indexing?)

**Verdict: ✅ CORRECT SCOPE** - Focus on retrieval is right, indexing can be added later.

## Impact Assessment

### Breaking Changes

**rank-refine → rank-rerank:**
- Crate name: `rank-refine` → `rank-rerank` (crates.io)
- PyPI name: `rank-refine` → `rank-rerank`
- GitHub repo: `rank-refine` → `rank-rerank`
- All internal references need updating

**rank-relax → rank-soft:**
- Crate name: `rank-relax` → `rank-soft` (crates.io)
- PyPI name: `rank-relax` → `rank-soft`
- GitHub repo: `rank-relax` → `rank-soft`
- All internal references need updating

### Migration Strategy

**Option A: New packages (Recommended)**
- Publish new packages (rank-rerank, rank-soft)
- Keep old packages with deprecation notice
- Users migrate over time

**Option B: Replace packages**
- Unpublish old, publish new (breaks existing users)
- Not recommended for published packages

**Option C: Alias packages**
- Keep old names as thin wrappers
- Redirect to new names
- More maintenance

**Recommendation: Option A** - Publish new packages, deprecate old ones.

## Final Verdict

✅ **All decisions are sound:**
1. rank-refine → rank-rerank: ✅ Correct (standard term)
2. rank-relax → rank-soft: ✅ Correct (clearer)
3. rank-learn for LTR: ✅ Correct (proper separation)
4. rank-retrieve scope: ✅ Correct (matches pipeline)

**Proceed with renames and creation.**

