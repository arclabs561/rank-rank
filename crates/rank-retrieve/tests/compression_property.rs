//! Property-based tests for compression correctness.

#[cfg(feature = "id-compression")]
mod tests {
    use proptest::prelude::*;
    use rank_retrieve::compression::{RocCompressor, IdSetCompressor};

    proptest! {
        #[test]
        fn prop_compression_round_trip_all_sizes(
            ids in prop::collection::vec(0u32..100000, 0..10000)
        ) {
            let mut sorted_ids = ids;
            sorted_ids.sort();
            sorted_ids.dedup();
            
            if sorted_ids.is_empty() {
                return Ok(());
            }
            
            let max_id = *sorted_ids.iter().max().unwrap();
            let universe_size = (max_id + 10000).max(100000);
            
            let compressor = RocCompressor::new();
            let compressed = compressor.compress_set(&sorted_ids, universe_size)?;
            let decompressed = compressor.decompress_set(&compressed, universe_size)?;
            
            prop_assert_eq!(sorted_ids, decompressed);
        }
        
        #[test]
        fn prop_compression_preserves_order(
            ids in prop::collection::vec(0u32..10000, 1..1000)
        ) {
            let mut sorted_ids = ids;
            sorted_ids.sort();
            sorted_ids.dedup();
            
            let max_id = *sorted_ids.iter().max().unwrap();
            let universe_size = (max_id + 1000).max(10000);
            
            let compressor = RocCompressor::new();
            let compressed = compressor.compress_set(&sorted_ids, universe_size)?;
            let decompressed = compressor.decompress_set(&compressed, universe_size)?;
            
            // Decompressed should be sorted
            prop_assert!(decompressed.windows(2).all(|w| w[0] < w[1]));
        }
        
        #[test]
        fn prop_compression_no_data_loss(
            ids in prop::collection::vec(0u32..50000, 1..5000)
        ) {
            let mut sorted_ids = ids;
            sorted_ids.sort();
            sorted_ids.dedup();
            
            let max_id = *sorted_ids.iter().max().unwrap();
            let universe_size = (max_id + 5000).max(50000);
            
            let compressor = RocCompressor::new();
            let compressed = compressor.compress_set(&sorted_ids, universe_size)?;
            let decompressed = compressor.decompress_set(&compressed, universe_size)?;
            
            // All original IDs should be present
            prop_assert_eq!(sorted_ids.len(), decompressed.len());
            for id in &sorted_ids {
                prop_assert!(decompressed.contains(id));
            }
        }
        
        #[test]
        fn prop_compression_ratio_monotonic(
            num_ids in 100usize..2000,
            spacing in 1u32..100
        ) {
            let universe_size = (num_ids as u32 * spacing * 3).max(10000);
            
            // Create two sets: one with more IDs
            let ids1: Vec<u32> = (0..num_ids).map(|i| i as u32 * spacing).collect();
            let ids2: Vec<u32> = (0..(num_ids * 2)).map(|i| i as u32 * spacing).collect();
            
            let compressor = RocCompressor::new();
            let compressed1 = compressor.compress_set(&ids1, universe_size)?;
            let compressed2 = compressor.compress_set(&ids2, universe_size)?;
            
            // Larger set should compress to larger size (but better ratio)
            prop_assert!(compressed2.len() >= compressed1.len());
        }
        
        #[test]
        fn prop_compression_deterministic(
            ids in prop::collection::vec(0u32..10000, 1..1000)
        ) {
            let mut sorted_ids = ids;
            sorted_ids.sort();
            sorted_ids.dedup();
            
            let max_id = *sorted_ids.iter().max().unwrap();
            let universe_size = (max_id + 1000).max(10000);
            
            let compressor = RocCompressor::new();
            let compressed1 = compressor.compress_set(&sorted_ids, universe_size)?;
            let compressed2 = compressor.compress_set(&sorted_ids, universe_size)?;
            
            // Same input should produce same output
            prop_assert_eq!(compressed1, compressed2);
        }
        
        #[test]
        fn prop_decompression_handles_all_universe_sizes(
            ids in prop::collection::vec(0u32..1000, 1..100),
            universe_size in 1000u32..100000
        ) {
            let mut sorted_ids = ids;
            sorted_ids.sort();
            sorted_ids.dedup();
            
            // Filter IDs that fit in universe
            sorted_ids.retain(|&id| id < universe_size);
            
            if sorted_ids.is_empty() {
                return Ok(());
            }
            
            let compressor = RocCompressor::new();
            let compressed = compressor.compress_set(&sorted_ids, universe_size)?;
            let decompressed = compressor.decompress_set(&compressed, universe_size)?;
            
            prop_assert_eq!(sorted_ids, decompressed);
        }
        
        #[test]
        fn prop_compression_works_with_boundary_values(
            universe_size in 100u32..10000
        ) {
            // Test with IDs at boundaries
            let ids = vec![0, universe_size - 1];
            
            let compressor = RocCompressor::new();
            let compressed = compressor.compress_set(&ids, universe_size)?;
            let decompressed = compressor.decompress_set(&compressed, universe_size)?;
            
            prop_assert_eq!(ids, decompressed);
        }
        
        #[test]
        fn prop_compression_handles_consecutive_ids(
            start in 0u32..10000,
            count in 10usize..1000
        ) {
            let ids: Vec<u32> = (start..start + count as u32).collect();
            let universe_size = (start + count as u32 + 1000).max(10000);
            
            let compressor = RocCompressor::new();
            let compressed = compressor.compress_set(&ids, universe_size)?;
            let decompressed = compressor.decompress_set(&compressed, universe_size)?;
            
            prop_assert_eq!(ids, decompressed);
        }
        
        #[test]
        fn prop_compression_handles_sparse_ids(
            start in 0u32..10000,
            count in 10usize..500,
            spacing in 10u32..1000
        ) {
            let ids: Vec<u32> = (0..count).map(|i| start + i as u32 * spacing).collect();
            let universe_size = (start + count as u32 * spacing + 10000).max(100000);
            
            let compressor = RocCompressor::new();
            let compressed = compressor.compress_set(&ids, universe_size)?;
            let decompressed = compressor.decompress_set(&compressed, universe_size)?;
            
            prop_assert_eq!(ids, decompressed);
        }
    }
}
