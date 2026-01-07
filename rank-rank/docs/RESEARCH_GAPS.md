# Research Gaps & Expansion Opportunities

Analysis of what the rank-* ecosystem is missing compared to the state of the art (December 2025).

## Executive Summary

The rank-* ecosystem has strong foundations in:
- Late interaction (MaxSim) via `rank-refine`
- Rank fusion algorithms via `rank-fusion`
- Differentiable ranking via `rank-relax`
- IR evaluation metrics via `rank-eval`

### Progress Made (December 2025)

✅ **Completed:**
1. BEIR dataset loader with 13 datasets (rank-eval)
2. MS MARCO passage ranking loader (rank-eval)
3. LambdaLoss, RankNet, ApproxNDCG loss functions (rank-relax)
4. FlashRank integration example (rank-refine-python)
5. `rerankers` library integration example (rank-refine-python)
6. Advanced metrics: ERR, RBP, α-nDCG, context precision/recall (rank-eval)
7. 60+ additional benchmark dataset metadata (rank-eval)
8. Statistical significance testing (paired bootstrap)
9. mxbai-rerank cross-encoder integration
10. Fairness metrics: exposure disparity, demographic parity, group NDCG (rank-eval)
11. LLM-as-Judge framework: G-Eval prompts, pairwise comparison, faithfulness (rank-eval)
12. Session-based metrics: session NDCG, success@T, time-to-success (rank-eval)
13. Calibration metrics: Brier score, ECE, calibration curves (rank-eval)
14. Full pipeline examples: rank-refine + rank-eval integration (examples/)
15. **CrossEncoderModel trait** in Rust with ORT (ONNX Runtime) implementation
16. **LLM-based listwise reranking** (llm_rerank.py): RankGPT-style prompts, sliding window
17. **ONNX export utilities** (onnx_export.py): encoder + cross-encoder export, quantization
18. **Multimodal support** (multimodal.py): CLIP, SigLIP embedders, MaxSim for images
19. **RAG-specific dynamic reranking** (rag_dynamic.py): InfoGain, adaptive top-k, feedback loops

### Remaining Gaps

All major gaps have been addressed! ✅

~~1. No native cross-encoder~~ → CrossEncoderModel trait in Rust + ONNX support
~~2. No LLM-based rerankers~~ → llm_rerank.py module (RankGPT-style)
~~3. Limited inference optimization~~ → ONNX export utilities + quantization
~~4. No multimodal support~~ → multimodal.py (CLIP, SigLIP integration)

---

## Recent Additions (December 2025)

### New Metrics in rank-eval

**User Behavior Metrics**
- ERR (Expected Reciprocal Rank) - Cascade user model
- RBP (Rank-Biased Precision) - Geometric decay model

**Diversity Metrics**
- α-nDCG for subtopic coverage

**Calibration Metrics**
- Brier score - Mean squared calibration error
- ECE (Expected Calibration Error) - Binned calibration
- Calibration curves for reliability diagrams

**Session-Based Metrics**
- Session NDCG - Multi-turn aggregation
- Success@T - Success within T turns
- Time-to-Success - Turns until relevant result

**RAG-Specific Metrics**
- Context Precision - Answer-supporting doc relevance
- Context Recall - Fact coverage
- Answer Faithfulness - Groundedness check

**Fairness Metrics (NEW)**
- Exposure Disparity (EE-D) - Actual vs target exposure
- Exposure-Relevance Alignment (EE-R) - Merit correlation
- Demographic Parity Ratio - Protected group exposure
- Group NDCG Disparity - Per-group quality

**LLM-as-Judge Framework (NEW)**
- G-Eval style chain-of-thought prompts
- Pairwise comparison prompts
- Faithfulness/hallucination check prompts
- ARES-style RAG evaluation prompts
- Pre-defined criteria: RETRIEVAL_RELEVANCE, ANSWER_FAITHFULNESS, etc.

