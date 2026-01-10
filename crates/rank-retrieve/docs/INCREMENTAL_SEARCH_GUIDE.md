# Incremental Search Guide

Guide for implementing incremental k-NN search patterns for recommendation systems and dynamic datasets.

## Overview

Incremental k-NN search addresses the challenge of maintaining search indexes when data is continuously updated. This is critical for recommendation systems where:
- **Data volume**: Billions of vectors processed
- **Display volume**: Only thousands shown to users
- **Update frequency**: Continuous additions and updates
- **Performance requirement**: Sub-second response times

## Challenge

Most vector databases require full index rebuilds for updates, which is impractical for:
- Real-time recommendation systems
- Streaming data pipelines
- Dynamic content platforms
- Live search systems

## Incremental Update Patterns

### Pattern 1: Append-Only Indexes

**Use case**: New data only, no deletions or updates

**Implementation**:
```rust
use rank_retrieve::dense::DenseRetriever;

struct IncrementalIndex {
    base_index: DenseRetriever,  // Pre-built index
    incremental_vectors: Vec<(u32, Vec<f32>)>,  // New vectors
    incremental_threshold: usize,  // Rebuild when threshold reached
}

impl IncrementalIndex {
    fn search(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        // Search base index
        let mut results = self.base_index.retrieve(query, k * 2)?;
        
        // Search incremental vectors (brute force for small set)
        for (id, vec) in &self.incremental_vectors {
            let dist = cosine_distance(query, vec);
            results.push((*id, dist));
        }
        
        // Sort and return top-k
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);
        results
    }
    
    fn add(&mut self, id: u32, vector: Vec<f32>) {
        self.incremental_vectors.push((id, vector));
        
        // Rebuild when threshold reached
        if self.incremental_vectors.len() >= self.incremental_threshold {
            self.rebuild();
        }
    }
    
    fn rebuild(&mut self) {
        // Add incremental vectors to base index
        for (id, vec) in self.incremental_vectors.drain(..) {
            self.base_index.add_document(id, vec)?;
        }
        self.base_index.build()?;
    }
}
```

**Trade-offs**:
- ✅ Simple implementation
- ✅ Fast for small incremental sets
- ⚠️ Performance degrades as incremental set grows
- ⚠️ Requires periodic rebuilds

### Pattern 2: Multi-Index Strategy

**Use case**: Large-scale systems with frequent updates

**Implementation**:
```rust
struct MultiIndexSystem {
    // Multiple indexes at different update frequencies
    daily_index: DenseRetriever,      // Rebuilt daily
    hourly_index: DenseRetriever,     // Rebuilt hourly
    realtime_vectors: Vec<(u32, Vec<f32>)>,  // Last hour's vectors
}

impl MultiIndexSystem {
    fn search(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        let mut all_results = Vec::new();
        
        // Search all indexes
        all_results.extend(self.daily_index.retrieve(query, k)?);
        all_results.extend(self.hourly_index.retrieve(query, k)?);
        
        // Search real-time vectors
        for (id, vec) in &self.realtime_vectors {
            let dist = cosine_distance(query, vec);
            all_results.push((*id, dist));
        }
        
        // Merge and deduplicate
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        for (id, score) in all_results {
            if !seen.contains(&id) {
                seen.insert(id);
                results.push((id, score));
            }
        }
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);
        results
    }
}
```

**Trade-offs**:
- ✅ Handles large update volumes
- ✅ Balances freshness and performance
- ⚠️ More complex to manage
- ⚠️ Requires coordination between indexes

### Pattern 3: Online Learning Indexes

**Use case**: Streaming data with concept drift

**Implementation**:
```rust
use rank_retrieve::dense::ivf_pq::OnlineProductQuantizer;

struct OnlineIndex {
    quantizer: OnlineProductQuantizer,
    vectors: Vec<(u32, Vec<u8>)>,  // Quantized vectors
}

impl OnlineIndex {
    fn add(&mut self, id: u32, vector: Vec<f32>) -> Result<(), RetrieveError> {
        // Update quantizer online
        let codes = self.quantizer.update(&vector)?;
        self.vectors.push((id, codes));
        Ok(())
    }
    
    fn search(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        let mut results = Vec::new();
        
        for (id, codes) in &self.vectors {
            let dist = self.quantizer.approximate_distance(query, codes);
            results.push((*id, dist));
        }
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);
        results
    }
}
```

**Trade-offs**:
- ✅ Adapts to changing data distributions
- ✅ No periodic rebuilds needed
- ⚠️ Requires careful tuning of learning rates
- ⚠️ May have slightly lower accuracy than batch methods

### Pattern 4: Delta Indexes

**Use case**: Large base corpus with small, frequent updates

**Implementation**:
```rust
struct DeltaIndex {
    base_index: IVFPQIndex,  // Large, stable index
    delta_index: DenseRetriever,  // Small, frequently updated
    deleted_ids: HashSet<u32>,  // Soft deletes
}

impl DeltaIndex {
    fn search(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
        // Search base index
        let mut base_results = self.base_index.search(query, k * 2)?;
        
        // Filter deleted IDs
        base_results.retain(|(id, _)| !self.deleted_ids.contains(id));
        
        // Search delta index
        let delta_results = self.delta_index.retrieve(query, k)?;
        
        // Merge results (delta takes precedence)
        let mut seen = HashSet::new();
        let mut results = Vec::new();
        
        // Add delta results first (newer data)
        for (id, score) in delta_results {
            seen.insert(id);
            results.push((id, score));
        }
        
        // Add base results (excluding duplicates and deleted)
        for (id, score) in base_results {
            if !seen.contains(&id) && !self.deleted_ids.contains(&id) {
                seen.insert(id);
                results.push((id, score));
            }
        }
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results.truncate(k);
        results
    }
    
    fn update(&mut self, id: u32, vector: Vec<f32>) {
        // Soft delete from base, add to delta
        self.deleted_ids.insert(id);
        self.delta_index.add_document(id, vector)?;
    }
    
    fn rebuild_base(&mut self) {
        // Periodically merge delta into base and rebuild
        // This is expensive, done during low-traffic periods
    }
}
```

