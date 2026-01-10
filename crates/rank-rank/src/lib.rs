//! # rank-rank
//!
//! The unified entry point for the `rank-*` information retrieval ecosystem.
//!
//! This crate serves two purposes:
//! 1. **Facade**: Re-exports all `rank-*` crates (`retrieve`, `fusion`, `rerank`, `eval`)
//!    into a single, consistent namespace.
//! 2. **Orchestrator**: Provides a [`Pipeline`] abstraction to compose these stages
//!    into a complete RAG or search system.
//!
//! # Quick Start
//!
//! ```rust
//! use rank_rank::prelude::*;
//!
//! // 1. Define a pipeline
//! // let pipeline = Pipeline::builder()
//! //     .retrieve(Bm25::new(...))
//! //     .fuse(Rrf::default())
//! //     .rerank(MaxSim::new(...))
//! //     .build();
//!
//! // 2. Search
//! // let results = pipeline.search("machine learning");
//! ```
//!
//! # Architecture
//!
//! The ecosystem is composed of four stages:
//!
//! 1. **Retrieve** (`rank-retrieve`): First-stage retrieval. Fast, coarse-grained.
//!    - BM25, TF-IDF, Dense (ANN), Sparse (SPLADE), Generative (LTRGR).
//!    - Goal: 10M docs -> 1,000 candidates.
//!
//! 2. **Fuse** (`rank-fusion`): Hybrid search signal combination.
//!    - RRF, ISR, CombMNZ.
//!    - Goal: Combine text + semantic signals robustly.
//!
//! 3. **Rerank** (`rank-rerank`): Second-stage precision.
//!    - MaxSim (ColBERT), Cross-Encoders.
//!    - Goal: 1,000 candidates -> 100 results.
//!
//! 4. **Eval** (`rank-eval`): Metrics and measurement.
//!    - NDCG, MAP, MRR.
//!    - Goal: Quantify quality.

pub mod pipeline;

/// Re-exports of core crates.
pub mod retrieve {
    pub use rank_retrieve::*;
}

pub mod fusion {
    pub use rank_fusion::*;
}

pub mod rerank {
    pub use rank_rerank::*;
}

pub mod eval {
    pub use rank_eval::*;
}

pub mod prelude {
    // Retrieve prelude (BM25, Dense, etc.)
    pub use crate::retrieve::prelude::*;
    
    // Fusion algorithms (RRF, ISR, etc.)
    // Note: We don't export `RetrieverId` from fusion to avoid conflict with retrieve::prelude::RetrieverId
    pub use crate::fusion::{
        rrf, rrf_k, rrf_multi, rrf_multi_k,
        isr, isr_k, isr_multi, isr_multi_k,
        combmnz, combmnz_k, combmnz_multi, combmnz_multi_k,
        combsum, combsum_k, combsum_multi, combsum_multi_k,
        borda, borda_k, borda_multi, borda_multi_k,
        dbsf, dbsf_k, dbsf_multi, dbsf_multi_k,
        weighted, weighted_k, weighted_multi, weighted_multi_k,
        RrfConfig, IsrConfig, CombMnzConfig, DbsfConfig, WeightedConfig,
        FusionError,
    };

    // Rerank prelude (MaxSim, etc.)
    pub use crate::rerank::prelude::*;
    
    // Eval metrics
    pub use crate::eval::{
        // Binary metrics
        binary::{ndcg_at_k, average_precision, reciprocal_rank, precision_at_k, recall_at_k},
        // Graded metrics
        graded::{compute_ndcg, compute_err},
    };
    
    pub use crate::pipeline::{Pipeline, PipelineBuilder};
}
