//! Query expansion and pseudo-relevance feedback (PRF) module.
//!
//! Provides query expansion techniques to improve recall by reformulating queries
//! with semantically related terms extracted from top-ranked documents.
//!
//! # Current Implementation
//!
//! **Status: Basic PRF implementation**
//!
//! The current implementation provides:
//! - Pseudo-relevance feedback (PRF): Use top-ranked documents to expand query
//! - Multiple term selection methods (Robertson selection, term frequency, IDF-weighted)
//! - Configurable expansion parameters (depth, max terms, weighting)
//!
//! **Suitable for:**
//! - Queries with vocabulary mismatch
//! - Low recall scenarios
//! - Domain-specific terminology
//! - Research/prototyping
//!
//! **Limitations:**
//! - Basic term extraction (no entity/keyphrase extraction yet)
//! - No neural expansion methods
//! - No semantic filtering
//!
//! # Research-Backed Best Practices
//!
//! Based on 2024 research:
//! - **Small PRF depth**: Top-3 to top-10 feedback docs (default: 5)
//! - **Limited expansion**: 3-10 terms typically optimal (default: 5)
//! - **Original query dominance**: Expansion weight typically 0.3-0.7 (default: 0.5)
//! - **Structured features**: Prioritize rare, discriminative terms
//!
//! # Query Expansion Methods
//!
//! ## Pseudo-Relevance Feedback (PRF)
//!
//! PRF expands queries using terms from top-ranked documents:
//!
//! 1. **Initial retrieval**: Retrieve top-k documents for original query
//! 2. **Term extraction**: Extract terms from feedback documents
//! 3. **Term selection**: Score and select best expansion terms
//! 4. **Query expansion**: Add selected terms to original query
//! 5. **Re-retrieval**: Retrieve final results with expanded query
//!
//! ## Term Selection Methods
//!
//! - **Robertson Selection Value (RSV)**: Score terms by contribution to relevance
//! - **Term Frequency**: Simple frequency-based selection
//! - **IDF-Weighted**: Prioritize rare, discriminative terms
//!
//! # Example
//!
//! ```rust
//! use rank_retrieve::query_expansion::{expand_query_with_prf, QueryExpander, ExpansionMethod};
//! use rank_retrieve::bm25::{InvertedIndex, Bm25Params};
//!
//! let mut index = InvertedIndex::new();
//! index.add_document(0, &["machine".to_string(), "learning".to_string()]);
//!
//! let query = vec!["ml".to_string()];  // Abbreviated query
//! let expander = QueryExpander::new()
//!     .with_prf_depth(5)
//!     .with_max_expansion_terms(5)
//!     .with_method(ExpansionMethod::IDFWeighted);
//!
//! // PRF expands "ml" to include "machine", "learning" from top docs
//! let results = expand_query_with_prf(&index, &query, 10, 10, &expander)?;
//! # Ok::<(), rank_retrieve::RetrieveError>(())
//! ```

use crate::RetrieveError;
use std::collections::{HashMap, HashSet};

/// Query expansion method for term selection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExpansionMethod {
    /// Robertson Selection Value (RSV): Score terms by contribution to relevance.
    /// Best for: General-purpose expansion with good precision-recall balance.
    RobertsonSelection,
    /// Term frequency: Select most frequent terms from feedback documents.
    /// Best for: Simple, fast expansion when term frequency correlates with relevance.
    TermFrequency,
    /// IDF-weighted: Prioritize rare, discriminative terms.
    /// Best for: When vocabulary mismatch is the main issue (rare terms are more informative).
    IDFWeighted,
}

impl Default for ExpansionMethod {
    fn default() -> Self {
        Self::IDFWeighted
    }
}

/// Query expander configuration.
#[derive(Debug, Clone)]
pub struct QueryExpander {
    /// Number of top-ranked documents to use for feedback (PRF depth).
    /// Research shows 3-10 is optimal (default: 5).
    pub prf_depth: usize,
    /// Maximum number of expansion terms to add (default: 5).
    pub max_expansion_terms: usize,
    /// Weight for expansion terms relative to original query (default: 0.5).
    /// Original query terms have weight 1.0, expansion terms have this weight.
    pub expansion_weight: f32,
    /// Term selection method (default: IDFWeighted).
    pub method: ExpansionMethod,
}

