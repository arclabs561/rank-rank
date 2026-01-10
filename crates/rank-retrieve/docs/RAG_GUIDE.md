# RAG (Retrieval-Augmented Generation) Guide

Comprehensive guide for building RAG pipelines with `rank-retrieve`, based on vector database research and best practices.

## Overview

RAG (Retrieval-Augmented Generation) combines information retrieval with language generation to enhance LLM capabilities. This guide follows the three-phase workflow identified in vector database research:

1. **Data Storage Phase**: Preprocess, chunk, embed, and index documents
2. **Information Retrieval Phase**: Query embedding and similarity search
3. **Content Generation Phase**: LLM context injection and response generation

## Three-Phase RAG Workflow

### Phase 1: Data Storage

**Objective**: Convert unstructured data into searchable vector representations.

**Steps**:

1. **Preprocessing**
   - Clean and normalize text (remove HTML, normalize whitespace)
   - Handle multiple formats (PDF, DOC, MD, CSV, etc.)
   - Language detection and normalization

2. **Chunking**
   - Split documents into smaller chunks (typically 100-500 tokens)
   - Overlap between chunks (10-20%) to preserve context
   - Preserve metadata (source, section, timestamp)

3. **Embedding**
   - Generate dense embeddings using embedding models (e.g., BERT, sentence-transformers)
   - Normalize embeddings (L2 normalization for cosine similarity)
   - Store embeddings with document IDs and metadata

4. **Indexing**
   - Build retrieval indexes (BM25, dense ANN, sparse)
   - Choose indexing method based on corpus size and requirements

**Example: Data Storage Phase**

```rust
use rank_retrieve::bm25::{Bm25Params, InvertedIndex};
use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::retrieve_bm25;
use rank_retrieve::retrieve_dense;

// 1. Preprocess and chunk documents
struct DocumentChunk {
    id: u32,
    text: String,
    source: String,
    chunk_index: usize,
}

fn preprocess_and_chunk(documents: Vec<String>) -> Vec<DocumentChunk> {
    documents
        .into_iter()
        .enumerate()
        .flat_map(|(doc_id, text)| {
            // Simple chunking (in production, use proper tokenization)
            text.split("\n\n")
                .enumerate()
                .map(move |(chunk_idx, chunk)| DocumentChunk {
                    id: (doc_id * 1000 + chunk_idx) as u32,
                    text: chunk.to_string(),
                    source: format!("doc_{}", doc_id),
                    chunk_index: chunk_idx,
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

// 2. Build indexes
fn build_indexes(chunks: &[DocumentChunk], embeddings: &[[f32; 384]]) -> (InvertedIndex, DenseRetriever) {
    // BM25 index (lexical matching)
    let mut bm25_index = InvertedIndex::new();
    for chunk in chunks {
        let terms: Vec<String> = chunk.text
            .split_whitespace()
            .map(|s| s.to_lowercase())
            .collect();
        bm25_index.add_document(chunk.id, &terms);
    }
    
    // Dense index (semantic matching)
    let mut dense_retriever = DenseRetriever::new();
    for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
        dense_retriever.add_document(chunk.id, embedding.to_vec());
    }
    
    (bm25_index, dense_retriever)
}
```

### Phase 2: Information Retrieval

**Objective**: Find relevant document chunks for a user query.

**Steps**:

1. **Query Embedding**
   - Generate embedding for user query using same embedding model
   - Normalize query embedding (L2 normalization)

2. **Similarity Search**
   - Retrieve candidates using multiple methods (BM25, dense, sparse)
   - Use approximate nearest neighbor search for large corpora
   - Return top-k candidates (typically 100-1000)

3. **Result Fusion** (Optional)
   - Combine results from multiple retrieval methods
   - Use reciprocal rank fusion (RRF) or weighted combination

**Example: Information Retrieval Phase**

