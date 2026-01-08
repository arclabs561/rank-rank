//! Example: Generative retrieval with LTRGR.
//!
//! This example demonstrates how to use generative retrieval for passage retrieval.
//! Generative retrieval generates identifiers (titles, substrings, pseudo-queries)
//! and maps them to passages using heuristic scoring.

use rank_retrieve::generative::{
    AutoregressiveModel, GenerativeRetriever, HeuristicScorer, IdentifierGenerator,
    SimpleIdentifierGenerator,
};

// Mock model for demonstration
struct ExampleModel;

impl AutoregressiveModel for ExampleModel {
    fn generate(
        &self,
        query: &str,
        prefix: &str,
        beam_size: usize,
        _constraint_fn: Option<&dyn Fn(&str) -> bool>,
    ) -> Result<Vec<(String, f32)>, rank_retrieve::RetrieveError> {
        // Simple mock: generate identifiers based on query
        let mut identifiers = Vec::new();
        let terms: Vec<&str> = query
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .take(beam_size)
            .collect();

        for (i, term) in terms.iter().enumerate() {
            let identifier = match prefix {
                "title" => format!("{} Topic", term.to_uppercase()),
                "substring" => format!("{} is a key concept", term),
                "pseudo-query" => format!("what is {}", term),
                _ => term.to_string(),
            };
            let score = 10.0 - (i as f32 * 0.5);
            identifiers.push((identifier, score));
        }

        if identifiers.is_empty() {
            identifiers.push((format!("{} result", prefix), 5.0));
        }

        Ok(identifiers)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generative Retrieval Example");
    println!("============================\n");

    // Create model and retriever
    let model = ExampleModel;
    let mut retriever = GenerativeRetriever::new(model)
        .with_beam_size(10)
        .with_scorer(HeuristicScorer::new().with_case_insensitive(true));

    // Add documents to index
    retriever.add_document(
        0,
        "Prime Rate in Canada is a guideline interest rate used by banks on loans for their most creditworthy clients.",
    );
    retriever.add_document(
        1,
        "Machine learning is a subset of artificial intelligence that focuses on algorithms and statistical models.",
    );
    retriever.add_document(
        2,
        "The Bank of Canada sets the overnight rate which influences the prime rate.",
    );
    retriever.add_document(
        3,
        "Deep learning uses neural networks with multiple layers to learn complex patterns.",
    );

    // Query
    let query = "What is prime rate in Canada?";
    println!("Query: {}\n", query);

    // Retrieve passages
    let results = retriever.retrieve(query, 10)?;

    println!("Retrieved Passages (top {}):", results.len());
    println!("----------------------------");
    for (i, (passage_id, score)) in results.iter().enumerate() {
        println!("\n{}. Passage {} (score: {:.2})", i + 1, passage_id, score);
    }

    // Demonstrate identifier generation
    println!("\n\nIdentifier Generation Example");
    println!("==============================\n");

    let generator = SimpleIdentifierGenerator::new();
    let sample_passages = vec![
        (0, "Prime Rate in Canada is a guideline interest rate used by banks on loans for their most creditworthy clients."),
        (1, "Machine learning is a subset of artificial intelligence that focuses on algorithms and statistical models."),
    ];

    for (passage_id, passage_text) in &sample_passages {
        let identifiers = generator.generate(passage_text, *passage_id);
        println!("Passage {}:", passage_id);
        println!("  Title: {}", identifiers.title);
        println!(
            "  Substring: {}...",
            &identifiers.substring[..identifiers.substring.len().min(50)]
        );
        println!("  Pseudo-query: {}", identifiers.pseudo_query);
        println!();
    }

    Ok(())
}
