//! Generative retrieval with learning-to-rank (LTRGR).
//!
//! This module implements generative retrieval, where autoregressive language models
//! generate identifiers (titles, substrings, pseudo-queries) for relevant passages.
//! LTRGR adds a learning-to-rank training phase to optimize passage ranking directly.
//!
//! # Overview
//!
//! Generative retrieval differs from traditional retrieval methods:
//! - **BM25/Dense**: Score passages directly
//! - **Generative**: Generate identifiers, then map to passages
//!
//! LTRGR bridges the gap between identifier generation and passage ranking by adding
//! a margin-based rank loss that optimizes passage ranking directly.
//!
//! # When to Use Generative Retrieval
//!
//! **Use generative retrieval when:**
//! - Researching novel retrieval paradigms
//! - Have access to autoregressive models (BART, T5)
//! - Need to experiment with identifier-based retrieval
//! - Building specialized retrieval systems
//!
//! **Do NOT use generative retrieval when:**
//! - Building RAG pipelines (use BM25 + dense)
//! - Need fast retrieval (generative is slower)
//! - Don't have autoregressive model infrastructure
//! - Need simple, well-understood retrieval
//!
//! **Performance characteristics:**
//! - Slower than BM25/dense (requires model inference)
//! - Higher quality for some queries (2-36% improvement over baseline)
//! - Requires external model integration
//! - More complex setup and training
//!
//! # Requirements
//!
//! - Autoregressive language model (BART, T5, etc.)
//! - Model must implement `AutoregressiveModel` trait
//! - Optional: FM-index for constrained generation
//! - Optional: LTR training pipeline for full LTRGR
//!
//! # Key Components
//!
//! - **`GenerativeRetriever`**: Main retriever that generates identifiers and scores passages
//! - **`IdentifierType`**: Enum for multiview identifiers (title, substring, pseudo-query)
//! - **`HeuristicScorer`**: Converts predicted identifiers to passage scores
//! - **`AutoregressiveModel`**: Trait for language models that generate identifiers
//! - **`LTRGRTrainer`**: Margin-based rank loss for learning-to-rank training
//!
//! # Example
//!
//! ```rust,no_run
//! use rank_retrieve::generative::{GenerativeRetriever, MockAutoregressiveModel};
//!
//! // Create retriever with model (implement AutoregressiveModel for your model)
//! let model = MockAutoregressiveModel::new();
//! let mut retriever = GenerativeRetriever::new(model);
//!
//! // Add documents
//! retriever.add_document(0, "Prime Rate in Canada is a guideline interest rate");
//! retriever.add_document(1, "Machine learning is a subset of AI");
//!
//! // Retrieve passages
//! let query = "What is prime rate in Canada?";
//! let results = retriever.retrieve(query, 100).unwrap();
//! ```

pub mod identifier;
pub mod ltrgr;
pub mod model;
pub mod scorer;

// Re-export key types for convenience
pub use identifier::{
    IdentifierGenerator, IdentifierType, MultiviewIdentifier, SimpleIdentifierGenerator,
};
pub use ltrgr::{LTRGRConfig, LTRGRTrainer};
pub use model::AutoregressiveModel;
pub use scorer::HeuristicScorer;

// Re-export MockAutoregressiveModel for testing (always available, not just in test mode)
pub use model::MockAutoregressiveModel;

/// Generative retriever that uses identifier generation for retrieval.
///
/// Follows the same pattern as other retrievers: stores passages internally
/// and retrieves based on query. Unlike BM25/Dense/Sparse which pre-compute
/// representations, generative retrieval needs passage text at retrieval time
/// to match generated identifiers.
pub struct GenerativeRetriever<M: AutoregressiveModel> {
    model: M,
    scorer: HeuristicScorer,
    beam_size: usize,
    /// Document ID -> Passage text
    passages: Vec<(u32, String)>,
}

impl<M: AutoregressiveModel> GenerativeRetriever<M> {
    /// Create a new generative retriever.
    pub fn new(model: M) -> Self {
        Self {
            model,
            scorer: HeuristicScorer::default(),
            beam_size: 15,
            passages: Vec::new(),
        }
    }

    /// Set the beam size for generation.
    ///
    /// # Arguments
    ///
    /// * `beam_size` - Number of candidate identifiers to generate per view (default: 15)
    ///
    /// # Performance
    ///
    /// Larger beam sizes improve recall but increase latency. Typical values: 10-20.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::generative::{GenerativeRetriever, MockAutoregressiveModel};
    ///
    /// let model = MockAutoregressiveModel::new();
    /// let retriever = GenerativeRetriever::new(model)
    ///     .with_beam_size(20);
    /// ```
    pub fn with_beam_size(mut self, beam_size: usize) -> Self {
        self.beam_size = beam_size;
        self
    }

    /// Set the heuristic scorer configuration.
    ///
    /// # Arguments
    ///
    /// * `scorer` - Heuristic scorer for matching identifiers to passages
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::generative::{GenerativeRetriever, MockAutoregressiveModel, HeuristicScorer};
    ///
    /// let model = MockAutoregressiveModel::new();
    /// let scorer = HeuristicScorer::default();
    /// let retriever = GenerativeRetriever::new(model)
    ///     .with_scorer(scorer);
    /// ```
    pub fn with_scorer(mut self, scorer: HeuristicScorer) -> Self {
        self.scorer = scorer;
        self
    }

