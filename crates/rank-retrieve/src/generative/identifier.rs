//! Multiview identifier generation for generative retrieval.
//!
//! This module is part of the `generative` module. See `generative/mod.rs` for the main entry point.
//!
//! Identifiers are distinctive strings that represent passages. LTRGR uses three
//! types of identifiers to represent passages from different perspectives:
//! - **Title**: Passage title (e.g., "Prime Rate in Canada")
//! - **Substring**: Random substring from passage body
//! - **Pseudo-query**: Query-like representation of the passage


/// Type of identifier for multiview representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdentifierType {
    /// Passage title identifier.
    Title,
    /// Substring from passage body.
    Substring,
    /// Pseudo-query representation.
    PseudoQuery,
}

impl IdentifierType {
    /// Get the prefix string for this identifier type.
    pub fn prefix(&self) -> &'static str {
        match self {
            IdentifierType::Title => "title",
            IdentifierType::Substring => "substring",
            IdentifierType::PseudoQuery => "pseudo-query",
        }
    }
}

/// Multiview identifier representation of a passage.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiviewIdentifier {
    /// Title identifier.
    pub title: String,
    /// Substring identifier.
    pub substring: String,
    /// Pseudo-query identifier.
    pub pseudo_query: String,
}

impl MultiviewIdentifier {
    /// Create a new multiview identifier.
    pub fn new(title: String, substring: String, pseudo_query: String) -> Self {
        Self {
            title,
            substring,
            pseudo_query,
        }
    }

    /// Get all identifiers as a vector.
    pub fn all(&self) -> Vec<String> {
        vec![
            self.title.clone(),
            self.substring.clone(),
            self.pseudo_query.clone(),
        ]
    }

    /// Get identifier by type.
    pub fn get(&self, identifier_type: IdentifierType) -> &str {
        match identifier_type {
            IdentifierType::Title => &self.title,
            IdentifierType::Substring => &self.substring,
            IdentifierType::PseudoQuery => &self.pseudo_query,
        }
    }
}

/// Generator for multiview identifiers from passages.
pub trait IdentifierGenerator {
    /// Generate multiview identifiers for a passage.
    ///
    /// # Arguments
    ///
    /// * `passage` - The passage text
    /// * `passage_id` - Unique identifier for the passage
    ///
    /// # Returns
    ///
    /// A `MultiviewIdentifier` containing title, substring, and pseudo-query.
    fn generate(&self, passage: &str, passage_id: u32) -> MultiviewIdentifier;
}

/// Simple identifier generator using heuristics.
///
/// This is a basic implementation that:
/// - Extracts title from first line or metadata
/// - Samples random substring from passage
/// - Generates pseudo-query from key terms
pub struct SimpleIdentifierGenerator {
    /// Minimum substring length.
    min_substring_len: usize,
    /// Maximum substring length.
    max_substring_len: usize,
}

impl Default for SimpleIdentifierGenerator {
    fn default() -> Self {
        Self {
            min_substring_len: 20,
            max_substring_len: 100,
        }
    }
}

impl SimpleIdentifierGenerator {
    /// Create a new simple identifier generator.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set substring length bounds.
    pub fn with_substring_bounds(mut self, min: usize, max: usize) -> Self {
        self.min_substring_len = min;
        self.max_substring_len = max;
        self
    }

    /// Extract title from passage (first line or first sentence).
    fn extract_title(&self, passage: &str) -> String {
        // Try first line
        if let Some(first_line) = passage.lines().next() {
            let trimmed = first_line.trim();
            if !trimmed.is_empty() && trimmed.len() < 200 {
                return trimmed.to_string();
            }
        }

        // Fallback: first sentence
        if let Some(sentence_end) = passage.find(&['.', '!', '?'][..]) {
            let sentence = &passage[..sentence_end].trim();
            if sentence.len() < 200 {
                return sentence.to_string();
            }
        }

        // Final fallback: first 50 characters
        passage.chars().take(50).collect()
    }