```rust
use rank_fusion::rrf_multi;

fn retrieve_candidates(
    query: &str,
    query_embedding: &[f32],
    bm25_index: &InvertedIndex,
    dense_retriever: &DenseRetriever,
) -> Result<Vec<(u32, f32)>, rank_retrieve::RetrieveError> {
    // 1. BM25 retrieval (lexical matching)
    let query_terms: Vec<String> = query
        .split_whitespace()
        .map(|s| s.to_lowercase())
        .collect();
    let bm25_results = retrieve_bm25(
        bm25_index,
        &query_terms,
        1000,
        Bm25Params::default(),
    )?;
    
    // 2. Dense retrieval (semantic matching)
    let dense_results = retrieve_dense(dense_retriever, query_embedding, 1000)?;
    
    // 3. Fuse results (hybrid search)
    let fused = rrf_multi(
        &[&bm25_results, &dense_results],
        Default::default(),
    );
    
    Ok(fused)
}
```

### Phase 3: Content Generation

**Objective**: Generate LLM response using retrieved context.

**Steps**:

1. **Reranking** (Optional but recommended)
   - Rerank top candidates using cross-encoder or MaxSim
   - Select top-k for context (typically 5-10 chunks)

2. **Context Assembly**
   - Combine retrieved chunks into prompt context
   - Format for LLM (e.g., "Context: ...\n\nQuestion: ...")

3. **LLM Generation**
   - Pass prompt to LLM (GPT, Claude, etc.)
   - Generate response with citations/references

**Example: Content Generation Phase**

```rust
use rank_rerank::simd;

fn generate_response(
    query: &str,
    candidates: &[(u32, f32)],
    chunks: &[DocumentChunk],
    query_tokens: &[String],
) -> String {
    // 1. Rerank top candidates
    let top_100: Vec<u32> = candidates.iter().take(100).map(|(id, _)| *id).collect();
    let doc_tokens_list: Vec<Vec<String>> = top_100
        .iter()
        .map(|id| {
            chunks.iter()
                .find(|c| c.id == *id)
                .map(|c| c.text.split_whitespace().map(|s| s.to_string()).collect())
                .unwrap_or_default()
        })
        .collect();
    
    let reranked = simd::maxsim_batch(query_tokens, &doc_tokens_list);
    
    // 2. Assemble context from top-k chunks
    let context: String = reranked.iter().take(5)
        .enumerate()
        .map(|(i, (id, _))| {
            chunks.iter()
                .find(|c| c.id == *id)
                .map(|c| format!("[{}] {}\n", i + 1, c.text))
                .unwrap_or_default()
        })
        .collect();
    
    // 3. Format prompt for LLM
    format!(
        "Context:\n{}\n\nQuestion: {}\n\nAnswer:",
        context, query
    )
    
    // In production, call LLM API here:
    // let response = llm_client.generate(&prompt).await?;
    // response
}
```

## Complete RAG Pipeline Example

```rust
use rank_retrieve::prelude::*;
use rank_fusion::rrf_multi;
use rank_rerank::simd;

struct RAGPipeline {
    bm25_index: InvertedIndex,
    dense_retriever: DenseRetriever,
    chunks: Vec<DocumentChunk>,
}

impl RAGPipeline {
    fn new(documents: Vec<String>, embeddings: &[[f32; 384]]) -> Self {
        // Phase 1: Data Storage
        let chunks = preprocess_and_chunk(documents);
        let (bm25_index, dense_retriever) = build_indexes(&chunks, embeddings);
        
        Self {
            bm25_index,
            dense_retriever,
            chunks,
        }
    }
    
    fn query(&self, query: &str, query_embedding: &[f32]) -> Result<String, rank_retrieve::RetrieveError> {
        // Phase 2: Information Retrieval
        let candidates = retrieve_candidates(
            query,
            query_embedding,
            &self.bm25_index,
            &self.dense_retriever,
        )?;
        
        // Phase 3: Content Generation
        let query_tokens: Vec<String> = query.split_whitespace().map(|s| s.to_string()).collect();
        let response = generate_response(query, &candidates, &self.chunks, &query_tokens);
        
        Ok(response)
    }
}
```

## Best Practices

### 1. Chunking Strategy

**Optimal chunk size**: 100-500 tokens
- Too small: Loses context
- Too large: Dilutes relevance, harder to retrieve

**Overlap**: 10-20% between chunks
- Preserves context across boundaries
- Improves recall for queries spanning chunks

