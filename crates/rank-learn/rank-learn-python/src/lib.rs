//! Python bindings for rank-learn.

use pyo3::prelude::*;

/// Python module for rank-learn.
#[pymodule]
fn rank_learn(_py: Python, m: &PyModule) -> PyResult<()> {
    // TODO: Expose LTR functions to Python
    Ok(())
}

