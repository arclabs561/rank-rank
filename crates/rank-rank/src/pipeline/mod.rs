use crate::prelude::*;
use std::marker::PhantomData;

/// A unified search pipeline.
///
/// Coordinates the flow of data through:
/// Retrieve -> Fuse -> Rerank
pub struct Pipeline<Id = u32> {
    // Placeholder for pipeline state
    _marker: PhantomData<Id>,
}

impl<Id> Pipeline<Id> {
    pub fn builder() -> PipelineBuilder<Id> {
        PipelineBuilder::new()
    }
}

pub struct PipelineBuilder<Id> {
    _marker: PhantomData<Id>,
}

impl<Id> PipelineBuilder<Id> {
    pub fn new() -> Self {
        Self { _marker: PhantomData }
    }

    // TODO: Add methods to attach specific retrievers, fusers, rerankers
    // This requires defining a common trait for "Retriever" that wraps the various
    // rank-retrieve implementations, or just accepting closures.
    
    pub fn build(self) -> Pipeline<Id> {
        Pipeline { _marker: PhantomData }
    }
}