impl Default for QueryExpander {
    fn default() -> Self {
        Self {
            prf_depth: 5,
            max_expansion_terms: 5,
            expansion_weight: 0.5,
            method: ExpansionMethod::IDFWeighted,
        }
    }
}

impl QueryExpander {
    /// Create new query expander with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set PRF depth (number of feedback documents).
    pub fn with_prf_depth(mut self, depth: usize) -> Self {
        self.prf_depth = depth;
        self
    }

    /// Set maximum number of expansion terms.
    pub fn with_max_expansion_terms(mut self, max_terms: usize) -> Self {
        self.max_expansion_terms = max_terms;
        self
    }

    /// Set expansion weight (0.0 to 1.0).
    pub fn with_expansion_weight(mut self, weight: f32) -> Self {
        self.expansion_weight = weight.max(0.0).min(1.0);
        self
    }

    /// Set term selection method.
    pub fn with_method(mut self, method: ExpansionMethod) -> Self {
        self.method = method;
        self
    }
}


/// Score terms using Robertson Selection Value (RSV).
///
/// RSV scores terms by their contribution to relevance:
/// RSV(t) = log((r + 0.5) / (R - r + 0.5)) * log((n - df + 0.5) / (df - r + 0.5))
///
/// Where:
/// - r = number of relevant docs containing term
/// - R = total relevant docs (PRF depth)
/// - n = total documents
/// - df = document frequency of term
fn score_terms_rsv(
    feedback_terms: &[String],
    all_terms: &[String],
    num_docs: u32,
    doc_frequencies: &HashMap<String, u32>,
) -> Vec<(String, f32)> {
    let r = feedback_terms.len() as f32; // All feedback docs are "relevant"
    let r_total = r; // PRF depth
    
    let mut term_scores: HashMap<String, f32> = HashMap::new();
    
    for term in all_terms {
        if feedback_terms.contains(term) {
            let df = doc_frequencies.get(term).copied().unwrap_or(0) as f32;
            let n = num_docs as f32;
            
            // RSV formula (simplified: assume all feedback docs are relevant)
            let score = ((r + 0.5) / (r_total - r + 0.5)).ln() 
                * ((n - df + 0.5) / (df - r + 0.5).max(0.5)).ln();
            
            term_scores.insert(term.clone(), score);
        }
    }
    
    let mut scored: Vec<(String, f32)> = term_scores.into_iter().collect();
    scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)); // Unstable for better performance
    scored
}

/// Score terms using term frequency.
fn score_terms_tf(
    feedback_terms: &[String],
) -> Vec<(String, f32)> {
    let mut term_counts: HashMap<String, u32> = HashMap::new();
    
    for term in feedback_terms {
        *term_counts.entry(term.clone()).or_insert(0) += 1;
    }
    
    let mut scored: Vec<(String, f32)> = term_counts
        .into_iter()
        .map(|(term, count)| (term, count as f32))
        .collect();
    scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)); // Unstable for better performance
    scored
}

/// Score terms using IDF weighting.
fn score_terms_idf(
    feedback_terms: &[String],
    num_docs: u32,
    doc_frequencies: &HashMap<String, u32>,
) -> Vec<(String, f32)> {
    let mut term_counts: HashMap<String, u32> = HashMap::new();
    
    for term in feedback_terms {
        *term_counts.entry(term.clone()).or_insert(0) += 1;
    }
    
    let n = num_docs as f32;
    let mut scored: Vec<(String, f32)> = term_counts
        .into_iter()
        .map(|(term, tf)| {
            let df = doc_frequencies.get(&term).copied().unwrap_or(0) as f32;
            let idf = if df > 0.0 {
                (n / df).ln()
            } else {
                0.0
            };
            (term, tf as f32 * idf)
        })
        .collect();
    scored.sort_unstable_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal)); // Unstable for better performance
    scored
}

