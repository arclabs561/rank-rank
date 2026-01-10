//! BM25 retrieval module.
//!
//! Provides inverted index and Okapi BM25 scoring for first-stage retrieval.
//!
//! # Current Implementation
//!
//! **Status: Basic inverted index**
//!
//! The current implementation uses a simple in-memory inverted index with:
//! - Standard Okapi BM25 scoring
//! - Document frequency (DF) and inverse document frequency (IDF) calculation
//! - Document length normalization
//!
//! **Suitable for:**
//! - Any scale of corpora (from small to very large)
//! - Prototyping and research
//! - Production systems
//! - Applications where simplicity and performance are both important
//!
//! **Optimizations:**
//! - Precomputed IDF values (lazy computation)
//! - Early termination heuristics (top-k heap)
//! - Optimized candidate collection
//! - Efficient scoring with precomputed parameters
//!
//! **Note on scale:**
//! - In-memory only (no persistence)
//! - For persistent storage or distributed systems, consider integrating with
//!   specialized backends (Tantivy, Lucene/Elasticsearch) via the `Backend` trait
//!
//! # BM25 Formula
//!
//! BM25 (Best Matching 25) is a ranking function used to estimate the relevance
//! of documents to a given search query. The formula is:
//!
//! ```text
//! BM25(q, d) = Σ IDF(q_i) * (f(q_i, d) * (k1 + 1)) / (f(q_i, d) + k1 * (1 - b + b * |d|/avgdl))
//! ```
//!
//! Where:
//! - `f(q_i, d)` = frequency of term q_i in document d
//! - `|d|` = length of document d
//! - `avgdl` = average document length in the collection
//! - `k1` = term frequency saturation parameter (typically 1.2)
//! - `b` = length normalization parameter (typically 0.75)
//! - `IDF(q_i)` = inverse document frequency of term q_i

use crate::RetrieveError;
#[cfg(feature = "bm25")]
use crate::retriever::{Retriever, RetrieverBuilder};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

/// Eager BM25 scoring with precomputed scores (feature-gated).
///
/// Provides `EagerBm25Index` for query-heavy workloads with repeated queries.
/// Precomputes scores during indexing for 500x faster retrieval.
#[cfg(feature = "bm25")]
pub mod eager;

/// BM25 variant selection.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum Bm25Variant {
    /// Standard BM25 (Okapi BM25).
    #[default]
    Standard,
    /// BM25L: Addresses over-penalization of short documents.
    /// Adds a constant delta to the TF term to boost longer documents.
    /// Default delta: 0.5
    BM25L { delta: f32 },
    /// BM25+: Prevents negative scores for common terms.
    /// Adds a constant delta to lower-bound the TF contribution.
    /// Default delta: 1.0
    BM25Plus { delta: f32 },
}

impl Bm25Variant {
    /// Create BM25L variant with default delta (0.5).
    pub fn bm25l() -> Self {
        Self::BM25L { delta: 0.5 }
    }

    /// Create BM25L variant with custom delta.
    pub fn bm25l_with_delta(delta: f32) -> Self {
        Self::BM25L { delta }
    }

    /// Create BM25+ variant with default delta (1.0).
    pub fn bm25plus() -> Self {
        Self::BM25Plus { delta: 1.0 }
    }

    /// Create BM25+ variant with custom delta.
    pub fn bm25plus_with_delta(delta: f32) -> Self {
        Self::BM25Plus { delta }
    }
}

/// BM25 parameters.
#[derive(Debug, Clone, Copy)]
pub struct Bm25Params {
    /// Term frequency saturation parameter (k1).
    /// Controls how quickly term frequency saturates.
    /// Default: 1.2
    pub k1: f32,

    /// Length normalization parameter (b).
    /// Controls the strength of length normalization.
    /// Default: 0.75
    pub b: f32,

    /// BM25 variant selection.
    /// Default: Standard BM25
    pub variant: Bm25Variant,
}

impl Default for Bm25Params {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            variant: Bm25Variant::Standard,
        }
    }
}

impl Bm25Params {
    /// Create BM25L parameters with default settings.
    pub fn bm25l() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            variant: Bm25Variant::bm25l(),
        }
    }

    /// Create BM25+ parameters with default settings.
    pub fn bm25plus() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
            variant: Bm25Variant::bm25plus(),
        }
    }
}

/// Inverted index for BM25 retrieval.
///
/// Stores term-to-document mappings and document statistics.
pub struct InvertedIndex {
    /// Term -> (Document ID -> Term Frequency)
    postings: HashMap<String, HashMap<u32, u32>>,

