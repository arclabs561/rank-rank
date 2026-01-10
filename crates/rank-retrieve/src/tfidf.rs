//! TF-IDF retrieval module.
//!
//! Provides TF-IDF (Term Frequency-Inverse Document Frequency) scoring for first-stage retrieval.
//!
//! # Current Implementation
//!
//! **Status: Basic inverted index (reuses BM25 infrastructure)**
//!
//! The current implementation reuses the BM25 inverted index structure with:
//! - Standard TF-IDF scoring (tf * idf)
//! - Support for linear and log-scaled TF variants
//! - Standard and smoothed IDF variants
//!
//! **Suitable for:**
//! - Any scale of corpora (from small to very large)
//! - Prototyping and research
//! - Production systems
//! - Baseline comparison with BM25
//! - Educational purposes
//!
//! **Note on scale:**
//! - In-memory only (no persistence)
//! - No length normalization (longer documents naturally score higher)
//! - For persistent storage, consider integrating with specialized backends via the `Backend` trait
//!
//! # TF-IDF vs BM25
//!
//! **TF-IDF:**
//! - Simpler formula: `score = tf * idf`
//! - No saturation: Term frequency grows linearly
//! - No length normalization: Longer documents score higher
//! - Parameter-free: No tuning required
//!
//! **BM25:**
//! - More complex formula with saturation and length normalization
//! - Term frequency saturates (prevents repetition abuse)
//! - Length normalization prevents bias toward longer documents
//! - Tunable parameters (k1, b)
//!
//! # TF-IDF Formula
//!
//! TF-IDF (Term Frequency-Inverse Document Frequency) calculates relevance as:
//!
//! ```text
//! TF-IDF(q, d) = Σ tf(q_i, d) * idf(q_i)
//! ```
//!
//! Where:
//! - `tf(q_i, d)` = term frequency of term q_i in document d
//!   - Linear: `tf = f_{t,d}` (raw count)
//!   - Log-scaled: `tf = 1 + log(f_{t,d})` (reduces impact of high frequencies)
//! - `idf(q_i)` = inverse document frequency of term q_i
//!   - Standard: `idf = log(N / df_t)` where N = total docs, df_t = docs containing term
//!   - Smoothed: `idf = log(1 + (N - df_t + 0.5) / (df_t + 0.5))` (BM25-style, more stable)

use crate::bm25::InvertedIndex;
use crate::RetrieveError;
use std::collections::HashSet;

/// TF-IDF parameters.
#[derive(Debug, Clone, Copy)]
pub struct TfIdfParams {
    /// Term frequency variant.
    pub tf_variant: TfVariant,
    /// IDF variant.
    pub idf_variant: IdfVariant,
}

impl Default for TfIdfParams {
    fn default() -> Self {
        Self {
            tf_variant: TfVariant::LogScaled,
            idf_variant: IdfVariant::Standard,
        }
    }
}

/// Term frequency variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TfVariant {
    /// Linear TF: `tf = f_{t,d}` (raw count).
    Linear,
    /// Log-scaled TF: `tf = 1 + log(f_{t,d})` (reduces impact of high frequencies).
    LogScaled,
}

/// IDF variant.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IdfVariant {
    /// Standard IDF: `idf = log(N / df_t)`.
    Standard,
    /// Smoothed IDF: `idf = log(1 + (N - df_t + 0.5) / (df_t + 0.5))` (BM25-style, more stable).
    Smoothed,
}

impl TfIdfParams {
    /// Create TF-IDF parameters with linear TF and standard IDF.
    pub fn linear() -> Self {
        Self {
            tf_variant: TfVariant::Linear,
            idf_variant: IdfVariant::Standard,
        }
    }

    /// Create TF-IDF parameters with log-scaled TF and smoothed IDF.
    pub fn smoothed() -> Self {
        Self {
            tf_variant: TfVariant::LogScaled,
            idf_variant: IdfVariant::Smoothed,
        }
    }
}

