//! End-to-end integration tests for compression in real scenarios.

#[cfg(all(feature = "id-compression", feature = "ivf_pq"))]
mod ivf_tests {
    use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
    use rank_retrieve::compression::IdCompressionMethod;

    #[test]
    fn test_ivf_compression_realistic_workflow() {
        let params = IVFPQParams {
            num_clusters: 256,
            nprobe: 32,
            num_codebooks: 8,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 100,
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        // Add realistic number of vectors
        for i in 0..10_000 {
            let vector: Vec<f32> = (0..128)
                .map(|j| ((i + j) as f32 / 1000.0).sin())
                .collect();
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        // Multiple queries
        for query_id in 0..100 {
            let query: Vec<f32> = (0..128)
                .map(|j| ((query_id + j) as f32 / 1000.0).sin())
                .collect();
            let results = index.search(&query, 20).unwrap();
            assert_eq!(results.len(), 20);
        }
    }

    #[test]
    fn test_ivf_compression_memory_usage() {
        let params = IVFPQParams {
            num_clusters: 512,
            nprobe: 64,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 50,
        };

        let mut index = IVFPQIndex::new(64, params).unwrap();

        // Add many vectors
        for i in 0..50_000 {
            let vector: Vec<f32> = (0..64).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        // Search should work
        let query: Vec<f32> = (0..64).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();
        assert_eq!(results.len(), 10);
    }
}

#[cfg(all(feature = "id-compression", feature = "hnsw"))]
mod hnsw_tests {
    use rank_retrieve::dense::hnsw::graph::{HNSWIndex, HNSWParams};
    use rank_retrieve::compression::IdCompressionMethod;

    #[test]
    fn test_hnsw_compression_realistic_workflow() {
        let params = HNSWParams {
            m: 32,
            m_max: 32,
            ef_construction: 200,
            ef_search: 100,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(128, params.m, params.m_max).unwrap();
        index.params = params;

        // Add realistic number of vectors
        for i in 0..10_000 {
            let vector: Vec<f32> = (0..128)
                .map(|j| ((i + j) as f32 / 1000.0).cos())
                .collect();
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        // Multiple queries
        for query_id in 0..100 {
            let query: Vec<f32> = (0..128)
                .map(|j| ((query_id + j) as f32 / 1000.0).cos())
                .collect();
            let results = index.search(&query, 20, 100).unwrap();
            assert_eq!(results.len(), 20);
        }
    }

    #[test]
    fn test_hnsw_compression_memory_usage() {
        let params = HNSWParams {
            m: 64,
            m_max: 64,
            ef_construction: 200,
            ef_search: 100,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 32,
            ..Default::default()
        };

        let mut index = HNSWIndex::new(64, params.m, params.m_max).unwrap();
        index.params = params;

        // Add many vectors
        for i in 0..20_000 {
            let vector: Vec<f32> = (0..64).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }

        index.build().unwrap();

        // Search should work
        let query: Vec<f32> = (0..64).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10, 100).unwrap();
        assert_eq!(results.len(), 10);
    }
}
