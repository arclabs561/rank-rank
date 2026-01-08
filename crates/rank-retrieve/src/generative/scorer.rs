//! Heuristic scoring function for generative retrieval.
//!
//! This module is part of the `generative` module. See `generative/mod.rs` for the main entry point.
//!
//! Converts predicted identifiers to passage scores using the heuristic:
//! s(q, p) = Σ_{ip ∈ Ip} s_ip
//!
//! Where:
//! - Ip = set of predicted identifiers that appear in passage p
//! - s_ip = language model score of identifier ip

/// Heuristic scorer that converts identifiers to passage scores.
#[derive(Debug, Clone)]
pub struct HeuristicScorer {
    /// Whether to use case-insensitive matching.
    case_insensitive: bool,
    /// Minimum identifier length to consider.
    min_identifier_len: usize,
}

impl Default for HeuristicScorer {
    fn default() -> Self {
        Self {
            case_insensitive: true,
            min_identifier_len: 3,
        }
    }
}

impl HeuristicScorer {
    /// Create a new heuristic scorer with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case-insensitive matching.
    pub fn with_case_insensitive(mut self, case_insensitive: bool) -> Self {
        self.case_insensitive = case_insensitive;
        self
    }

    /// Set minimum identifier length.
    pub fn with_min_identifier_len(mut self, min_len: usize) -> Self {
        self.min_identifier_len = min_len;
        self
    }

    /// Score a single passage given predicted identifiers.
    ///
    /// Computes the heuristic score: `s(q, p) = Σ_{ip ∈ Ip} s_ip` where `Ip` is the set
    /// of predicted identifiers that appear in passage `p`, and `s_ip` is the language
    /// model score of identifier `ip`.
    ///
    /// # Arguments
    ///
    /// * `passage` - The passage text to score
    /// * `predicted_identifiers` - Vector of (identifier, score) pairs from the model
    ///
    /// # Returns
    ///
    /// The total score for this passage (sum of matching identifier scores).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::generative::HeuristicScorer;
    ///
    /// let scorer = HeuristicScorer::new();
    /// let passage = "Prime Rate in Canada is a guideline interest rate";
    /// let identifiers = vec![
    ///     ("Prime Rate in Canada".to_string(), 5.0),
    ///     ("interest rate".to_string(), 3.0),
    ///     ("unrelated term".to_string(), 2.0),
    /// ];
    ///
    /// let score = scorer.score_passage(passage, &identifiers);
    /// assert_eq!(score, 8.0); // 5.0 + 3.0 (unrelated term doesn't match)
    /// ```
    ///
    /// # Performance
    ///
    /// Time complexity: O(n * m) where n is number of identifiers and m is passage length.
    /// For typical workloads (10-50 identifiers, 100-500 char passages), this is <1ms.
    /// Case-insensitive matching adds overhead but improves recall.
    /// Unicode normalization (when enabled) adds additional overhead but improves matching
    /// across different Unicode representations (e.g., "é" vs "e\u{0301}").
    pub fn score_passage(&self, passage: &str, predicted_identifiers: &[(String, f32)]) -> f32 {
        if predicted_identifiers.is_empty() {
            return 0.0;
        }

        // Normalize passage (case-insensitive and optionally Unicode normalization)
        let passage_normalized = {
            let mut normalized = if self.case_insensitive {
                passage.to_lowercase()
            } else {
                passage.to_string()
            };

            #[cfg(feature = "unicode")]
            {
                use unicode_normalization::UnicodeNormalization;
                normalized = normalized.nfc().collect::<String>();
            }

            normalized
        };

        let mut total_score = 0.0;

        for (identifier, score) in predicted_identifiers {
            // Filter short identifiers (check before normalization)
            if identifier.len() < self.min_identifier_len {
                continue;
            }

            // Normalize identifier (case-insensitive and optionally Unicode normalization)
            let identifier_normalized = {
                let mut normalized = if self.case_insensitive {
                    identifier.to_lowercase()
                } else {
                    identifier.clone()
                };

                #[cfg(feature = "unicode")]
                {
                    use unicode_normalization::UnicodeNormalization;
                    normalized = normalized.nfc().collect::<String>();
                }

                normalized
            };

            // Check if identifier appears in passage
            if passage_normalized.contains(&identifier_normalized) {
                total_score += score;
            }
        }

        total_score
    }

