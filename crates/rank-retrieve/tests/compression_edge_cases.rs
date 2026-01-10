//! Edge case tests for compression.

#[cfg(feature = "id-compression")]
mod tests {
    use rank_retrieve::compression::{RocCompressor, IdSetCompressor, CompressionError};

    #[test]
    fn test_empty_set() {
        let compressor = RocCompressor::new();
        let compressed = compressor.compress_set(&[], 1000).unwrap();
        assert!(compressed.is_empty());
        
        let decompressed = compressor.decompress_set(&[], 1000).unwrap();
        assert!(decompressed.is_empty());
    }
    
    #[test]
    fn test_single_element() {
        let compressor = RocCompressor::new();
        let ids = vec![42];
        
        let compressed = compressor.compress_set(&ids, 1000).unwrap();
        let decompressed = compressor.decompress_set(&compressed, 1000).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_two_elements() {
        let compressor = RocCompressor::new();
        let ids = vec![10, 20];
        
        let compressed = compressor.compress_set(&ids, 1000).unwrap();
        let decompressed = compressor.decompress_set(&compressed, 1000).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_all_zeros() {
        let compressor = RocCompressor::new();
        let ids = vec![0, 1, 2, 3];
        
        let compressed = compressor.compress_set(&ids, 1000).unwrap();
        let decompressed = compressor.decompress_set(&compressed, 1000).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_max_universe_size() {
        let compressor = RocCompressor::new();
        let universe_size = u32::MAX;
        let ids = vec![0, 1000, 10000, universe_size - 1];
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_very_small_universe() {
        let compressor = RocCompressor::new();
        let universe_size = 10;
        let ids = vec![0, 5, 9];
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_universe_size_one() {
        let compressor = RocCompressor::new();
        let universe_size = 1;
        let ids = vec![0];
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_id_equals_universe_size() {
        let compressor = RocCompressor::new();
        let universe_size = 1000;
        let ids = vec![1000];  // Equal to universe_size (should fail)
        
        let result = compressor.compress_set(&ids, universe_size);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_id_exceeds_universe_size() {
        let compressor = RocCompressor::new();
        let universe_size = 1000;
        let ids = vec![1001];  // Exceeds universe_size
        
        let result = compressor.compress_set(&ids, universe_size);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_very_large_deltas() {
        let compressor = RocCompressor::new();
        let ids = vec![0, 1_000_000, 2_000_000];
        let universe_size = 10_000_000;
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_duplicate_ids_rejected() {
        let compressor = RocCompressor::new();
        let ids = vec![1, 2, 2, 3];  // Duplicate
        
        let result = compressor.compress_set(&ids, 1000);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CompressionError::InvalidInput(_)));
    }
    
    #[test]
    fn test_unsorted_ids_rejected() {
        let compressor = RocCompressor::new();
        let ids = vec![3, 1, 2];  // Not sorted
        
        let result = compressor.compress_set(&ids, 1000);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CompressionError::InvalidInput(_)));
    }
    
    #[test]
    fn test_descending_order_rejected() {
        let compressor = RocCompressor::new();
        let ids = vec![10, 5, 1];  // Descending
        
        let result = compressor.compress_set(&ids, 1000);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_corrupted_compressed_data() {
        let compressor = RocCompressor::new();
        
        // Invalid varint encoding
        let corrupted = vec![0xFF, 0xFF, 0xFF, 0xFF];
        let result = compressor.decompress_set(&corrupted, 1000);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_truncated_compressed_data() {
        let compressor = RocCompressor::new();
        let ids = vec![1, 2, 3, 4, 5];
        let compressed = compressor.compress_set(&ids, 1000).unwrap();
        
        // Remove last byte
        let truncated = &compressed[..compressed.len() - 1];
        let result = compressor.decompress_set(truncated, 1000);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_empty_compressed_data() {
        let compressor = RocCompressor::new();
        let decompressed = compressor.decompress_set(&[], 1000).unwrap();
        assert!(decompressed.is_empty());
    }
    
    #[test]
    fn test_extra_data_after_compressed() {
        let compressor = RocCompressor::new();
        let ids = vec![1, 2, 3];
        let mut compressed = compressor.compress_set(&ids, 1000).unwrap();
        compressed.push(0xFF);  // Add extra byte
        
        let result = compressor.decompress_set(&compressed, 1000);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_very_large_set() {
        let compressor = RocCompressor::new();
        let num_ids = 100_000;
        let universe_size = 10_000_000;
        let ids: Vec<u32> = (0..num_ids).map(|i| i * 100).collect();
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_set_with_one_id_at_each_boundary() {
        let compressor = RocCompressor::new();
        let universe_size = 1000;
        let ids = vec![0, universe_size - 1];
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_all_ids_in_universe() {
        let compressor = RocCompressor::new();
        let universe_size = 100;
        let ids: Vec<u32> = (0..universe_size).collect();
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        assert_eq!(ids, decompressed);
    }
    
    #[test]
    fn test_estimate_size_edge_cases() {
        let compressor = RocCompressor::new();
        
        // Zero IDs
        assert_eq!(compressor.estimate_size(0, 1000), 0);
        
        // Single ID
        let estimate = compressor.estimate_size(1, 1000);
        assert!(estimate > 0);
        assert!(estimate < 100);  // Should be small
        
        // Very large set
        let estimate = compressor.estimate_size(1_000_000, 10_000_000);
        assert!(estimate > 0);
        assert!(estimate < 4_000_000);  // Less than uncompressed
    }
    
    #[test]
    fn test_bits_per_id_edge_cases() {
        let compressor = RocCompressor::new();
        
        // Zero IDs
        assert_eq!(compressor.bits_per_id(0, 1000), 0.0);
        
        // Single ID
        let bits = compressor.bits_per_id(1, 1000);
        assert!(bits > 0.0);
        assert!(bits < 20.0);  // Less than log2(1000)
        
        // Large set
        let bits = compressor.bits_per_id(10000, 1_000_000);
        assert!(bits > 0.0);
        assert!(bits < 30.0);  // Less than log2(1_000_000)
    }
    
    #[test]
    fn test_compression_with_different_precisions() {
        let ids: Vec<u32> = (0..1000).collect();
        let universe_size = 100_000;
        
        let compressor1 = RocCompressor::new();
        let compressor2 = RocCompressor::with_precision(1 << 16);
        
        let compressed1 = compressor1.compress_set(&ids, universe_size).unwrap();
        let compressed2 = compressor2.compress_set(&ids, universe_size).unwrap();
        
        // Both should decompress correctly
        let decompressed1 = compressor1.decompress_set(&compressed1, universe_size).unwrap();
        let decompressed2 = compressor2.decompress_set(&compressed2, universe_size).unwrap();
        
        assert_eq!(ids, decompressed1);
        assert_eq!(ids, decompressed2);
    }
    
    #[test]
    fn test_round_trip_preserves_uniqueness() {
        let compressor = RocCompressor::new();
        let ids = vec![1, 5, 10, 20, 50];
        let universe_size = 1000;
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        
        // Decompressed should have same length (no duplicates)
        assert_eq!(ids.len(), decompressed.len());
        
        // All IDs should be unique
        let mut sorted = decompressed.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), decompressed.len());
    }
}