    /// Add a document to the index.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document identifier (u32 to match other retrievers)
    /// * `passage_text` - Full passage text (needed for identifier matching)
    pub fn add_document(&mut self, doc_id: u32, passage_text: &str) {
        self.passages.push((doc_id, passage_text.to_string()));
    }

    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    ///
    /// * `query` - Query text
    /// * `k` - Number of documents to retrieve
    ///
    /// # Returns
    ///
    /// Vector of (document_id, score) pairs, sorted by score descending
    ///
    /// # Errors
    ///
    /// Returns `RetrieveError::EmptyQuery` if query is empty.
    /// Returns `RetrieveError::EmptyIndex` if index has no documents.
    pub fn retrieve(&self, query: &str, k: usize) -> Result<Vec<(u32, f32)>, crate::RetrieveError> {
        if query.trim().is_empty() {
            return Err(crate::RetrieveError::EmptyQuery);
        }

        if self.passages.is_empty() {
            return Err(crate::RetrieveError::EmptyIndex);
        }

        // Generate identifiers for all three views
        let mut all_identifiers = Vec::new();

        for identifier_type in [
            IdentifierType::Title,
            IdentifierType::Substring,
            IdentifierType::PseudoQuery,
        ] {
            let prefix = identifier_type.prefix();
            let identifiers = self.model.generate(
                query,
                prefix,
                self.beam_size,
                None, // No FM-index constraint for now
            )?;
            all_identifiers.extend(identifiers);
        }

        // Deduplicate identifiers (same identifier from different views)
        // Keep the highest-scoring instance of each identifier
        let mut deduplicated: std::collections::HashMap<String, f32> =
            std::collections::HashMap::new();
        for (identifier, score) in all_identifiers {
            deduplicated
                .entry(identifier)
                .and_modify(|existing_score| {
                    // Keep the maximum score for duplicate identifiers
                    if score > *existing_score {
                        *existing_score = score;
                    }
                })
                .or_insert(score);
        }
        let all_identifiers: Vec<(String, f32)> = deduplicated.into_iter().collect();

        // Score passages using heuristic function
        // For large corpora, use heap-based top-k selection (O(n log k) instead of O(n log n))
        // This is more efficient when k << n (e.g., k=10, n=10000)
        let passage_scores = if k < self.passages.len() / 10 && k > 0 {
            // Use min-heap for top-k (more efficient for large corpora)
            // We need a wrapper for f32 since it doesn't implement Ord
            #[derive(PartialEq, PartialOrd)]
            struct ScoreWrapper(f32);

            impl Eq for ScoreWrapper {}

            impl Ord for ScoreWrapper {
                fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                    self.0
                        .partial_cmp(&other.0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
            }

            use std::collections::BinaryHeap;

            // Min-heap that keeps only top-k elements
            // We use Reverse to make it a min-heap (smallest score at top)
            // Then we'll reverse the order at the end
            let mut heap: BinaryHeap<std::cmp::Reverse<(ScoreWrapper, u32)>> =
                BinaryHeap::with_capacity(k + 1);

            for (doc_id, passage_text) in &self.passages {
                let score = self.scorer.score_passage(passage_text, &all_identifiers);

                // Push to heap
                heap.push(std::cmp::Reverse((ScoreWrapper(score), *doc_id)));

                // Keep only top-k (remove smallest if heap size > k)
                if heap.len() > k {
                    heap.pop(); // Remove smallest
                }
            }

            // Extract and reverse to get descending order
            let mut result: Vec<(u32, f32)> = heap
                .into_iter()
                .map(|std::cmp::Reverse((ScoreWrapper(score), doc_id))| (doc_id, score))
                .collect();

            // Sort descending (heap gives us ascending, so reverse)
            result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            result
        } else {
            // For small k relative to n, full sort is more efficient
            let mut passage_scores: Vec<(u32, f32)> = self
                .passages
                .iter()
                .map(|(doc_id, passage_text)| {
                    let score = self.scorer.score_passage(passage_text, &all_identifiers);
                    (*doc_id, score)
                })
                .collect();

            passage_scores
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            passage_scores.truncate(k);
            passage_scores
        };

        Ok(passage_scores)
    }
}

#[cfg(test)]
mod tests {
    use super::model::MockAutoregressiveModel;
    use super::*;

    #[test]
    fn test_generative_retriever_basic() {
        let model = MockAutoregressiveModel::new();
        let mut retriever = GenerativeRetriever::new(model);

        retriever.add_document(0, "Prime Rate in Canada is a guideline interest rate");
        retriever.add_document(1, "Machine learning is a subset of artificial intelligence");

        let results = retriever.retrieve("What is prime rate?", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_generative_retriever_empty_query() {
        let model = MockAutoregressiveModel::new();
        let retriever = GenerativeRetriever::new(model);

        assert!(retriever.retrieve("", 10).is_err());
        assert!(retriever.retrieve("   ", 10).is_err());
    }

    #[test]
    fn test_generative_retriever_empty_index() {
        let model = MockAutoregressiveModel::new();
        let retriever = GenerativeRetriever::new(model);

        assert!(retriever.retrieve("test query", 10).is_err());
    }

    #[test]
    fn test_generative_retriever_builder() {
        let model = MockAutoregressiveModel::new();
        let scorer = HeuristicScorer::new().with_case_insensitive(false);
        let mut retriever = GenerativeRetriever::new(model)
            .with_beam_size(20)
            .with_scorer(scorer);

        retriever.add_document(0, "Test passage");
        let results = retriever.retrieve("test", 10).unwrap();
        assert!(!results.is_empty());
    }
}