    /// Score multiple passages in batch.
    ///
    /// Efficiently scores multiple passages using the same set of predicted identifiers.
    /// Results are automatically sorted by score (descending).
    ///
    /// # Performance Optimizations
    ///
    /// - **Normalized identifier caching**: Normalizes identifiers once and reuses for all passages
    /// - **Batch processing**: Processes all passages in a single pass
    ///
    /// # Arguments
    ///
    /// * `passages` - Vector of (passage_id, passage_text) tuples
    /// * `predicted_identifiers` - Vector of (identifier, score) pairs
    ///
    /// # Returns
    ///
    /// Vector of (passage_id, score) tuples, sorted by score (descending).
    ///
    /// # Example
    ///
    /// ```rust
    /// use rank_retrieve::generative::HeuristicScorer;
    ///
    /// let scorer = HeuristicScorer::new();
    /// let passages = vec![
    ///     (0u32, "Prime Rate in Canada"),
    ///     (1u32, "Machine learning tutorial"),
    ///     (2u32, "Interest rate guidelines"),
    /// ];
    /// let identifiers = vec![
    ///     ("Prime Rate".to_string(), 5.0),
    ///     ("interest rate".to_string(), 3.0),
    /// ];
    ///
    /// let results = scorer.score_batch(&passages, &identifiers);
    /// // Results are sorted by score descending
    /// assert!(results[0].1 >= results[1].1);
    /// ```
    ///
    /// # Performance
    ///
    /// Time complexity: O(p * n * m) where p is number of passages, n is number of identifiers,
    /// and m is average passage length. For 100 passages with 20 identifiers, typically <10ms.
    /// Normalized identifier caching reduces overhead for repeated normalization.
    pub fn score_batch(
        &self,
        passages: &[(u32, &str)],
        predicted_identifiers: &[(String, f32)],
    ) -> Vec<(u32, f32)> {
        // Pre-normalize identifiers once (caching optimization)
        // This avoids repeated normalization for each passage
        let normalized_identifiers: Vec<(String, f32)> = predicted_identifiers
            .iter()
            .filter(|(id, _)| id.len() >= self.min_identifier_len)
            .map(|(identifier, score)| {
                let normalized = {
                    let norm = if self.case_insensitive {
                        identifier.to_lowercase()
                    } else {
                        identifier.clone()
                    };

                    #[cfg(feature = "unicode")]
                    {
                        use unicode_normalization::UnicodeNormalization;
                        norm.nfc().collect::<String>()
                    }
                    #[cfg(not(feature = "unicode"))]
                    {
                        norm
                    }
                };
                (normalized, *score)
            })
            .collect();

        // Score passages using pre-normalized identifiers
        let mut passage_scores: Vec<(u32, f32)> = passages
            .iter()
            .map(|(passage_id, passage_text)| {
                // Normalize passage once
                let passage_normalized = {
                    let normalized = if self.case_insensitive {
                        passage_text.to_lowercase()
                    } else {
                        passage_text.to_string()
                    };

                    #[cfg(feature = "unicode")]
                    {
                        use unicode_normalization::UnicodeNormalization;
                        normalized.nfc().collect::<String>()
                    }
                    #[cfg(not(feature = "unicode"))]
                    {
                        normalized
                    }
                };

                // Score using pre-normalized identifiers
                let mut total_score = 0.0;
                for (identifier_normalized, score) in &normalized_identifiers {
                    if passage_normalized.contains(identifier_normalized) {
                        total_score += score;
                    }
                }

                (*passage_id, total_score)
            })
            .collect();

        // Sort by score (descending)
        passage_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        passage_scores
    }