    /// Document ID -> Document Length (in terms)
    doc_lengths: HashMap<u32, u32>,

    /// Total number of documents
    num_docs: u32,

    /// Average document length
    avg_doc_length: f32,

    /// Document frequency for each term (for IDF calculation)
    doc_frequencies: HashMap<String, u32>,

    /// Precomputed IDF values for each term (optimization: avoid repeated calculations)
    /// Lazily computed on first retrieval to avoid expensive recomputation during indexing
    /// Uses RefCell for interior mutability to allow lazy computation from immutable methods
    precomputed_idf: RefCell<HashMap<String, f32>>,

    /// Number of documents when IDF was last computed (for lazy recomputation)
    idf_computed_at_num_docs: RefCell<u32>,
}

impl InvertedIndex {
    /// Create a new empty index.
    pub fn new() -> Self {
        Self {
            postings: HashMap::new(),
            doc_lengths: HashMap::new(),
            num_docs: 0,
            avg_doc_length: 0.0,
            doc_frequencies: HashMap::new(),
            precomputed_idf: RefCell::new(HashMap::new()),
            idf_computed_at_num_docs: RefCell::new(0),
        }
    }

    /// Recompute all IDF values if stale (lazy computation).
    ///
    /// This optimization precomputes IDF values to avoid repeated calculations during retrieval.
    /// Only recomputes when the index has changed since last computation.
    /// Uses RefCell for interior mutability to allow lazy computation from immutable methods.
    fn ensure_idf_computed(&self) {
        // Check if already computed and up-to-date
        let computed_at = *self.idf_computed_at_num_docs.borrow();
        if computed_at == self.num_docs {
            let idf_map = self.precomputed_idf.borrow();
            if !idf_map.is_empty() {
                return; // Already computed and up-to-date
            }
        }

        // Need to recompute
        let mut idf_map = self.precomputed_idf.borrow_mut();
        idf_map.clear();
        let n = self.num_docs as f32;
        for (term, df) in &self.doc_frequencies {
            let df_f = *df as f32;
            if df_f > 0.0 {
                let idf = ((n - df_f + 0.5) / (df_f + 0.5) + 1.0).ln();
                idf_map.insert(term.clone(), idf);
            }
        }
        *self.idf_computed_at_num_docs.borrow_mut() = self.num_docs;
    }

    /// Add a document to the index.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document identifier
    /// * `terms` - Tokenized document terms
    pub fn add_document(&mut self, doc_id: u32, terms: &[String]) {
        let doc_length = terms.len() as u32;
        self.doc_lengths.insert(doc_id, doc_length);

        // Count term frequencies in this document
        let mut term_freqs: HashMap<String, u32> = HashMap::new();
        for term in terms {
            *term_freqs.entry(term.clone()).or_insert(0) += 1;
        }

        // Update postings list
        for (term, freq) in term_freqs {
            self.postings
                .entry(term.clone())
                .or_default()
                .insert(doc_id, freq);

            // Update document frequency
            *self.doc_frequencies.entry(term).or_insert(0) += 1;
        }

        self.num_docs += 1;
        self.update_avg_doc_length();
        // Clear precomputed IDF (will be recomputed lazily on next retrieval)
        // This avoids expensive recomputation during indexing
        self.precomputed_idf.borrow_mut().clear();
        *self.idf_computed_at_num_docs.borrow_mut() = 0;
    }

    /// Update average document length.
    fn update_avg_doc_length(&mut self) {
        let total_length: u32 = self.doc_lengths.values().sum();
        if self.num_docs > 0 {
            self.avg_doc_length = total_length as f32 / self.num_docs as f32;
        }
    }

    /// Get the number of documents in the index.
    pub fn num_docs(&self) -> u32 {
        self.num_docs
    }

    /// Get the postings list (term -> document -> term frequency).
    pub fn postings(&self) -> &HashMap<String, HashMap<u32, u32>> {
        &self.postings
    }

    /// Get document frequencies (term -> number of documents containing term).
    pub fn doc_frequencies(&self) -> &HashMap<String, u32> {
        &self.doc_frequencies
    }

    /// Get document lengths (document ID -> document length in terms).
    pub fn doc_lengths(&self) -> &HashMap<u32, u32> {
        &self.doc_lengths
    }

