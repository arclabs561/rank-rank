//! Query likelihood language model retrieval module.
//!
//! Provides probabilistic retrieval using language models, ranking documents by
//! the probability that the document's language model generated the query: P(Q|D).
//!
//! # Current Implementation
//!
//! **Status: Basic query likelihood with smoothing**
//!
//! The current implementation provides:
//! - Jelinek-Mercer smoothing: Interpolates document and corpus language models
//! - Dirichlet smoothing: Bayesian approach with automatic length adaptation
//! - Log-probability scoring to avoid numerical underflow
//!
//! **Suitable for:**
//! - Research/prototyping scenarios
//! - Queries where probabilistic approach helps
//! - When theoretical grounding is important
//! - As a baseline for comparison with BM25/TF-IDF
//!
//! **Limitations:**
//! - Basic unigram model (no n-grams or skip-grams)
//! - No document priors (authority, genre, etc.)
//! - In-memory only (no persistence)
//!
//! # Query Likelihood Model
//!
//! Query likelihood ranks documents by:
//!
//! ```text
//! score(Q, D) = P(Q|D) = ∏ P(q_i|D)
//! ```
//!
//! Where `P(q_i|D)` is the probability of term `q_i` in document D's language model.
//!
//! # Smoothing Techniques
//!
//! ## Jelinek-Mercer Smoothing
//!
//! Interpolates between document and corpus language models:
//!
//! ```text
//! P(q_i|D) = λ * P(q_i|D) + (1 - λ) * P(q_i|C)
//! ```
//!
//! Where `λ` controls the balance (default: 0.5).
//!
//! ## Dirichlet Smoothing
//!
//! Uses Bayesian approach with automatic length adaptation:
//!
//! ```text
//! P(q_i|D) = (c(q_i, D) + μ * P(q_i|C)) / (|D| + μ)
//! ```
//!
//! Where `μ` controls smoothing strength (default: 1000).
//!
//! # Example
//!
//! ```rust
//! use rank_retrieve::query_likelihood::{retrieve_query_likelihood, QueryLikelihoodParams, SmoothingMethod};
//! use rank_retrieve::bm25::InvertedIndex;
//!
//! let mut index = InvertedIndex::new();
//! index.add_document(0, &["machine".to_string(), "learning".to_string()]);
//!
//! let query = vec!["machine".to_string(), "learning".to_string()];
//! let params = QueryLikelihoodParams {
//!     smoothing: SmoothingMethod::Dirichlet { mu: 1000.0 },
//! };
//! let results = retrieve_query_likelihood(&index, &query, 10, params)?;
//! # Ok::<(), rank_retrieve::RetrieveError>(())
//! ```

use crate::bm25::InvertedIndex;
use crate::RetrieveError;
use std::collections::{HashMap, HashSet};

/// Smoothing method for query likelihood.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmoothingMethod {
    /// Jelinek-Mercer smoothing: Interpolates document and corpus language models.
    /// Lambda (λ) controls the balance (default: 0.5).
    /// Lower lambda (0.1-0.3): More weight to corpus, better for short documents.
    /// Higher lambda (0.5-0.7): More weight to document, better for long documents.
    JelinekMercer { lambda: f32 },
    /// Dirichlet smoothing: Bayesian approach with automatic length adaptation.
    /// Mu (μ) controls smoothing strength (default: 1000).
    /// Higher mu: More smoothing, better for short documents.
    /// Lower mu: Less smoothing, better for long documents.
    Dirichlet { mu: f32 },
}

impl Default for SmoothingMethod {
    fn default() -> Self {
        Self::Dirichlet { mu: 1000.0 }
    }
}

impl SmoothingMethod {
    /// Create Jelinek-Mercer smoothing with default lambda (0.5).
    pub fn jelinek_mercer() -> Self {
        Self::JelinekMercer { lambda: 0.5 }
    }

    /// Create Jelinek-Mercer smoothing with custom lambda.
    pub fn jelinek_mercer_with_lambda(lambda: f32) -> Self {
        Self::JelinekMercer { lambda: lambda.max(0.0).min(1.0) }
    }

    /// Create Dirichlet smoothing with default mu (1000.0).
    pub fn dirichlet() -> Self {
        Self::Dirichlet { mu: 1000.0 }
    }

    /// Create Dirichlet smoothing with custom mu.
    pub fn dirichlet_with_mu(mu: f32) -> Self {
        Self::Dirichlet { mu: mu.max(0.0) }
    }
}

/// Query likelihood parameters.
#[derive(Debug, Clone, Copy)]
pub struct QueryLikelihoodParams {
    /// Smoothing method to use.
    pub smoothing: SmoothingMethod,
}

