//! Integration tests for IVF-PQ with ID compression.

#[cfg(all(feature = "id-compression", feature = "ivf_pq"))]
mod tests {
    use rank_retrieve::dense::ivf_pq::{IVFPQIndex, IVFPQParams};
    use rank_retrieve::compression::IdCompressionMethod;

    #[test]
    fn test_ivf_with_compression() {
        let params = IVFPQParams {
            num_clusters: 64,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: Some(IdCompressionMethod::Roc),
            compression_threshold: 10,  // Compress clusters with > 10 IDs
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        // Add vectors
        for i in 0..1000 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }

        // Build (compression happens here)
        index.build().unwrap();

        // Search should work
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();

        assert_eq!(results.len(), 10);
        assert!(results.iter().all(|(_, score)| score.is_finite()));
    }

    #[test]
    fn test_ivf_without_compression() {
        let params = IVFPQParams {
            num_clusters: 64,
            nprobe: 10,
            num_codebooks: 4,
            codebook_size: 256,
            id_compression: None,
            compression_threshold: 100,
        };

        let mut index = IVFPQIndex::new(128, params).unwrap();

        // Add vectors
        for i in 0..1000 {
            let vector: Vec<f32> = (0..128).map(|j| (i + j) as f32 / 1000.0).collect();
            index.add(i, vector).unwrap();
        }

        // Build
        index.build().unwrap();

        // Search should work
        let query: Vec<f32> = (0..128).map(|i| i as f32 / 1000.0).collect();
        let results = index.search(&query, 10).unwrap();

        assert_eq!(results.len(), 10);
    }
}