    /// Find which identifiers match a passage.
    ///
    /// Useful for debugging and explainability.
    pub fn find_matching_identifiers(
        &self,
        passage: &str,
        predicted_identifiers: &[(String, f32)],
    ) -> Vec<(String, f32)> {
        // Normalize passage (case-insensitive and optionally Unicode normalization)
        let passage_normalized = {
            let mut normalized = if self.case_insensitive {
                passage.to_lowercase()
            } else {
                passage.to_string()
            };

            #[cfg(feature = "unicode")]
            {
                use unicode_normalization::UnicodeNormalization;
                normalized = normalized.nfc().collect::<String>();
            }

            normalized
        };

        let mut matching = Vec::new();

        for (identifier, score) in predicted_identifiers {
            if identifier.len() < self.min_identifier_len {
                continue;
            }

            // Normalize identifier (case-insensitive and optionally Unicode normalization)
            let identifier_normalized = {
                let mut normalized = if self.case_insensitive {
                    identifier.to_lowercase()
                } else {
                    identifier.clone()
                };

                #[cfg(feature = "unicode")]
                {
                    use unicode_normalization::UnicodeNormalization;
                    normalized = normalized.nfc().collect::<String>();
                }

                normalized
            };

            if passage_normalized.contains(&identifier_normalized) {
                matching.push((identifier.clone(), *score));
            }
        }

        matching
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_passage_basic() {
        let scorer = HeuristicScorer::new();
        let passage = "Prime Rate in Canada is a guideline interest rate";
        let identifiers = vec![
            ("Prime Rate in Canada".to_string(), 5.0),
            ("interest rate".to_string(), 3.0),
            ("unrelated term".to_string(), 2.0),
        ];

        let score = scorer.score_passage(passage, &identifiers);
        assert_eq!(score, 8.0); // 5.0 + 3.0
    }

    #[test]
    fn test_score_passage_case_insensitive() {
        let scorer = HeuristicScorer::new().with_case_insensitive(true);
        let passage = "prime rate in canada";
        let identifiers = vec![("Prime Rate".to_string(), 5.0), ("CANADA".to_string(), 3.0)];

        let score = scorer.score_passage(passage, &identifiers);
        assert_eq!(score, 8.0);
    }

    #[test]
    fn test_score_passage_case_sensitive() {
        let scorer = HeuristicScorer::new().with_case_insensitive(false);
        let passage = "prime rate in canada";
        let identifiers = vec![
            ("Prime Rate".to_string(), 5.0), // Won't match
            ("prime rate".to_string(), 3.0), // Will match
        ];

        let score = scorer.score_passage(passage, &identifiers);
        assert_eq!(score, 3.0);
    }

    #[test]
    fn test_score_passage_min_length() {
        let scorer = HeuristicScorer::new().with_min_identifier_len(6);
        let passage = "Prime Rate in Canada";
        let identifiers = vec![
            ("Prime".to_string(), 5.0),      // Too short (5 chars < 6), filtered
            ("Prime Rate".to_string(), 3.0), // Matches (10 chars >= 6)
        ];

        let score = scorer.score_passage(passage, &identifiers);
        // Only "Prime Rate" should match (3.0), "Prime" is filtered
        assert_eq!(score, 3.0);
    }

    #[test]
    fn test_score_batch() {
        let scorer = HeuristicScorer::new();
        let passages = vec![
            (0u32, "Prime Rate in Canada"),
            (1u32, "Machine learning tutorial"),
            (2u32, "Interest rate guidelines"),
        ];
        let identifiers = vec![
            ("Prime Rate".to_string(), 5.0),
            ("interest rate".to_string(), 3.0),
            ("machine learning".to_string(), 4.0),
        ];

        let results = scorer.score_batch(&passages, &identifiers);

        // Should be sorted by score descending
        assert_eq!(results[0].0, 0); // "Prime Rate" + "interest rate" = 8.0
        assert_eq!(results[1].0, 1); // "machine learning" = 4.0
        assert_eq!(results[2].0, 2); // "interest rate" = 3.0
    }

    #[test]
    fn test_find_matching_identifiers() {
        let scorer = HeuristicScorer::new();
        let passage = "Prime Rate in Canada is important";
        let identifiers = vec![
            ("Prime Rate".to_string(), 5.0),
            ("Canada".to_string(), 3.0),
            ("unrelated".to_string(), 2.0),
        ];

        let matching = scorer.find_matching_identifiers(passage, &identifiers);
        assert_eq!(matching.len(), 2);
        assert!(matching.iter().any(|(id, _)| id == "Prime Rate"));
        assert!(matching.iter().any(|(id, _)| id == "Canada"));
    }

    #[test]
    fn test_score_passage_empty_identifiers() {
        let scorer = HeuristicScorer::new();
        let passage = "Some passage";
        let identifiers = vec![];

        let score = scorer.score_passage(passage, &identifiers);
        assert_eq!(score, 0.0);
    }
}
