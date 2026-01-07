//! Python bindings for rank-retrieve.

use pyo3::prelude::*;

/// Python module for rank-retrieve.
#[pymodule]
fn rank_retrieve(_py: Python, m: &PyModule) -> PyResult<()> {
    // TODO: Expose retrieval functions to Python
    Ok(())
}