**Statistical Testing**
- Paired bootstrap significance testing
- Metric correlation (Spearman, Kendall)

### New Datasets Metadata (60+ benchmarks)
- **Multilingual**: MIRACL, MLDR, Mr.TyDi, XOR-TyDi, MKQA, CLIR-Matrix
- **Multimodal (VDR)**: MMDocIR, ViDoRe V2, MIRACL-VISION, Jina-VDR, M-LongDoc
- **Code/Tool-Use**: CoIR (ACL 2025), Text-to-SQL retrieval, APPS
- **Reasoning**: BRIGHT (reasoning-intensive retrieval)
- **Long-Context**: LongBench, SCROLLS, ZeroSCROLLS, L-Eval
- **Temporal/Robustness**: FutureQueryEval, Robustness-Gym
- **Agentic**: AgentBench-RAG, WebAgent-Retrieval
- **RAG-Specific**: DataMorgana, ARES, RAG-Truth

---

## 1. Models & Architectures

### Currently Supported
| Model Type | Implementation | Status |
|------------|---------------|--------|
| Late Interaction (MaxSim) | rank-refine (Rust) | Production |
| Cross-Encoder | External (mxbai-rerank) | Integration example only |

### Missing Models

#### Cross-Encoders (High Priority)
State-of-the-art cross-encoders we could integrate or implement:

| Model | Provider | Params | NDCG@10 (BEIR) | Notes |
|-------|----------|--------|----------------|-------|
| `bge-reranker-v2.5-gemma2` | BAAI | 2.6B | ~0.60 | Layerwise LLM reranker |
| `mxbai-rerank-v2` | MixedBread | 0.5-1.5B | 0.5749 | Already integrated |
| `ms-marco-MiniLM-L-6-v2` | Sentence-Transformers | 22M | ~0.52 | Fast, good baseline |
| `MonoT5-base` | Castorini | 220M | ~0.54 | T5-based pointwise |
| `Jina-reranker-v2` | Jina AI | 137M | ~0.55 | Multimodal capable |

**Recommendation**: Add `CrossEncoderModel` trait to rank-refine with implementations for:
- Sentence-Transformers cross-encoders
- MonoT5-style seq2seq rerankers
- BGE layerwise rerankers

#### LLM-Based Listwise Rerankers (Medium Priority)
These achieve highest quality but are expensive:

| Model | Type | Latency | Quality |
|-------|------|---------|---------|
| RankGPT (GPT-4) | Listwise API | 5-10s | SOTA on familiar domains |
| RankZephyr | Listwise local | 2-5s | Open-source alternative |
| RankVicuna | Listwise local | 2-5s | Smaller model option |

**Research**: "How Good are LLM-based Rerankers?" (EMNLP 2025) shows LLM rerankers outperform cross-encoders on familiar domains but generalize worse.

#### Efficient Retrieval Engines (Low Priority - Different Scope)
For reference, these are retrieval systems rather than rerankers:

| Engine | Architecture | Notes |
|--------|-------------|-------|
| PLAID | ColBERTv2 optimized | 45x CPU speedup |
| WARP | XTR + PLAID hybrid | 3x faster than ColBERTv2 |
| XTR | Compressed ColBERT | Smaller index |

---

## 2. Datasets & Benchmarks

### Currently Supported ✅
- **13 BEIR datasets** with automatic download and loading
- **MS MARCO passage ranking** format support
- **60+ additional benchmark metadata** (multilingual, multimodal, RAG, code, etc.)
- Custom eval_results.json format

### Standard Benchmarks (Implemented)

#### BEIR Benchmark (High Priority)
The standard for zero-shot evaluation. 18 diverse datasets:

| Dataset | Domain | Queries | Docs | Task Type |
|---------|--------|---------|------|-----------|
| MS MARCO | Web | 6.9k | 8.8M | Passage ranking |
| Natural Questions | Wikipedia | 3.5k | 2.7M | QA retrieval |
| HotpotQA | Wikipedia | 7.4k | 5.2M | Multi-hop QA |
| TREC-COVID | Scientific | 50 | 171k | Bio-medical |
| NFCorpus | Medical | 323 | 3.6k | Medical |
| SciFact | Scientific | 300 | 5k | Fact verification |
| ArguAna | Arguments | 1.4k | 8.7k | Argument retrieval |
| FiQA | Financial | 648 | 57k | Financial QA |
| Quora | Duplicate | 10k | 523k | Duplicate detection |

**Action**: Create `rank-eval/datasets/` with BEIR integration:
```python
from rank_eval.datasets import load_beir
queries, corpus, qrels = load_beir("scifact")
```

#### MS MARCO (High Priority)
The most common training dataset for rerankers:
- **MS MARCO Passage**: 8.8M passages, 500k training queries
- **MS MARCO Document**: 3.2M documents
- Standard format: TSV with qid, pid, query, passage

#### TREC Datasets
- TREC Deep Learning 2019/2020/2021
- TREC Robust04
- TREC-COVID

---

## 3. Learning-to-Rank Methods

### Currently Supported ✅
- Simple score-based ranking
- MaxSim late interaction
- **All major LTR losses implemented in rank-relax:**
  - RankNet (pairwise logistic)
  - LambdaLoss (NDCG-weighted pairwise)
  - ListNet (listwise KL divergence)
  - ListMLE (permutation likelihood)
  - ApproxNDCG (differentiable NDCG)

### Loss Functions (Implemented ✅)

#### Pairwise
- ✅ **RankNet**: `rank_relax::ranknet_loss(predictions, targets)`
- ✅ **LambdaLoss**: `rank_relax::lambda_loss(predictions, targets, k)`

#### Listwise
- ✅ **ListNet**: `rank_relax::listnet_loss(predictions, targets, tau)`
- ✅ **ListMLE**: `rank_relax::listmle_loss(predictions, targets, tau)`
- ✅ **ApproxNDCG**: `rank_relax::approx_ndcg(predictions, targets, tau, k)`

**Usage in rank-relax:**
```rust
use rank_relax::{ranknet_loss, lambda_loss, approx_ndcg, listnet_loss, listmle_loss};

let predictions = vec![0.9, 0.3, 0.7, 0.5];
let relevance = vec![2.0, 0.0, 1.0, 0.0];

let ranknet = ranknet_loss(&predictions, &relevance);
let lambda = lambda_loss(&predictions, &relevance, Some(10));
let approx_ndcg_loss = approx_ndcg(&predictions, &relevance, 1.0, Some(10));
```

---

## 4. Python Ecosystem Integration

### Implemented Integrations ✅

#### AnswerDotAI/rerankers ✅
Integration example created: `rank-refine-python/examples/rerankers_unified.py`
```python
# Custom MaxSim backend for rerankers
from rerankers import Reranker
# Can use rank-refine alongside other backends
```

#### FlashRank (CPU-optimized) ✅
Integration example created: `rank-refine-python/examples/flashrank_integration.py`
```python
from flashrank import Ranker, RerankRequest
ranker = Ranker(model_name="ms-marco-MiniLM-L-12-v2")
# Two-stage: MaxSim -> FlashRank pipeline demonstrated
```

#### mxbai-rerank ✅
Integration example created: `rank-refine-python/examples/comprehensive_reranking_eval.py`
```python
from mxbai_rerank import MxbaiRerankV2
# Two-stage: MaxSim -> mxbai-rerank pipeline demonstrated
```

### Potential Future Integrations

#### FlagEmbedding (BAAI)
Official library for BGE models:
```python
from FlagEmbedding import FlagReranker
reranker = FlagReranker("BAAI/bge-reranker-v2-m3", use_fp16=True)
scores = reranker.rerank(query, docs)
```

