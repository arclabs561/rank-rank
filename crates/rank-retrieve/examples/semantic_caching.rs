//! Example: Semantic Caching for LLM Applications
//!
//! This demonstrates how to use `rank-retrieve` as a semantic cache to reduce
//! LLM API costs and improve response times.
//!
//! **What is Semantic Caching?**
//! Semantic caching stores query embeddings and pre-generated LLM responses.
//! When a semantically similar query arrives, the cached response is returned
//! instead of calling the LLM API, reducing costs and latency.
//!
//! **Key Components:**
//! 1. **Embedding Generation**: Convert queries to embeddings
//! 2. **In-Memory Caching**: Store embeddings and responses
//! 3. **Similarity Search**: Find semantically similar queries
//!
//! **Benefits:**
//! - **Cost Reduction**: Avoid redundant LLM API calls
//! - **Latency Improvement**: Cache hits are much faster than LLM calls
//! - **Scalability**: Handle large volumes of queries efficiently
//!
//! **When to Use:**
//! - High query volume with semantic similarity
//! - Cost-sensitive applications
//! - Latency-critical systems
//!
//! **Performance:**
//! - Cache hit: ~1-5ms (similarity search)
//! - Cache miss: ~100-1000ms (LLM API call)
//! - Cost savings: 50-90% for similar queries

use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::retrieve_dense;
use std::collections::HashMap;

/// Semantic cache entry storing query embedding and response.
struct CacheEntry {
    query_embedding: Vec<f32>,
    response: String,
    metadata: HashMap<String, String>,
}

/// Semantic cache using dense retrieval for similarity matching.
pub struct SemanticCache {
    retriever: DenseRetriever,
    entries: HashMap<u32, CacheEntry>,
    next_id: u32,
    similarity_threshold: f32,
}

impl SemanticCache {
    /// Create new semantic cache.
    ///
    /// # Arguments
    /// * `similarity_threshold` - Minimum cosine similarity for cache hit (0.0-1.0)
    ///   - Higher threshold: More strict matching (fewer false positives)
    ///   - Lower threshold: More lenient matching (more cache hits)
    ///   - Recommended: 0.85-0.95 for most use cases
    pub fn new(similarity_threshold: f32) -> Self {
        Self {
            retriever: DenseRetriever::new(),
            entries: HashMap::new(),
            next_id: 0,
            similarity_threshold,
        }
    }
    
    /// Check cache for semantically similar query.
    ///
    /// Returns cached response if similar query found, None otherwise.
    pub fn get(&self, query_embedding: &[f32]) -> Option<&CacheEntry> {
        // Search for similar queries
        let results = retrieve_dense(&self.retriever, query_embedding, 1)
            .ok()?;
        
        // Check if top result meets similarity threshold
        if let Some((id, similarity)) = results.first() {
            // Convert distance to similarity (cosine similarity: 1.0 - distance)
            let similarity_score = 1.0 - similarity;
            
            if similarity_score >= self.similarity_threshold {
                return self.entries.get(id);
            }
        }
        
        None
    }
    
    /// Store query and response in cache.
    ///
    /// # Arguments
    /// * `query_embedding` - Query embedding (should be L2-normalized)
    /// * `response` - LLM-generated response
    /// * `metadata` - Optional metadata (timestamp, model, etc.)
    pub fn put(
        &mut self,
        query_embedding: Vec<f32>,
        response: String,
        metadata: HashMap<String, String>,
    ) {
        let id = self.next_id;
        self.next_id += 1;
        
        // Store in retriever for similarity search
        self.retriever.add_document(id, query_embedding.clone());
        
        // Store entry
        self.entries.insert(id, CacheEntry {
            query_embedding,
            response,
            metadata,
        });
    }
    
    /// Get cache statistics.
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            num_entries: self.entries.len(),
            similarity_threshold: self.similarity_threshold,
        }
    }
}

/// Cache statistics.
pub struct CacheStats {
    pub num_entries: usize,
    pub similarity_threshold: f32,
}

/// LLM client interface (mock for example).
trait LLMClient {
    fn generate(&self, query: &str) -> String;
}

/// Mock LLM client for demonstration.
struct MockLLMClient;

impl LLMClient for MockLLMClient {
    fn generate(&self, query: &str) -> String {
        // Simulate LLM API call latency
        std::thread::sleep(std::time::Duration::from_millis(100));
        
        format!("LLM response for: {}", query)
    }
}

