# Research Connections: Papers Relevant to rank-* Repositories

This document tracks research papers that are relevant to our rank-* repositories, helping us stay aligned with the latest research and identify implementation opportunities.

## rank-relax: Differentiable Ranking

### Gumbel Reranking (ACL 2025) ⭐ **Highly Relevant**

**Paper**: "Gumbel Reranking: Differentiable End-to-End Reranker Optimization"  
**Authors**: Siyuan Huang, Zhiyuan Ma, Jintao Du, et al.  
**Venue**: ACL 2025

**Summary**: End-to-end training of rerankers in RAG systems using Gumbel-Softmax and relaxed top-k sampling. Reformulates reranking as learning a differentiable attention mask.

**Key Techniques**:
- Gumbel-Softmax trick for differentiable discrete sampling
- Relaxed Top-k: sample k times, take element-wise max
- Differentiable Masked Attention (DMA) for end-to-end optimization

**Connection to rank-relax**:
- ✅ rank-relax has `differentiable_topk()` but uses sigmoid, not Gumbel
- ⏳ Should add Gumbel-Softmax variant for better exploration
- ⏳ Should add relaxed top-k with multiple samples
- ✅ Core infrastructure (differentiable operations) already exists

**Status**: ✅ **Implemented** - Available in `rank-relax` with `gumbel` feature

**Implementation**:
- ✅ `gumbel_softmax()` - Gumbel-Softmax sampling
- ✅ `relaxed_topk_gumbel()` - Relaxed top-k with multiple samples
- ✅ `gumbel_attention_mask()` - Convenience function for RAG reranking
- ✅ Tests passing (5 tests)
- ✅ Example: `examples/gumbel_reranking.rs`
- ✅ Documentation: `rank-relax/docs/GUMBEL_RERANKING.md`

**Usage**:
```bash
cargo add rank-relax --features gumbel
cargo run --example gumbel_reranking --features gumbel
```

## rank-refine: Late Interaction Scoring

### ColBERT (SIGIR 2020)

**Paper**: "ColBERT: Efficient and Effective Passage Search via Contextualized Late Interaction over BERT"  
**Authors**: Omar Khattab, Matei Zaharia

**Connection**: rank-refine implements MaxSim (ColBERT-style token-level similarity)

**Status**: ✅ Implemented

### mxbai-rerank-v2 (March 2025) - Cross-Encoder Complement

**Paper**: "ProRank: Prompt Warmup via Reinforcement Learning for Small Language Models Reranking"  
**Authors**: Xianming Li, Aamir Shakir, Rui Huang, Julius Lipp, Jing Li  
**Venue**: arXiv:2506.03487 (2025)

**Summary**: RL-optimized cross-encoder achieving SOTA on BEIR (57.49 NDCG@10). Uses Qwen-2.5 base with three-stage training: GRPO + contrastive learning + preference learning.

**Key Features**:
- 100+ language support
- 8K context length (32K compatible)
- 8x faster than comparable cross-encoders (0.89s latency on A100)
- Apache 2.0 license
- Models: `mxbai-rerank-base-v2` (0.5B), `mxbai-rerank-large-v2` (1.5B)

**Connection to rank-refine**:
- **Different approach**: Cross-encoder (joint encoding) vs Late Interaction (MaxSim)
- **Complementary**: Use MaxSim for 100-1000 candidates, mxbai-rerank for final top-10-50
- **Integration**: Can implement `CrossEncoderModel` trait using mxbai-rerank
- **Pipeline**: Dense → MaxSim (rank-refine) → Cross-encoder (mxbai-rerank)

**When to use which**:
| Approach | Candidates | Speed | Quality | Pre-computation |
|----------|-----------|-------|---------|-----------------|
| MaxSim (rank-refine) | 100-1000 | Fast (~61ms) | Good | Needs token embeddings |
| mxbai-rerank | 10-100 | Medium (~0.9s) | Best | Raw text input |

**Status**: ✅ **Integrated** - Tested integration with evaluation

**Examples**:
- `rank-refine-python/examples/two_stage_reranking.py` - Basic pipeline demo
- `rank-refine-python/examples/comprehensive_reranking_eval.py` - Full eval with metrics