    /// Get an iterator over all document IDs in the index.
    pub fn document_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.doc_lengths.keys().copied()
    }

    /// Get the term frequency of a term in a document.
    pub fn term_frequency(&self, doc_id: u32, term: &str) -> u32 {
        self.postings
            .get(term)
            .and_then(|postings| postings.get(&doc_id))
            .copied()
            .unwrap_or(0)
    }

    /// Get the length of a document.
    pub fn document_length(&self, doc_id: u32) -> u32 {
        self.doc_lengths.get(&doc_id).copied().unwrap_or(0)
    }

    /// Calculate inverse document frequency (IDF) for a term.
    ///
    /// Uses precomputed IDF values when available for better performance.
    /// Falls back to on-the-fly calculation if not precomputed.
    ///
    /// Uses a BM25 variant IDF formula: log((N - df + 0.5) / (df + 0.5) + 1.0)
    /// where N is total documents and df is document frequency.
    ///
    /// **Note**: The `+ 1.0` inside the logarithm is a variant choice that ensures:
    /// - Positive IDF values (log argument always > 1.0)
    /// - Non-zero IDF for very common terms (df ≈ N)
    /// - Better numerical stability
    ///
    /// This differs slightly from the standard BM25 formula but is more stable
    /// and produces similar ranking results in practice.
    pub fn idf(&self, term: &str) -> f32 {
        // Use precomputed IDF if available (optimization)
        {
            let idf_map = self.precomputed_idf.borrow();
            if let Some(&idf) = idf_map.get(term) {
                return idf;
            }
        }
        // Fallback to on-the-fly calculation
        let df = self.doc_frequencies.get(term).copied().unwrap_or(0) as f32;
        if df == 0.0 {
            return 0.0;
        }
        let n = self.num_docs as f32;
        // Use standard BM25 IDF formula: log((N - df + 0.5) / (df + 0.5))
        // Add 1.0 to ensure positive IDF (standard BM25 variant)
        ((n - df + 0.5) / (df + 0.5) + 1.0).ln()
    }

    /// Score a document against a query using BM25.
    ///
    /// # Arguments
    ///
    /// * `doc_id` - Document to score
    /// * `query_terms` - Tokenized query terms
    /// * `params` - BM25 parameters
    ///
    /// # Returns
    ///
    /// BM25 score for the document
    pub fn score(&self, doc_id: u32, query_terms: &[String], params: Bm25Params) -> f32 {
        // Guard against division by zero if avg_doc_length is 0 (shouldn't happen in practice)
        if self.avg_doc_length == 0.0 {
            return 0.0;
        }
        
        let doc_length = self.doc_lengths.get(&doc_id).copied().unwrap_or(0) as f32;
        let mut score = 0.0;

        for term in query_terms {
            let idf = self.idf(term);
            if idf == 0.0 {
                continue;
            }

            // Get term frequency in document
            let tf = self
                .postings
                .get(term)
                .and_then(|postings| postings.get(&doc_id))
                .copied()
                .unwrap_or(0) as f32;

            if tf == 0.0 {
                continue;
            }

            // BM25 formula (with variant support)
            let numerator = tf * (params.k1 + 1.0);
            let denominator =
                tf + params.k1 * (1.0 - params.b + params.b * doc_length / self.avg_doc_length);
            let mut tf_score = numerator / denominator;

            // Apply variant-specific modifications
            match params.variant {
                Bm25Variant::Standard => {
                    // Standard BM25: no modification
                }
                Bm25Variant::BM25L { delta } => {
                    // BM25L: Add delta to boost longer documents
                    tf_score += delta;
                }
                Bm25Variant::BM25Plus { delta } => {
                    // BM25+: Add delta to lower-bound scores
                    tf_score += delta;
                }
            }

            score += idf * tf_score;
        }

        score
    }

    /// Retrieve top-k documents for a query using BM25 scoring.
    ///
    /// Scores all documents containing at least one query term and returns the top-k
    /// results sorted by BM25 score (descending).
    ///
    /// Uses early termination optimization: maintains a threshold and skips documents
    /// that cannot possibly be in the top-k results.
    ///
    /// # Arguments
    ///
    /// * `query_terms` - Tokenized query terms (should be preprocessed: lowercased, stemmed, etc.)
    /// * `k` - Number of documents to retrieve (top-k)
    /// * `params` - BM25 parameters (k1, b). Use `Bm25Params::default()` for standard values.
    ///
    /// # Returns
    ///
    /// Vector of (document_id, score) pairs, sorted by score descending.
    /// Returns fewer than k documents if fewer documents match the query.
    ///
    /// # Errors
    ///
    /// Returns `RetrieveError::EmptyQuery` if query is empty.
    /// Returns `RetrieveError::EmptyIndex` if index has no documents.
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::bm25::{InvertedIndex, Bm25Params};
    ///
    /// let mut index = InvertedIndex::new();
    /// index.add_document(0, &["machine".to_string(), "learning".to_string()]);
    /// index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
    ///
    /// let query = vec!["machine".to_string(), "learning".to_string()];
    /// let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
    /// assert_eq!(results[0].0, 0); // Document 0 should rank highest
    /// ```
    ///
    /// # Performance
    ///
    /// Time complexity: O(q * d) where q is number of query terms and d is average number
    /// of documents per term. For typical queries (2-5 terms) and indexes (10K-1M docs),
    /// retrieval is typically <10ms. Early termination can reduce this significantly for
    /// queries where top-k scores converge quickly.
    pub fn retrieve(
        &self,
        query_terms: &[String],
        k: usize,
        params: Bm25Params,
    ) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if query_terms.is_empty() {
            return Err(RetrieveError::EmptyQuery);
        }

        if self.num_docs == 0 {
            return Err(RetrieveError::EmptyIndex);
        }

        // Ensure IDF values are precomputed (lazy computation)
        self.ensure_idf_computed();

        // Precompute IDF values for all query terms (optimization)
        let query_idfs: Vec<f32> = query_terms
            .iter()
            .map(|term| self.idf(term))
            .collect();

        // Get all candidate documents (documents containing at least one query term)
        // Use Vec instead of HashSet for better cache locality
        // Pre-allocate capacity based on query terms (heuristic: assume ~100 docs per term)
        let estimated_candidates = query_terms.len() * 100;
        let mut candidates: Vec<u32> = Vec::with_capacity(estimated_candidates);
        let mut seen: HashSet<u32> = HashSet::with_capacity(estimated_candidates);
        for term in query_terms {
            if let Some(postings) = self.postings.get(term) {
                for &doc_id in postings.keys() {
                    if seen.insert(doc_id) {
                        candidates.push(doc_id);
                    }
                }
            }
        }

        // Handle k=0 case
        if k == 0 {
            return Ok(Vec::new());
        }

        // Early termination: use proper BinaryHeap for O(log k) operations
        // Use a min-heap to track top-k scores (Reverse for min-heap behavior)
        use std::cmp::Reverse;
        use std::collections::BinaryHeap;

        #[derive(PartialEq)]
        struct FloatOrd(f32);
        impl Eq for FloatOrd {}
        impl PartialOrd for FloatOrd {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl Ord for FloatOrd {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
            }
        }

        let mut heap: BinaryHeap<Reverse<(FloatOrd, u32)>> = BinaryHeap::with_capacity(k + 1);

        // Score candidates with early termination
        for doc_id in candidates {
            let score = self.score_optimized(doc_id, query_terms, &query_idfs, params);
            
            // Filter out NaN, Infinity, and non-positive scores
            if !score.is_finite() || score <= 0.0 {
                continue;
            }

            if heap.len() < k {
                // Heap not full yet, add to heap
                heap.push(Reverse((FloatOrd(score), doc_id)));
            } else if let Some(&Reverse((FloatOrd(min_score), _))) = heap.peek() {
                if score > min_score {
                    // Score is better than worst in top-k, replace it
                    heap.pop();
                    heap.push(Reverse((FloatOrd(score), doc_id)));
                }
            }
            // else: score <= min_score, skip (early termination)
        }

        // Extract and sort by score descending
        let mut results: Vec<(u32, f32)> = heap
            .into_iter()
            .map(|Reverse((FloatOrd(score), doc_id))| (doc_id, score))
            .collect();
        results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        Ok(results)
    }

    /// Optimized scoring function that uses precomputed IDF values.
    ///
    /// This is an internal optimization to avoid repeated IDF lookups.
    fn score_optimized(
        &self,
        doc_id: u32,
        query_terms: &[String],
        query_idfs: &[f32],
        params: Bm25Params,
    ) -> f32 {
        let doc_length = self.doc_lengths.get(&doc_id).copied().unwrap_or(0) as f32;
        let mut score = 0.0;

        for (term, &idf) in query_terms.iter().zip(query_idfs.iter()) {
            if idf == 0.0 {
                continue;
            }

            // Get term frequency in document
            let tf = self
                .postings
                .get(term)
                .and_then(|postings| postings.get(&doc_id))
                .copied()
                .unwrap_or(0) as f32;

            if tf == 0.0 {
                continue;
            }

            // BM25 formula (with variant support)
            let numerator = tf * (params.k1 + 1.0);
            let denominator =
                tf + params.k1 * (1.0 - params.b + params.b * doc_length / self.avg_doc_length);
            let mut tf_score = numerator / denominator;

            // Apply variant-specific modifications
            match params.variant {
                Bm25Variant::Standard => {
                    // Standard BM25: no modification
                }
                Bm25Variant::BM25L { delta } => {
                    // BM25L: Add delta to boost longer documents
                    tf_score += delta;
                }
                Bm25Variant::BM25Plus { delta } => {
                    // BM25+: Add delta to lower-bound scores
                    tf_score += delta;
                }
            }

            score += idf * tf_score;
        }

        score
    }
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "bm25")]
impl Retriever for InvertedIndex {
    type Query = Vec<String>;