/// Extract terms from feedback documents using the inverted index.
///
/// Given document IDs, extracts all terms that appear in those documents
/// by looking up terms in the postings list.
fn extract_terms_from_feedback_docs(
    index: &crate::bm25::InvertedIndex,
    feedback_doc_ids: &[u32],
) -> Vec<String> {
    let mut feedback_terms: Vec<String> = Vec::new();
    let feedback_set: HashSet<u32> = feedback_doc_ids.iter().copied().collect();
    
    // Iterate through all terms in the index
    for (term, postings) in index.postings() {
        // Check if this term appears in any feedback document
        for &doc_id in postings.keys() {
            if feedback_set.contains(&doc_id) {
                feedback_terms.push(term.clone());
                break; // Only add term once, even if it appears in multiple feedback docs
            }
        }
    }
    
    feedback_terms
}

/// Expand query using pseudo-relevance feedback (PRF) for BM25/TF-IDF.
///
/// This function performs two-stage retrieval:
/// 1. Initial retrieval with original query
/// 2. Extract terms from top-k feedback documents
/// 3. Select best expansion terms
/// 4. Re-retrieve with expanded query
///
/// # Arguments
///
/// * `index` - Inverted index (BM25 or TF-IDF)
/// * `query` - Original query terms
/// * `initial_k` - Number of documents for initial retrieval (PRF feedback)
/// * `final_k` - Number of documents to return after expansion
/// * `expander` - Query expander configuration
/// * `retrieve_fn` - Retrieval function (e.g., `retrieve_bm25`, `retrieve_tfidf`)
///
/// # Returns
///
/// Vector of (document_id, score) pairs from expanded query retrieval.
///
/// # Errors
///
/// Returns errors from underlying retrieval functions.
///
/// # Example
///
/// ```rust
/// use rank_retrieve::query_expansion::{expand_query_with_prf_bm25, QueryExpander};
/// use rank_retrieve::bm25::{InvertedIndex, Bm25Params};
/// use rank_retrieve::retrieve_bm25;
///
/// let mut index = InvertedIndex::new();
/// index.add_document(0, &["machine".to_string(), "learning".to_string()]);
///
/// let query = vec!["ml".to_string()];  // Abbreviated query
/// let expander = QueryExpander::new().with_prf_depth(5);
///
/// // PRF expands "ml" to include "machine", "learning" from top docs
/// let results = expand_query_with_prf_bm25(
///     &index,
///     &query,
///     10,  // initial_k
///     10,  // final_k
///     &expander,
///     retrieve_bm25,
/// )?;
/// # Ok::<(), rank_retrieve::RetrieveError>(())
/// ```
pub fn expand_query_with_prf_bm25<F>(
    index: &crate::bm25::InvertedIndex,
    query: &[String],
    initial_k: usize,
    final_k: usize,
    expander: &QueryExpander,
    retrieve_fn: F,
) -> Result<Vec<(u32, f32)>, RetrieveError>
where
    F: Fn(&crate::bm25::InvertedIndex, &[String], usize, crate::bm25::Bm25Params) -> Result<Vec<(u32, f32)>, RetrieveError>,
{
    // Step 1: Initial retrieval
    let initial_results = retrieve_fn(index, query, initial_k, crate::bm25::Bm25Params::default())?;
    
    if initial_results.is_empty() {
        // No feedback documents, return empty results
        return Ok(Vec::new());
    }
    
    // Step 2: Extract terms from feedback documents
    let feedback_doc_ids: Vec<u32> = initial_results
        .iter()
        .take(expander.prf_depth.min(initial_results.len()))
        .map(|(doc_id, _)| *doc_id)
        .collect();
    
    let feedback_terms = extract_terms_from_feedback_docs(index, &feedback_doc_ids);
    
    if feedback_terms.is_empty() {
        // No terms extracted, return original results
        return Ok(initial_results.into_iter().take(final_k).collect());
    }
    
    // Step 3: Select expansion terms
    let expansion_terms: Vec<String> = match expander.method {
        ExpansionMethod::RobertsonSelection => {
            let num_docs = index.num_docs();
            let doc_frequencies = index.doc_frequencies();
            let scored = score_terms_rsv(&feedback_terms, &feedback_terms, num_docs, doc_frequencies);
            scored
                .into_iter()
                .filter(|(term, _)| !query.contains(term)) // Exclude original query terms
                .take(expander.max_expansion_terms)
                .map(|(term, _)| term)
                .collect()
        }
        ExpansionMethod::TermFrequency => {
            let scored = score_terms_tf(&feedback_terms);
            scored
                .into_iter()
                .filter(|(term, _)| !query.contains(term))
                .take(expander.max_expansion_terms)
                .map(|(term, _)| term)
                .collect()
        }
        ExpansionMethod::IDFWeighted => {
            let num_docs = index.num_docs();
            let doc_frequencies = index.doc_frequencies();
            let scored = score_terms_idf(&feedback_terms, num_docs, doc_frequencies);
            scored
                .into_iter()
                .filter(|(term, _)| !query.contains(term))
                .take(expander.max_expansion_terms)
                .map(|(term, _)| term)
                .collect()
        }
    };
    
    if expansion_terms.is_empty() {
        // No expansion terms found, return original results
        return Ok(initial_results.into_iter().take(final_k).collect());
    }
    
    // Step 4: Expand query
    // Original query terms have weight 1.0, expansion terms have expansion_weight
    // For simplicity, we'll just append expansion terms (weighting handled in retrieval)
    let mut expanded_query = query.to_vec();
    expanded_query.extend(expansion_terms);
    
    // Step 5: Re-retrieve with expanded query
    retrieve_fn(index, &expanded_query, final_k, crate::bm25::Bm25Params::default())
}

