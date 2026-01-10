//! Tests for Product Quantization variants.

#[cfg(all(feature = "ivf_pq", feature = "scann"))]
mod tests {
    use rank_retrieve::dense::ivf_pq::pq::ProductQuantizer;
    use rank_retrieve::dense::ivf_pq::OptimizedProductQuantizer;
    use rank_retrieve::dense::ivf_pq::OnlineProductQuantizer;
    use rank_retrieve::RetrieveError;

    fn generate_test_vectors(num_vectors: usize, dimension: usize) -> Vec<f32> {
        let mut flat = Vec::with_capacity(num_vectors * dimension);
        for i in 0..num_vectors {
            let mut vec = vec![0.0f32; dimension];
            for j in 0..dimension {
                vec[j] = ((i * dimension + j) as f32) * 0.01;
            }
            // L2 normalize
            let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            for x in &mut vec {
                *x /= norm;
            }
            flat.extend_from_slice(&vec);
        }
        flat
    }

    #[test]
    fn test_pq_basic() -> Result<(), RetrieveError> {
        let dimension = 128;
        let num_codebooks = 8;
        let codebook_size = 256;
        let num_vectors = 1000;

        let vectors = generate_test_vectors(num_vectors, dimension);
        let mut pq = ProductQuantizer::new(dimension, num_codebooks, codebook_size)?;
        pq.fit(&vectors, num_vectors)?;

        let test_vector = &vectors[0..dimension];
        let codes = pq.quantize(test_vector);
        assert_eq!(codes.len(), num_codebooks);

        let distance = pq.approximate_distance(test_vector, &codes);
        assert!(distance.is_finite());
        assert!(distance >= 0.0);

        Ok(())
    }

    #[test]
    fn test_opq_basic() -> Result<(), RetrieveError> {
        let dimension = 128;
        let num_codebooks = 8;
        let codebook_size = 256;
        let num_vectors = 1000;

        let vectors = generate_test_vectors(num_vectors, dimension);
        let mut opq = OptimizedProductQuantizer::new(dimension, num_codebooks, codebook_size)?;
        opq.fit(&vectors, num_vectors, 3)?; // 3 iterations for test

        let test_vector = &vectors[0..dimension];
        let codes = opq.quantize(test_vector);
        assert_eq!(codes.len(), num_codebooks);

        let distance = opq.approximate_distance(test_vector, &codes);
        assert!(distance.is_finite());
        assert!(distance >= 0.0);

        Ok(())
    }

    #[test]
    fn test_online_pq_basic() -> Result<(), RetrieveError> {
        let dimension = 128;
        let num_codebooks = 8;
        let codebook_size = 256;
        let num_vectors = 1000;

        let vectors = generate_test_vectors(num_vectors, dimension);
        let mut opq = OnlineProductQuantizer::new(
            dimension,
            num_codebooks,
            codebook_size,
            0.1,  // learning_rate
            0.01, // forgetting_rate
        )?;

        // Initialize with first batch
        let init_size = 500;
        opq.initialize(&vectors[..init_size * dimension], init_size)?;

        // Update with remaining vectors
        for i in init_size..num_vectors {
            let vec = &vectors[i * dimension..(i + 1) * dimension];
            let codes = opq.update(vec)?;
            assert_eq!(codes.len(), num_codebooks);
        }

        // Test quantization
        let test_vector = &vectors[0..dimension];
        let codes = opq.quantize(test_vector);
        assert_eq!(codes.len(), num_codebooks);

        let distance = opq.approximate_distance(test_vector, &codes);
        assert!(distance.is_finite());
        assert!(distance >= 0.0);

        Ok(())
    }

    #[test]
    fn test_opq_vs_pq_accuracy() -> Result<(), RetrieveError> {
        let dimension = 128;
        let num_codebooks = 8;
        let codebook_size = 256;
        let num_vectors = 2000;

        let vectors = generate_test_vectors(num_vectors, dimension);
        let test_vector = &vectors[0..dimension];

        // Standard PQ
        let mut pq = ProductQuantizer::new(dimension, num_codebooks, codebook_size)?;
        pq.fit(&vectors, num_vectors)?;
        let pq_codes = pq.quantize(test_vector);
        let pq_distance = pq.approximate_distance(test_vector, &pq_codes);

        // OPQ
        let mut opq = OptimizedProductQuantizer::new(dimension, num_codebooks, codebook_size)?;
        opq.fit(&vectors, num_vectors, 5)?;
        let opq_codes = opq.quantize(test_vector);
        let opq_distance = opq.approximate_distance(test_vector, &opq_codes);

        // OPQ should generally have better (lower) distance
        // (though not guaranteed for all data distributions)
        println!("PQ distance: {:.6}, OPQ distance: {:.6}", pq_distance, opq_distance);
        assert!(pq_distance.is_finite());
        assert!(opq_distance.is_finite());

        Ok(())
    }

    #[test]
    fn test_online_pq_adaptation() -> Result<(), RetrieveError> {
        let dimension = 64;
        let num_codebooks = 4;
        let codebook_size = 128;

        let mut opq = OnlineProductQuantizer::new(
            dimension,
            num_codebooks,
            codebook_size,
            0.2,  // Higher learning rate for faster adaptation
            0.05, // Forgetting rate
        )?;

        // Initialize with vectors from one distribution
        let mut init_vectors = Vec::new();
        for i in 0..100 {
            let mut vec = vec![0.0f32; dimension];
            for j in 0..dimension {
                vec[j] = ((i + j) as f32) * 0.01; // Distribution 1
            }
            let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            for x in &mut vec {
                *x /= norm;
            }
            init_vectors.extend_from_slice(&vec);
        }
        opq.initialize(&init_vectors, 100)?;

        // Update with vectors from different distribution
        for i in 0..50 {
            let mut vec = vec![0.0f32; dimension];
            for j in 0..dimension {
                vec[j] = ((i * 2 + j * 3) as f32) * 0.01; // Different distribution
            }
            let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
            for x in &mut vec {
                *x /= norm;
            }
            opq.update(&vec)?;
        }

        // Should still work after adaptation
        let test_vec = vec![0.1f32; dimension];
        let codes = opq.quantize(&test_vec);
        assert_eq!(codes.len(), num_codebooks);

        Ok(())
    }
}