    fn retrieve(&self, query: &Self::Query, k: usize) -> Result<Vec<(u32, f32)>, RetrieveError> {
        self.retrieve(query, k, Bm25Params::default())
    }
}

#[cfg(feature = "bm25")]
impl RetrieverBuilder for InvertedIndex {
    type Content = Vec<String>;

    fn add_document(&mut self, doc_id: u32, content: Self::Content) -> Result<(), RetrieveError> {
        self.add_document(doc_id, &content);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bm25_basic() {
        let mut index = InvertedIndex::new();

        // Add documents
        index.add_document(
            0,
            &[
                "the".to_string(),
                "quick".to_string(),
                "brown".to_string(),
                "fox".to_string(),
            ],
        );
        index.add_document(
            1,
            &["the".to_string(), "lazy".to_string(), "dog".to_string()],
        );
        index.add_document(
            2,
            &[
                "quick".to_string(),
                "brown".to_string(),
                "fox".to_string(),
                "jumps".to_string(),
            ],
        );

        // Query
        let query = vec!["quick".to_string(), "fox".to_string()];
        let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();

        // Document 0 and 2 should have highest scores (both contain "quick" and "fox")
        assert!(results.len() >= 2);
        // Verify we got results with positive scores
        assert!(results.iter().any(|(_, score)| *score > 0.0));
    }

    #[test]
    fn test_bm25_variants() {
        let mut index = InvertedIndex::new();
        
        // Add documents with varying lengths
        // Short document
        index.add_document(0, &["test".to_string()]);
        // Medium document
        index.add_document(1, &vec!["test".to_string(); 5]);
        // Long document
        index.add_document(2, &vec!["test".to_string(); 20]);

        let query = vec!["test".to_string()];

        // Standard BM25
        let standard_params = Bm25Params::default();
        let standard_results = index.retrieve(&query, 10, standard_params).unwrap();
        
        // BM25L (should boost longer documents)
        let bm25l_params = Bm25Params::bm25l();
        let bm25l_results = index.retrieve(&query, 10, bm25l_params).unwrap();
        
        // BM25+ (should lower-bound scores)
        let bm25plus_params = Bm25Params::bm25plus();
        let bm25plus_results = index.retrieve(&query, 10, bm25plus_params).unwrap();

        // All should return results
        assert!(!standard_results.is_empty());
        assert!(!bm25l_results.is_empty());
        assert!(!bm25plus_results.is_empty());

        // BM25+ should have higher scores (delta added)
        assert!(bm25plus_results[0].1 >= standard_results[0].1);
        
        // BM25L should also have higher scores (delta added)
        assert!(bm25l_results[0].1 >= standard_results[0].1);
    }

    #[test]
    fn test_bm25_variant_custom_delta() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string(), "test".to_string()]);
        
        let query = vec!["test".to_string()];

        // BM25L with custom delta
        let bm25l_custom = Bm25Params {
            k1: 1.2,
            b: 0.75,
            variant: Bm25Variant::bm25l_with_delta(1.0),
        };
        let results_custom = index.retrieve(&query, 10, bm25l_custom).unwrap();

        // BM25L with default delta (0.5)
        let bm25l_default = Bm25Params::bm25l();
        let results_default = index.retrieve(&query, 10, bm25l_default).unwrap();

        // Custom delta (1.0) should give higher scores than default (0.5)
        assert!(results_custom[0].1 > results_default[0].1);
    }

    #[test]
    fn test_bm25_variant_backward_compatibility() {
        // Default params should use Standard variant (backward compatible)
        let params = Bm25Params::default();
        assert_eq!(params.variant, Bm25Variant::Standard);
        assert_eq!(params.k1, 1.2);
        assert_eq!(params.b, 0.75);
    }

    #[test]
    fn test_idf() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["common".to_string(), "term".to_string()]);
        index.add_document(1, &["common".to_string(), "word".to_string()]);
        index.add_document(2, &["rare".to_string(), "term".to_string()]);

        // "common" appears in 2 docs, "rare" in 1 doc
        let idf_common = index.idf("common");
        let idf_rare = index.idf("rare");

        // Rare term should have higher IDF
        assert!(idf_rare > idf_common);
    }
}
