//! Example: Filtering and Faceting in Vector Search
//!
//! Demonstrates how to use filters and facets together for metadata-based retrieval.
//!
//! **Key Concepts**:
//! - **Filters**: Narrow search by metadata (e.g., category=1, region=2)
//! - **Facets**: Discover available filter values and their counts
//!
//! **Use Cases**:
//! - E-commerce: Filter products by category, show available brands with counts
//! - Content search: Filter by topic, show available authors
//! - RAG systems: Filter by document type, show available sources
//!
//! Run: `cargo run --example filtering_and_faceting --features dense`

use rank_retrieve::dense::DenseRetriever;
use rank_retrieve::filtering::{FilterPredicate, MetadataStore};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Filtering and Faceting Example ===\n");

    // Create retriever with metadata support
    let mut retriever = DenseRetriever::with_metadata();

    // Simulate a product catalog with embeddings and metadata
    // In practice, embeddings would come from a model (e.g., sentence-transformers)
    let products = vec![
        (0, vec![0.9, 0.1, 0.0], ("laptop", "electronics", "brand_a")),
        (1, vec![0.8, 0.2, 0.0], ("laptop", "electronics", "brand_b")),
        (2, vec![0.7, 0.3, 0.0], ("laptop", "electronics", "brand_a")),
        (3, vec![0.1, 0.9, 0.0], ("book", "media", "brand_c")),
        (4, vec![0.2, 0.8, 0.0], ("book", "media", "brand_c")),
        (5, vec![0.3, 0.7, 0.0], ("phone", "electronics", "brand_a")),
        (6, vec![0.4, 0.6, 0.0], ("phone", "electronics", "brand_b")),
        (7, vec![0.5, 0.5, 0.0], ("tablet", "electronics", "brand_a")),
    ];

    // Category mapping: category_name -> category_id
    let category_map: HashMap<&str, u32> = [
        ("laptop", 0),
        ("book", 1),
        ("phone", 2),
        ("tablet", 3),
    ]
    .iter()
    .cloned()
    .collect();

    // Department mapping: department_name -> department_id
    let department_map: HashMap<&str, u32> = [
        ("electronics", 0),
        ("media", 1),
    ]
    .iter()
    .cloned()
    .collect();

    // Brand mapping: brand_name -> brand_id
    let brand_map: HashMap<&str, u32> = [
        ("brand_a", 0),
        ("brand_b", 1),
        ("brand_c", 2),
    ]
    .iter()
    .cloned()
    .collect();

    // Add documents with metadata
    for (doc_id, embedding, (category, department, brand)) in &products {
        retriever.add_document(*doc_id, embedding.clone());

        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), *category_map.get(category).unwrap());
        metadata.insert("department".to_string(), *department_map.get(department).unwrap());
        metadata.insert("brand".to_string(), *brand_map.get(brand).unwrap());

        retriever.add_metadata(*doc_id, metadata)?;
    }

    println!("Added {} products with metadata\n", products.len());

    // === FACETING: Discover available filter values ===
    println!("=== Step 1: Faceting - Discover Available Values ===\n");

    // Get metadata store from retriever
    let metadata_store = retriever.metadata().unwrap();
    let categories = metadata_store.get_all_values("category");
    println!("Available categories: {:?}", categories);

    // Get category counts (faceting)
    let category_counts = metadata_store.get_value_counts("category");
    println!("\nCategory distribution:");
    for (category_id, count) in &category_counts {
        let category_name = category_map
            .iter()
            .find(|(_, &id)| id == *category_id)
            .map(|(name, _)| *name)
            .unwrap_or("unknown");
        println!("  {} (id={}): {} products", category_name, category_id, count);
    }

    // Get brand counts
    let brand_counts = metadata_store.get_value_counts("brand");
    println!("\nBrand distribution:");
    for (brand_id, count) in &brand_counts {
        let brand_name = brand_map
            .iter()
            .find(|(_, &id)| id == *brand_id)
            .map(|(name, _)| *name)
            .unwrap_or("unknown");
        println!("  {} (id={}): {} products", brand_name, brand_id, count);
    }

    // === FILTERING: Narrow search by metadata ===
    println!("\n=== Step 2: Filtering - Narrow Search ===\n");

    let query_embedding = vec![0.85, 0.15, 0.0]; // Query for "laptop-like" products

    // Unfiltered search
    println!("Unfiltered search (top 5):");
    let unfiltered = retriever.retrieve(&query_embedding, 5)?;
    for (i, (doc_id, score)) in unfiltered.iter().enumerate() {
        println!("  {}. Doc {}: {:.4}", i + 1, doc_id, score);
    }

    // Filtered search: only electronics
    println!("\nFiltered search (department=electronics, top 5):");
    let electronics_filter = FilterPredicate::equals("department", 0);
    let filtered = retriever.retrieve_with_filter(&query_embedding, 5, &electronics_filter)?;
    for (i, (doc_id, score)) in filtered.iter().enumerate() {
        println!("  {}. Doc {}: {:.4}", i + 1, doc_id, score);
    }

    // Filtered search: only laptops
    println!("\nFiltered search (category=laptop, top 5):");
    let laptop_filter = FilterPredicate::equals("category", 0);
    let laptop_results = retriever.retrieve_with_filter(&query_embedding, 5, &laptop_filter)?;
    for (i, (doc_id, score)) in laptop_results.iter().enumerate() {
        println!("  {}. Doc {}: {:.4}", i + 1, doc_id, score);
    }

    // Combined filter: electronics AND laptop
    println!("\nCombined filter (department=electronics AND category=laptop, top 5):");
    let combined_filter = FilterPredicate::And(vec![
        FilterPredicate::equals("department", 0),
        FilterPredicate::equals("category", 0),
    ]);
    let combined_results = retriever.retrieve_with_filter(&query_embedding, 5, &combined_filter)?;
    for (i, (doc_id, score)) in combined_results.iter().enumerate() {
        println!("  {}. Doc {}: {:.4}", i + 1, doc_id, score);
    }

    // === FILTERED FACETING: Show available values in filtered results ===
    println!("\n=== Step 3: Filtered Faceting - Values in Filtered Results ===\n");

    // Get brand counts for electronics only
    println!("Brand distribution in Electronics department:");
    let electronics_brand_counts = metadata_store.get_value_counts_filtered("brand", &electronics_filter);
    for (brand_id, count) in &electronics_brand_counts {
        let brand_name = brand_map
            .iter()
            .find(|(_, &id)| id == *brand_id)
            .map(|(name, _)| *name)
            .unwrap_or("unknown");
        println!("  {} (id={}): {} products", brand_name, brand_id, count);
    }

    // Get category counts for electronics only
    println!("\nCategory distribution in Electronics department:");
    let electronics_category_counts = metadata_store.get_value_counts_filtered("category", &electronics_filter);
    for (category_id, count) in &electronics_category_counts {
        let category_name = category_map
            .iter()
            .find(|(_, &id)| id == *category_id)
            .map(|(name, _)| *name)
            .unwrap_or("unknown");
        println!("  {} (id={}): {} products", category_name, category_id, count);
    }

    // === SELECTIVITY ESTIMATION ===
    println!("\n=== Step 4: Selectivity Estimation ===\n");

    let laptop_selectivity = metadata_store.estimate_selectivity(&laptop_filter);
    println!(
        "Laptop filter selectivity: {:.1}%",
        laptop_selectivity.unwrap_or(0.0) * 100.0
    );

    let electronics_selectivity = metadata_store.estimate_selectivity(&electronics_filter);
    println!(
        "Electronics filter selectivity: {:.1}%",
        electronics_selectivity.unwrap_or(0.0) * 100.0
    );

    let combined_selectivity = metadata_store.estimate_selectivity(&combined_filter);
    println!(
        "Combined filter selectivity: {:.1}%",
        combined_selectivity.unwrap_or(0.0) * 100.0
    );

    println!("\n=== Summary ===");
    println!("✅ Faceting: Discover available filter values and counts");
    println!("✅ Filtering: Narrow search by metadata constraints");
    println!("✅ Filtered Faceting: Show values available in filtered results");
    println!("✅ Selectivity: Estimate filter effectiveness for oversampling");
    println!("\nSee docs/FACETS_VS_FILTERS.md for detailed analysis.");

    Ok(())
}
