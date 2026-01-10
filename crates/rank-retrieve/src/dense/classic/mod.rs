//! Classic ANN methods implementation.

#[cfg(feature = "lsh")]
pub mod lsh;

#[cfg(any(feature = "annoy", feature = "kdtree", feature = "balltree", feature = "rptree", feature = "kmeans_tree"))]
pub mod trees;