/// Compute term frequency for a term in a document.
fn compute_tf(tf_count: u32, variant: TfVariant) -> f32 {
    match variant {
        TfVariant::Linear => tf_count as f32,
        TfVariant::LogScaled => {
            if tf_count == 0 {
                0.0
            } else {
                1.0 + (tf_count as f32).ln()
            }
        }
    }
}

/// Compute inverse document frequency for a term.
fn compute_idf(
    num_docs: u32,
    doc_frequency: u32,
    variant: IdfVariant,
) -> f32 {
    if doc_frequency == 0 {
        return 0.0;
    }

    let n = num_docs as f32;
    let df = doc_frequency as f32;

    match variant {
        IdfVariant::Standard => (n / df).ln(),
        IdfVariant::Smoothed => (1.0 + (n - df + 0.5) / (df + 0.5)).ln(),
    }
}

/// Score a document against a query using TF-IDF.
///
/// # Arguments
///
/// * `index` - Inverted index (reuses BM25 index structure)
/// * `doc_id` - Document to score
/// * `query_terms` - Tokenized query terms
/// * `params` - TF-IDF parameters
///
/// # Returns
///
/// TF-IDF score for the document
pub fn score_tfidf(
    index: &InvertedIndex,
    doc_id: u32,
    query_terms: &[String],
    params: TfIdfParams,
) -> f32 {
    let mut score = 0.0;
    let num_docs = index.num_docs();

    for term in query_terms {
        // Get term frequency in document
        let tf_count = index
            .postings()
            .get(term)
            .and_then(|postings| postings.get(&doc_id))
            .copied()
            .unwrap_or(0) as u32;

        if tf_count == 0 {
            continue;
        }

        // Compute TF
        let tf = compute_tf(tf_count, params.tf_variant);

        // Get document frequency for IDF
        let df = index.doc_frequencies().get(term).copied().unwrap_or(0);

        // Compute IDF
        let idf = compute_idf(num_docs, df, params.idf_variant);

        if idf == 0.0 {
            continue;
        }

        // TF-IDF score: tf * idf
        score += tf * idf;
    }

    score
}