**Metadata preservation**: Store source, section, timestamp
- Enables filtering and citation
- Helps with deduplication

### 2. Embedding Model Selection

**General purpose**: `sentence-transformers/all-MiniLM-L6-v2` (384 dim)
- Fast, good quality
- Suitable for most use cases

**High quality**: `sentence-transformers/all-mpnet-base-v2` (768 dim)
- Better quality, slower
- Use when quality is critical

**Domain-specific**: Fine-tune on domain data
- Improves relevance for specialized domains
- Requires training data

### 3. Hybrid Retrieval

**Why hybrid**: Combines lexical (BM25) and semantic (dense) matching
- BM25: Handles exact keyword matches
- Dense: Handles synonyms, paraphrases
- Fusion: Best of both worlds

**Fusion method**: Reciprocal Rank Fusion (RRF)
- Simple, effective
- No tuning required
- Handles different score scales

### 4. Reranking

**When to rerank**: Always for production systems
- First-stage retrieval: Fast, high recall
- Reranking: Precise, high precision
- Improves final quality significantly

**Reranking methods**:
- **MaxSim** (ColBERT): Fast, good quality
- **Cross-encoder**: Slower, best quality
- Use MaxSim for most cases, cross-encoder for critical applications

### 5. Context Window Management

**Top-k selection**: 5-10 chunks for context
- More chunks: Better coverage, but may dilute relevance
- Fewer chunks: More focused, but may miss information

**Context formatting**: Clear structure
- Number chunks for citation
- Separate context from query
- Include metadata (source, section)

### 6. Error Handling

**Retrieval failures**: Graceful degradation
- Fallback to single method if fusion fails
- Return partial results if some methods fail

**Empty results**: Handle gracefully
- Return informative message
- Suggest query reformulation

## Performance Optimization

### Indexing Performance

- **Batch indexing**: Process documents in batches
- **Parallel processing**: Use multiple threads for embedding generation
- **Incremental updates**: Add new documents without rebuilding entire index

### Retrieval Performance

- **ANN algorithms**: Use HNSW or IVF-PQ for large corpora (>1M docs)
- **Caching**: Cache frequent queries (see Semantic Caching guide)
- **Early termination**: Stop search when sufficient candidates found

### Generation Performance

- **Async processing**: Use async for LLM API calls
- **Streaming**: Stream LLM responses for better UX
- **Batching**: Batch reranking operations

## Integration with Vector Databases

For production systems requiring persistence and scaling:

**Qdrant Integration**: See `examples/qdrant_real_integration.rs`
- Persistent storage
- Horizontal scaling
- Metadata filtering

**Pinecone Integration**: Similar pattern to Qdrant
- Managed service
- Automatic scaling
- Pay-per-use pricing

**Hybrid approach**: Use vector database for dense, `rank-retrieve` for BM25
- Best of both worlds
- Combines managed infrastructure with in-memory performance

## Evaluation

**Metrics to track**:
- **Recall@k**: Percentage of relevant documents in top-k
- **NDCG@k**: Normalized discounted cumulative gain
- **Latency**: End-to-end query time
- **Cost**: LLM API costs per query

**Evaluation setup**:
```rust
use rank_eval::binary::ndcg_at_k;

// Evaluate retrieval quality
let relevance_scores = vec![1.0, 0.0, 1.0, 0.0, 1.0]; // Ground truth
let ndcg = ndcg_at_k(&candidates, &relevance_scores, 10);
```

## Common Pitfalls

1. **Chunking too small/large**: Test different chunk sizes
2. **No reranking**: Always rerank for production
3. **Single retrieval method**: Use hybrid retrieval
4. **Ignoring metadata**: Preserve and use metadata
5. **No evaluation**: Measure and improve continuously

## See Also

- [Semantic Caching Guide](SEMANTIC_CACHING_GUIDE.md) - Reduce LLM API costs
- [Vector Database Integration](VECTOR_DATABASE_INTEGRATION.md) - Production scaling
- [Late Interaction Guide](LATE_INTERACTION_GUIDE.md) - ColBERT/ColPali reranking
- [Qdrant Integration Example](../examples/qdrant_real_integration.rs) - Production RAG pipeline