/// RAG system with semantic caching.
pub struct CachedRAGSystem {
    cache: SemanticCache,
    llm_client: Box<dyn LLMClient>,
    embedding_model: Box<dyn Fn(&str) -> Vec<f32>>,
}

impl CachedRAGSystem {
    /// Create new cached RAG system.
    pub fn new(
        similarity_threshold: f32,
        llm_client: Box<dyn LLMClient>,
        embedding_model: Box<dyn Fn(&str) -> Vec<f32>>,
    ) -> Self {
        Self {
            cache: SemanticCache::new(similarity_threshold),
            llm_client,
            embedding_model,
        }
    }
    
    /// Process query with semantic caching.
    ///
    /// Returns (response, was_cached).
    pub fn query(&mut self, query: &str) -> (String, bool) {
        // Generate query embedding
        let query_embedding = (self.embedding_model)(query);
        
        // Check cache
        if let Some(entry) = self.cache.get(&query_embedding) {
            println!("Cache HIT: Similar query found (similarity >= {:.2})", 
                     self.cache.similarity_threshold);
            return (entry.response.clone(), true);
        }
        
        println!("Cache MISS: Generating new response");
        
        // Generate response from LLM
        let response = self.llm_client.generate(query);
        
        // Store in cache
        let mut metadata = HashMap::new();
        metadata.insert("timestamp".to_string(), 
                       std::time::SystemTime::now()
                           .duration_since(std::time::UNIX_EPOCH)
                           .unwrap()
                           .as_secs()
                           .to_string());
        metadata.insert("model".to_string(), "gpt-4".to_string());
        
        self.cache.put(query_embedding, response.clone(), metadata);
        
        (response, false)
    }
    
    /// Get cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
}

fn main() {
    println!("=== Semantic Caching Example ===\n");
    
    // Mock embedding model (in production, use real embedding model)
    let embedding_model = Box::new(|query: &str| -> Vec<f32> {
        // Simple hash-based embedding for demo (use real model in production)
        let mut embedding = vec![0.0; 384];
        for (i, byte) in query.as_bytes().iter().enumerate() {
            embedding[i % 384] += *byte as f32 / 255.0;
        }
        // L2 normalize
        let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        embedding.iter().map(|x| x / norm).collect()
    });
    
    // Create cached RAG system
    let mut system = CachedRAGSystem::new(
        0.90, // 90% similarity threshold
        Box::new(MockLLMClient),
        embedding_model,
    );
    
    // Simulate queries
    let queries = vec![
        "What is machine learning?",
        "Explain machine learning",
        "Tell me about ML",
        "What is artificial intelligence?",
        "Explain AI",
    ];
    
    let mut cache_hits = 0;
    let mut cache_misses = 0;
    
    for query in queries {
        println!("\nQuery: {}", query);
        let (response, was_cached) = system.query(query);
        
        if was_cached {
            cache_hits += 1;
            println!("Response (cached): {}", response);
        } else {
            cache_misses += 1;
            println!("Response (new): {}", response);
        }
    }
    
    // Print statistics
    println!("\n=== Cache Statistics ===");
    let stats = system.cache_stats();
    println!("Total entries: {}", stats.num_entries);
    println!("Cache hits: {}", cache_hits);
    println!("Cache misses: {}", cache_misses);
    println!("Hit rate: {:.1}%", 
             (cache_hits as f32 / (cache_hits + cache_misses) as f32) * 100.0);
    
    // Cost savings estimate
    let estimated_cost_per_query = 0.002; // $0.002 per LLM query
    let cost_saved = cache_hits as f32 * estimated_cost_per_query;
    println!("Estimated cost saved: ${:.4}", cost_saved);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_semantic_cache() {
        let mut cache = SemanticCache::new(0.85);
        
        // Add entry
        let embedding1 = vec![1.0, 0.0, 0.0];
        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "test".to_string());
        cache.put(embedding1.clone(), "Response 1".to_string(), metadata);
        
        // Similar query should hit cache
        let similar_embedding = vec![0.99, 0.01, 0.0];
        assert!(cache.get(&similar_embedding).is_some());
        
        // Different query should miss cache
        let different_embedding = vec![0.0, 1.0, 0.0];
        assert!(cache.get(&different_embedding).is_none());
    }
}
