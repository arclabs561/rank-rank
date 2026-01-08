# Research Gaps & Expansion Opportunities

Analysis of what the rank-* ecosystem is missing compared to the state of the art (December 2025).

## Executive Summary

The rank-* ecosystem has strong foundations in:
- Late interaction (MaxSim) via `rank-rerank`
- Rank fusion algorithms via `rank-fusion`
- Differentiable ranking via `rank-soft`
- IR evaluation metrics via `rank-eval`

**Recently Addressed (December 2025):**
- BEIR dataset loader added to rank-eval-python
- LambdaLoss, ApproxNDCG, RankNet added to rank-soft
- FlashRank integration example in rank-rerank-python
- rerankers library integration example

**Remaining Gaps:**
1. **No native cross-encoder** - rely on external libs (mxbai-rerank, FlashRank)
2. **No LLM-based rerankers** - RankGPT, RankZephyr listwise approaches
3. **Limited inference optimization** - no ONNX export for MaxSim

---

## 1. Models & Architectures

### Currently Supported
| Model Type | Implementation | Status |
|------------|---------------|--------|
| Late Interaction (MaxSim) | rank-rerank (Rust) | Production |
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

**Recommendation**: Add `CrossEncoderModel` trait to rank-rerank with implementations for:
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

### Currently Used
- Synthetic datasets in examples
- Custom eval_results.json format

### Missing Standard Benchmarks

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

### Currently Supported in rank-soft

| Loss Function | Type | Status | Notes |
|--------------|------|--------|-------|
| RankNet | Pairwise | Implemented | Logistic loss on score differences |
| LambdaLoss | Pairwise (NDCG-weighted) | Implemented | ΔNDCG weighting for direct metric optimization |
| ApproxNDCG | Listwise | Implemented | Differentiable NDCG via soft ranking |
| ListNet | Listwise | Implemented | KL divergence on score distributions |
| ListMLE | Listwise | Implemented | Likelihood of correct permutation |
| Spearman Loss | Correlation | Implemented | Differentiable Spearman correlation |

**Usage**:
```rust
use rank_relax::{ranknet_loss, lambda_loss, approx_ndcg, listnet_loss, listmle_loss};

let predictions = vec![0.9, 0.3, 0.7, 0.5, 0.8];
let relevance = vec![2.0, 0.0, 1.0, 0.0, 2.0];

let rn = ranknet_loss(&predictions, &relevance);
let ll = lambda_loss(&predictions, &relevance, Some(10));
let an = approx_ndcg(&predictions, &relevance, 1.0, Some(10));
```

### Not Yet Implemented

- **SoftRank Loss**: Full optimal transport soft ranking
- **LambdaMART**: Gradient boosting with LambdaLoss
- **Permutahedron Projection**: O(n log n) differentiable sorting

---

## 4. Python Ecosystem Integration

### Existing Libraries We Could Integrate With

#### AnswerDotAI/rerankers (High Value)
Unified API for rerankers (1.6k stars):
```python
from rerankers import Reranker
ranker = Reranker('cross-encoder')  # or 'colbert', 't5', 'cohere', etc.
results = ranker.rank(query, docs)
```

Supports: Cross-encoders, ColBERT, T5, RankGPT, Cohere, Jina, FlashRank, MonoVLM

**Integration opportunity**: Add rank-rerank as a backend:
```python
ranker = Reranker('rank-rerank', model_type='maxsim')
```

#### FlagEmbedding (BAAI)
Official library for BGE models:
```python
from FlagEmbedding import FlagReranker
reranker = FlagReranker("BAAI/bge-reranker-v2-m3", use_fp16=True)
scores = reranker.rerank(query, docs)
```

Includes training scripts for fine-tuning.

#### FlashRank (CPU-optimized)
Ultra-fast ONNX rerankers for CPU:
```python
from flashrank import Ranker, RerankRequest
ranker = Ranker(model_name="ms-marco-TinyBERT-L-2-v2")
results = ranker.rerank(RerankRequest(query=q, passages=docs))
```

4MB models, no PyTorch dependency.

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
Could extend rank-rerank's `TokenEmbeddings` to handle vision tokens.

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

### Completed (December 2025)
1. ~~Add BEIR dataset loader to rank-eval~~ - `rank_eval.datasets.load_beir()`
2. ~~Add LambdaLoss to rank-soft~~ - Plus RankNet, ApproxNDCG, ListNet, ListMLE
3. ~~Create FlashRank integration example~~ - `examples/flashrank_integration.py`
4. ~~Integration with AnswerDotAI/rerankers~~ - `examples/rerankers_unified.py`

### High Priority (Next Sprint)
1. Add CrossEncoderModel trait to rank-rerank (native Rust implementation)
2. ONNX export for MaxSim encoder
3. Integration tests with real BEIR datasets (scifact, nfcorpus)

### Medium Priority (Next Quarter)
4. Native BGE reranker support
5. Batch inference optimization
6. GPU acceleration for MaxSim (CUDA/Metal)

### Low Priority (Future)
7. LLM-based listwise reranking
8. Multi-modal support (images, video)
9. RAG-specific dynamic reranking

---

## 9. Competitive Landscape

| Library | Focus | Language | Stars |
|---------|-------|----------|-------|
| rank-rerank | Late interaction (MaxSim) | Rust/Python | - |
| rerankers | Unified API | Python | 1.6k |
| FlagEmbedding | BGE models | Python | 6k+ |
| FlashRank | Fast ONNX | Python | 1k+ |
| RankLLM | LLM listwise | Python | 800+ |
| ColBERT | Late interaction | Python | 2k+ |
| Sentence-Transformers | Cross-encoders | Python | 14k+ |

### rank-* Differentiators
1. **Rust performance** - Native speed without Python overhead
2. **Modular design** - Separate crates for fusion, refinement, relaxation, eval
3. **Differentiable ranking** - rank-soft for ML training
4. **Evaluation suite** - rank-eval with comprehensive metrics

---

## References

1. ColBERTv2: Effective and Efficient Retrieval via Lightweight Late Interaction (2021)
2. BEIR: A Heterogeneous Benchmark for Zero-shot Evaluation of Information Retrieval Models (2021)
3. How Good are LLM-based Rerankers? (EMNLP 2025)
4. jina-reranker-v3: Last but Not Late Interaction for Listwise Document Reranking (2025)
5. FIRST: Faster Listwise Reranking with Single-Token Decoding (EMNLP 2024)
6. WARP: Efficient XTR Retrieval via Centroid Pruning (2025)

