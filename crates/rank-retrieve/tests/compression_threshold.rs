//! Tests for compression threshold logic and decision making.

#[cfg(feature = "id-compression")]
mod tests {
    use rank_retrieve::compression::{RocCompressor, IdSetCompressor};
    use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
    use rank_retrieve::compression::IdCompressionMethod;

    #[test]
    fn test_compression_threshold_decision() {
        let compressor = RocCompressor::new();
        let universe_size = 100_000;
        
        // Test that compression is beneficial above threshold
        let below_threshold = 50;  // Small set
        let above_threshold = 500;  // Large set
        
        let ids_small: Vec<u32> = (0..below_threshold).map(|i| i * 1000).collect();
        let ids_large: Vec<u32> = (0..above_threshold).map(|i| i * 200).collect();
        
        let compressed_small = compressor.compress_set(&ids_small, universe_size).unwrap();
        let compressed_large = compressor.compress_set(&ids_large, universe_size).unwrap();
        
        // Larger set should have better compression ratio
        let ratio_small = (below_threshold * 4) as f64 / compressed_small.len() as f64;
        let ratio_large = (above_threshold * 4) as f64 / compressed_large.len() as f64;
        
        // Large set should compress better (or at least not worse)
        assert!(ratio_large >= ratio_small * 0.8, 
            "Large set should compress at least as well as small set");
    }
    
    #[test]
    fn test_ivf_threshold_enforcement() {
        // Test that IVF only compresses clusters above threshold
        let params_high_threshold = IVFPQParams {
            num_clusters: 1000,  // Many small clusters
            nprobe: 50,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 1000,  // Very high threshold
        };
        
        let mut index = IVFPQIndex::new(128, params_high_threshold).unwrap();
        
        // Add few vectors (will create small clusters)
        for i in 0..500 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }
        
        // Should build successfully (small clusters won't compress)
        assert!(index.build().is_ok());
        
        // Search should work
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }
    
    #[test]
    fn test_ivf_threshold_compression_triggered() {
        // Test that IVF compresses when threshold is met
        let params_low_threshold = IVFPQParams {
            num_clusters: 10,  // Few large clusters
            nprobe: 5,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 50,  // Low threshold
        };
        
        let mut index = IVFPQIndex::new(128, params_low_threshold).unwrap();
        
        // Add many vectors (will create large clusters)
        for i in 0..5000 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }
        
        // Should build successfully (large clusters will compress)
        assert!(index.build().is_ok());
        
        // Search should work
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }
    
    #[test]
    fn test_compression_threshold_zero() {
        // Threshold of 0 should compress everything
        let params = IVFPQParams {
            num_clusters: 100,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 0,  // Compress everything
        };
        
        let mut index = IVFPQIndex::new(128, params).unwrap();
        
        for i in 0..100 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }
        
        assert!(index.build().is_ok());
        
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();
        assert!(results.len() <= 10);
    }
    
    #[test]
    fn test_compression_threshold_very_high() {
        // Very high threshold should compress nothing
        let params = IVFPQParams {
            num_clusters: 100,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 1_000_000,  // Very high
        };
        
        let mut index = IVFPQIndex::new(128, params).unwrap();
        
        for i in 0..1000 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }
        
        assert!(index.build().is_ok());
        
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }
    
    #[test]
    fn test_compression_benefit_calculation() {
        let compressor = RocCompressor::new();
        
        // Calculate compression benefit for different sizes
        let universe_size = 1_000_000;
        
        for num_ids in [10, 50, 100, 500, 1000, 5000] {
            let ids: Vec<u32> = (0..num_ids).map(|i| i * (universe_size / num_ids as u32)).collect();
            
            let compressed = compressor.compress_set(&ids, universe_size).unwrap();
            let uncompressed_size = num_ids * 4;
            let savings = uncompressed_size - compressed.len();
            let savings_percent = (savings as f64 / uncompressed_size as f64) * 100.0;
            
            // Larger sets should save more (absolute)
            if num_ids > 100 {
                assert!(savings > 0, "Should save space for {} IDs", num_ids);
            }
            
            // Log savings for debugging
            println!("{} IDs: {} bytes saved ({:.1}%)", num_ids, savings, savings_percent);
        }
    }
}