    /// Sample a random substring from passage.
    fn sample_substring(&self, passage: &str) -> String {
        let passage_len = passage.len();
        if passage_len < self.min_substring_len {
            return passage.to_string();
        }

        // Try to find a good substring (prefer word boundaries)
        let max_start = passage_len.saturating_sub(self.min_substring_len);
        if max_start == 0 {
            return passage.to_string();
        }

        // Sample start position (prefer beginning, but allow randomness)
        let start = if passage_len > self.max_substring_len * 2 {
            // For long passages, prefer beginning
            passage_len / 4
        } else {
            0
        };

        let end = (start + self.max_substring_len).min(passage_len);
        let substring = &passage[start..end];

        // Try to align to word boundaries
        let trimmed = substring.trim();
        if trimmed.len() >= self.min_substring_len {
            trimmed.to_string()
        } else {
            substring.to_string()
        }
    }

    /// Generate pseudo-query from passage key terms.
    fn generate_pseudo_query(&self, passage: &str) -> String {
        // Simple heuristic: extract key terms (words that appear frequently)
        let words: Vec<&str> = passage
            .split_whitespace()
            .filter(|w| w.len() > 3) // Filter short words
            .collect();

        if words.is_empty() {
            return "what is".to_string();
        }

        // Count word frequencies
        use std::collections::HashMap;
        let mut freq: HashMap<&str, usize> = HashMap::new();
        for word in &words {
            *freq.entry(word).or_insert(0) += 1;
        }

        // Get top 3-5 most frequent words
        let mut sorted_words: Vec<(&str, usize)> = freq.into_iter().collect();
        sorted_words.sort_by(|a, b| b.1.cmp(&a.1));
        
        let top_words: Vec<&str> = sorted_words
            .iter()
            .take(5)
            .map(|(word, _)| *word)
            .collect();

        // Format as query
        if top_words.is_empty() {
            "what is".to_string()
        } else {
            format!("what is {}", top_words.join(" "))
        }
    }
}

impl IdentifierGenerator for SimpleIdentifierGenerator {
    fn generate(&self, passage: &str, _passage_id: u32) -> MultiviewIdentifier {
        let title = self.extract_title(passage);
        let substring = self.sample_substring(passage);
        let pseudo_query = self.generate_pseudo_query(passage);

        MultiviewIdentifier::new(title, substring, pseudo_query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_type_prefix() {
        assert_eq!(IdentifierType::Title.prefix(), "title");
        assert_eq!(IdentifierType::Substring.prefix(), "substring");
        assert_eq!(IdentifierType::PseudoQuery.prefix(), "pseudo-query");
    }

    #[test]
    fn test_multiview_identifier() {
        let id = MultiviewIdentifier::new(
            "Title".to_string(),
            "Substring".to_string(),
            "Pseudo-query".to_string(),
        );

        assert_eq!(id.get(IdentifierType::Title), "Title");
        assert_eq!(id.get(IdentifierType::Substring), "Substring");
        assert_eq!(id.get(IdentifierType::PseudoQuery), "Pseudo-query");
        assert_eq!(id.all().len(), 3);
    }

    #[test]
    fn test_simple_identifier_generator() {
        let generator = SimpleIdentifierGenerator::new();
        let passage = "Prime Rate in Canada is a guideline interest rate used by banks on loans for their most creditworthy clients. The prime rate rises and falls with the ebb and flow of the Canadian economy.";

        let identifiers = generator.generate(passage, 0);

        assert!(!identifiers.title.is_empty());
        assert!(!identifiers.substring.is_empty());
        assert!(!identifiers.pseudo_query.is_empty());
        assert!(identifiers.substring.len() >= 20);
    }

    #[test]
    fn test_extract_title() {
        let generator = SimpleIdentifierGenerator::new();
        let passage = "Prime Rate in Canada\n\nThis is a detailed explanation...";
        let title = generator.extract_title(passage);
        assert_eq!(title, "Prime Rate in Canada");
    }

    #[test]
    fn test_sample_substring() {
        let generator = SimpleIdentifierGenerator::new();
        let passage = "This is a long passage with many words that we want to sample a substring from. It should be between 20 and 100 characters ideally.";
        let substring = generator.sample_substring(passage);
        assert!(substring.len() >= 20);
        assert!(substring.len() <= 100);
    }

    #[test]
    fn test_generate_pseudo_query() {
        let generator = SimpleIdentifierGenerator::new();
        let passage = "Machine learning is a subset of artificial intelligence that focuses on algorithms and statistical models.";
        let pseudo_query = generator.generate_pseudo_query(passage);
        assert!(pseudo_query.contains("what is"));
        assert!(pseudo_query.len() > 10);
    }
}