#### RankLLM (LLM listwise)
Unified LLM reranking:
```bash
pip install rank-llm[all]
```
Supports RankGPT, RankZephyr, RankVicuna.

---

## 5. Inference Optimization

### Currently Missing

#### ONNX Export (High Priority)
Export cross-encoders and MaxSim to ONNX for:
- Faster inference
- CPU optimization
- Edge deployment

#### Quantization (Medium Priority)
- INT8 quantization for transformer encoders
- Product quantization for token embeddings

#### Batching Strategies
- Dynamic batching for variable-length inputs
- Sequence length bucketing

---

## 6. Multi-Modal Reranking

### Emerging Area
- **MonoQwen2-VL**: Multi-modal reranker for PDFs, images
- **Jina-reranker-v2**: Text + image
- **Video-ColBERT**: Text-to-video retrieval

### Relevance to rank-*
Could extend rank-refine's `TokenEmbeddings` to handle vision tokens.

---

## 7. RAG-Specific Innovations

### Recent Papers (2024-2025)

| Paper | Key Idea |
|-------|----------|
| DynamicRAG | Use LLM output as feedback for dynamic reranking |
| InfoGain-RAG | Rerank by document information gain |
| LevelRAG | Multi-hop logic planning over rerankers |
| RE-RAG | Relevance estimator for interpretable reranking |

### Relevance
RAG systems increasingly need:
- Dynamic passage selection (not fixed top-k)
- Multi-hop query decomposition
- Feedback loops from generation to retrieval

---

## 8. Recommended Priorities

### High Priority (Completed ✅)
1. ✅ Add BEIR dataset loader to rank-eval (13 datasets + 60+ additional benchmarks)
2. ✅ Add LambdaLoss to rank-relax (+ RankNet, ApproxNDCG, ListNet, ListMLE)
3. ✅ Create FlashRank integration example (rank-refine-python)

### Medium Priority (Completed ✅)
4. ✅ Integration with AnswerDotAI/rerankers (example created)
5. ✅ CrossEncoderModel trait in rank-refine (Rust trait + ORT implementation)
6. ✅ ONNX export utilities (onnx_export.py with quantization support)

### Low Priority (Completed ✅)
7. ✅ LLM-based listwise reranking (llm_rerank.py - RankGPT, sliding window)
8. ✅ Multi-modal support (multimodal.py - CLIP, SigLIP embedders)
9. ✅ RAG-specific dynamic reranking (rag_dynamic.py - InfoGain, adaptive top-k)

---

## 9. Competitive Landscape

| Library | Focus | Language | Stars |
|---------|-------|----------|-------|
| rank-refine | Late interaction (MaxSim) | Rust/Python | - |
| rerankers | Unified API | Python | 1.6k |
| FlagEmbedding | BGE models | Python | 6k+ |
| FlashRank | Fast ONNX | Python | 1k+ |
| RankLLM | LLM listwise | Python | 800+ |
| ColBERT | Late interaction | Python | 2k+ |
| Sentence-Transformers | Cross-encoders | Python | 14k+ |

### rank-* Differentiators
1. **Rust performance** - Native speed without Python overhead
2. **Modular design** - Separate crates for fusion, refinement, relaxation, eval
3. **Differentiable ranking** - rank-relax for ML training
4. **Evaluation suite** - rank-eval with comprehensive metrics

---

## References

1. ColBERTv2: Effective and Efficient Retrieval via Lightweight Late Interaction (2021)
2. BEIR: A Heterogeneous Benchmark for Zero-shot Evaluation of Information Retrieval Models (2021)
3. How Good are LLM-based Rerankers? (EMNLP 2025)
4. jina-reranker-v3: Last but Not Late Interaction for Listwise Document Reranking (2025)
5. FIRST: Faster Listwise Reranking with Single-Token Decoding (EMNLP 2024)
6. WARP: Efficient XTR Retrieval via Centroid Pruning (2025)