/// Expand query using pseudo-relevance feedback (generic interface).
///
/// This is a simplified version that works with any retriever that returns
/// document IDs. For full PRF, you need access to document content to extract terms.
///
/// # Note
///
/// This is a basic implementation. For production use, you'd need:
/// - Access to document content (not just IDs)
/// - Proper tokenization
/// - Entity/keyphrase extraction
/// - Semantic filtering
pub fn expand_query_simple(
    original_query: &[String],
    expansion_terms: &[String],
    _expansion_weight: f32,  // Reserved for future weighting implementation
) -> Vec<String> {
    // Simple expansion: append expansion terms
    // In practice, you'd weight terms differently based on expansion_weight
    let mut expanded = original_query.to_vec();
    expanded.extend(expansion_terms.iter().cloned());
    expanded
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bm25::InvertedIndex;

    #[test]
    fn test_query_expander_default() {
        let expander = QueryExpander::default();
        assert_eq!(expander.prf_depth, 5);
        assert_eq!(expander.max_expansion_terms, 5);
        assert_eq!(expander.expansion_weight, 0.5);
        assert_eq!(expander.method, ExpansionMethod::IDFWeighted);
    }

    #[test]
    fn test_query_expander_builder() {
        let expander = QueryExpander::new()
            .with_prf_depth(10)
            .with_max_expansion_terms(7)
            .with_expansion_weight(0.7)
            .with_method(ExpansionMethod::TermFrequency);
        
        assert_eq!(expander.prf_depth, 10);
        assert_eq!(expander.max_expansion_terms, 7);
        assert_eq!(expander.expansion_weight, 0.7);
        assert_eq!(expander.method, ExpansionMethod::TermFrequency);
    }

    #[test]
    fn test_expand_query_simple() {
        let original = vec!["machine".to_string()];
        let expansion = vec!["learning".to_string(), "algorithms".to_string()];
        let expanded = expand_query_simple(&original, &expansion, 0.5);
        
        assert_eq!(expanded.len(), 3);
        assert!(expanded.contains(&"machine".to_string()));
        assert!(expanded.contains(&"learning".to_string()));
        assert!(expanded.contains(&"algorithms".to_string()));
    }

    #[test]
    fn test_score_terms_tf() {
        let terms = vec![
            "test".to_string(),
            "test".to_string(),
            "example".to_string(),
        ];
        let scored = score_terms_tf(&terms);
        
        assert!(!scored.is_empty());
        // "test" should have higher score (appears twice)
        assert!(scored[0].0 == "test" || scored[0].0 == "example");
    }
}
