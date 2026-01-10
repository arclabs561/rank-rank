//! Integration tests for HNSW with ID compression.

#[cfg(all(feature = "id-compression", feature = "hnsw"))]
mod tests {
    use rank_retrieve::dense::hnsw::graph::{HNSWIndex, HNSWParams};
    use rank_retrieve::compression::IdCompressionMethod;

    #[test]
    fn test_hnsw_with_compression() {
        let params = HNSWParams {
            m: 32,  // Large enough to trigger compression
            m_max: 32,
            ef_construction: 100,
            ef_search: 50,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,  // Compress if m >= 32
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        // Add vectors
        for i in 0..1000 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }

        // Build (compression happens here)
        index.build().unwrap();

        // Search should work
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10, 50).unwrap();

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|(_, score)| score.is_finite()));
    }

    #[test]
    fn test_hnsw_without_compression() {
        let params = HNSWParams {
            m: 16,  // Too small for compression
            m_max: 16,
            ef_construction: 100,
            ef_search: 50,
            id_compression: None,
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        // Add vectors
        for i in 0..1000 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }

        // Build
        index.build().unwrap();

        // Search should work
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10, 50).unwrap();

        assert_eq!(results.len(), 10);
    }
}