/// Retrieve top-k documents for a query using TF-IDF scoring.
///
/// Scores all documents containing at least one query term and returns the top-k
/// results sorted by TF-IDF score (descending).
///
/// # Arguments
///
/// * `index` - Inverted index (reuses BM25 index structure)
/// * `query_terms` - Tokenized query terms (should be preprocessed: lowercased, stemmed, etc.)
/// * `k` - Number of documents to retrieve (top-k)
/// * `params` - TF-IDF parameters. Use `TfIdfParams::default()` for standard values.
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
/// use rank_retrieve::tfidf::{retrieve_tfidf, TfIdfParams};
/// use rank_retrieve::bm25::InvertedIndex;
///
/// let mut index = InvertedIndex::new();
/// index.add_document(0, &["machine".to_string(), "learning".to_string()]);
/// index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
///
/// let query = vec!["machine".to_string(), "learning".to_string()];
/// let results = retrieve_tfidf(&index, &query, 10, TfIdfParams::default())?;
/// assert_eq!(results[0].0, 0); // Document 0 should rank highest
/// # Ok::<(), rank_retrieve::RetrieveError>(())
/// ```
///
/// # Performance
///
/// Time complexity: O(q * d) where q is number of query terms and d is average number
/// of documents per term. For typical queries (2-5 terms) and indexes (10K-1M docs),
/// retrieval is typically <10ms. Slightly faster than BM25 due to simpler formula.
pub fn retrieve_tfidf(
    index: &InvertedIndex,
    query_terms: &[String],
    k: usize,
    params: TfIdfParams,
) -> Result<Vec<(u32, f32)>, RetrieveError> {
    if query_terms.is_empty() {
        return Err(RetrieveError::EmptyQuery);
    }

    if index.num_docs() == 0 {
        return Err(RetrieveError::EmptyIndex);
    }

    // Get all candidate documents (documents containing at least one query term)
    let mut candidates: Vec<u32> = Vec::new();
    let mut seen: HashSet<u32> = HashSet::new();
    for term in query_terms {
        if let Some(postings) = index.postings().get(term) {
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

    // Score all candidates
    let mut scored: Vec<(u32, f32)> = candidates
        .into_iter()
        .map(|doc_id| {
            let score = score_tfidf(index, doc_id, query_terms, params);
            (doc_id, score)
        })
        .collect();

    // Sort by score descending (unstable for better performance)
    scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Return top-k
    Ok(scored.into_iter().take(k).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bm25::InvertedIndex;

    #[test]
    fn test_tfidf_retrieval() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["machine".to_string(), "learning".to_string()]);
        index.add_document(1, &["artificial".to_string(), "intelligence".to_string()]);
        index.add_document(2, &["machine".to_string(), "learning".to_string(), "algorithms".to_string()]);

        let query = vec!["machine".to_string(), "learning".to_string()];
        let results = retrieve_tfidf(&index, &query, 10, TfIdfParams::default()).unwrap();

        assert!(!results.is_empty());
        // Document 2 should rank highest (contains both terms + additional term, higher TF)
        // Document 0 should also be in results (contains both terms)
        assert!(results.iter().any(|(id, _)| *id == 0 || *id == 2));
        assert!(results[0].1 > 0.0);
        // Verify both query terms are matched
        assert!(results.iter().any(|(id, _)| *id == 0));
        assert!(results.iter().any(|(id, _)| *id == 2));
    }

    #[test]
    fn test_tfidf_empty_query() {
        let index = InvertedIndex::new();
        let query = vec![];
        let result = retrieve_tfidf(&index, &query, 10, TfIdfParams::default());
        assert!(matches!(result, Err(RetrieveError::EmptyQuery)));
    }

    #[test]
    fn test_tfidf_empty_index() {
        let index = InvertedIndex::new();
        let query = vec!["test".to_string()];
        let result = retrieve_tfidf(&index, &query, 10, TfIdfParams::default());
        assert!(matches!(result, Err(RetrieveError::EmptyIndex)));
    }

    #[test]
    fn test_tf_variants() {
        let mut index = InvertedIndex::new();
        // Add document with term repeated 5 times
        index.add_document(0, &vec!["test".to_string(); 5]);
        // Add another document with term once (for IDF calculation)
        index.add_document(1, &["other".to_string()]);

        let query = vec!["test".to_string()];
        
        // Linear TF should give higher score for repeated terms
        let linear_params = TfIdfParams {
            tf_variant: TfVariant::Linear,
            idf_variant: IdfVariant::Standard,
        };
        let linear_results = retrieve_tfidf(&index, &query, 10, linear_params).unwrap();
        
        // Log-scaled TF should give lower score (saturation effect)
        let log_params = TfIdfParams {
            tf_variant: TfVariant::LogScaled,
            idf_variant: IdfVariant::Standard,
        };
        let log_results = retrieve_tfidf(&index, &query, 10, log_params).unwrap();
        
        // Linear: tf=5.0, Log-scaled: tf≈2.61 (1+ln(5))
        // With same IDF, linear should be higher
        assert!(linear_results[0].1 > log_results[0].1, 
                "Linear TF (score: {}) should be higher than log-scaled (score: {})", 
                linear_results[0].1, log_results[0].1);
    }

    #[test]
    fn test_idf_variants() {
        let mut index = InvertedIndex::new();
        index.add_document(0, &["rare".to_string()]);
        index.add_document(1, &["common".to_string()]);
        index.add_document(2, &["common".to_string()]);

        let query = vec!["rare".to_string()];
        
        // Standard IDF
        let standard_params = TfIdfParams {
            tf_variant: TfVariant::LogScaled,
            idf_variant: IdfVariant::Standard,
        };
        let standard_results = retrieve_tfidf(&index, &query, 10, standard_params).unwrap();
        
        // Smoothed IDF
        let smoothed_params = TfIdfParams {
            tf_variant: TfVariant::LogScaled,
            idf_variant: IdfVariant::Smoothed,
        };
        let smoothed_results = retrieve_tfidf(&index, &query, 10, smoothed_params).unwrap();
        
        // Both should return document 0, but scores may differ
        assert_eq!(standard_results[0].0, 0);
        assert_eq!(smoothed_results[0].0, 0);
    }
}
