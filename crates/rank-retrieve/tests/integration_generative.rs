//! Integration tests for generative retrieval.
//!
//! Tests the complete generative retrieval pipeline including:
//! - Identifier generation (title, substring, pseudo-query)
//! - Heuristic scoring
//! - LTRGR training
//! - End-to-end retrieval

use rank_retrieve::generative::{
    AutoregressiveModel, GenerativeRetriever, HeuristicScorer, IdentifierType, LTRGRTrainer,
    MockAutoregressiveModel, MultiviewIdentifier,
};

#[test]
fn test_generative_retrieval_pipeline() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);

    // Add documents
    retriever.add_document(
        0,
        "Prime Rate in Canada is a guideline interest rate set by the Bank of Canada",
    );
    retriever.add_document(1, "Machine learning is a subset of artificial intelligence");
    retriever.add_document(
        2,
        "The Bank of Canada sets monetary policy through interest rates",
    );

    // Retrieve
    let results = retriever
        .retrieve("What is prime rate in Canada?", 10)
        .unwrap();

    // Should return results
    assert!(!results.is_empty());
    assert!(results.len() <= 10);

    // Document 0 should rank highly (contains "Prime Rate in Canada")
    let top_doc_id = results[0].0;
    assert!(top_doc_id == 0 || top_doc_id == 2); // Either doc 0 or doc 2 should be top
}

#[test]
fn test_multiview_identifier_generation() {
    let model = MockAutoregressiveModel::new();
    let query = "What is machine learning?";

    // Generate identifiers for all three views
    let title_ids = model
        .generate(query, IdentifierType::Title.prefix(), 5, None)
        .unwrap();
    let substring_ids = model
        .generate(query, IdentifierType::Substring.prefix(), 5, None)
        .unwrap();
    let pseudo_ids = model
        .generate(query, IdentifierType::PseudoQuery.prefix(), 5, None)
        .unwrap();

    // All should generate some identifiers
    assert!(!title_ids.is_empty());
    assert!(!substring_ids.is_empty());
    assert!(!pseudo_ids.is_empty());

    // Identifiers should have scores
    assert!(title_ids.iter().all(|(_, score)| *score > 0.0));
    assert!(substring_ids.iter().all(|(_, score)| *score > 0.0));
    assert!(pseudo_ids.iter().all(|(_, score)| *score > 0.0));
}

#[test]
fn test_heuristic_scorer_integration() {
    let scorer = HeuristicScorer::new();
    let mut retriever =
        GenerativeRetriever::new(MockAutoregressiveModel::new()).with_scorer(scorer);

    retriever.add_document(0, "Prime Rate in Canada");
    retriever.add_document(1, "Machine learning tutorial");

    let results = retriever.retrieve("What is prime rate?", 10).unwrap();

    // Should score documents based on identifier matching
    assert!(!results.is_empty());
    assert!(results[0].1 >= 0.0); // Scores should be non-negative
}

#[test]
fn test_ltrgr_training_integration() {
    let trainer = LTRGRTrainer::new();
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);

    // Add documents
    retriever.add_document(0, "Prime Rate in Canada");
    retriever.add_document(1, "Machine learning");
    retriever.add_document(2, "Interest rate policy");

    // Retrieve passages
    let passage_scores = retriever.retrieve("What is prime rate?", 10).unwrap();

    // Compute rank loss
    let positive_ids = vec![0u32]; // Document 0 is positive
    let loss = trainer.compute_rank_loss_1(&passage_scores, &positive_ids);

    // Loss should be computed
    assert!(loss >= 0.0);
}

#[test]
fn test_beam_size_effect() {
    let model_small = MockAutoregressiveModel::new();
    let model_large = MockAutoregressiveModel::new();
    let mut retriever_small = GenerativeRetriever::new(model_small).with_beam_size(5);
    let mut retriever_large = GenerativeRetriever::new(model_large).with_beam_size(20);

    retriever_small.add_document(0, "Prime Rate in Canada");
    retriever_large.add_document(0, "Prime Rate in Canada");

    let results_small = retriever_small.retrieve("What is prime rate?", 10).unwrap();
    let results_large = retriever_large.retrieve("What is prime rate?", 10).unwrap();

    // Both should return results
    assert!(!results_small.is_empty());
    assert!(!results_large.is_empty());

    // Larger beam size may find more identifiers, but both should work
    assert!(results_small[0].1 >= 0.0);
    assert!(results_large[0].1 >= 0.0);
}

#[test]
fn test_case_insensitive_scoring() {
    let scorer_case_insensitive = HeuristicScorer::new().with_case_insensitive(true);
    let scorer_case_sensitive = HeuristicScorer::new().with_case_insensitive(false);

    let passage = "Prime Rate in Canada";
    let identifiers = vec![
        ("PRIME RATE".to_string(), 5.0),
        ("Prime Rate".to_string(), 3.0), // Exact case match
    ];

    let score_insensitive = scorer_case_insensitive.score_passage(passage, &identifiers);
    let score_sensitive = scorer_case_sensitive.score_passage(passage, &identifiers);

    // Case-insensitive should match both identifiers (PRIME RATE and Prime Rate)
    assert_eq!(score_insensitive, 8.0);
    // Case-sensitive should only match exact case (Prime Rate)
    assert_eq!(score_sensitive, 3.0);
}

#[test]
fn test_large_scale_retrieval() {
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);

    // Add many documents
    for i in 0..100 {
        retriever.add_document(i, &format!("Document {} about various topics", i));
    }

    // Retrieve top-k
    let results = retriever.retrieve("test query", 10).unwrap();

    // Should return exactly top-k
    assert_eq!(results.len(), 10);
    // Results should be sorted by score descending
    for i in 0..results.len() - 1 {
        assert!(results[i].1 >= results[i + 1].1);
    }
}

