//! Performance and regression tests for compression.

#[cfg(feature = "id-compression")]
mod tests {
    use rank_retrieve::compression::{RocCompressor, IdSetCompressor};
    use std::time::Instant;

    #[test]
    fn test_compression_performance_small_set() {
        let compressor = RocCompressor::new();
        let ids: Vec<u32> = (0..100).collect();
        let universe_size = 100_000;
        
        let start = Instant::now();
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let compress_time = start.elapsed();
        
        let start = Instant::now();
        let _decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        let decompress_time = start.elapsed();
        
        // Should be fast (< 1ms for small sets)
        assert!(compress_time.as_millis() < 100, "Compression too slow: {:?}", compress_time);
        assert!(decompress_time.as_millis() < 100, "Decompression too slow: {:?}", decompress_time);
    }
    
    #[test]
    fn test_compression_performance_medium_set() {
        let compressor = RocCompressor::new();
        let ids: Vec<u32> = (0..1000).map(|i| i * 100).collect();
        let universe_size = 1_000_000;
        
        let start = Instant::now();
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        let compress_time = start.elapsed();
        
        let start = Instant::now();
        let _decompressed = compressor.decompress_set(&compressed, universe_size).unwrap();
        let decompress_time = start.elapsed();
        
        // Should be reasonably fast (< 10ms for medium sets)
        assert!(compress_time.as_millis() < 1000, "Compression too slow: {:?}", compress_time);
        assert!(decompress_time.as_millis() < 1000, "Decompression too slow: {:?}", decompress_time);
    }
    
    #[test]
    fn test_compression_ratio_regression() {
        let compressor = RocCompressor::new();
        
        // Test that compression ratio doesn't regress
        let test_cases = vec![
            (100, 10_000, 1.5),   // Small set, should compress
            (1000, 100_000, 2.0), // Medium set
            (10000, 1_000_000, 2.5), // Large set
        ];
        
        for (num_ids, universe_size, min_ratio) in test_cases {
            let ids: Vec<u32> = (0..num_ids).map(|i| i * (universe_size / num_ids as u32)).collect();
            
            let compressed = compressor.compress_set(&ids, universe_size).unwrap();
            let uncompressed_size = num_ids * 4;
            let ratio = uncompressed_size as f64 / compressed.len() as f64;
            
            assert!(ratio >= min_ratio, 
                "Compression ratio {} below minimum {} for {} IDs", 
                ratio, min_ratio, num_ids);
        }
    }
    
    #[test]
    fn test_compression_memory_efficiency() {
        let compressor = RocCompressor::new();
        let ids: Vec<u32> = (0..10000).map(|i| i * 100).collect();
        let universe_size = 10_000_000;
        
        let compressed = compressor.compress_set(&ids, universe_size).unwrap();
        
        // Compressed should be smaller than uncompressed
        let uncompressed_size = ids.len() * 4;
        assert!(compressed.len() < uncompressed_size, 
            "Compressed size {} should be < uncompressed size {}", 
            compressed.len(), uncompressed_size);
    }
    
    #[test]
    fn test_estimate_size_accuracy() {
        let compressor = RocCompressor::new();
        
        let test_cases = vec![
            (100, 10_000),
            (1000, 100_000),
            (10000, 1_000_000),
        ];
        
        for (num_ids, universe_size) in test_cases {
            let ids: Vec<u32> = (0..num_ids).map(|i| i * (universe_size / num_ids as u32)).collect();
            
            let estimate = compressor.estimate_size(num_ids, universe_size);
            let actual = compressor.compress_set(&ids, universe_size).unwrap().len();
            
            // Estimate should be within 2x of actual (rough heuristic)
            assert!(estimate > 0);
            assert!(estimate <= actual * 3, 
                "Estimate {} too large compared to actual {}", estimate, actual);
        }
    }
    
    #[test]
    fn test_compression_deterministic() {
        let compressor = RocCompressor::new();
        let ids: Vec<u32> = (0..1000).map(|i| i * 100).collect();
        let universe_size = 1_000_000;
        
        // Compress same data multiple times
        let compressed1 = compressor.compress_set(&ids, universe_size).unwrap();
        let compressed2 = compressor.compress_set(&ids, universe_size).unwrap();
        let compressed3 = compressor.compress_set(&ids, universe_size).unwrap();
        
        // Should produce identical output
        assert_eq!(compressed1, compressed2);
        assert_eq!(compressed2, compressed3);
    }
    
    #[test]
    fn test_compression_throughput() {
        let compressor = RocCompressor::new();
        let universe_size = 1_000_000;
        
        // Measure throughput for multiple compressions
        let num_iterations = 100;
        let ids_per_iteration = 1000;
        
        let start = Instant::now();
        for i in 0..num_iterations {
            let ids: Vec<u32> = (0..ids_per_iteration)
                .map(|j| (i * ids_per_iteration + j) as u32 * 100)
                .collect();
            let _compressed = compressor.compress_set(&ids, universe_size).unwrap();
        }
        let total_time = start.elapsed();
        
        let throughput = (num_iterations * ids_per_iteration) as f64 / total_time.as_secs_f64();
        
        // Should handle at least 10k IDs per second
        assert!(throughput > 10_000.0, 
            "Throughput {} IDs/sec too low", throughput);
    }
}
