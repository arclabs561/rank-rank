# Plan

This is a "scratch" planning document that tracks constraints, architecture,
design and implementation details for the rank-* ecosystem. The primary use
cases driving everything are:

* Building Rust-native RAG pipelines without Python FFI overhead
* Hybrid search combining multiple retrieval methods (BM25, dense, sparse)
* Late interaction reranking for precision without cross-encoder latency
* Differentiable ranking operations for ML training
* IR evaluation metrics for benchmarking

Of course, I tried to keep generality in mind as well, as I've used IR systems
in many other contexts too. And tried to think about how others might use this
code, but didn't step too far outside my comfort zone.

## Table of Contents

* [Novelty](#novelty)
* [Things rank-* should NOT do](#things-rank--should-not-do)
* [Things rank-* should do](#things-rank--should-do)
* [Motivation and gap analysis](#motivation-and-gap-analysis)
* [README critique](#readme-critique)
* [Implementation constraints](#implementation-constraints)
* [Ecosystem integration](#ecosystem-integration)

## Novelty

Before diving into details, it should be noted that almost no technique
described in this ecosystem is novel. Maybe the _combination_ of these things is
novel, but it's hard to say. The rank-* crates make use of:

* SIMD-accelerated similarity scoring (common in vector search libraries)
* Rank fusion algorithms (RRF, ISR, CombMNZ - well-established in IR literature)
* Late interaction (ColBERT-style MaxSim - from research papers)
* Differentiable ranking (soft ranking, neural sort - from ML research)
* Learning to rank (LambdaRank, LambdaMART - standard LTR algorithms)

One thing that I did think was somewhat novel was providing all of these in a
unified Rust ecosystem with zero dependencies by default and seamless
integration between crates. But even that is debatable - Python frameworks like
LlamaIndex provide unified APIs, just not in Rust.

In general, this sort of information has been extremely difficult to find. In
doing research, I read lots of code, comments, papers and mailing list posts.
Frequently, folks would refer to "techniques" that have been used "since
forever," and yet, I found it difficult to find authoritative sources on them.
They are almost always buried in platform specific details, particular
interpretations of standards, or experiments on implementations.

## Things rank-* should NOT do

We really want to try to strike a balance between being simple enough that
rank-* is flexible and embeddable in other applications, but heavy enough that
it actually handles most of the complexity in many common use cases. Striking
this balance in an IR ecosystem is hard. Still, there are some things we can
rule out fairly easily:

* rank-* will not handle tokenization beyond simplistic approaches. rank-retrieve
  might provide some standard whitespace or ngram tokenizers, but it is
  otherwise up to the caller to plug in more sophisticated things like language
  aware stemmers or lemmatizers.
* rank-* will not have a notion of "fields" or any such thing. Every document
  is just a multi-set of terms. In this sense, there is no schema as there is
  only one field for every document.
* Similar to the above, rank-retrieve will strictly be an index. It will not store
  document content. (Though rank-rerank does store token-level embeddings for
  late interaction.)
* rank-retrieve will not provide persistent storage by default. It is in-memory
  only. For large scale, integrate with specialized libraries (Tantivy, HNSW,
  FAISS).
* rank-* will not be a full RAG framework. It provides retrieval, fusion,
  reranking and evaluation, but not document loading, chunking, LLM integration.
* rank-retrieve will not optimize individual retrieval methods for production
  scale. Use specialized crates (tantivy for BM25, hnsw_rs for dense ANN) for
  that. rank-retrieve focuses on unified API and hybrid search coordination.

These alone draw a fairly bright dividing line between rank-* and more full
fledged IR systems like Lucene or Python frameworks like LlamaIndex. rank-* is
built for simpler tasks, and we acknowledge that more complex tasks are better
solved by other tools.

## Things rank-* should do

We've established some things that rank-* won't do, but what are some things
that it should do?

* It should provide unified APIs across multiple retrieval methods (BM25, dense,
  sparse, generative) with consistent output format (`Vec<(u32, f32)>`) for
  easy integration.
* It should handle the complexity of rank fusion (RRF, ISR, CombMNZ) without
  score normalization, since different retrievers use incompatible score scales.
* It should provide SIMD-accelerated late interaction scoring (MaxSim) for
  precise reranking without cross-encoder latency.
* It should support differentiable ranking operations for ML training, enabling
  gradient flow through ranking operations.
* It should provide IR evaluation metrics (NDCG, MAP, MRR) with TREC format
  parsing.
* It should have zero dependencies by default, making it lightweight and
  embeddable.
* It should seamlessly integrate between crates - rank-retrieve → rank-fusion →
  rank-rerank → rank-eval should work without adapter code.

The other major design point that rank-* could have occupied---and I did
carefully consider---is a library for just a single stage (e.g., only fusion,
or only reranking). This is tempting because this design space only requires
focusing on one algorithm and the file format, and _not_ on ecosystem
integration. Why? Because only one crate needs to be maintained and there's no
coordination overhead.

While it's plausible I may build such standalone libraries, it's likely to end
up as separate crates anyway. The problem with only offering that is that it
isn't broadly useful. It really only applies to some specific niches, and in
order to get it to work in a more complete pipeline, you really need to wind
up building a lot of the infrastructure that rank-* could have done for you.
Finally, on top of all of that, RAG pipelines absolutely require building out
that infrastructure, so I must do it anyway. We might as well package it up and
make it generally useful.

Plus, ever since I started learning about information retrieval, I've always
wanted to build an IR ecosystem. But I wanted to do something different than what
Lucene has done. rank-* isn't _that_ different than Lucene, but I think it will
occupy enough of a niche for it to be successful. Time will tell.

## Motivation and gap analysis

The README currently states:

> Rust ecosystem lacks unified APIs for IR pipelines. Python frameworks provide
> unified retrieval, but Rust developers must manually compose multiple crates.
> These crates fill that gap: unified APIs, seamless integration, zero
> dependencies by default.

This is accurate but incomplete. Let me expand:

### Python Ecosystem

**LlamaIndex, Haystack, LangChain** provide unified retrieval APIs:
- Unified retriever interfaces that abstract BM25, dense, and sparse methods
- Hybrid search built-in (combining multiple retrieval methods)
- Vector store abstractions supporting 50+ backends
- Full RAG frameworks (not just retrieval)

**Limitations for Rust users:**
- Python-only (requires FFI/PyO3 for Rust integration)
- Heavy dependencies and runtime overhead
- Not suitable for high-performance, low-latency systems
- Full RAG frameworks (more than just retrieval)

### Rust Ecosystem

**Current state (2025):**

1. **Individual components exist:**
   - `tantivy`: Full-text search with BM25
   - `bm25` crate: BM25 sparse vector generation
   - `hnsw_rs`, `faiss` bindings: Dense ANN search
   - `qdrant-client`, `weaviate-client`: Vector database clients

2. **Missing:**
   - **No unified API** across BM25, dense, sparse
   - **No hybrid search coordination** (score fusion, normalization)
   - **No generative retrieval** implementations
   - **No ecosystem integration** (fusion, reranking, evaluation)

**Gap analysis:**
- Rust developers must manually compose multiple crates
- No standard patterns for hybrid search
- No unified interface for switching between methods
- Limited examples and documentation for retrieval pipelines

### Unique Value Proposition

1. **Unified API for Multiple Retrieval Methods**
   - Single interface for BM25, dense, sparse, and generative retrieval
   - Consistent API patterns across all methods
   - Easy switching between methods for experimentation
   - Competitive advantage: No equivalent in Rust ecosystem

2. **Ecosystem Integration**
   - Seamless integration between crates (retrieve → fuse → rerank → evaluate)
   - Consistent data formats across crates (`Vec<(u32, f32)>`)
   - No adapter code needed
   - Competitive advantage: Purpose-built for rank-* ecosystem

3. **Zero Dependencies by Default**
   - Lightweight and embeddable
   - Feature-gated implementations (users opt into specific methods)
   - Competitive advantage: Minimal overhead, maximum flexibility

4. **Generative Retrieval (LTRGR)**
   - Complete LTRGR implementation (~1000+ lines)
   - Unique in Rust ecosystem
   - Even Python frameworks don't provide LTRGR implementations

## README critique

The current README is minimal and accurate, but could be improved:

### What's good

* **Minimal and focused**: Doesn't overwhelm with details
* **Clear crate organization**: Pipeline stages vs Training is logical
* **All artifacts linked**: crates.io, docs.rs, PyPI, npm links present
* **Modern tooling**: Uses `uv pip install` instead of `pip install`
* **Motivation present**: Explains the gap being filled

### What's missing or could be better

1. **Concrete examples**: The README shows installation but no usage examples.
   A minimal example showing the pipeline (retrieve → fuse → rerank) would
   help users understand the value immediately.

2. **Limitations not explicit**: The README doesn't mention what rank-* doesn't
   do. Users might assume it's a full RAG framework or production-scale
   retrieval system. Should explicitly state:
   - In-memory only (no persistence)
   - Basic implementations (not optimized for large scale)
   - Not a full RAG framework

3. **"Zero dependencies" claim needs verification**: The README says "zero
   dependencies by default" but doesn't verify this. Let me check:
   - rank-fusion: `default = []` - verified zero deps
   - rank-retrieve: Has features but default might have deps
   - rank-rerank: Has features but default might have deps
   - Should verify and state explicitly which crates have zero deps

4. **Pipeline flow not obvious**: The README lists crates but doesn't show how
   they fit together. A simple diagram or example showing:
   ```
   10M docs → rank-retrieve → 1000 candidates
   1000 → rank-fusion → combined results
   1000 → rank-rerank → 100 results
   100 → rank-eval → metrics
   ```
   would help.

5. **Comparison to alternatives**: The README mentions Python frameworks but
   doesn't compare to Rust alternatives. Should mention:
   - When to use rank-* vs tantivy (BM25 only, needs persistence)
   - When to use rank-* vs Python frameworks (Rust-native, performance)
   - When NOT to use rank-* (need full RAG, very large scale)

6. **Training crates**: rank-soft provides both differentiable ranking operations and Learning to Rank algorithms (LambdaRank, Ranking SVM, neural LTR). Used for training ranking models, not part of the inference pipeline.

7. **WASM/npm only for some crates**: rank-fusion and rank-rerank have npm
   packages, but rank-retrieve doesn't. This is inconsistent and should be
   explained or fixed.

## Implementation constraints

### rank-retrieve

**What it optimizes for:**
- Concrete function API as primary interface (simple, direct)
- Unified output format (`Vec<(u32, f32)>`) for easy integration
- Ecosystem integration over standalone functionality
- Simplicity over feature completeness
- Prototyping/research over production scale
- Rust-native over cross-language compatibility
- Feature-gated implementations for lightweight usage

**What it does NOT optimize for:**
- Individual method performance (use specialized crates)
- Persistent storage (use `tantivy` or vector databases)
- Large-scale optimization (use specialized backends)
- Full RAG framework (use Python frameworks)
- Production-scale single methods (use optimized backends)

**Constraints:**
- In-memory indexes only (no persistence)
- Basic implementations (not optimized for large scale)
- Maximum 2^32 documents per segment (doc IDs are u32)
- No phrase queries (positions not indexed)

### rank-fusion

**What it optimizes for:**
- Zero dependencies by default
- Score-agnostic fusion (RRF doesn't need normalization)
- Multiple fusion algorithms (RRF, ISR, CombMNZ, Borda, DBSF)
- Simple API (just functions, no traits)

**Constraints:**
- Requires all input lists to have same doc ID format
- No score normalization (by design - RRF avoids this)
- Limited to list fusion (no learning-based fusion)

### rank-rerank

**What it optimizes for:**
- SIMD acceleration (hand-written, no linear algebra crate)
- Late interaction (MaxSim, ColBERT-style)
- Performance target: 61ms per query for 100-1000 candidates
- Zero dependencies by default

**Constraints:**
- No cross-encoder implementation (disabled, waiting for ort 2.0)
- Limited to token-level similarity (no full transformer reranking)
- Requires pre-computed token embeddings

### rank-soft

**What it optimizes for:**
- Differentiable ranking operations
- Framework agnostic (works with PyTorch, JAX, Rust ML)
- Multiple algorithms (sigmoid-based, NeuralSort, SoftRank, SmoothI)

**Constraints:**
- Not for inference (only training)
- Requires regularization parameter tuning
- May not converge to exact discrete ranking

### rank-soft (includes LTR algorithms)

**What it optimizes for:**
- Differentiable ranking operations (soft ranking, sorting, loss functions)
- Learning to Rank algorithms (LambdaRank, Ranking SVM, neural LTR)
- Framework agnostic (works with PyTorch, JAX, Rust ML)

**Constraints:**
- Not for inference (only training)
- Requires regularization parameter tuning
- May not converge to exact discrete ranking
- Requires labeled training data for LTR algorithms

### rank-eval

**What it optimizes for:**
- Standard IR metrics (NDCG, MAP, MRR)
- TREC format parsing
- Zero dependencies

**Constraints:**
- Limited to offline evaluation (no online metrics)
- Requires relevance judgments

## Ecosystem integration

The README claims "seamless integration" but doesn't show it. Here's what that
actually means:

1. **Consistent data format**: All crates use `Vec<(u32, f32)>` for doc ID + score
2. **No adapter code**: Can chain retrieve → fuse → rerank → eval directly
3. **Shared types**: Doc IDs are u32 across all crates
4. **Feature coordination**: Optional features align between crates

**What "seamless" does NOT mean:**
- Automatic pipeline construction (you still write the code)
- Shared configuration (each crate has its own config)
- Coordinated versioning (crates version independently)

## Recommendations for README improvement

1. **Add a minimal example** showing the pipeline:
   ```rust
   // Retrieve
   let results = rank_retrieve::bm25(&index, &query, 1000)?;
   
   // Fuse with dense results
   let fused = rank_fusion::rrf(&[bm25_results, dense_results], 60)?;
   
   // Rerank
   let reranked = rank_rerank::maxsim(&query_tokens, &candidates, 100)?;
   
   // Evaluate
   let ndcg = rank_eval::ndcg(&reranked, &relevance, 10)?;
   ```

2. **Add a "When to use" section**:
   - Building Rust-native RAG pipelines
   - Need hybrid search (BM25 + dense + sparse)
   - Researching/experimenting with retrieval methods
   - Need generative retrieval (LTRGR)
   - Integrating with rank-* ecosystem

3. **Add a "When NOT to use" section**:
   - Only need one method (use specialized crate)
   - Need persistent storage (use `tantivy` or vector DB)
   - Very large scale (use specialized backends)
   - Need full RAG framework (use Python frameworks)

4. **Clarify "zero dependencies"**: State explicitly which crates have zero
   deps and which have optional features.

5. **Add pipeline diagram**: Show the flow from retrieval to evaluation.

6. **Explain training crate**: Clarify that rank-soft provides both differentiable ranking operations and LTR algorithms for training ranking models, not part of the inference pipeline.

7. **Fix WASM inconsistency**: Either add WASM to all crates or explain why
   only some have it.
