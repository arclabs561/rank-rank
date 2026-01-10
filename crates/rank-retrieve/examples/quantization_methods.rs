//! Example: Product Quantization Methods
//!
//! Demonstrates different product quantization approaches:
//! - Standard Product Quantization (PQ)
//! - Optimized Product Quantization (OPQ) - minimizes quantization distortions
//! - Online Product Quantization (O-PQ) - adapts to dynamic datasets
//!
//! **When to use each:**
//! - **PQ**: Standard quantization, good baseline
//! - **OPQ**: Better accuracy, willing to pay for optimization time
//! - **O-PQ**: Streaming data, dynamic datasets, online learning scenarios
//!
//! **Performance Trade-offs:**
//! - PQ: Fast training, good compression
//! - OPQ: Slower training (optimization), better accuracy
//! - O-PQ: Continuous updates, adapts to data distribution changes

#[cfg(all(feature = "ivf_pq", feature = "scann"))]
use rank_retrieve::dense::ivf_pq::pq::ProductQuantizer;
#[cfg(all(feature = "ivf_pq", feature = "scann"))]
use rank_retrieve::dense::ivf_pq::OptimizedProductQuantizer;
#[cfg(all(feature = "ivf_pq", feature = "scann"))]
use rank_retrieve::dense::ivf_pq::OnlineProductQuantizer;
#[cfg(all(feature = "ivf_pq", feature = "scann"))]
use rank_retrieve::RetrieveError;

#[cfg(all(feature = "ivf_pq", feature = "scann"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Product Quantization Methods Example ===\n");

    let dimension = 128;
    let num_codebooks = 8;
    let codebook_size = 256;
    let num_vectors = 10000;

    // Generate sample vectors
    let mut vectors = Vec::new();
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
        vectors.push(vec);
    }

    // Flatten for SoA format
    let mut flat_vectors = Vec::with_capacity(num_vectors * dimension);
    for vec in &vectors {
        flat_vectors.extend_from_slice(vec);
    }

    // Test vector
    let test_vector = &vectors[0];

    // Example 1: Standard Product Quantization
    println!("1. Standard Product Quantization (PQ)");
    let mut pq = ProductQuantizer::new(dimension, num_codebooks, codebook_size)?;
    pq.fit(&flat_vectors, num_vectors)?;
    
    let pq_codes = pq.quantize(test_vector);
    let pq_distance = pq.approximate_distance(test_vector, &pq_codes);
    println!("   Codes: {:?}", &pq_codes[..4.min(pq_codes.len())]);
    println!("   Approximate distance: {:.6}\n", pq_distance);

    // Example 2: Optimized Product Quantization (OPQ)
    println!("2. Optimized Product Quantization (OPQ)");
    let mut opq = OptimizedProductQuantizer::new(dimension, num_codebooks, codebook_size)?;
    opq.fit(&flat_vectors, num_vectors, 5)?; // 5 iterations
    
    let opq_codes = opq.quantize(test_vector);
    let opq_distance = opq.approximate_distance(test_vector, &opq_codes);
    println!("   Codes: {:?}", &opq_codes[..4.min(opq_codes.len())]);
    println!("   Approximate distance: {:.6}", opq_distance);
    println!("   Improvement over PQ: {:.2}%", 
             ((pq_distance - opq_distance) / pq_distance) * 100.0);
    println!();

    // Example 3: Online Product Quantization (O-PQ)
    println!("3. Online Product Quantization (O-PQ)");
    let mut opq_online = OnlineProductQuantizer::new(
        dimension,
        num_codebooks,
        codebook_size,
        0.1,  // learning_rate
        0.01, // forgetting_rate
    )?;
    
    // Initialize with first batch
    let init_batch_size = 1000;
    let mut init_flat = Vec::with_capacity(init_batch_size * dimension);
    for vec in &vectors[..init_batch_size] {
        init_flat.extend_from_slice(vec);
    }
    opq_online.initialize(&init_flat, init_batch_size)?;
    
    // Update with new vectors (simulating streaming)
    println!("   Initialized with {} vectors", init_batch_size);
    for i in init_batch_size..(init_batch_size + 100) {
        let codes = opq_online.update(&vectors[i])?;
        if i == init_batch_size {
            println!("   Updated with new vector, codes: {:?}", &codes[..4.min(codes.len())]);
        }
    }
    println!("   Updated with {} additional vectors", 100);
    
    let online_codes = opq_online.quantize(test_vector);
    let online_distance = opq_online.approximate_distance(test_vector, &online_codes);
    println!("   Approximate distance: {:.6}\n", online_distance);

    // Comparison
    println!("=== Comparison ===");
    println!("PQ distance:     {:.6}", pq_distance);
    println!("OPQ distance:   {:.6} ({:.2}% better)", 
             opq_distance, ((pq_distance - opq_distance) / pq_distance) * 100.0);
    println!("O-PQ distance:  {:.6}", online_distance);
    println!();
    println!("**Key Takeaways:**");
    println!("- OPQ optimizes space decomposition for better accuracy");
    println!("- O-PQ adapts to new data without full retraining");
    println!("- Choose based on accuracy vs. adaptability trade-offs");

    Ok(())
}

#[cfg(not(all(feature = "ivf_pq", feature = "scann")))]
fn main() {
    println!("This example requires 'ivf_pq' and 'scann' features.");
    println!("Run with: cargo run --example quantization_methods --features ivf_pq,scann");
}
