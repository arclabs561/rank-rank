//! Autoregressive model interface for generative retrieval.
//!
//! This module is part of the `generative` module. See `generative/mod.rs` for the main entry point.
//!
//! This module defines the trait for autoregressive language models used in
//! generative retrieval. Models generate identifiers (titles, substrings, pseudo-queries)
//! given a query and identifier prefix.

use crate::RetrieveError;

/// Trait for autoregressive language models used in generative retrieval.
///
/// Models should support:
/// - Generating identifiers given a query and prefix
/// - Beam search for diverse generation
/// - Constrained generation (via FM-index or similar)
pub trait AutoregressiveModel {
    /// Generate identifiers for a query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query text
    /// * `prefix` - Identifier prefix (e.g., "title", "substring", "pseudo-query")
    /// * `beam_size` - Number of beams for beam search
    /// * `constraint_fn` - Optional function to check if a token is valid (for constrained generation)
    ///
    /// # Returns
    ///
    /// Vector of (identifier, score) pairs, where score is the language model score.
    fn generate(
        &self,
        query: &str,
        prefix: &str,
        beam_size: usize,
        constraint_fn: Option<&dyn Fn(&str) -> bool>,
    ) -> Result<Vec<(String, f32)>, RetrieveError>;
}

/// Mock autoregressive model for testing and benchmarking.
///
/// Returns simple identifiers based on query keywords.
pub struct MockAutoregressiveModel;

impl MockAutoregressiveModel {
    pub fn new() -> Self {
        Self {}
    }
}

impl AutoregressiveModel for MockAutoregressiveModel {
    fn generate(
        &self,
        query: &str,
        prefix: &str,
        beam_size: usize,
        _constraint_fn: Option<&dyn Fn(&str) -> bool>,
    ) -> Result<Vec<(String, f32)>, RetrieveError> {
        // Simple mock: generate identifiers based on query
        let mut identifiers = Vec::new();

        // Extract key terms from query
        let terms: Vec<&str> = query
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .take(beam_size)
            .collect();

        for (i, term) in terms.iter().enumerate() {
            let identifier = match prefix {
                "title" => format!("{} Topic", term.to_uppercase()),
                "substring" => format!("{} is a key concept", term),
                "pseudo-query" => format!("what is {}", term),
                _ => term.to_string(),
            };
            // Decreasing scores
            let score = 10.0 - (i as f32 * 0.5);
            identifiers.push((identifier, score));
        }

        // Ensure we return at least one identifier
        if identifiers.is_empty() {
            identifiers.push((format!("{} result", prefix), 5.0));
        }

        Ok(identifiers)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_model_generate() {
        let model = MockAutoregressiveModel::new();
        let identifiers = model
            .generate("What is prime rate?", "title", 5, None)
            .unwrap();
        
        assert!(!identifiers.is_empty());
        assert!(identifiers[0].1 > 0.0);
    }

    #[test]
    fn test_mock_model_different_prefixes() {
        let model = MockAutoregressiveModel::new();
        
        let title_ids = model.generate("test query", "title", 3, None).unwrap();
        let substring_ids = model.generate("test query", "substring", 3, None).unwrap();
        
        assert_ne!(title_ids[0].0, substring_ids[0].0);
    }
}