impl Default for QueryLikelihoodParams {
    fn default() -> Self {
        Self {
            smoothing: SmoothingMethod::default(),
        }
    }
}

/// Compute corpus-level statistics for query likelihood.
///
/// Returns:
/// - `corpus_term_freqs`: Total count of each term across all documents
/// - `corpus_size`: Total number of terms in corpus
fn compute_corpus_stats(index: &InvertedIndex) -> (HashMap<String, u32>, u32) {
    let mut corpus_term_freqs: HashMap<String, u32> = HashMap::new();
    let mut corpus_size: u32 = 0;

    // Iterate through all terms in the index
    for (term, postings) in index.postings() {
        // Sum term frequencies across all documents
        let total_tf: u32 = postings.values().sum();
        corpus_term_freqs.insert(term.clone(), total_tf);
        corpus_size += total_tf;
    }

    (corpus_term_freqs, corpus_size)
}

/// Compute corpus probability P(term|C) for a term.
fn corpus_probability(term: &str, corpus_term_freqs: &HashMap<String, u32>, corpus_size: u32) -> f32 {
    if corpus_size == 0 {
        return 0.0;
    }
    let term_freq = corpus_term_freqs.get(term).copied().unwrap_or(0) as f32;
    term_freq / corpus_size as f32
}

/// Compute document probability P(term|D) for a term in a document.
fn document_probability(index: &InvertedIndex, doc_id: u32, term: &str) -> f32 {
    let doc_length = index.document_length(doc_id) as f32;
    if doc_length == 0.0 {
        return 0.0;
    }
    let term_freq = index.term_frequency(doc_id, term) as f32;
    term_freq / doc_length
}

/// Score a document using Jelinek-Mercer smoothing.
///
/// Formula: P(q_i|D) = λ * P(q_i|D) + (1 - λ) * P(q_i|C)
fn score_jelinek_mercer(
    index: &InvertedIndex,
    doc_id: u32,
    query_terms: &[String],
    lambda: f32,
    corpus_term_freqs: &HashMap<String, u32>,
    corpus_size: u32,
) -> f32 {
    let mut log_score = 0.0;

    for term in query_terms {
        let p_doc = document_probability(index, doc_id, term);
        let p_corpus = corpus_probability(term, corpus_term_freqs, corpus_size);

        // Jelinek-Mercer: P(q_i|D) = λ * P(q_i|D) + (1 - λ) * P(q_i|C)
        let p_smoothed = lambda * p_doc + (1.0 - lambda) * p_corpus;

        // Use log probabilities to avoid underflow
        if p_smoothed > 0.0 {
            log_score += p_smoothed.ln();
        }
    }

    log_score
}

/// Score a document using Dirichlet smoothing.
///
/// Formula: P(q_i|D) = (c(q_i, D) + μ * P(q_i|C)) / (|D| + μ)
fn score_dirichlet(
    index: &InvertedIndex,
    doc_id: u32,
    query_terms: &[String],
    mu: f32,
    corpus_term_freqs: &HashMap<String, u32>,
    corpus_size: u32,
) -> f32 {
    let doc_length = index.document_length(doc_id) as f32;
    let mut log_score = 0.0;

    for term in query_terms {
        let term_freq = index.term_frequency(doc_id, term) as f32;
        let p_corpus = corpus_probability(term, corpus_term_freqs, corpus_size);

        // Dirichlet: P(q_i|D) = (c(q_i, D) + μ * P(q_i|C)) / (|D| + μ)
        let numerator = term_freq + mu * p_corpus;
        let denominator = doc_length + mu;
        let p_smoothed = numerator / denominator;

        // Use log probabilities to avoid underflow
        if p_smoothed > 0.0 {
            log_score += p_smoothed.ln();
        }
    }

    log_score
}