**Benchmark Results** (synthetic data, CPU):
| Method | NDCG@10 | MAP | Latency |
|--------|---------|-----|---------|
| MaxSim | 0.63-0.73 | 0.48-0.56 | ~1ms |
| Two-Stage | 1.00 | 1.00 | ~1500ms |

**Installation**:
```bash
pip install mxbai-rerank rank-refine rank-eval
```

**Links**:
- [GitHub](https://github.com/mixedbread-ai/mxbai-rerank)
- [Blog](https://www.mixedbread.com/blog/mxbai-rerank-v2)
- [HuggingFace](https://huggingface.co/mixedbread-ai/mxbai-rerank-base-v2)

## rank-fusion: Rank Fusion Algorithms

### Reciprocal Rank Fusion (RRF)

**Paper**: "Reciprocal Rank Fusion outperforms Condorcet and individual Rank Learning Methods"  
**Authors**: Cormack et al.

**Connection**: rank-fusion implements RRF as primary fusion method

**Status**: ✅ Implemented

## rank-eval: Evaluation Metrics

### TREC Evaluation

**Connection**: rank-eval implements standard TREC format parsing and evaluation metrics

**Status**: ✅ Implemented

### Comprehensive Evaluation Framework (December 2025)

rank-eval-python now provides a comprehensive evaluation suite:

#### Core Metrics (Rust-backed)
- NDCG, MAP, MRR, Precision, Recall, DCG, Success@k, R-Precision

#### Advanced Metrics (Python)
- **User Behavior**: ERR (cascade model), RBP (geometric decay)
- **Diversity**: α-nDCG (subtopic coverage)
- **Calibration**: Brier score, ECE, calibration curves
- **Session-based**: Session NDCG, Success@T, Time-to-success
- **RAG-specific**: Context precision/recall, Answer faithfulness
- **Fairness**: Exposure disparity, Demographic parity, Group NDCG

#### LLM-as-Judge Framework
- G-Eval style chain-of-thought prompts
- Pairwise comparison templates
- Faithfulness/hallucination detection
- ARES-style RAG evaluation

#### Statistical Testing
- Paired bootstrap significance tests
- Metric correlation (Spearman, Kendall)

**Status**: ✅ **Implemented** - See `rank-eval-python/examples/`

## How to Use This Document

1. **When reading papers**: If a paper is relevant to any rank-* repo, add it here
2. **Implementation planning**: Use this to identify what techniques we should add
3. **Documentation**: Link from repo-specific docs to this central reference
4. **Research alignment**: Ensure we're aware of latest techniques in the field

## Additional Models & Libraries (December 2025 Research)

### Cross-Encoder Models

#### BGE Reranker v2.5 (BAAI)

**Models**: `bge-reranker-v2.5-gemma2-lightweight`, `bge-reranker-v2-m3`  
**Library**: FlagEmbedding (`pip install FlagEmbedding`)

**Key Features**:
- Layerwise LLM reranker based on Gemma2/MiniCPM
- Strong zero-shot performance on BEIR
- Supports fine-tuning with pairwise data

**Status**: 📋 **Documented** - Integration opportunity identified

#### Sentence-Transformers Cross-Encoders

**Models**: `cross-encoder/ms-marco-MiniLM-L-6-v2`, `ms-marco-TinyBERT-L-6`  
**Library**: sentence-transformers

**Key Features**:
- Fast, lightweight (22M params)
- Good baseline for MS MARCO
- Easy to use

**Status**: 📋 **Documented**

#### MonoT5 (Castorini)

**Models**: `castorini/monot5-base-msmarco`, `castorini/monot5-large-msmarco`  
**Type**: T5-based seq2seq reranker

**Key Features**:
- T5 architecture generates "true"/"false" for relevance
- Strong BEIR performance
- Supports many languages

**Status**: 📋 **Documented**

### LLM-Based Listwise Rerankers

#### RankLLM / RankGPT / RankZephyr

**Paper**: "Is ChatGPT Good at Search?" (EMNLP 2023)  
**Library**: rank-llm (`pip install rank-llm[all]`)

**Key Features**:
- Listwise ranking: LLM sees multiple documents and outputs permutation
- RankZephyr: Open-source 7B alternative to GPT
- RankVicuna: Smaller model option
- Achieves highest quality for small k, but high latency (5s+)

**Status**: 📋 **Documented** - External library reference

#### jina-reranker-v3 (September 2025)

**Paper**: "jina-reranker-v3: Last but Not Late Interaction for Listwise Document Reranking"  
**Authors**: Feng Wang, Yuqing Li, Han Xiao

**Key Innovation**: "Last but not late" interaction - causal attention between query and all candidates, not separate encoding like ColBERT

**Status**: 📋 **Documented**

### Fast/Lightweight Rerankers

#### FlashRank

**Library**: FlashRank (`pip install flashrank`)  
**GitHub**: PrithivirajDamodaran/FlashRank

**Key Features**:
- ONNX-optimized cross-encoders
- Ultra-fast CPU inference
- Tiny models (~4MB)
- No PyTorch dependency

**Status**: ✅ **Integrated** - See `rank-refine-python/examples/flashrank_integration.py`

### Unified Reranker Libraries

#### AnswerDotAI/rerankers (1.6k stars)

**Library**: rerankers (`pip install rerankers`)

Unified API supporting:
- Cross-encoders (MiniLM, BERT)
- ColBERT-style late interaction
- T5-based (MonoT5, InRanker)
- LLM-based (RankGPT, RankLLM)
- API providers (Cohere, Jina, Voyage)
- FlashRank (ONNX)
- Multi-modal (MonoVLM)

**Integration Opportunity**: Add rank-refine as a backend option

**Status**: ✅ **Integrated** - See `rank-refine-python/examples/rerankers_unified.py`

---

## Learning-to-Rank Loss Functions

### Pairwise Losses

#### RankNet (2005)
Logistic loss on score differences: σ(s_i - s_j)

#### LambdaRank / LambdaLoss
RankNet + ΔNDCG weighting for pairs that matter to ranking metrics

**Connection to rank-relax**: Could add as differentiable loss function

### Listwise Losses

#### ListNet
KL divergence between predicted and ground-truth score distributions

#### ApproxNDCG
Differentiable NDCG via soft ranking (smooth sorting)

**Paper**: "FIRST: Faster Listwise Reranking with Single-Token Decoding" (EMNLP 2024)  
Shows weighted RankNet outperforms LambdaRank and ListNet for LLM rerankers

**Status**: ✅ **Implemented** in rank-relax

```rust
use rank_relax::{ranknet_loss, lambda_loss, approx_ndcg};
let loss = ranknet_loss(&predictions, &relevance);
let lambda = lambda_loss(&predictions, &relevance, Some(10));
let approx = approx_ndcg(&predictions, &relevance, 1.0, Some(10));
```

---

## Standard Benchmarks

### BEIR (Heterogeneous IR Benchmark)

18 diverse datasets for zero-shot evaluation:

| Dataset | Domain | Queries | Task |
|---------|--------|---------|------|
| MS MARCO | Web | 6.9k | Passage ranking |
| Natural Questions | Wikipedia | 3.5k | QA retrieval |
| HotpotQA | Wikipedia | 7.4k | Multi-hop QA |
| TREC-COVID | Medical | 50 | Bio-medical |
| SciFact | Scientific | 300 | Fact verification |
| FiQA | Financial | 648 | Financial QA |
| ArguAna | Arguments | 1.4k | Argument retrieval |

**Metric**: NDCG@10 (via TREC evaluation toolkit)

**Status**: ✅ **Implemented** in rank-eval-python

```python
from rank_eval import load_beir, list_beir_datasets, evaluate_beir
dataset = load_beir("scifact")
metrics = evaluate_beir("scifact", run)
```

### MS MARCO

Standard training dataset:
- 8.8M passages
- 500k training queries
- TSV format (qid, pid, query, passage)

**Status**: ✅ **Implemented** in rank-eval-python

```python
from rank_eval import load_msmarco_passage
```

### MTEB Leaderboard

Massive Text Embedding Benchmark on HuggingFace:
- Tracks embedding and reranker models
- Standard evaluation protocol

---

## Efficiency Optimizations

### PLAID (ColBERTv2 Engine)

Centroid interaction + pruning for 45x CPU speedup

### WARP (XTR Engine)

PLAID-like pruning + XTR compression:
- Dynamic similarity imputation
- Implicit decompression
- 3x faster than ColBERTv2/PLAID

### Quantization
- INT8 for transformer encoders
- Product quantization for embeddings

**Status**: 📋 **Documented** - Future optimization opportunity

---

## RAG-Specific Reranking (2024-2025)

### DynamicRAG (May 2025)
Use LLM output as feedback for dynamic reranking

### InfoGain-RAG (September 2025)
Rerank by document information gain

### LevelRAG (February 2025)
Multi-hop logic planning over rerankers

**Status**: ✅ **Implemented** in `rank-refine-python/rank_refine/rag_dynamic.py`

Implementations:
- `DynamicRAGReranker`: Feedback loop with generation quality evaluation
- `InfoGainReranker`: Diversity-aware reranking based on novelty
- `AdaptiveTopK`: Dynamic cutoff selection (not fixed k)
- `UncertaintySamplingReranker`: Focus on documents resolving uncertainty

```python
from rank_refine.rag_dynamic import InfoGainReranker, DynamicRAGReranker
reranker = InfoGainReranker(base_scorer=my_scorer, novelty_weight=0.5)
context = reranker.rerank("query", documents, k=5)
```

---

## Multi-Modal Reranking

### Emerging Area
- **MonoQwen2-VL**: Multi-modal reranker for PDFs, images
- **Jina-reranker-v2**: Text + image
- **ColPali**: Late interaction for document images
- **Video-ColBERT**: Text-to-video retrieval

**Status**: ✅ **Implemented** in `rank-refine-python/rank_refine/multimodal.py`

Implementations:
- `CLIPEmbedder`: OpenAI CLIP for text-image similarity
- `SigLIPEmbedder`: Google SigLIP (improved zero-shot)
- `MultimodalReranker`: Unified API for text/image/multimodal docs
- `multimodal_maxsim`: MaxSim with patch-level late interaction

```python
from rank_refine.multimodal import CLIPEmbedder, MultimodalReranker
embedder = CLIPEmbedder("openai/clip-vit-base-patch32")
reranker = MultimodalReranker(embedder)
results = reranker.rerank("sunset photo", image_docs)
```

---

## LLM-Based Listwise Reranking

### RankGPT / RankZephyr Style

**Paper**: "Is ChatGPT Good at Search?" (EMNLP 2023)

**Status**: ✅ **Implemented** in `rank-refine-python/rank_refine/llm_rerank.py`

Implementations:
- `LLMReranker`: Pluggable LLM call function (OpenAI, Anthropic, local)
- `SlidingWindowReranker`: Heapsort-style for large candidate sets
- `build_rankgpt_prompt`: Standard RankGPT prompt format
- `build_cot_prompt`: Chain-of-thought reasoning prompts
- `parse_ranking_output`: Robust parsing of LLM ranking output

```python
from rank_refine.llm_rerank import LLMReranker, SlidingWindowReranker
reranker = LLMReranker(call_fn=my_llm_call)
ranked = reranker.rerank("query", documents)

# For many documents
sw = SlidingWindowReranker(reranker, window_size=10, step_size=5)
ranked = sw.rerank("query", large_doc_list)
```

---

## ONNX Export & Optimization

### Inference Optimization

**Status**: ✅ **Implemented** in `rank-refine-python/rank_refine/onnx_export.py`

Implementations:
- `export_sentence_transformer`: Export encoders to ONNX
- `export_cross_encoder`: Export rerankers to ONNX
- `export_clip_text_encoder`: Export CLIP text encoder
- `optimize_onnx_model`: Graph optimization + INT8 quantization
- `ONNXEncoder`: Fast ONNX-based encoding
- `ONNXCrossEncoder`: Fast ONNX-based reranking

```python
from rank_refine.onnx_export import export_cross_encoder, ONNXCrossEncoder
export_cross_encoder("cross-encoder/ms-marco-MiniLM-L-6-v2", "reranker.onnx")
reranker = ONNXCrossEncoder("reranker.onnx", "cross-encoder/ms-marco-MiniLM-L-6-v2")
scores = reranker.score("query", ["doc1", "doc2"])
```

---

## Adding New Papers

When adding a paper, include:
- **Title, authors, venue**
- **Summary** (1-2 sentences)
- **Key techniques** (bullet points)
- **Connection** to specific rank-* repo
- **Status** (✅ Implemented, ⏳ To Implement, 📋 Documented)
- **Action items** (if any)