**Trade-offs**:
- ✅ Efficient for large base + small updates
- ✅ Supports updates and deletes
- ⚠️ Requires periodic base rebuilds
- ⚠️ Soft deletes consume memory

## Recommendation System Patterns

### Pattern A: User-Item Recommendations

**Scenario**: Recommend items to users based on user-item interaction vectors

```rust
struct RecommendationSystem {
    user_index: IncrementalIndex,  // User embeddings
    item_index: IncrementalIndex,  // Item embeddings
}

impl RecommendationSystem {
    fn recommend(&self, user_id: u32, k: usize) -> Vec<(u32, f32)> {
        // Get user embedding
        let user_embedding = self.get_user_embedding(user_id)?;
        
        // Search item index
        self.item_index.search(&user_embedding, k)
    }
    
    fn update_user(&mut self, user_id: u32, interactions: &[u32]) {
        // Update user embedding based on new interactions
        let new_embedding = self.compute_user_embedding(interactions);
        self.user_index.add(user_id, new_embedding);
    }
    
    fn add_item(&mut self, item_id: u32, features: Vec<f32>) {
        self.item_index.add(item_id, features);
    }
}
```

### Pattern B: Real-Time Personalization

**Scenario**: Update recommendations based on recent user behavior

```rust
struct RealTimeRecommendation {
    base_index: IVFPQIndex,  // Pre-computed item embeddings
    user_history: HashMap<u32, VecDeque<u32>>,  // Recent interactions
    history_window: usize,  // Number of recent items to consider
}

impl RealTimeRecommendation {
    fn recommend(&self, user_id: u32, k: usize) -> Vec<(u32, f32)> {
        // Get recent interactions
        let recent_items = self.user_history.get(&user_id)
            .map(|h| h.iter().take(self.history_window).cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        
        // Compute query from recent items
        let query = self.compute_query_embedding(&recent_items);
        
        // Search base index
        self.base_index.search(&query, k)
    }
    
    fn record_interaction(&mut self, user_id: u32, item_id: u32) {
        let history = self.user_history.entry(user_id).or_insert_with(VecDeque::new);
        history.push_back(item_id);
        if history.len() > self.history_window {
            history.pop_front();
        }
    }
}
```

## Performance Optimization

### 1. Batch Updates

Update indexes in batches rather than one-by-one:

```rust
fn batch_update(&mut self, updates: Vec<(u32, Vec<f32>)>) {
    // Collect updates
    for (id, vec) in updates {
        self.pending_updates.push((id, vec));
    }
    
    // Apply in batch when threshold reached
    if self.pending_updates.len() >= 1000 {
        self.apply_batch_updates();
    }
}
```

### 2. Asynchronous Rebuilds

Perform expensive rebuilds asynchronously:

```rust
use std::sync::Arc;
use std::sync::Mutex;

struct AsyncIndex {
    current_index: Arc<Mutex<DenseRetriever>>,
    rebuild_queue: Arc<Mutex<Vec<(u32, Vec<f32>)>>>,
}

impl AsyncIndex {
    fn start_rebuild_worker(&self) {
        // Background thread for rebuilds
        std::thread::spawn(move || {
            loop {
                let updates = self.rebuild_queue.lock().unwrap().drain(..).collect::<Vec<_>>();
                if !updates.is_empty() {
                    let new_index = self.build_index(updates);
                    *self.current_index.lock().unwrap() = new_index;
                }
                std::thread::sleep(Duration::from_secs(60)); // Rebuild every minute
            }
        });
    }
}
```

### 3. Incremental Index Selection

Choose index type based on update pattern:

- **Low update rate** (< 1% per day): Use standard indexes with periodic rebuilds
- **Medium update rate** (1-10% per day): Use append-only or delta indexes
- **High update rate** (> 10% per day): Use online learning indexes

## Best Practices

1. **Monitor index freshness**: Track time since last update
2. **Balance accuracy and latency**: More frequent updates = better accuracy but higher latency
3. **Use appropriate indexes**: Match index type to update pattern
4. **Batch operations**: Group updates to reduce overhead
5. **Async processing**: Perform expensive operations in background
6. **Graceful degradation**: Handle index rebuilds without blocking queries

## Limitations

Current `rank-retrieve` implementations:
- ✅ Support incremental additions (append-only)
- ⚠️ Limited support for updates/deletes (requires rebuild)
- ⚠️ No built-in incremental search patterns (implement custom)

**For production systems requiring full incremental support**:
- Consider vector databases with native incremental support (Qdrant, Milvus)
- Or implement custom incremental patterns using `rank-retrieve` as building blocks

## See Also

- [Vector Database Integration](VECTOR_DATABASE_INTEGRATION.md) - Production scaling
- [Online Product Quantization](../src/dense/ivf_pq/online_pq.rs) - Online learning for dynamic datasets
- [RAG Guide](RAG_GUIDE.md) - Retrieval-augmented generation patterns
