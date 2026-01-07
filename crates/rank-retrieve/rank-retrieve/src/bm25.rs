//! BM25 retrieval module.
//!
//! Provides inverted index and Okapi BM25 scoring for first-stage retrieval.
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

use std::collections::{HashMap, HashSet};
use crate::error::RetrieveError;

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
}

impl Default for Bm25Params {
    fn default() -> Self {
        Self {
            k1: 1.2,
            b: 0.75,
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
        }
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
                .or_insert_with(HashMap::new)
                .insert(doc_id, freq);
            
            // Update document frequency
            *self.doc_frequencies.entry(term).or_insert(0) += 1;
        }
        
        self.num_docs += 1;
        self.update_avg_doc_length();
    }
    
    /// Update average document length.
    fn update_avg_doc_length(&mut self) {
        let total_length: u32 = self.doc_lengths.values().sum();
        if self.num_docs > 0 {
            self.avg_doc_length = total_length as f32 / self.num_docs as f32;
        }
    }
    
    /// Calculate inverse document frequency (IDF) for a term.
    ///
    /// Uses the standard IDF formula: log((N - df + 0.5) / (df + 0.5))
    /// where N is total documents and df is document frequency.
    pub fn idf(&self, term: &str) -> f32 {
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
        let doc_length = self.doc_lengths.get(&doc_id).copied().unwrap_or(0) as f32;
        let mut score = 0.0;
        
        for term in query_terms {
            let idf = self.idf(term);
            if idf == 0.0 {
                continue;
            }
            
            // Get term frequency in document
            let tf = self.postings
                .get(term)
                .and_then(|postings| postings.get(&doc_id))
                .copied()
                .unwrap_or(0) as f32;
            
            if tf == 0.0 {
                continue;
            }
            
            // BM25 formula
            let numerator = tf * (params.k1 + 1.0);
            let denominator = tf + params.k1 * (1.0 - params.b + params.b * doc_length / self.avg_doc_length);
            score += idf * (numerator / denominator);
        }
        
        score
    }
    
    /// Retrieve top-k documents for a query.
    ///
    /// # Arguments
    ///
    /// * `query_terms` - Tokenized query terms
    /// * `k` - Number of documents to retrieve
    /// * `params` - BM25 parameters
    ///
    /// # Returns
    ///
    /// Vector of (document_id, score) pairs, sorted by score descending
    ///
    /// # Errors
    ///
    /// Returns `RetrieveError::EmptyQuery` if query is empty.
    /// Returns `RetrieveError::EmptyIndex` if index has no documents.
    pub fn retrieve(&self, query_terms: &[String], k: usize, params: Bm25Params) -> Result<Vec<(u32, f32)>, RetrieveError> {
        if query_terms.is_empty() {
            return Err(RetrieveError::EmptyQuery);
        }
        
        if self.num_docs == 0 {
            return Err(RetrieveError::EmptyIndex);
        }
        
        // Get all candidate documents (documents containing at least one query term)
        let mut candidates: HashSet<u32> = HashSet::new();
        for term in query_terms {
            if let Some(postings) = self.postings.get(term) {
                candidates.extend(postings.keys());
            }
        }
        
        // Score all candidates
        let mut scored: Vec<(u32, f32)> = candidates
            .into_iter()
            .map(|doc_id| {
                let score = self.score(doc_id, query_terms, params);
                (doc_id, score)
            })
            .collect();
        
        // Sort by score descending
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        // Return top-k
        Ok(scored.into_iter().take(k).collect())
    }
}

impl Default for InvertedIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bm25_basic() {
        let mut index = InvertedIndex::new();
        
        // Add documents
        index.add_document(0, &["the".to_string(), "quick".to_string(), "brown".to_string(), "fox".to_string()]);
        index.add_document(1, &["the".to_string(), "lazy".to_string(), "dog".to_string()]);
        index.add_document(2, &["quick".to_string(), "brown".to_string(), "fox".to_string(), "jumps".to_string()]);
        
        // Query
        let query = vec!["quick".to_string(), "fox".to_string()];
        let results = index.retrieve(&query, 10, Bm25Params::default()).unwrap();
        
        // Document 0 and 2 should have highest scores (both contain "quick" and "fox")
        assert!(results.len() >= 2);
        // Verify we got results with positive scores
        assert!(results.iter().any(|(_, score)| *score > 0.0));
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

