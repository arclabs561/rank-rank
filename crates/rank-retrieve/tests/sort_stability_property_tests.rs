//! Property-based tests for sort_unstable_by correctness.
//!
//! Verifies that unstable sorting produces correct results for ranking,
//! even though it may not preserve order of equal elements.

use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sort_unstable_by_correctness(
        scores in prop::collection::vec(
            (-1000.0f32..1000.0f32),
            1..1000
        )
    ) {
        // Property: sort_unstable_by should produce correctly sorted results
        let mut data: Vec<(usize, f32)> = scores.iter().enumerate().map(|(i, &s)| (i, s)).collect();
        data.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));

        // Check that results are sorted descending (allowing for NaN which total_cmp puts at end)
        for i in 0..data.len().saturating_sub(1) {
            let a = data[i].1;
            let b = data[i + 1].1;
            // For finite values, check descending order
            // NaN values are handled by total_cmp (they go to the end)
            if a.is_finite() && b.is_finite() {
                prop_assert!(
                    a >= b,
                    "Results should be sorted descending: {} >= {}",
                    a,
                    b
                );
            }
        }

        // Check that all original elements are present
        let original_ids: std::collections::HashSet<usize> = (0..scores.len()).collect();
        let sorted_ids: std::collections::HashSet<usize> = data.iter().map(|(id, _)| *id).collect();
        prop_assert_eq!(original_ids, sorted_ids, "All elements should be present");
    }

    #[test]
    fn test_sort_unstable_by_vs_stable_same_scores(
        scores in prop::collection::vec(
            (-1000.0f32..1000.0f32),
            1..100
        )
    ) {
        // Property: sort_unstable_by and sort_by should produce same score ordering
        // (ignoring order of equal elements, which is fine for ranking)
        let mut unstable: Vec<(usize, f32)> = scores.iter().enumerate().map(|(i, &s)| (i, s)).collect();
        let mut stable: Vec<(usize, f32)> = scores.iter().enumerate().map(|(i, &s)| (i, s)).collect();

        unstable.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
        stable.sort_by(|a, b| b.1.total_cmp(&a.1));

        // Check that scores are in same order (allowing for floating-point differences)
        for i in 0..unstable.len() {
            let tolerance = (unstable[i].1.abs() * 1e-5).max(1e-6);
            prop_assert!(
                (unstable[i].1 - stable[i].1).abs() < tolerance,
                "Scores should match: unstable[{}]={}, stable[{}]={}",
                i,
                unstable[i].1,
                i,
                stable[i].1
            );
        }
    }

    #[test]
    fn test_sort_unstable_by_top_k_consistency(
        scores in prop::collection::vec(
            (-1000.0f32..1000.0f32),
            10..1000
        ),
        k in 1usize..100
    ) {
        // Property: Top-k from full sort should match top-k from partial sort
        let k = k.min(scores.len());

        let mut full: Vec<(usize, f32)> = scores.iter().enumerate().map(|(i, &s)| (i, s)).collect();
        full.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));
        let full_top_k: Vec<f32> = full.iter().take(k).map(|(_, score)| *score).collect();

        // Simulate heap-based top-k
        use std::collections::BinaryHeap;
        use std::cmp::Reverse;

        #[derive(PartialEq, PartialOrd)]
        struct FloatOrd(f32);
        impl Eq for FloatOrd {}
        impl Ord for FloatOrd {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
            }
        }

        let mut heap: BinaryHeap<Reverse<(FloatOrd, usize)>> = BinaryHeap::with_capacity(k + 1);
        for (id, &score) in scores.iter().enumerate() {
            let score_ord = FloatOrd(score);
            if heap.len() < k {
                heap.push(Reverse((score_ord, id)));
            } else if let Some(&Reverse((FloatOrd(min_score), _))) = heap.peek() {
                // min_score is the f32 value extracted from FloatOrd wrapper
                if score > min_score {
                    heap.pop();
                    heap.push(Reverse((score_ord, id)));
                }
            }
        }

        let mut heap_top_k: Vec<f32> = heap
            .into_iter()
            .map(|Reverse((FloatOrd(score), _))| score) // score is f32 extracted from FloatOrd
            .collect();
        heap_top_k.sort_unstable_by(|a, b| b.total_cmp(a));

        // Top-k scores should match (allowing for floating-point differences)
        prop_assert_eq!(
            full_top_k.len(),
            heap_top_k.len(),
            "Top-k lengths should match"
        );
        for i in 0..full_top_k.len() {
            let tolerance = (full_top_k[i].abs() * 1e-5).max(1e-6);
            prop_assert!(
                (full_top_k[i] - heap_top_k[i]).abs() < tolerance,
                "Top-k scores should match: full[{}]={}, heap[{}]={}",
                i,
                full_top_k[i],
                i,
                heap_top_k[i]
            );
        }
    }

    #[test]
    fn test_sort_unstable_by_handles_nan(
        scores in prop::collection::vec(
            prop::num::f32::ANY,
            1..100
        )
    ) {
        // Property: sort_unstable_by should handle NaN values gracefully (no panic)
        let mut scores_with_nan = scores.clone();
        
        // Add some NaN values
        if !scores_with_nan.is_empty() {
            let len = scores_with_nan.len();
            scores_with_nan[0] = f32::NAN;
            if len > 1 {
                scores_with_nan[len / 2] = f32::NAN;
            }
        }

        let mut data: Vec<(usize, f32)> = scores_with_nan.iter().enumerate().map(|(i, &s)| (i, s)).collect();

        // Should not panic - this is the main property we're testing
        data.sort_unstable_by(|a, b| b.1.total_cmp(&a.1));

        // Verify that all elements are still present (no crashes)
        prop_assert_eq!(data.len(), scores_with_nan.len(), "All elements should be present");
        
        // Verify that finite values are sorted correctly (NaN handling is implementation-dependent)
        let finite_scores: Vec<f32> = data.iter()
            .filter(|(_, score)| score.is_finite())
            .map(|(_, score)| *score)
            .collect();
        
        for i in 0..finite_scores.len().saturating_sub(1) {
            prop_assert!(
                finite_scores[i] >= finite_scores[i + 1],
                "Finite values should be sorted descending"
            );
        }
    }
}