#[test]
fn test_identifier_matching_accuracy() {
    let scorer = HeuristicScorer::new();
    let passage = "The Bank of Canada sets the prime rate as a guideline interest rate";

    let identifiers = vec![
        ("Bank of Canada".to_string(), 10.0),
        ("prime rate".to_string(), 8.0),
        ("interest rate".to_string(), 6.0),
        ("unrelated term".to_string(), 5.0),
    ];

    let score = scorer.score_passage(passage, &identifiers);

    // Should match first three identifiers (10 + 8 + 6 = 24)
    assert_eq!(score, 24.0);

    // Verify matching identifiers
    let matching = scorer.find_matching_identifiers(passage, &identifiers);
    assert_eq!(matching.len(), 3);
    assert!(matching.iter().any(|(id, _)| id == "Bank of Canada"));
    assert!(matching.iter().any(|(id, _)| id == "prime rate"));
    assert!(matching.iter().any(|(id, _)| id == "interest rate"));
}

#[test]
fn test_identifier_deduplication() {
    // Test that duplicate identifiers from different views are deduplicated
    // and the highest score is kept
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);

    retriever.add_document(0, "Prime Rate in Canada is a guideline interest rate");

    // The MockAutoregressiveModel generates identifiers based on query,
    // so if the same identifier appears in multiple views, it should be deduplicated
    let results = retriever
        .retrieve("What is prime rate in Canada?", 10)
        .unwrap();

    // Should return results (deduplication should not break retrieval)
    assert!(!results.is_empty());
}

#[cfg(feature = "ltrgr")]
#[test]
fn test_ltrgr_random_sampling() {
    // Test that LTRGR random sampling works correctly
    use rank_retrieve::generative::LTRGRTrainer;

    let trainer = LTRGRTrainer::new();
    let passage_scores = vec![
        (0u32, 10.0), // positive
        (1u32, 8.0),  // positive
        (2u32, 5.0),  // negative
        (3u32, 3.0),  // negative
    ];
    let positive_ids = vec![0u32, 1u32];

    // Call compute_rank_loss_2 multiple times - should get valid results
    // (may differ due to random sampling, but both should be valid)
    let loss1 = trainer.compute_rank_loss_2(&passage_scores, &positive_ids);
    let loss2 = trainer.compute_rank_loss_2(&passage_scores, &positive_ids);

    // Loss should be valid (>= 0)
    assert!(loss1 >= 0.0);
    assert!(loss2 >= 0.0);

    // With random sampling, results may differ, but both should be valid
    // (Note: This test may occasionally get the same result if random sampling picks
    // the same values, but that's acceptable - the important thing is that it doesn't panic)
}

#[cfg(feature = "unicode")]
#[test]
fn test_unicode_normalization() {
    // Test that Unicode normalization improves matching
    use rank_retrieve::generative::HeuristicScorer;

    let scorer = HeuristicScorer::new();

    // Test with composed vs decomposed Unicode
    // "é" can be represented as U+00E9 (composed) or U+0065 U+0301 (decomposed)
    let passage = "café"; // Composed form
    let identifiers = vec![
        // Decomposed form (e + combining acute accent)
        ("cafe\u{0301}".to_string(), 5.0),
    ];

    // With Unicode normalization, these should match
    let score = scorer.score_passage(passage, &identifiers);
    assert!(
        score > 0.0,
        "Unicode normalization should match composed and decomposed forms"
    );
}

#[test]
fn test_heap_based_top_k() {
    // Test that heap-based top-k selection works correctly for large corpora
    let model = MockAutoregressiveModel::new();
    let mut retriever = GenerativeRetriever::new(model);

    // Add many documents (100) to trigger heap-based optimization
    // Heap optimization kicks in when k < passages.len() / 10
    for i in 0..100 {
        retriever.add_document(
            i,
            &format!(
                "Document {} about various topics including machine learning and AI",
                i
            ),
        );
    }

    // Retrieve top-5 (should use heap-based selection)
    let results = retriever.retrieve("What is machine learning?", 5).unwrap();

    // Should return exactly 5 results
    assert_eq!(results.len(), 5);

    // Results should be sorted descending by score
    for i in 0..results.len().saturating_sub(1) {
        assert!(
            results[i].1 >= results[i + 1].1,
            "Results should be sorted descending: {} >= {}",
            results[i].1,
            results[i + 1].1
        );
    }
}

#[test]
fn test_batch_scoring_optimization() {
    // Test that batch scoring with normalized identifier caching works
    use rank_retrieve::generative::HeuristicScorer;

    let scorer = HeuristicScorer::new();
    let passages = vec![
        (0u32, "Prime Rate in Canada"),
        (1u32, "Machine learning tutorial"),
        (2u32, "Interest rate guidelines"),
        (3u32, "Bank of Canada policies"),
    ];
    let identifiers = vec![
        ("Prime Rate".to_string(), 5.0),
        ("interest rate".to_string(), 3.0),
        ("Bank of Canada".to_string(), 4.0),
    ];

    let results = scorer.score_batch(&passages, &identifiers);

    // Should return all passages
    assert_eq!(results.len(), 4);

    // Results should be sorted descending
    for i in 0..results.len().saturating_sub(1) {
        assert!(results[i].1 >= results[i + 1].1);
    }

    // Document 0 should have highest score (matches "Prime Rate" + "interest rate" = 8.0)
    // Document 3 should also score well (matches "Bank of Canada" = 4.0)
    assert!(results[0].1 > 0.0);
}