/// Retrieve top-k documents for a query using query likelihood language models.
///
/// Scores all documents and returns the top-k results sorted by query likelihood
/// score (descending).
///
/// # Arguments
///
/// * `index` - The inverted index containing document statistics.
/// * `query_terms` - Tokenized query terms (should be preprocessed: lowercased, stemmed, etc.)
/// * `k` - Number of documents to retrieve (top-k)
/// * `params` - Query likelihood parameters. Use `QueryLikelihoodParams::default()` for standard values.
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
/// use rank_retrieve::query_likelihood::{retrieve_query_likelihood, QueryLikelihoodParams, SmoothingMethod};
/// use rank_retrieve::bm25::InvertedIndex;
///
/// let mut index = InvertedIndex::new();
/// index.add_document(0, &["machine".to_string(), "learning".to_string()]);
/// index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
///
/// let query = vec!["machine".to_string(), "learning".to_string()];
/// let params = QueryLikelihoodParams {
///     smoothing: SmoothingMethod::Dirichlet { mu: 1000.0 },
/// };
/// let results = retrieve_query_likelihood(&index, &query, 10, params).unwrap();
/// assert_eq!(results[0].0, 0); // Document 0 should rank highest
/// ```
pub fn retrieve_query_likelihood(
    index: &InvertedIndex,
    query_terms: &[String],
    k: usize,
    params: QueryLikelihoodParams,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    if query_terms.is_empty() {
        return Err(RetrieveError::EmptyQuery);
    }

    if index.num_docs() == 0 {
        return Err(RetrieveError::EmptyIndex);
    }

    // Compute corpus statistics (lazy computation)
    let (corpus_term_freqs, corpus_size) = compute_corpus_stats(index);

    // Get all candidate documents (documents containing at least one query term)
    let mut candidates: HashSet<u32> = HashSet::new();
    for term in query_terms {
        if let Some(postings) = index.postings().get(term) {
            for &doc_id in postings.keys() {
                candidates.insert(doc_id);
            }
        }
    }

    // If no candidates, score all documents (smoothing allows non-matching docs to have non-zero scores)
    if candidates.is_empty() {
        candidates = index.document_ids().collect();
    }

    // Handle k=0 case
    if k == 0 {
        return Ok(Vec::new());
    }

    // Score all candidate documents
    let mut results: Vec<(u32, f32)> = Vec::with_capacity(candidates.len());
    for doc_id in candidates {
        let score = match params.smoothing {
            SmoothingMethod::JelinekMercer { lambda } => {
                score_jelinek_mercer(index, doc_id, query_terms, lambda, &corpus_term_freqs, corpus_size)
            }
            SmoothingMethod::Dirichlet { mu } => {
                score_dirichlet(index, doc_id, query_terms, mu, &corpus_term_freqs, corpus_size)
            }
        };

        // Only include documents with positive scores (log probabilities can be negative)
        if score > f32::NEG_INFINITY {
            results.push((doc_id, score));
        }
    }

    // Sort by score descending and take top-k (unstable for better performance)
    results.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    results.truncate(k);

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bm25::InvertedIndex;

    #[test]
    fn test_query_likelihood_retrieval() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
        index.add_document(2, &["machine".to_string(), "learning".to_string(), "algorithms".to_string()]);

        let query = vec!["machine".to_string(), "learning".to_string()];
        let results = retrieve_query_likelihood(&index, &query, 10, QueryLikelihoodParams::default()).unwrap();

        assert!(!results.is_empty());
        // Document 0 or 2 should rank highest (both contain both query terms)
        assert!(results[0].0 == 0 || results[0].0 == 2);
        // Document 1 should have lower score (only contains "machine", not "learning")
        if let Some((_, score1)) = results.iter().find(|(id, _)| *id == 1) {
            assert!(results[0].1 > *score1);
        }
    }

    #[test]
    fn test_query_likelihood_empty_query() {
        let index = InvertedIndex::new();
        let query = vec![];
        let result = retrieve_query_likelihood(&index, &query, 10, QueryLikelihoodParams::default());
        assert!(matches!(result, Err(RetrieveError::EmptyQuery)));
    }

    #[test]
    fn test_query_likelihood_empty_index() {
        let index = InvertedIndex::new();
        let query = vec!["test".to_string()];
        let result = retrieve_query_likelihood(&index, &query, 10, QueryLikelihoodParams::default());
        assert!(matches!(result, Err(RetrieveError::EmptyIndex)));
    }

    #[test]
    fn test_smoothing_methods() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["test".to_string(), "test".to_string()]);

        let query = vec!["test".to_string()];

        // Jelinek-Mercer
        let jm_params = QueryLikelihoodParams {
            smoothing: SmoothingMethod::jelinek_mercer_with_lambda(0.5),
        };
        let jm_results = retrieve_query_likelihood(&index, &query, 10, jm_params).unwrap();

        // Dirichlet
        let dir_params = QueryLikelihoodParams {
            smoothing: SmoothingMethod::dirichlet_with_mu(1000.0),
        };
        let dir_results = retrieve_query_likelihood(&index, &query, 10, dir_params).unwrap();

        // Both should return results
        assert!(!jm_results.is_empty());
        assert!(!dir_results.is_empty());
    }

    #[test]
    fn test_smoothing_handles_unseen_terms() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["machine".to_string()]);

        // Query with term not in document (smoothing should handle this)
        let query = vec!["learning".to_string()];
        let results = retrieve_query_likelihood(&index, &query, 10, QueryLikelihoodParams::default()).unwrap();

        // Should still return results (smoothing assigns non-zero probability)
        assert!(!results.is_empty());
    }
}
